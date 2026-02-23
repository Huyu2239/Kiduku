use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveTime, TimeZone, Utc, Weekday};
use poise::serenity_prelude as serenity;

use crate::infrastructure::db::Db;
use crate::presentation::entry::util::{current_unix_timestamp, truncate};

const JST_OFFSET_SECS: i64 = 9 * 3600;
const BATCH_HOUR_JST: u32 = 8;
const ONE_MONTH_SECS: i64 = 30 * 24 * 3600;

pub fn start(ctx: serenity::Context, db: Db) {
    tokio::spawn(async move {
        loop {
            let sleep_duration = duration_until_next_monday_8am_jst();
            tokio::time::sleep(sleep_duration).await;

            let now_unix = current_unix_timestamp();

            if is_first_monday_of_month_jst(now_unix) {
                run_monthly_batch(&ctx, &db, now_unix).await;
            }
            run_weekly_batch(&ctx, &db).await;
        }
    });
}

async fn run_weekly_batch(ctx: &serenity::Context, db: &Db) {
    tracing::info!("週次バッチ開始");

    let targets = match db.fetch_unread_targets_for_weekly_batch().await {
        Ok(t) => t,
        Err(err) => {
            tracing::error!("週次バッチ: ターゲット取得失敗: {:?}", err);
            return;
        }
    };

    if targets.is_empty() {
        tracing::info!("週次バッチ: 未読ターゲットなし");
        return;
    }

    // user_id でグループ化
    let mut by_user: HashMap<u64, Vec<i64>> = HashMap::new();
    for (mention_id, user_id) in targets {
        by_user.entry(user_id).or_default().push(mention_id);
    }

    for (user_id, mention_ids) in &by_user {
        if let Err(err) = send_weekly_dm(ctx, *user_id, mention_ids).await {
            tracing::error!("週次DM送信失敗 user={}: {:?}", user_id, err);
        }
    }

    tracing::info!("週次バッチ完了: {}ユーザーに通知", by_user.len());
}

async fn send_weekly_dm(
    ctx: &serenity::Context,
    user_id: u64,
    mention_ids: &[i64],
) -> anyhow::Result<()> {
    let dm_channel = serenity::UserId::new(user_id)
        .create_dm_channel(&ctx.http)
        .await?;

    let count = mention_ids.len();
    let content = format!(
        "📬 **未読メンション通知**\n\
         未読・未解決のメンションが{}件あります。\n\
         `/通知一覧` コマンドで確認してください。",
        count
    );

    dm_channel
        .send_message(&ctx.http, serenity::CreateMessage::new().content(content))
        .await?;

    Ok(())
}

async fn run_monthly_batch(ctx: &serenity::Context, db: &Db, now_unix: i64) {
    tracing::info!("月次バッチ開始");

    let cutoff_unix = now_unix - ONE_MONTH_SECS;
    let targets = match db
        .fetch_expiring_targets_for_monthly_batch(cutoff_unix, now_unix)
        .await
    {
        Ok(t) => t,
        Err(err) => {
            tracing::error!("月次バッチ: ターゲット取得失敗: {:?}", err);
            return;
        }
    };

    if targets.is_empty() {
        tracing::info!("月次バッチ: 期限切れターゲットなし");
        return;
    }

    // user_id でグループ化
    let mut by_user: HashMap<u64, Vec<_>> = HashMap::new();
    for (user_id, item) in targets {
        by_user.entry(user_id).or_default().push(item);
    }

    for (user_id, items) in &by_user {
        if let Err(err) = send_monthly_dm(ctx, *user_id, items).await {
            tracing::error!("月次DM送信失敗 user={}: {:?}", user_id, err);
        }
    }

    tracing::info!("月次バッチ完了: {}ユーザーに通知", by_user.len());
}

async fn send_monthly_dm(
    ctx: &serenity::Context,
    user_id: u64,
    items: &[crate::infrastructure::db::MentionForTarget],
) -> anyhow::Result<()> {
    let dm_channel = serenity::UserId::new(user_id)
        .create_dm_channel(&ctx.http)
        .await?;

    let header = format!(
        "⚠️ **期限切れメンション通知**\n\
         1ヶ月以上前のメンションが{}件あります。\n\
         各メンションを延命するか無視するかを選択してください。",
        items.len()
    );

    dm_channel
        .send_message(&ctx.http, serenity::CreateMessage::new().content(header))
        .await?;

    for item in items {
        let message_link = format!(
            "https://discord.com/channels/{}/{}/{}",
            item.guild_id, item.channel_id, item.message_id
        );
        let snippet = truncate(&item.content, 80);

        let content = format!(
            "**メッセージ**: [リンク]({})\n**内容**: {}",
            message_link, snippet
        );

        let extend_button =
            serenity::CreateButton::new(format!("mm:extend:{}:{}", item.mention_id, user_id))
                .label("1ヶ月延命")
                .style(serenity::ButtonStyle::Primary);

        let ignore_button =
            serenity::CreateButton::new(format!("mm:ignore:{}:{}", item.mention_id, user_id))
                .label("無視")
                .style(serenity::ButtonStyle::Danger);

        let components = vec![serenity::CreateActionRow::Buttons(vec![
            extend_button,
            ignore_button,
        ])];

        dm_channel
            .send_message(
                &ctx.http,
                serenity::CreateMessage::new()
                    .content(content)
                    .components(components),
            )
            .await?;
    }

    Ok(())
}

fn duration_until_next_monday_8am_jst() -> std::time::Duration {
    let now_unix = current_unix_timestamp();
    let now_utc = Utc
        .timestamp_opt(now_unix, 0)
        .single()
        .unwrap_or_else(Utc::now);

    // 現在のJST日付
    let now_jst = now_utc + Duration::seconds(JST_OFFSET_SECS);
    let today = now_jst.date_naive();

    // 次の月曜日を探す
    let days_until_monday = match today.weekday() {
        Weekday::Mon => 0,
        Weekday::Tue => 6,
        Weekday::Wed => 5,
        Weekday::Thu => 4,
        Weekday::Fri => 3,
        Weekday::Sat => 2,
        Weekday::Sun => 1,
    };

    let next_monday = today + Duration::days(days_until_monday as i64);
    let batch_time = NaiveTime::from_hms_opt(BATCH_HOUR_JST, 0, 0).unwrap();
    let next_run_naive = next_monday.and_time(batch_time);

    // JST → UTC (UTC = JST - 9h)
    let next_run_unix = next_run_naive.and_utc().timestamp() - JST_OFFSET_SECS;

    let sleep_secs = (next_run_unix - now_unix).max(0) as u64;

    // 既に今日が月曜かつ8時を過ぎている場合は来週に
    if sleep_secs == 0 {
        let next_run_unix = next_run_unix + 7 * 24 * 3600;
        let sleep_secs = (next_run_unix - now_unix).max(1) as u64;
        return std::time::Duration::from_secs(sleep_secs);
    }

    std::time::Duration::from_secs(sleep_secs)
}

fn is_first_monday_of_month_jst(now_unix: i64) -> bool {
    let now_utc = Utc
        .timestamp_opt(now_unix, 0)
        .single()
        .unwrap_or_else(Utc::now);
    let now_jst = now_utc + Duration::seconds(JST_OFFSET_SECS);
    let date = now_jst.date_naive();

    date.weekday() == Weekday::Mon && date.day() <= 7
}

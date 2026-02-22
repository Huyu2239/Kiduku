use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context as _;
use poise::serenity_prelude as serenity;
use serenity::model::prelude::{MessageId, RoleId, UserId};

use crate::presentation::{Context, Error};
use crate::usecase::dto::{CheckReadsInputDto, MessageWithReactionsDto, UsecaseError};
use crate::usecase::slash_commands::check_reads as check_reads_usecase;

const DEFAULT_HOURS: i64 = 24;
const MAX_MESSAGES: usize = 10;
const FETCH_LIMIT: i64 = 50;

#[poise::command(slash_command, rename = "check-reads")]
pub async fn main(
    ctx: Context<'_>,
    #[description = "過去何時間を対象にするか（デフォルト24）"] hours: Option<i64>,
) -> Result<(), Error> {
    let hours = hours.unwrap_or(DEFAULT_HOURS);
    if hours <= 0 {
        ctx.send(
            poise::CreateReply::default()
                .content("hours は1以上の数値を指定してください。")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let guild_id = match ctx.guild_id() {
        Some(id) => id,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("このコマンドはサーバー内でのみ利用できます。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let author_id = ctx.author().id;
    let cutoff_unix = cutoff_unix_timestamp(hours)?;

    let mentions = match ctx
        .data()
        .db
        .fetch_mentions_for_author(author_id.get(), cutoff_unix, FETCH_LIMIT)
        .await
    {
        Ok(mentions) => mentions,
        Err(err) => {
            tracing::error!("failed to fetch mentions: {:?}", err);
            ctx.send(
                poise::CreateReply::default()
                    .content("既読状況の取得に失敗しました。DBの接続状態を確認してください。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    if mentions.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content("対象となるメンション付きメッセージが見つかりませんでした。")
                .ephemeral(true),
        )
        .await?;
        return Ok(());
    }

    let mut link_map: HashMap<u64, (u64, u64)> = HashMap::new();
    let dto_messages = mentions
        .iter()
        .map(|mention| {
            link_map.insert(mention.message_id, (mention.guild_id, mention.channel_id));
            map_to_message_with_reactions(mention)
        })
        .collect::<Vec<_>>();

    let input = CheckReadsInputDto {
        user_id: author_id,
        hours,
        messages: dto_messages,
    };

    let outputs = match check_reads_usecase::execute(input) {
        Ok(outputs) => outputs,
        Err(err) => {
            handle_usecase_error(&ctx, err).await?;
            return Ok(());
        }
    };

    let mut embed = serenity::CreateEmbed::new()
        .title("既読状況の確認結果")
        .description(format!(
            "過去{}時間に保存されたメンション付きメッセージを確認しました。",
            hours
        ));

    for (idx, output) in outputs.into_iter().enumerate() {
        if idx >= MAX_MESSAGES {
            embed = embed.field(
                "注意",
                format!("表示件数は最大{}件です。", MAX_MESSAGES),
                false,
            );
            break;
        }

        let message_id = output.message_id.get();
        let (link_guild_id, link_channel_id) = link_map
            .get(&message_id)
            .copied()
            .unwrap_or((guild_id.get(), ctx.channel_id().get()));
        let message_link = format!(
            "https://discord.com/channels/{}/{}/{}",
            link_guild_id, link_channel_id, message_id
        );
        let summary = format_message_summary(&output);
        embed = embed.field(message_link, summary, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

fn map_to_message_with_reactions(
    mention: &crate::infrastructure::db::StoredMention,
) -> MessageWithReactionsDto {
    MessageWithReactionsDto {
        message_id: MessageId::new(mention.message_id),
        content: mention.content.clone(),
        user_mentions: mention
            .target_user_ids
            .iter()
            .map(|id| UserId::new(*id))
            .collect(),
        role_mentions: Vec::<RoleId>::new(),
        expanded_role_member_ids: Vec::new(),
        mentions_everyone: false,
        everyone_member_ids: Vec::new(),
        reaction_user_ids: mention
            .read_user_ids
            .iter()
            .map(|id| UserId::new(*id))
            .collect(),
    }
}

fn cutoff_unix_timestamp(hours: i64) -> Result<i64, Error> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before UNIX_EPOCH")?
        .as_secs() as i64;
    let seconds = hours
        .checked_mul(60 * 60)
        .context("hours parameter is too large")?;
    Ok(now.saturating_sub(seconds))
}

async fn handle_usecase_error(ctx: &Context<'_>, err: UsecaseError) -> Result<(), Error> {
    let message = match err {
        UsecaseError::NoMessages | UsecaseError::NoMentionedUsers => {
            "対象となるメッセージが見つかりませんでした。".to_string()
        }
        UsecaseError::InvalidHoursParameter => {
            "hours は1以上の数値を指定してください。".to_string()
        }
        UsecaseError::InvalidMentionData => {
            "メンション情報が不正です。DB保存処理を確認してください。".to_string()
        }
        UsecaseError::RoleNotFound(role_id) => {
            format!("ロールが見つかりませんでした: {}", role_id.get())
        }
    };

    ctx.send(
        poise::CreateReply::default()
            .content(message)
            .ephemeral(true),
    )
    .await?;
    Ok(())
}

fn format_message_summary(output: &crate::usecase::dto::CheckReadsOutputDto) -> String {
    let snippet = truncate(&output.message_content, 100);
    let total = output.mentioned_users.len();
    let read = output.read_users.len();
    let percent = if total == 0 { 0 } else { (read * 100) / total };

    let read_users = format_user_mentions(&output.read_users);
    let unread_users = format_user_mentions(&output.unread_users);

    format!(
        "内容: {}\n既読: {}/{} ({}%)\n既読ユーザー: {}\n未読ユーザー: {}",
        snippet, read, total, percent, read_users, unread_users
    )
}

fn truncate(content: &str, max_chars: usize) -> String {
    let mut truncated = content.chars().take(max_chars).collect::<String>();
    if content.chars().count() > max_chars {
        truncated.push('…');
    }
    truncated
}

fn format_user_mentions(users: &[UserId]) -> String {
    if users.is_empty() {
        return "なし".into();
    }
    users
        .iter()
        .map(|id| format!("<@{}>", id.get()))
        .collect::<Vec<_>>()
        .join(" ")
}

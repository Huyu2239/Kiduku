use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;

use crate::infrastructure::db::MentionForTarget;
use crate::presentation::{Context, Error};

const PAGE_SIZE: usize = 5;

#[poise::command(slash_command, rename = "通知一覧")]
pub async fn main(
    ctx: Context<'_>,
    #[description = "解決済みも表示する"] show_done: Option<bool>,
) -> Result<(), Error> {
    let show_done = show_done.unwrap_or(false);
    let is_ephemeral = ctx.guild_id().is_some();
    let user_id = ctx.author().id;

    let items = fetch_page(&ctx.data().db, user_id.get(), 0, show_done).await?;

    if items.is_empty() {
        ctx.send(
            poise::CreateReply::default()
                .content("表示できるメンションがありません。")
                .ephemeral(is_ephemeral),
        )
        .await?;
        return Ok(());
    }

    let has_next = items.len() > PAGE_SIZE;
    let page_items = &items[..items.len().min(PAGE_SIZE)];
    let guild_id = ctx.guild_id();

    let embeds = build_embeds(page_items, guild_id);
    let components = build_nav_buttons(0, show_done, user_id.get(), false, has_next);

    let mut reply = poise::CreateReply::default()
        .components(components)
        .ephemeral(is_ephemeral);
    reply.embeds = embeds;
    ctx.send(reply).await?;
    Ok(())
}

pub async fn fetch_page(
    db: &crate::infrastructure::db::Db,
    user_id: u64,
    page: usize,
    show_done: bool,
) -> Result<Vec<MentionForTarget>, Error> {
    let offset = (page * PAGE_SIZE) as i64;
    // フィルタリングのため多めに取得する
    let fetch_limit = if show_done {
        (PAGE_SIZE + 1) as i64
    } else {
        // show_done=false の場合、done済みを除外するため余裕を持って取得
        (PAGE_SIZE * 3 + 1) as i64
    };

    let mut items = db
        .fetch_mentions_for_target(user_id, offset, fetch_limit)
        .await?;

    if !show_done {
        items.retain(|item| !item.is_done);
    }

    Ok(items)
}

pub fn build_embeds(
    items: &[MentionForTarget],
    guild_id: Option<serenity::GuildId>,
) -> Vec<serenity::CreateEmbed> {
    items
        .iter()
        .map(|item| {
            let date = DateTime::<Utc>::from_timestamp(item.created_at_unix, 0)
                .map(|dt| dt.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|| "不明".to_string());

            let snippet = truncate(&item.content, 100);

            let guild_id_val = guild_id.map(|g| g.get()).unwrap_or(item.guild_id);
            let message_link = format!(
                "https://discord.com/channels/{}/{}/{}",
                guild_id_val, item.channel_id, item.message_id
            );

            let status = if item.is_done {
                "解決済み 🔒"
            } else if item.is_read {
                "既読 ✅"
            } else {
                "未読 ❌"
            };

            serenity::CreateEmbed::new()
                .title(format!("メッセージ ({})", date))
                .description(snippet)
                .field("送信者", format!("<@{}>", item.author_id), true)
                .field("状態", status, true)
                .field("リンク", format!("[開く]({})", message_link), true)
        })
        .collect()
}

pub fn build_nav_buttons(
    page: usize,
    show_done: bool,
    user_id: u64,
    has_prev: bool,
    has_next: bool,
) -> Vec<serenity::CreateActionRow> {
    let show_done_flag = if show_done { 1u8 } else { 0u8 };

    let prev_button = serenity::CreateButton::new(format!(
        "mm:p:{}:{}:{}",
        page.saturating_sub(1),
        show_done_flag,
        user_id
    ))
    .label("◀ 前へ")
    .style(serenity::ButtonStyle::Secondary)
    .disabled(!has_prev);

    let next_button =
        serenity::CreateButton::new(format!("mm:p:{}:{}:{}", page + 1, show_done_flag, user_id))
            .label("次へ ▶")
            .style(serenity::ButtonStyle::Secondary)
            .disabled(!has_next);

    vec![serenity::CreateActionRow::Buttons(vec![
        prev_button,
        next_button,
    ])]
}

fn truncate(content: &str, max_chars: usize) -> String {
    let mut truncated = content.chars().take(max_chars).collect::<String>();
    if content.chars().count() > max_chars {
        truncated.push('…');
    }
    truncated
}

use chrono::{DateTime, Utc};
use poise::serenity_prelude as serenity;

use crate::infrastructure::db::{Db, MentionForTarget};
use crate::presentation::entry::util::truncate;
use crate::presentation::{Context, Error};

const PAGE_SIZE: usize = 5;
const UNKNOWN_CHANNEL_CODE: isize = 10003;
const UNKNOWN_MESSAGE_CODE: isize = 10008;

#[poise::command(slash_command, rename = "通知一覧")]
pub async fn main(
    ctx: Context<'_>,
    #[description = "解決済みも表示する"] show_done: Option<bool>,
) -> Result<(), Error> {
    let show_done = show_done.unwrap_or(false);
    let is_ephemeral = ctx.guild_id().is_some();
    let user_id = ctx.author().id;

    let items = fetch_page(
        &ctx.data().db,
        ctx.serenity_context(),
        user_id.get(),
        0,
        show_done,
    )
    .await?;

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
    db: &Db,
    serenity_ctx: &serenity::Context,
    user_id: u64,
    page: usize,
    show_done: bool,
) -> Result<Vec<MentionForTarget>, Error> {
    let mut offset = (page * PAGE_SIZE) as i64;
    let fetch_limit = (PAGE_SIZE + 1) as i64;
    let mut valid_items = Vec::new();

    loop {
        let fetched = db
            .fetch_mentions_for_target(user_id, offset, fetch_limit, show_done)
            .await?;

        if fetched.is_empty() {
            break;
        }

        let fetched_len = fetched.len() as i64;
        offset += fetched_len;

        for item in fetched {
            if should_skip_and_cleanup_item(db, serenity_ctx, user_id, &item).await {
                continue;
            }
            valid_items.push(item);
            if valid_items.len() > PAGE_SIZE {
                return Ok(valid_items);
            }
        }

        if fetched_len < fetch_limit {
            break;
        }
    }

    Ok(valid_items)
}

async fn should_skip_and_cleanup_item(
    db: &Db,
    serenity_ctx: &serenity::Context,
    user_id: u64,
    item: &MentionForTarget,
) -> bool {
    if is_deleted_message_item(serenity_ctx, item).await {
        match db.delete_mention_by_message_id(item.message_id).await {
            Ok(deleted) if deleted > 0 => {
                tracing::info!(
                    "removed stale mention record before display: message_id={}",
                    item.message_id
                );
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(
                    "failed to cleanup stale mention record: message_id={}, err={:?}",
                    item.message_id,
                    err
                );
            }
        }
        return true;
    }

    if item.mention_everyone
        && is_thread_channel(serenity_ctx, item.channel_id).await
        && !is_thread_member(serenity_ctx, item.channel_id, user_id).await
    {
        match db.delete_target_for_user(item.mention_id, user_id).await {
            Ok(deleted) if deleted > 0 => {
                tracing::info!(
                    "removed stale thread target before display: mention_id={}, user_id={}",
                    item.mention_id,
                    user_id
                );
            }
            Ok(_) => {}
            Err(err) => {
                tracing::warn!(
                    "failed to cleanup stale thread target: mention_id={}, user_id={}, err={:?}",
                    item.mention_id,
                    user_id,
                    err
                );
            }
        }
        return true;
    }

    false
}

async fn is_deleted_message_item(
    serenity_ctx: &serenity::Context,
    item: &MentionForTarget,
) -> bool {
    let channel_id = serenity::ChannelId::new(item.channel_id);
    let message_id = serenity::MessageId::new(item.message_id);
    match channel_id.message(serenity_ctx, message_id).await {
        Ok(_) => false,
        Err(err) if is_unknown_message_or_channel_error(&err) => true,
        Err(err) => {
            tracing::warn!(
                "failed to verify message existence before display: channel_id={}, message_id={}, err={:?}",
                item.channel_id,
                item.message_id,
                err
            );
            false
        }
    }
}

async fn is_thread_channel(serenity_ctx: &serenity::Context, channel_id: u64) -> bool {
    let channel_id = serenity::ChannelId::new(channel_id);
    match channel_id.to_channel(serenity_ctx).await {
        Ok(serenity::Channel::Guild(channel)) => matches!(
            channel.kind,
            serenity::ChannelType::PublicThread
                | serenity::ChannelType::PrivateThread
                | serenity::ChannelType::NewsThread
        ),
        Ok(_) => false,
        Err(err) => {
            tracing::warn!(
                "failed to resolve channel kind before display check: channel_id={}, err={:?}",
                channel_id.get(),
                err
            );
            false
        }
    }
}

async fn is_thread_member(serenity_ctx: &serenity::Context, channel_id: u64, user_id: u64) -> bool {
    let channel_id = serenity::ChannelId::new(channel_id);
    match channel_id.get_thread_members(&serenity_ctx.http).await {
        Ok(thread_members) => thread_members
            .iter()
            .any(|member| member.user_id.get() == user_id),
        Err(err) => {
            // 取得失敗時は誤削除を避けるため keep する
            tracing::warn!(
                "failed to fetch thread members before display check: channel_id={}, err={:?}",
                channel_id.get(),
                err
            );
            true
        }
    }
}

fn is_unknown_message_or_channel_error(err: &serenity::Error) -> bool {
    match err {
        serenity::Error::Http(serenity::http::HttpError::UnsuccessfulRequest(resp)) => {
            matches!(resp.error.code, UNKNOWN_CHANNEL_CODE | UNKNOWN_MESSAGE_CODE)
        }
        _ => false,
    }
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

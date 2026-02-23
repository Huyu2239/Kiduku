use std::time::{SystemTime, UNIX_EPOCH};

use poise::serenity_prelude as serenity;

use crate::presentation::entry::slash_commands::my_mentions;
use crate::presentation::Data;

pub async fn handle(ctx: &serenity::Context, data: &Data, comp: &serenity::ComponentInteraction) {
    let id = comp.data.custom_id.as_str();
    if id.starts_with("mm:p:") {
        handle_pagination(ctx, data, comp).await;
    } else if id.starts_with("mm:extend:") {
        handle_extend(ctx, data, comp).await;
    } else if id.starts_with("mm:ignore:") {
        handle_ignore(ctx, data, comp).await;
    }
}

async fn handle_pagination(
    ctx: &serenity::Context,
    data: &Data,
    comp: &serenity::ComponentInteraction,
) {
    // custom_id format: "mm:p:{page}:{show_done_01}:{user_id}"
    let parts: Vec<&str> = comp.data.custom_id.splitn(5, ':').collect();
    if parts.len() != 5 {
        tracing::warn!("unexpected pagination custom_id: {}", comp.data.custom_id);
        return;
    }
    let page: usize = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => return,
    };
    let show_done: bool = parts[3] == "1";
    let owner_user_id: u64 = match parts[4].parse() {
        Ok(v) => v,
        Err(_) => return,
    };

    if comp.user.id.get() != owner_user_id {
        if let Err(err) = comp
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("このボタンはあなた向けではありません。")
                        .ephemeral(true),
                ),
            )
            .await
        {
            tracing::error!("failed to respond to unauthorized pagination: {:?}", err);
        }
        return;
    }

    let items = match my_mentions::fetch_page(&data.db, owner_user_id, page, show_done).await {
        Ok(items) => items,
        Err(err) => {
            tracing::error!("failed to fetch page for pagination: {:?}", err);
            return;
        }
    };

    if items.is_empty() {
        if let Err(err) = comp
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("これ以上のメンションはありません。")
                        .embeds(vec![])
                        .components(vec![]),
                ),
            )
            .await
        {
            tracing::error!("failed to update message for empty page: {:?}", err);
        }
        return;
    }

    let has_next = items.len() > 5;
    let page_items = &items[..items.len().min(5)];
    let guild_id = comp.guild_id;

    let embeds = my_mentions::build_embeds(page_items, guild_id);
    let components =
        my_mentions::build_nav_buttons(page, show_done, owner_user_id, page > 0, has_next);

    if let Err(err) = comp
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embeds(embeds)
                    .components(components),
            ),
        )
        .await
    {
        tracing::error!("failed to update pagination message: {:?}", err);
    }
}

async fn handle_extend(
    ctx: &serenity::Context,
    data: &Data,
    comp: &serenity::ComponentInteraction,
) {
    // custom_id format: "mm:extend:{mention_id}:{user_id}"
    let parts: Vec<&str> = comp.data.custom_id.splitn(4, ':').collect();
    if parts.len() != 4 {
        return;
    }
    let mention_id: i64 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => return,
    };
    let owner_user_id: u64 = match parts[3].parse() {
        Ok(v) => v,
        Err(_) => return,
    };

    if comp.user.id.get() != owner_user_id {
        if let Err(err) = comp
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("このボタンはあなた向けではありません。")
                        .ephemeral(true),
                ),
            )
            .await
        {
            tracing::error!("failed to respond to unauthorized extend: {:?}", err);
        }
        return;
    }

    let now_unix = current_unix_timestamp();
    let extended_until = now_unix + 30 * 24 * 60 * 60;

    if let Err(err) = data
        .db
        .extend_mention_for_user(mention_id, owner_user_id, extended_until)
        .await
    {
        tracing::error!("failed to extend mention: {:?}", err);
        return;
    }

    if let Err(err) = comp
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("1ヶ月延命しました。")
                    .components(vec![]),
            ),
        )
        .await
    {
        tracing::error!("failed to update extend message: {:?}", err);
    }
}

async fn handle_ignore(
    ctx: &serenity::Context,
    data: &Data,
    comp: &serenity::ComponentInteraction,
) {
    // custom_id format: "mm:ignore:{mention_id}:{user_id}"
    let parts: Vec<&str> = comp.data.custom_id.splitn(4, ':').collect();
    if parts.len() != 4 {
        return;
    }
    let mention_id: i64 = match parts[2].parse() {
        Ok(v) => v,
        Err(_) => return,
    };
    let owner_user_id: u64 = match parts[3].parse() {
        Ok(v) => v,
        Err(_) => return,
    };

    if comp.user.id.get() != owner_user_id {
        if let Err(err) = comp
            .create_response(
                &ctx.http,
                serenity::CreateInteractionResponse::Message(
                    serenity::CreateInteractionResponseMessage::new()
                        .content("このボタンはあなた向けではありません。")
                        .ephemeral(true),
                ),
            )
            .await
        {
            tracing::error!("failed to respond to unauthorized ignore: {:?}", err);
        }
        return;
    }

    let now_unix = current_unix_timestamp();

    if let Err(err) = data
        .db
        .ignore_mention_for_user(mention_id, owner_user_id, now_unix)
        .await
    {
        tracing::error!("failed to ignore mention: {:?}", err);
        return;
    }

    if let Err(err) = comp
        .create_response(
            &ctx.http,
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .content("このメンションを無視しました。")
                    .components(vec![]),
            ),
        )
        .await
    {
        tracing::error!("failed to update ignore message: {:?}", err);
    }
}

fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

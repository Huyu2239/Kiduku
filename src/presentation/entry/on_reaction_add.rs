use std::time::{SystemTime, UNIX_EPOCH};

use poise::serenity_prelude as serenity;

use crate::presentation::Data;

const READ_EMOJI: &str = "✅";

pub async fn handle(ctx: &serenity::Context, data: &Data, reaction: &serenity::Reaction) {
    let user_id = match reaction.user_id {
        Some(user_id) => user_id,
        None => return,
    };
    if user_id == ctx.cache.current_user().id {
        return;
    }
    if !is_read_emoji(&reaction.emoji) {
        return;
    }

    let now_unix = match current_unix_timestamp() {
        Ok(ts) => ts,
        Err(err) => {
            tracing::error!("failed to get current time: {:?}", err);
            return;
        }
    };

    if let Err(err) = data
        .db
        .record_read(reaction.message_id.get(), user_id.get(), now_unix)
        .await
    {
        tracing::error!("failed to record read reaction: {:?}", err);
    }
}

fn is_read_emoji(emoji: &serenity::ReactionType) -> bool {
    matches!(emoji, serenity::ReactionType::Unicode(value) if value == READ_EMOJI)
}

fn current_unix_timestamp() -> anyhow::Result<i64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(anyhow::Error::from)?;
    Ok(now.as_secs() as i64)
}

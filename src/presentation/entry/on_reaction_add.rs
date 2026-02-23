use std::time::{SystemTime, UNIX_EPOCH};

use poise::serenity_prelude as serenity;

use crate::presentation::Data;

const KIDOKU_EMOJI_ID: u64 = 1475281418400698633;
const DONE_EMOJI_ID: u64 = 1475281416370524414;

enum ReactionKind {
    Read,
    Done,
}

pub async fn handle(ctx: &serenity::Context, data: &Data, reaction: &serenity::Reaction) {
    let user_id = match reaction.user_id {
        Some(user_id) => user_id,
        None => return,
    };
    if user_id == ctx.cache.current_user().id {
        return;
    }

    let kind = match reaction_kind(&reaction.emoji) {
        Some(kind) => kind,
        None => return,
    };

    let now_unix = match current_unix_timestamp() {
        Ok(ts) => ts,
        Err(err) => {
            tracing::error!("failed to get current time: {:?}", err);
            return;
        }
    };

    let message_id = reaction.message_id.get();
    let user_id_raw = user_id.get();

    match kind {
        ReactionKind::Read => {
            if let Err(err) = data.db.record_read(message_id, user_id_raw, now_unix).await {
                tracing::error!("failed to record read reaction: {:?}", err);
            }
        }
        ReactionKind::Done => {
            if let Err(err) = data.db.record_done(message_id, user_id_raw, now_unix).await {
                tracing::error!("failed to record done reaction: {:?}", err);
            }
        }
    }
}

fn reaction_kind(emoji: &serenity::ReactionType) -> Option<ReactionKind> {
    match emoji {
        serenity::ReactionType::Custom { id, .. } if id.get() == KIDOKU_EMOJI_ID => {
            Some(ReactionKind::Read)
        }
        serenity::ReactionType::Custom { id, .. } if id.get() == DONE_EMOJI_ID => {
            Some(ReactionKind::Done)
        }
        _ => None,
    }
}

fn current_unix_timestamp() -> anyhow::Result<i64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(anyhow::Error::from)?;
    Ok(now.as_secs() as i64)
}

use poise::serenity_prelude as serenity;

use crate::presentation::entry::util::{current_unix_timestamp, DONE_EMOJI_ID, KIDOKU_EMOJI_ID};
use crate::presentation::Data;

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

    let now_unix = current_unix_timestamp();
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

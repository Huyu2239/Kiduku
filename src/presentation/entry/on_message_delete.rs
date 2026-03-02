use poise::serenity_prelude as serenity;

use crate::presentation::Data;

pub async fn handle_single(data: &Data, deleted_message_id: serenity::MessageId) {
    match data
        .db
        .delete_mention_by_message_id(deleted_message_id.get())
        .await
    {
        Ok(0) => {}
        Ok(_) => {
            tracing::info!(
                "deleted mention record for deleted message: {}",
                deleted_message_id.get()
            );
        }
        Err(err) => {
            tracing::error!(
                "failed to delete mention record for message {}: {:?}",
                deleted_message_id.get(),
                err
            );
        }
    }
}

pub async fn handle_bulk(data: &Data, message_ids: &[serenity::MessageId]) {
    if message_ids.is_empty() {
        return;
    }

    let raw_ids = message_ids.iter().map(|id| id.get()).collect::<Vec<_>>();
    match data.db.delete_mentions_by_message_ids(&raw_ids).await {
        Ok(0) => {}
        Ok(deleted) => {
            tracing::info!(
                "deleted {} mention records for bulk message delete ({} messages)",
                deleted,
                raw_ids.len()
            );
        }
        Err(err) => {
            tracing::error!(
                "failed to delete mention records for bulk message delete: {:?}",
                err
            );
        }
    }
}

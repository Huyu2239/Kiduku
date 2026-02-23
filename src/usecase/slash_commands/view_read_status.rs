use serenity::model::prelude::UserId;

use crate::domain::policy::read_status_calc;
use crate::infrastructure::db::StoredMention;

pub struct ViewReadStatusOutput {
    pub guild_id: u64,
    pub channel_id: u64,
    pub message_id: u64,
    pub message_content: String,
    pub author_id: u64,
    pub read_users: Vec<UserId>,
    pub unread_users: Vec<UserId>,
    pub done_users: Vec<UserId>,
}

pub fn execute(mention: StoredMention) -> Option<ViewReadStatusOutput> {
    if mention.target_user_ids.is_empty() {
        return None;
    }

    let targets = mention
        .target_user_ids
        .iter()
        .map(|id| UserId::new(*id))
        .collect::<Vec<_>>();
    let reactions = mention
        .read_user_ids
        .iter()
        .map(|id| UserId::new(*id))
        .collect::<Vec<_>>();
    let (read_users, unread_users) = read_status_calc::calculate_read_status(&targets, &reactions);

    let done_users = {
        let mut done = mention
            .done_user_ids
            .iter()
            .map(|id| UserId::new(*id))
            .collect::<Vec<_>>();
        done.sort_by_key(|id| id.get());
        done.dedup_by_key(|id| id.get());
        done
    };

    Some(ViewReadStatusOutput {
        guild_id: mention.guild_id,
        channel_id: mention.channel_id,
        message_id: mention.message_id,
        message_content: mention.content,
        author_id: mention.author_id,
        read_users,
        unread_users,
        done_users,
    })
}

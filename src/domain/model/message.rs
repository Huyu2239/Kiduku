use serenity::model::prelude::{ChannelId, MessageId, RoleId, UserId};

#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub content: String,
    pub user_mentions: Vec<UserId>,
    pub role_mentions: Vec<RoleId>,
    pub mentions_everyone: bool,
    pub is_reply: bool,
}

impl Message {
    pub fn has_mention(&self) -> bool {
        !self.user_mentions.is_empty() || !self.role_mentions.is_empty() || self.mentions_everyone
    }
}

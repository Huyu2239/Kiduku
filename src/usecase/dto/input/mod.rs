use serenity::model::prelude::{ChannelId, MessageId, RoleId, UserId};

#[derive(Debug, Clone)]
pub struct MessageInput {
    pub content: String,
    pub channel_id: u64,
}

impl MessageInput {
    pub fn new(content: impl Into<String>, channel_id: u64) -> Self {
        Self {
            content: content.into(),
            channel_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageInputDto {
    pub message_id: MessageId,
    pub channel_id: ChannelId,
    pub author_id: UserId,
    pub content: String,
    pub user_mentions: Vec<UserId>,
    pub role_mentions: Vec<RoleId>,
    pub mentions_everyone: bool,
}

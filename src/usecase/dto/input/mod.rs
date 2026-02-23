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

#[derive(Debug, Clone)]
pub struct MessageWithReactionsDto {
    pub message_id: MessageId,
    pub content: String,
    pub user_mentions: Vec<UserId>,
    pub role_mentions: Vec<RoleId>,
    pub expanded_role_member_ids: Vec<UserId>,
    pub mentions_everyone: bool,
    pub everyone_member_ids: Vec<UserId>,
    pub reaction_user_ids: Vec<UserId>,
    pub done_user_ids: Vec<UserId>,
}

#[derive(Debug, Clone)]
pub struct CheckReadsInputDto {
    pub user_id: UserId,
    pub hours: i64,
    pub messages: Vec<MessageWithReactionsDto>,
}

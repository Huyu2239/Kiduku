use serenity::model::prelude::{MessageId, RoleId, UserId};

#[derive(Debug, Clone)]
pub struct AddReadReactionOutputDto {
    pub message_id: MessageId,
    pub should_add_reaction: bool,
}

#[derive(Debug, Clone)]
pub struct CheckReadsOutputDto {
    pub message_id: MessageId,
    pub message_content: String,
    pub mentioned_users: Vec<UserId>,
    pub read_users: Vec<UserId>,
    pub unread_users: Vec<UserId>,
}

#[derive(Debug, Clone)]
pub struct HelpCommandDto {
    pub name: String,
    pub description: String,
    pub example: String,
}

#[derive(Debug, Clone)]
pub struct HelpOutputDto {
    pub title: String,
    pub description: String,
    pub commands: Vec<HelpCommandDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsecaseError {
    NoMentionedUsers,
    NoMessages,
    RoleNotFound(RoleId),
    InvalidHoursParameter,
    InvalidMentionData,
}

impl std::fmt::Display for UsecaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            UsecaseError::NoMentionedUsers => "メンション対象者が見つかりませんでした。",
            UsecaseError::NoMessages => "対象となるメッセージが見つかりませんでした。",
            UsecaseError::RoleNotFound(role_id) => {
                return write!(f, "ロールが見つかりませんでした: {}", role_id.get());
            }
            UsecaseError::InvalidHoursParameter => "hours パラメータが不正です。",
            UsecaseError::InvalidMentionData => "メンション情報が不正です。",
        };
        write!(f, "{message}")
    }
}

impl std::error::Error for UsecaseError {}

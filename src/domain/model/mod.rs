pub mod message;

use serenity::model::prelude::{RoleId, UserId};

pub use message::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MentionType {
    User(UserId),
    Role(RoleId),
    Everyone,
    Here,
}

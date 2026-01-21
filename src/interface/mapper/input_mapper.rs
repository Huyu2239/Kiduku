use serenity::model::prelude::Message;

use crate::usecase::dto::MessageInput;

pub fn from_message(message: &Message) -> MessageInput {
    MessageInput::new(message.content.clone(), message.channel_id.get())
}

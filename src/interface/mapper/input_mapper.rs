use serenity::model::prelude::Message;

use crate::domain::model;
use crate::usecase::dto::{MessageInput, MessageInputDto};

pub fn from_message(message: &Message) -> MessageInput {
    MessageInput::new(message.content.clone(), message.channel_id.get())
}

pub fn from_message_to_message_input_dto(message: &Message) -> MessageInputDto {
    let user_mentions = message.mentions.iter().map(|user| user.id).collect();
    let role_mentions = message.mention_roles.clone();

    MessageInputDto {
        message_id: message.id,
        channel_id: message.channel_id,
        author_id: message.author.id,
        content: message.content.clone(),
        user_mentions,
        role_mentions,
        mentions_everyone: message.mention_everyone,
    }
}

pub fn to_domain_message(input: &MessageInputDto) -> model::Message {
    model::Message {
        id: input.message_id,
        channel_id: input.channel_id,
        author_id: input.author_id,
        content: input.content.clone(),
        user_mentions: input.user_mentions.clone(),
        role_mentions: input.role_mentions.clone(),
        mentions_everyone: input.mentions_everyone,
    }
}

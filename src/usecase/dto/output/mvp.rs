use serenity::model::prelude::MessageId;

#[derive(Debug, Clone)]
pub struct AddReadReactionOutputDto {
    pub message_id: MessageId,
    pub should_add_reaction: bool,
}

#[derive(Debug, Clone)]
pub enum UsecaseError {
    Internal,
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

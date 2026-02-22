use crate::domain::policy::mention_detection;
use crate::interface::mapper::input_mapper;
use crate::usecase::dto::{AddReadReactionOutputDto, MessageInputDto, UsecaseError};

pub fn execute(input: MessageInputDto) -> Result<AddReadReactionOutputDto, UsecaseError> {
    let message = input_mapper::to_domain_message(&input);
    let should_add_reaction = mention_detection::should_add_read_reaction(&message);

    Ok(AddReadReactionOutputDto {
        message_id: input.message_id,
        should_add_reaction,
    })
}

#[cfg(test)]
mod tests {
    use serenity::model::prelude::{ChannelId, MessageId, UserId};

    use super::execute;
    use crate::usecase::dto::MessageInputDto;

    #[test]
    fn returns_true_when_mentions_exist() {
        let input = MessageInputDto {
            message_id: MessageId::new(1),
            channel_id: ChannelId::new(1),
            author_id: UserId::new(1),
            content: "<@2> ping".into(),
            user_mentions: vec![UserId::new(2)],
            role_mentions: Vec::new(),
            mentions_everyone: false,
        };

        let output = execute(input).expect("usecase should succeed");
        assert!(output.should_add_reaction);
    }

    #[test]
    fn returns_false_when_no_mentions_exist() {
        let input = MessageInputDto {
            message_id: MessageId::new(1),
            channel_id: ChannelId::new(1),
            author_id: UserId::new(1),
            content: "hello".into(),
            user_mentions: Vec::new(),
            role_mentions: Vec::new(),
            mentions_everyone: false,
        };

        let output = execute(input).expect("usecase should succeed");
        assert!(!output.should_add_reaction);
    }
}

use serenity::model::prelude::UserId;

use crate::domain::policy::read_status_calc;
use crate::usecase::dto::{CheckReadsInputDto, CheckReadsOutputDto, UsecaseError};

pub fn execute(input: CheckReadsInputDto) -> Result<Vec<CheckReadsOutputDto>, UsecaseError> {
    if input.hours <= 0 {
        return Err(UsecaseError::InvalidHoursParameter);
    }
    if input.messages.is_empty() {
        return Err(UsecaseError::NoMessages);
    }

    let mut outputs = Vec::new();

    for message in input.messages {
        let mentioned_users = collect_mentioned_users(&message)?;
        if mentioned_users.is_empty() {
            continue;
        }

        let (read_users, unread_users) =
            read_status_calc::calculate_read_status(&mentioned_users, &message.reaction_user_ids);

        let done_users = dedup_sorted(message.done_user_ids.clone());

        outputs.push(CheckReadsOutputDto {
            message_id: message.message_id,
            message_content: message.content,
            mentioned_users,
            read_users,
            unread_users,
            done_users,
        });
    }

    if outputs.is_empty() {
        return Err(UsecaseError::NoMentionedUsers);
    }

    Ok(outputs)
}

fn collect_mentioned_users(
    message: &crate::usecase::dto::MessageWithReactionsDto,
) -> Result<Vec<UserId>, UsecaseError> {
    if message.mentions_everyone && message.everyone_member_ids.is_empty() {
        return Err(UsecaseError::InvalidMentionData);
    }

    let mut mentioned = Vec::new();
    mentioned.extend(message.user_mentions.iter().copied());
    mentioned.extend(message.expanded_role_member_ids.iter().copied());
    if message.mentions_everyone {
        mentioned.extend(message.everyone_member_ids.iter().copied());
    }

    Ok(dedup_sorted(mentioned))
}

fn dedup_sorted(mut users: Vec<UserId>) -> Vec<UserId> {
    users.sort_by_key(|id| id.get());
    users.dedup_by_key(|id| id.get());
    users
}

#[cfg(test)]
mod tests {
    use serenity::model::prelude::{MessageId, RoleId, UserId};

    use super::execute;
    use crate::usecase::dto::{CheckReadsInputDto, MessageWithReactionsDto};

    #[test]
    fn calculates_unread_users_from_mentions_and_reactions() {
        let message = MessageWithReactionsDto {
            message_id: MessageId::new(1),
            content: "ping".into(),
            user_mentions: vec![UserId::new(1), UserId::new(2)],
            role_mentions: vec![RoleId::new(10)],
            expanded_role_member_ids: vec![UserId::new(3)],
            mentions_everyone: false,
            everyone_member_ids: Vec::new(),
            reaction_user_ids: vec![UserId::new(2), UserId::new(3)],
            done_user_ids: Vec::new(),
        };

        let input = CheckReadsInputDto {
            user_id: UserId::new(99),
            hours: 24,
            messages: vec![message],
        };

        let outputs = execute(input).expect("usecase should succeed");
        let output = &outputs[0];

        assert_eq!(output.mentioned_users.len(), 3);
        assert_eq!(output.read_users, vec![UserId::new(2), UserId::new(3)]);
        assert_eq!(output.unread_users, vec![UserId::new(1)]);
    }

    #[test]
    fn rejects_invalid_hours() {
        let input = CheckReadsInputDto {
            user_id: UserId::new(1),
            hours: 0,
            messages: vec![MessageWithReactionsDto {
                message_id: MessageId::new(1),
                content: String::new(),
                user_mentions: Vec::new(),
                role_mentions: Vec::new(),
                expanded_role_member_ids: Vec::new(),
                mentions_everyone: false,
                everyone_member_ids: Vec::new(),
                reaction_user_ids: Vec::new(),
                done_user_ids: Vec::new(),
            }],
        };

        let err = execute(input).expect_err("expected validation error");
        assert_eq!(
            err,
            crate::usecase::dto::UsecaseError::InvalidHoursParameter
        );
    }
}

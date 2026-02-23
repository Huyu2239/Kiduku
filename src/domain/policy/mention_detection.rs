use crate::domain::model::{MentionType, Message};

pub fn should_add_read_reaction(message: &Message) -> bool {
    if message.is_reply {
        message.has_explicit_mention_in_content()
    } else {
        message.has_mention()
    }
}

pub fn extract_mentions(message: &Message) -> Vec<MentionType> {
    let mut mentions = Vec::new();

    for user_id in &message.user_mentions {
        mentions.push(MentionType::User(*user_id));
    }

    for role_id in &message.role_mentions {
        mentions.push(MentionType::Role(*role_id));
    }

    if message.mentions_everyone {
        let content_lower = message.content.to_lowercase();
        if content_lower.contains("@here") {
            mentions.push(MentionType::Here);
        }
        if content_lower.contains("@everyone") {
            mentions.push(MentionType::Everyone);
        }
        if !content_lower.contains("@here") && !content_lower.contains("@everyone") {
            mentions.push(MentionType::Everyone);
        }
    }

    mentions
}

#[cfg(test)]
mod tests {
    use serenity::model::prelude::{ChannelId, MessageId, RoleId, UserId};

    use super::{extract_mentions, should_add_read_reaction};
    use crate::domain::model::{MentionType, Message};

    fn base_message() -> Message {
        Message {
            id: MessageId::new(1),
            channel_id: ChannelId::new(1),
            author_id: UserId::new(1),
            content: String::new(),
            user_mentions: Vec::new(),
            role_mentions: Vec::new(),
            mentions_everyone: false,
            is_reply: false,
        }
    }

    #[test]
    fn skips_reply_with_only_auto_mention() {
        // オートメンションは content に <@> が入らないため反応しない
        let mut message = base_message();
        message.user_mentions = vec![UserId::new(10)];
        message.is_reply = true;

        assert!(!should_add_read_reaction(&message));
    }

    #[test]
    fn reacts_to_reply_with_explicit_text_mention() {
        // テキストに明示的に <@ID> を書いた場合は反応する
        let mut message = base_message();
        message.user_mentions = vec![UserId::new(10)];
        message.content = "<@10> 確認お願いします".into();
        message.is_reply = true;

        assert!(should_add_read_reaction(&message));
    }

    #[test]
    fn detects_user_mentions() {
        let mut message = base_message();
        message.user_mentions = vec![UserId::new(10)];

        assert!(should_add_read_reaction(&message));
        assert_eq!(
            extract_mentions(&message),
            vec![MentionType::User(UserId::new(10))]
        );
    }

    #[test]
    fn detects_role_mentions() {
        let mut message = base_message();
        message.role_mentions = vec![RoleId::new(20)];

        assert!(should_add_read_reaction(&message));
        assert_eq!(
            extract_mentions(&message),
            vec![MentionType::Role(RoleId::new(20))]
        );
    }

    #[test]
    fn detects_everyone_and_here_mentions() {
        let mut message = base_message();
        message.mentions_everyone = true;
        message.content = "@everyone @here".into();

        let mentions = extract_mentions(&message);
        assert!(mentions.contains(&MentionType::Everyone));
        assert!(mentions.contains(&MentionType::Here));
    }
}

use poise::serenity_prelude as serenity;
use serenity::model::prelude::UserId;

use crate::presentation::entry::util::truncate;
use crate::presentation::{Context, Error};
use crate::usecase::slash_commands::view_read_status as view_read_status_usecase;

const EMBED_FIELD_VALUE_MAX_LEN: usize = 1024;

#[poise::command(context_menu_command = "既読状況確認")]
pub async fn main(ctx: Context<'_>, msg: serenity::Message) -> Result<(), Error> {
    let mention = ctx
        .data()
        .db
        .fetch_mention_by_message_id(msg.id.get())
        .await?;

    let mention = match mention {
        Some(m) => m,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("このメッセージは記録されていません。メンションが含まれていないか、Bot起動前に送信された可能性があります。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let output = match view_read_status_usecase::execute(mention) {
        Some(o) => o,
        None => {
            ctx.send(
                poise::CreateReply::default()
                    .content("このメッセージにはメンション対象者が記録されていません。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let message_link = format!(
        "https://discord.com/channels/{}/{}/{}",
        output.guild_id, output.channel_id, output.message_id
    );
    let snippet = truncate(&output.message_content, 100);
    let total = output.read_users.len() + output.unread_users.len();
    let read_count = output.read_users.len();
    let percent = if total == 0 {
        0
    } else {
        (read_count * 100) / total
    };

    let read_summary = format!("{}/{} ({}%)", read_count, total, percent);
    let read_users_text = format_user_mentions_limited(
        &output.read_users,
        available_chars_for_read_users(&read_summary),
    );

    let embed = serenity::CreateEmbed::new()
        .title("既読状況確認")
        .description(format!("[メッセージを開く]({})", message_link))
        .field("内容", snippet, false)
        .field(
            "既読",
            format!("{}\n{}", read_summary, read_users_text),
            false,
        )
        .field(
            "未読",
            format_user_mentions_limited(&output.unread_users, EMBED_FIELD_VALUE_MAX_LEN),
            false,
        )
        .field(
            "解決済み",
            format_user_mentions_limited(&output.done_users, EMBED_FIELD_VALUE_MAX_LEN),
            false,
        );

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

fn available_chars_for_read_users(read_summary: &str) -> usize {
    let used = read_summary.chars().count() + 1;
    EMBED_FIELD_VALUE_MAX_LEN.saturating_sub(used)
}

fn format_user_mentions_limited(users: &[UserId], max_chars: usize) -> String {
    if users.is_empty() {
        return "なし".into();
    }

    let mentions = users
        .iter()
        .map(|id| format!("<@{}>", id.get()))
        .collect::<Vec<_>>();

    let mut body = String::new();
    for (index, mention) in mentions.iter().enumerate() {
        let separator = if body.is_empty() { "" } else { " " };
        let remaining_after = mentions.len().saturating_sub(index + 1);
        let predicted = format!("{}{}{}", body, separator, mention);
        if remaining_after == 0 {
            if predicted.chars().count() <= max_chars {
                body = predicted;
                return body;
            }
            append_omitted_suffix(&mut body, mentions.len().saturating_sub(index), max_chars);
            return body;
        }

        let suffix_if_truncated = format!(" …他{}人", remaining_after);
        if predicted.chars().count() + suffix_if_truncated.chars().count() <= max_chars {
            body = predicted;
            continue;
        }

        append_omitted_suffix(&mut body, mentions.len().saturating_sub(index), max_chars);
        return body;
    }

    if body.is_empty() {
        "なし".into()
    } else {
        body
    }
}

fn append_omitted_suffix(body: &mut String, omitted_count: usize, max_chars: usize) {
    if omitted_count == 0 {
        return;
    }
    let suffix = format!(" …他{}人", omitted_count);
    if body.chars().count() + suffix.chars().count() <= max_chars {
        body.push_str(&suffix);
        return;
    }

    let allowed_body_len = max_chars.saturating_sub(suffix.chars().count());
    let trimmed = body.chars().take(allowed_body_len).collect::<String>();
    *body = trimmed;
    body.push_str(&suffix);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn users(count: usize) -> Vec<UserId> {
        (1..=count as u64).map(UserId::new).collect()
    }

    #[test]
    fn returns_none_label_for_empty_users() {
        assert_eq!(format_user_mentions_limited(&[], 1024), "なし");
    }

    #[test]
    fn keeps_full_list_when_under_limit() {
        let text = format_user_mentions_limited(&users(3), 1024);
        assert_eq!(text, "<@1> <@2> <@3>");
    }

    #[test]
    fn truncates_with_omitted_count_when_over_limit() {
        let text = format_user_mentions_limited(&users(200), 80);
        assert!(text.contains("他"));
        assert!(text.chars().count() <= 80);
    }

    #[test]
    fn read_field_fits_embed_limit_for_many_users() {
        let summary = "123/456 (27%)";
        let users_text =
            format_user_mentions_limited(&users(500), available_chars_for_read_users(summary));
        let field = format!("{summary}\n{users_text}");
        assert!(field.chars().count() <= EMBED_FIELD_VALUE_MAX_LEN);
    }
}

use poise::serenity_prelude as serenity;
use serenity::model::prelude::UserId;

use crate::presentation::{Context, Error};
use crate::usecase::slash_commands::view_read_status as view_read_status_usecase;

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

    let embed = serenity::CreateEmbed::new()
        .title("既読状況確認")
        .description(format!("[メッセージを開く]({})", message_link))
        .field("内容", snippet, false)
        .field(
            "既読",
            format!(
                "{}/{} ({}%)\n{}",
                read_count,
                total,
                percent,
                format_user_mentions(&output.read_users)
            ),
            false,
        )
        .field("未読", format_user_mentions(&output.unread_users), false)
        .field("解決済み", format_user_mentions(&output.done_users), false);

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

fn truncate(content: &str, max_chars: usize) -> String {
    let mut truncated = content.chars().take(max_chars).collect::<String>();
    if content.chars().count() > max_chars {
        truncated.push('…');
    }
    truncated
}

fn format_user_mentions(users: &[UserId]) -> String {
    if users.is_empty() {
        return "なし".into();
    }
    users
        .iter()
        .map(|id| format!("<@{}>", id.get()))
        .collect::<Vec<_>>()
        .join(" ")
}

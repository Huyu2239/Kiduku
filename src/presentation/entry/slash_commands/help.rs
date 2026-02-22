use crate::presentation::{Context, Error};
use crate::usecase::slash_commands::help as help_usecase;

#[poise::command(slash_command, rename = "help")]
pub async fn main(ctx: Context<'_>) -> Result<(), Error> {
    let help = match help_usecase::execute() {
        Ok(help) => help,
        Err(err) => {
            tracing::error!("usecase error: {:?}", err);
            ctx.send(
                poise::CreateReply::default()
                    .content("ヘルプの生成に失敗しました。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };

    let mut embed = poise::serenity_prelude::CreateEmbed::new()
        .title(help.title)
        .description(help.description);
    for command in help.commands {
        let value = format!("{}\n例: `{}`", command.description, command.example);
        embed = embed.field(command.name, value, false);
    }

    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true))
        .await?;
    Ok(())
}

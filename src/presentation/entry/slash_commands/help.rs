use crate::presentation::discord_exec;
use crate::presentation::{Context, Error};
use crate::usecase::slash_commands::help as help_usecase;

#[poise::command(slash_command, rename = "help")]
pub async fn main(ctx: Context<'_>) -> Result<(), Error> {
    let plan = match help_usecase::execute() {
        Ok(plan) => plan,
        Err(err) => {
            tracing::error!("usecase error: {:?}", err);
            ctx.send(
                poise::CreateReply::default()
                    .content("不正なリクエストのため処理できません。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
    };
    discord_exec::execute_from_interaction(ctx, plan).await?;
    Ok(())
}

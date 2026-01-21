use poise::serenity_prelude as serenity;

use crate::interface::mapper::input_mapper;
use crate::presentation::discord_exec;
use crate::presentation::entry::on_error;
use crate::usecase::on_message::greeting;

pub async fn handle(ctx: &serenity::Context, message: &serenity::Message) {
    if message.author.bot {
        return;
    }

    let input = input_mapper::from_message(message);
    let plan = match greeting::execute(input) {
        Ok(plan) => plan,
        Err(err) => {
            tracing::error!("usecase error: {:?}", err);
            return;
        }
    };

    if let Err(err) = discord_exec::execute(ctx, plan).await {
        on_error::handle_exec_error(err);
    }
}

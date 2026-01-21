use anyhow::Context as _;
use serenity::prelude::*;

use kiduku::infrastructure::config::{set_dev_mode, AppConfig};
use kiduku::presentation::build_framework;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    set_dev_mode(is_cargo_run());
    let config = AppConfig::load()?;
    let AppConfig {
        discord_bot_token,
        env_filter,
        dev_mode,
    } = config;
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(dev_mode)
        .with_file(dev_mode)
        .with_line_number(dev_mode)
        .init();

    let intents =
        GatewayIntents::GUILDS | GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let framework = build_framework();
    let mut client = Client::builder(discord_bot_token, intents)
        .framework(framework)
        .await
        .context("failed to create Discord client")?;

    if let Err(err) = client.start().await {
        tracing::error!("client error: {:?}", err);
        return Err(err).context("failed to start Discord client");
    }

    Ok(())
}

fn is_cargo_run() -> bool {
    std::env::var("CARGO").is_ok() && std::env::var("CARGO_BIN_NAME").is_ok()
}

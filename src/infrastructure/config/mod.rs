use std::env;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Context as _;
use tracing_subscriber::EnvFilter;

#[derive(Debug)]
pub struct AppConfig {
    pub discord_bot_token: String,
    pub env_filter: EnvFilter,
    pub dev_mode: bool,
}

static DEV_MODE: AtomicBool = AtomicBool::new(false);

pub fn set_dev_mode(enabled: bool) {
    DEV_MODE.store(enabled, Ordering::Relaxed);
}

fn dev_mode() -> bool {
    DEV_MODE.load(Ordering::Relaxed)
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let dev_mode = dev_mode();
        let env_filter = build_env_filter(dev_mode);
        let discord_bot_token =
            env::var("DISCORD_BOT_TOKEN").context("DISCORD_BOT_TOKEN is not set")?;
        Ok(Self {
            discord_bot_token,
            env_filter,
            dev_mode,
        })
    }
}

fn build_env_filter(dev_mode: bool) -> EnvFilter {
    if let Ok(level) = env::var("LOG_LEVEL") {
        return EnvFilter::try_new(level).unwrap_or_else(|err| {
            eprintln!("invalid LOG_LEVEL; falling back to default: {err}");
            default_env_filter(dev_mode)
        });
    }

    if let Ok(filter) = EnvFilter::try_from_default_env() {
        return filter;
    }

    default_env_filter(dev_mode)
}

fn default_env_filter(dev_mode: bool) -> EnvFilter {
    let level = if dev_mode { "debug" } else { "info" };
    EnvFilter::new(level)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_env_filter_dev_mode() {
        // dev_mode=trueの場合、debugレベルでフィルタが作成される
        // EnvFilterの内部構造はテストできないため、
        // パニックしないことのみ確認
        let _filter = default_env_filter(true);
    }

    #[test]
    fn test_default_env_filter_production_mode() {
        // dev_mode=falseの場合、infoレベルでフィルタが作成される
        // EnvFilterの内部構造はテストできないため、
        // パニックしないことのみ確認
        let _filter = default_env_filter(false);
    }
}

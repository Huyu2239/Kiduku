use std::time::{SystemTime, UNIX_EPOCH};

pub const KIDOKU_EMOJI_ID: u64 = 1475281418400698633;
pub const KIDOKU_EMOJI_NAME: &str = "KIDOKU";
pub const DONE_EMOJI_ID: u64 = 1475281416370524414;
pub const DONE_EMOJI_NAME: &str = "DONE";

pub fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

pub fn truncate(content: &str, max_chars: usize) -> String {
    let mut truncated = content.chars().take(max_chars).collect::<String>();
    if content.chars().count() > max_chars {
        truncated.push('…');
    }
    truncated
}

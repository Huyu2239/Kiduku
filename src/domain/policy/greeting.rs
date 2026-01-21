pub const GREETING_TEXT: &str = "おはよう";

pub fn is_greeting(content: &str) -> bool {
    content.trim() == GREETING_TEXT
}

#[cfg(test)]
mod tests {
    use super::{is_greeting, GREETING_TEXT};

    #[test]
    fn matches_exact_greeting() {
        assert!(is_greeting(GREETING_TEXT));
    }

    #[test]
    fn trims_whitespace() {
        let with_spaces = format!("  {GREETING_TEXT}  ");
        assert!(is_greeting(&with_spaces));
    }

    #[test]
    fn rejects_other_text() {
        assert!(!is_greeting("こんにちは"));
    }
}

#[derive(Debug, Clone)]
pub struct MessageInput {
    pub content: String,
    pub channel_id: u64,
}

impl MessageInput {
    pub fn new(content: impl Into<String>, channel_id: u64) -> Self {
        Self {
            content: content.into(),
            channel_id,
        }
    }
}

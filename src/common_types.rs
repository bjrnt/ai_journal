use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct JournalMessage {
    pub from_bot: bool,
    pub text: String,
    pub timestamp: DateTime<Utc>,
}

impl JournalMessage {
    pub fn new(text: String, from_bot: bool) -> Self {
        JournalMessage {
            from_bot,
            text,
            timestamp: Utc::now(),
        }
    }
}

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

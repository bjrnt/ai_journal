use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct JournalMessage {
    pub from_bot: bool,
    pub text: String,
    pub timestamp: DateTime<Utc>,
}

pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

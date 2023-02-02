use chrono::Utc;
use log::info;
use serde::{Deserialize, Serialize};

use crate::common_types::JournalMessage;

type Response = Result<JournalMessage, Box<dyn std::error::Error + Send + Sync>>;

const BASE_PROMPT: &str = "You are Socrates. Please help me with an issue in my life. Please ask me questions to try to understand what my issue is and help me unpack it. You can start the conversation however you feel best.\nYou:";

#[derive(Debug, Serialize)]
struct RequestData {
    model: String,
    prompt: String,
    temperature: f32,
    max_tokens: u32,
    top_p: u32,
    frequency_penalty: f32,
    prescence_penalty: f32,
    stop: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    choices: Vec<Choice>,
    usage: UsageData,
}

#[derive(Debug, Deserialize)]
struct UsageData {
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct Choice {
    text: String,
}

#[derive(Clone)]
pub struct OpenAiApi {
    api_token: String,
}

impl OpenAiApi {
    pub fn new(api_token: String) -> Self {
        OpenAiApi { api_token }
    }

    pub async fn begin(self) -> Response {
        self.complete(BASE_PROMPT).await
    }

    pub async fn complete_next(self, msgs: &[JournalMessage]) -> Response {
        let prompt = convert_to_prompt_format(msgs);
        self.complete(&prompt).await
    }

    async fn complete(self, prompt: &str) -> Response {
        let data = serde_json::json!({
            "model": "text-davinci-003",
            "prompt": prompt,
            "temperature": 0.9,
            "max_tokens": 150,
            "top_p": 1,
            "frequency_penalty": 0.0,
            "presence_penalty": 0.6,
            "stop": vec!["You:", "Me:"],
        });
        let resp = reqwest::Client::new()
            .post("https://api.openai.com/v1/completions")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&data)
            .send()
            .await?
            .json::<ResponseData>()
            .await?;
        let message = resp
            .choices
            .first()
            .ok_or_else(|| format!("No choice in Open AI reply: {:?}", resp))?;

        info!(
            "Received completion (total {} tokens =~ ${}): '{}'",
            resp.usage.total_tokens,
            approximate_cost(resp.usage.total_tokens),
            message.text
        );

        let text = message
            .text
            .trim_end_matches("You: ")
            .trim_end_matches("Me: ");

        Ok(JournalMessage {
            from_bot: true,
            text: text.to_owned(),
            timestamp: Utc::now(),
        })
    }
}

fn convert_to_prompt_format(msgs: &[JournalMessage]) -> String {
    let mut prompt = String::new();
    prompt.push_str(BASE_PROMPT.trim_end_matches("You: "));
    for msg in msgs.iter() {
        let speaker = if msg.from_bot { "You: " } else { "Me: " };
        prompt.push_str(speaker);
        prompt.push_str(&msg.text);
        prompt.push('\n');
    }
    prompt.push_str("You:");
    prompt
}

fn approximate_cost(num_tokens: u32) -> f32 {
    num_tokens as f32 * 0.00002
}

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

fn approximate_cost(num_tokens: u32) -> f32 {
    num_tokens as f32 * 0.00002
}

async fn complete_prompt(prompt: &str) -> Response {
    info!("Completing prompt: '{}'", prompt);

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
    let open_ai_api_token =
        std::env::var("OPEN_AI_API_TOKEN").expect("OPEN_AI_API_TOKEN must be set");
    let resp = reqwest::Client::new()
        .post("https://api.openai.com/v1/completions")
        .header("Authorization", format!("Bearer {open_ai_api_token}"))
        .json(&data)
        .send()
        .await?
        .json::<ResponseData>()
        .await?;
    let message = resp.choices.first().expect("no text message back");

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

fn convert_to_prompt_format(msgs: &Vec<JournalMessage>) -> String {
    let mut prompt = String::new();
    prompt.push_str(BASE_PROMPT.trim_end_matches("You: "));
    for msg in msgs.iter() {
        let speaker = if msg.from_bot { "You: " } else { "Me: " };
        prompt.push_str(speaker);
        prompt.push_str(&msg.text);
        prompt.push_str("\n");
    }
    prompt.push_str("You:");
    prompt
}

pub async fn begin() -> Response {
    complete_prompt(BASE_PROMPT).await
}

pub async fn complete_next(msgs: &Vec<JournalMessage>) -> Response {
    let prompt = convert_to_prompt_format(msgs);
    complete_prompt(&prompt).await
}

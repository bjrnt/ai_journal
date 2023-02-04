use std::borrow::Cow;
use tiktoken_rs::tiktoken::{p50k_base, CoreBPE};

use log::info;
use serde::{Deserialize, Serialize};

use crate::common_types::JournalMessage;

type Response = Result<JournalMessage, Box<dyn std::error::Error + Send + Sync>>;

const MAX_PROMPT_TOKENS: usize = 2048;
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

    pub async fn complete_next(&self, msgs: &[JournalMessage]) -> Response {
        let prompt = self.prompt(msgs)?;
        self.complete(&prompt).await
    }

    fn prompt(
        &self,
        msgs: &[JournalMessage],
    ) -> Result<Cow<'static, str>, Box<dyn std::error::Error + Send + Sync>> {
        lazy_static! {
            static ref BPE: CoreBPE = p50k_base().unwrap();
        }

        for skip_n_messages in (0..msgs.len()).step_by(5) {
            let prompt = convert_to_prompt_format(&msgs[skip_n_messages..]);
            if BPE.encode_with_special_tokens(&prompt).len() > MAX_PROMPT_TOKENS {
                continue;
            }
            return Ok(prompt);
        }
        Err("could not find a prompt with valid length under max tokens".into()).into()
    }

    async fn complete(&self, prompt: &str) -> Response {
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
            .trim_start()
            .trim_end_matches("You: ")
            .trim_end_matches("Me: ")
            .trim_end();

        Ok(JournalMessage::new(text.into(), true))
    }
}

fn convert_to_prompt_format(msgs: &[JournalMessage]) -> Cow<'static, str> {
    let prompt = BASE_PROMPT.trim_end_matches("You:");
    let messages: String = msgs
        .iter()
        .map(|msg| {
            format!(
                "{}: {}\n",
                if msg.from_bot { "You" } else { "Me" },
                msg.text
            )
        })
        .collect();
    format!("{}{}You:", prompt, messages).into()
}

fn approximate_cost(num_tokens: u32) -> f32 {
    num_tokens as f32 * 0.00002
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_correct_prompt() {
        let out = convert_to_prompt_format(&[
            JournalMessage::new("Hello".into(), true),
            JournalMessage::new("What's up?".into(), false),
        ]);
        assert!(out.starts_with(BASE_PROMPT));
        assert!(out.ends_with("You: Hello\nMe: What's up?\nYou:"));
    }
}

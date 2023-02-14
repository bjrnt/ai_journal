use log::*;
use serde::{Deserialize, Serialize};
use teloxide::{dispatching::dialogue::ErasedStorage, prelude::*};

use crate::{
    common_types::{HandlerResult, JournalMessage},
    open_ai::OpenAiApi,
};

pub type BotDialogue = Dialogue<State, ErasedStorage<State>>;
pub type DialogueStorage = std::sync::Arc<ErasedStorage<State>>;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub enum State {
    #[default]
    Start,
    Chatting {
        messages: Vec<JournalMessage>,
    },
}

pub async fn start(
    bot: Bot,
    dialogue: BotDialogue,
    open_ai_api: OpenAiApi,
    msg: Message,
) -> HandlerResult {
    let message = open_ai_api.begin().await?;
    info!("Starting chat: '{}'", message.text);

    bot.send_message(msg.chat.id, message.text.clone()).await?;
    dialogue
        .update(State::Chatting {
            messages: vec![message],
        })
        .await?;
    Ok(())
}

pub async fn chatting(
    bot: Bot,
    dialogue: BotDialogue,
    messages: Vec<JournalMessage>,
    open_ai_api: OpenAiApi,
    msg: Message,
) -> HandlerResult {
    let mut next_messages = messages.clone();
    let Some(user_text) = msg.text() else {
        warn!("Received irregular message: {:?}", msg);
        return Ok(())
    };

    info!(
        "Received message from {}: '{}'",
        msg.chat.username().unwrap_or("<unknown>"),
        user_text
    );
    next_messages.push(JournalMessage {
        from_bot: false,
        text: user_text.trim_start().trim_end().to_owned(),
        timestamp: msg.date,
    });

    let next_message = open_ai_api.complete_next(&next_messages).await?;
    next_messages.push(next_message.clone());

    info!(
        "Replied to {}: '{}'",
        msg.chat.username().unwrap_or("<unknown>"),
        next_message.text
    );

    bot.send_message(msg.chat.id, next_message.text).await?;
    dialogue
        .update(State::Chatting {
            messages: next_messages,
        })
        .await?;
    Ok(())
}

use dialogue::DialogueStorage;
use dotenv::dotenv;
use log::warn;
use teloxide::{
    dispatching::dialogue::{serializer::Json, ErasedStorage, SqliteStorage, Storage},
    prelude::*,
};

#[macro_use]
extern crate lazy_static;

mod common_types;

use crate::{dialogue::State, open_ai::OpenAiApi};

mod commands;
mod dialogue;
mod open_ai;

fn username_allowlist_filter(allowlist: &[String], msg: Message) -> bool {
    msg.from()
        .and_then(|user| user.username.as_ref())
        .map(|username| allowlist.contains(username))
        .unwrap_or_default()
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let telegram_api_token =
        std::env::var("TELEGRAM_API_TOKEN").expect("TELEGRAM_API_TOKEN must be set");

    let open_ai_api_token =
        std::env::var("OPEN_AI_API_TOKEN").expect("OPEN_AI_API_TOKEN must be set");

    let username_allowlist: Vec<String> = std::env::var("USERNAME_ALLOWLIST")
        .expect("USERNAME_ALLOWLIST must be set")
        .split(',')
        .map(|v| v.to_owned())
        .collect();
    assert!(
        !username_allowlist.is_empty(),
        "USERNAME_ALLOWLIST must contain at least one username"
    );

    let db_path = std::env::var("DB_PATH").expect("DB_PATH must be set");

    let storage: DialogueStorage = SqliteStorage::open(&db_path, Json).await.unwrap().erase();

    let bot = Bot::new(telegram_api_token);

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .filter(move |msg: Message| username_allowlist_filter(&username_allowlist, msg))
            .enter_dialogue::<Message, ErasedStorage<dialogue::State>, dialogue::State>()
            .branch(
                dptree::entry()
                    .filter_command::<commands::Command>()
                    .endpoint(commands::handler),
            )
            .branch(
                dptree::entry()
                    .branch(dptree::case![State::Start].endpoint(dialogue::start))
                    .branch(
                        dptree::case![State::Chatting { messages }].endpoint(dialogue::chatting),
                    ),
            ),
    )
    .dependencies(dptree::deps![OpenAiApi::new(open_ai_api_token), storage])
    .default_handler(|upd| async move {
        warn!("Unhandled update: {:?}", upd);
    })
    .error_handler(LoggingErrorHandler::with_custom_text(
        "An error has occurred in the dispatcher",
    ))
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

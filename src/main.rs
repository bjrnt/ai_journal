use dotenv::dotenv;
use log::warn;
use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

mod common_types;

use crate::dialogue::State;

mod commands;
mod dialogue;
mod open_ai;

const USERNAME_ALLOWLIST: [&str; 1] = ["bjrnt"];

fn username_allowlist_filter(msg: Message) -> bool {
    msg.from()
        .and_then(|user| user.username.as_ref())
        .map(|username| USERNAME_ALLOWLIST.contains(&username.as_str()))
        .unwrap_or_default()
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let telegram_api_token =
        std::env::var("TELEGRAM_API_TOKEN").expect("TELEGRAM_API_TOKEN must be set");
    let bot = Bot::new(telegram_api_token);

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .filter(username_allowlist_filter)
            .enter_dialogue::<Message, InMemStorage<dialogue::State>, dialogue::State>()
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
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
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

use regex::Regex;
use teloxide::{prelude::*, types::InputFile, utils::command::BotCommands};

use crate::{
    common_types::{HandlerResult, JournalMessage},
    dialogue::{BotDialogue, State},
};

#[derive(BotCommands, Clone)]
#[command(
    rename_rule = "lowercase",
    description = "The following commands are supported: "
)]
pub enum Command {
    #[command(description = "show this text")]
    Help,
    #[command(description = "export this session's chat history so far")]
    Export,
    #[command(
        description = "end your current conversation (start a new one by sending any message)"
    )]
    End,
}

pub async fn handler(bot: Bot, msg: Message, dialogue: BotDialogue, cmd: Command) -> HandlerResult {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Export => {
            let current_state = dialogue.get_or_default().await?;
            export_handler(bot, msg, current_state).await?;
        }
        Command::End => {
            dialogue.reset().await?;
            bot.send_message(msg.chat.id, "Our conversation has ended. Thank you for taking the time to talk with me today! Type /start to begin a new conversation at a later time.").await?;
        }
    };
    Ok(())
}

async fn export_handler(bot: Bot, msg: Message, state: State) -> HandlerResult {
    match state {
        State::Start => {
            bot.send_message(msg.chat.id, "There aren't any messages to export yet. Try chatting with me for a while before running this command!").await?;
        }
        State::Chatting { messages } => {
            bot.send_message(
                msg.chat.id,
                "Exporting our current conversation so far. Give me one second!",
            )
            .await?;

            let file_content = messages_to_export_format(&messages);
            let file = InputFile::memory(file_content).file_name("SocratesJournalBot.txt");
            bot.send_document(msg.chat.id, file).await?;
        }
    };
    Ok(())
}

fn messages_to_export_format(msgs: &[JournalMessage]) -> String {
    lazy_static! {
        static ref NEWLINE_COLLAPSING_RE: Regex = Regex::new("\n{3,}").unwrap();
    }

    let exported: String = msgs
        .iter()
        .map(|msg| {
            format!(
                "{}: {}\n\n",
                if msg.from_bot { "Socrates" } else { "Me" },
                msg.text
            )
        })
        .collect();
    NEWLINE_COLLAPSING_RE
        .replace(exported.trim_start().trim_end(), "\n\n")
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_collapses_newlines() {
        let out = messages_to_export_format(&[
            JournalMessage::new("Hello\n\n\n\n\n".into(), true),
            JournalMessage::new("What's up?\n".into(), false),
        ]);
        let expected = "Socrates: Hello\n\nMe: What's up?";
        assert_eq!(out, expected)
    }
}

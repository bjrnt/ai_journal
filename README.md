# AI Journal

AI journal is a Telegram bot that can be used for AI-assisted journaling on the go. You'll be chatting with an AI-version of Socrates, who has received instructions to help you work through any issues you might have, primarily by asking questions. The bot supports exporting your conversation as a text file once you're done, in case you want to save it in a digital journal somewhere.

## Installation

1. Add the necessary environment variables to run the bot
    - `TELEGRAM_API_TOKEN` can be generated using the [Telegram tutorial](https://core.telegram.org/bots/tutorial#getting-ready)
    - `OPEN_AI_API_TOKEN` can be retrieved by [creating an API key](https://platform.openai.com/account/api-keys)
    - `USERNAME_ALLOWLIST` is a comma-separated list of Telegram usernames that have access to the bot. Keep the bot restricted to ensure your Open AI API costs don't explode.
    - `DB_PATH` is a path to the SQLite database that will be used to store dialogues.

```
cp ./.env.example ./.env
```

2. Run the bot

```
cargo run
```
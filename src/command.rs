use clap::{Parser, Subcommand};
use color_eyre::Result;

#[derive(Parser, Debug)]
#[command()]
pub struct BotCommand {
    #[command(subcommand)]
    pub action: BotAction,
}

#[derive(Subcommand, Debug)]
pub enum BotAction {
    #[command(name = "quit")]
    Quit,
}

#[derive(thiserror::Error, Debug)]
pub enum BotCommandError {
    #[error("Missing required prefix for command")]
    MissingPrefix,

    #[error("Failed to parse command")]
    ParseFailed,

    #[error("Clap parsing error")]
    Clap(#[from] clap::Error),
}

pub fn parse_chat_command(text: &str) -> Result<BotCommand, BotCommandError> {
    if !text.starts_with('!') {
        return Err(BotCommandError::MissingPrefix);
    }

    let command_text = text.trim_start_matches('!');
    let mut split = shell_words::split(command_text).map_err(|_| BotCommandError::ParseFailed)?;
    // add a dummy command name to the start of the vec
    split.insert(0, "!".to_string());
    let command = BotCommand::try_parse_from(split)?;

    Ok(command)
}

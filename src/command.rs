use clap::{Parser, Subcommand};
use color_eyre::Result;
use grammers_client::InputMessage;

use crate::Context;

#[derive(Parser, Debug)]
#[command()]
pub struct BotCommand {
    #[command(subcommand)]
    pub action: BotAction,
}

#[derive(Subcommand, Debug)]
pub enum BotAction {
    Quit,

    Ping,

    MsgId,

    ChatId,
}

pub enum ActionResponse {
    Delete,
    Edit(InputMessage),
    Reply(InputMessage),
}

pub struct ActionResult {
    pub quit: bool,
    pub response: Option<ActionResponse>,
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

pub fn parse_chat_command(context: &Context) -> Result<ActionResult, BotCommandError> {
    let text = context.message.text().trim();

    if !text.starts_with('!') {
        return Err(BotCommandError::MissingPrefix);
    }

    let command_text = text.trim_start_matches('!');
    let mut split = shell_words::split(command_text).map_err(|_| BotCommandError::ParseFailed)?;
    // add a dummy command name to the start of the vec
    split.insert(0, "!".to_string());
    let command = BotCommand::try_parse_from(split)?;

    match command.action {
        BotAction::Quit => Ok(ActionResult::quit(true)),
        BotAction::Ping => Ok(ActionResult::reply("Pong!".into())),
        BotAction::MsgId => Ok(ActionResult::edit(InputMessage::markdown(format!(
            "`{}`",
            context.message.id()
        )))),
        BotAction::ChatId => Ok(ActionResult::edit(InputMessage::markdown(format!(
            "`{}`",
            context.chat.id()
        )))),
    }
}

impl ActionResult {
    pub fn quit(delete: bool) -> Self {
        Self {
            quit: true,
            response: if delete {
                Some(ActionResponse::Delete)
            } else {
                None
            },
        }
    }

    pub fn edit(new_message: InputMessage) -> Self {
        Self {
            quit: false,
            response: Some(ActionResponse::Edit(new_message)),
        }
    }

    pub fn reply(response: InputMessage) -> Self {
        Self {
            quit: false,
            response: Some(ActionResponse::Reply(response)),
        }
    }
}

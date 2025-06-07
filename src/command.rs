use clap::{Parser, Subcommand};
use color_eyre::Result;
use grammers_client::InputMessage;

use crate::Context;

use self::{case::CaseArgs, dice::DiceArgs};

mod case;
mod dice;

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

    #[command(alias = "c")]
    Case(CaseArgs),

    Dice(DiceArgs),
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

    #[error("Grammers invocation failed: {0}")]
    Grammers(#[from] grammers_client::InvocationError),
}

pub async fn parse_chat_command(context: &Context) -> Result<ActionResult, BotCommandError> {
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
        BotAction::MsgId => {
            let id = context
                .message
                .reply_to_message_id()
                .unwrap_or(context.message.id());
            Ok(ActionResult::edit(InputMessage::markdown(format!(
                "Message ID: `{}`",
                id
            ))))
        }
        BotAction::ChatId => Ok(ActionResult::edit(InputMessage::markdown(format!(
            "Chat ID: `{}`",
            context.chat.id()
        )))),
        BotAction::Case(args) => {
            let reply = context
                .message
                .get_reply()
                .await?
                .map(|m| m.text().to_string());

            args.handle(reply.as_deref())
        }
        BotAction::Dice(args) => args.handle(),
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

use std::str::FromStr;

use clap::{Args, Parser, Subcommand};
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

    #[command(alias = "c")]
    Case(CaseArgs),
}

#[derive(Clone, Copy, Debug)]
pub enum CaseMode {
    Upcase,
    Downcase,
    Invert,
    Randomize,
    Alternate,
}

impl CaseMode {
    pub fn transform(&self, text: &str) -> String {
        match self {
            CaseMode::Upcase => text.to_uppercase(),
            CaseMode::Downcase => text.to_lowercase(),
            CaseMode::Invert => text
                .chars()
                .map(|c| {
                    if c.is_uppercase() {
                        c.to_lowercase().to_string()
                    } else if c.is_lowercase() {
                        c.to_uppercase().to_string()
                    } else {
                        c.to_string()
                    }
                })
                .collect(),
            CaseMode::Randomize => {
                use rand::Rng;
                let mut rng = rand::rng();
                text.chars()
                    .map(|c| {
                        if rng.random_bool(0.5) {
                            c.to_uppercase().to_string()
                        } else {
                            c.to_lowercase().to_string()
                        }
                    })
                    .collect()
            }
            CaseMode::Alternate => text
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if i % 2 == 0 {
                        c.to_uppercase().to_string()
                    } else {
                        c.to_lowercase().to_string()
                    }
                })
                .collect(),
        }
    }
}

impl FromStr for CaseMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "U" | "UPCASE" => Ok(CaseMode::Upcase),
            "D" | "DOWNCASE" => Ok(CaseMode::Downcase),
            "I" | "INVERT" => Ok(CaseMode::Invert),
            "R" | "RANDOMIZE" => Ok(CaseMode::Randomize),
            "A" | "ALTERNATE" => Ok(CaseMode::Alternate),
            _ => Err(format!("Unknown case mode: {}", s)),
        }
    }
}

#[derive(Args, Debug)]
pub struct CaseArgs {
    #[arg()]
    pub mode: CaseMode,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub text: Vec<String>,
}

impl CaseArgs {
    pub fn handle(&self) -> Result<ActionResult, BotCommandError> {
        let text = self.text.join(" ");
        let transformed_text = self.mode.transform(&text);
        Ok(ActionResult::edit(transformed_text.into()))
    }
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
            if let Some(reply_to) = context.message.get_reply().await? {
                let reply_text = reply_to.text();
                let transformed_text = args.mode.transform(reply_text);
                Ok(ActionResult::edit(transformed_text.into()))
            } else {
                args.handle()
            }
        }
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

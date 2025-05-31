use clap::Parser;

use crate::logging::LogLevel;

const ENV_LOG_LEVEL: &str = "SHABBY_LOG_LEVEL";
const ENV_API_ID: &str = "SHABBY_TG_API_ID";
const ENV_API_HASH: &str = "SHABBY_TG_API_HASH";
const ENV_PHONE_NUMBER: &str = "SHABBY_TG_PHONE_NUMBER";

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    /// Specifies desired logging level.
    ///
    /// If this option is specified, the verbosity and (-v) and quiet (-q) flags
    /// will be ignored.
    #[arg(short, long, env = ENV_LOG_LEVEL, alias = "log-level", global = true)]
    pub log_level: Option<LogLevel>,

    /// Specifies the API ID for Telegram.
    #[arg(
        long,
        env = ENV_API_ID,
        alias = "api-id",
        global = true,
    )]
    pub api_id: Option<i32>,

    /// Specifies the API hash for Telegram.
    #[arg(
        long,
        env = ENV_API_HASH,
        alias = "api-hash",
        global = true,
    )]
    pub api_hash: Option<String>,

    /// Specifies the phone number for Telegram login.
    #[arg(
        long,
        env = ENV_PHONE_NUMBER,
        alias = "phone-number",
        global = true,
    )]
    pub phone_number: Option<String>,
}

impl Cli {
    pub fn log_level(&self) -> Option<LogLevel> {
        if let Some(ll) = self.log_level {
            return Some(ll);
        }

        None
    }
}

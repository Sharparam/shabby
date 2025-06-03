use std::path::PathBuf;

use clap::Parser;

use crate::logging::LogLevel;

use self::verbose::Verbosity;

mod verbose;

const ENV_LOG_LEVEL: &str = "SHABBY_LOG_LEVEL";
const ENV_CONFIG: &str = "SHABBY_CONFIG";
const ENV_API_ID: &str = "SHABBY_TG_API_ID";
const ENV_API_HASH: &str = "SHABBY_TG_API_HASH";
const ENV_PHONE_NUMBER: &str = "SHABBY_TG_PHONE_NUMBER";
const ENV_SESSION: &str = "SHABBY_SESSION";

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Cli {
    #[command(flatten)]
    pub verbose: Verbosity,

    /// Specifies desired logging level.
    ///
    /// Cannot be combined with `--verbose` or `--quiet`.
    #[arg(
        short,
        long,
        env = ENV_LOG_LEVEL,
        alias = "log-level",
        global = true,
        group = "verbosity",
    )]
    pub log_level: Option<LogLevel>,

    /// Specifies the path to the configuration file.
    #[arg(
        short,
        long,
        env = ENV_CONFIG,
        global = true
    )]
    pub config: Option<PathBuf>,

    /// Specifies the API ID for Telegram.
    #[arg(
        short = 'i',
        long,
        env = ENV_API_ID,
        alias = "api-id",
        global = true,
    )]
    pub api_id: Option<i32>,

    /// Specifies the API hash for Telegram.
    #[arg(
        short = 'H',
        long,
        env = ENV_API_HASH,
        alias = "api-hash",
        global = true,
    )]
    pub api_hash: Option<String>,

    /// Specifies the phone number for Telegram login.
    #[arg(
        short,
        long,
        env = ENV_PHONE_NUMBER,
        alias = "phone-number",
        global = true,
    )]
    pub phone_number: Option<String>,

    /// Specifies the path to the session file.
    #[arg(short, long, env = ENV_SESSION, global = true)]
    pub session: Option<PathBuf>,
}

impl Cli {
    pub fn log_level(&self) -> Option<LogLevel> {
        if let Some(ll) = self.log_level {
            return Some(ll);
        }

        match self.verbose.is_present() {
            true => Some(self.verbose.level()),
            false => None,
        }
    }
}

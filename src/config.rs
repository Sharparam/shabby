use std::path::PathBuf;

use color_eyre::{Result, eyre::OptionExt};

use crate::cli::Cli;

const DEFAULT_SESSION_FILENAME: &str = "shabby.session";

#[derive(Debug)]
pub struct Config {
    api_id: i32,
    api_hash: String,
    phone_number: String,
    session_filename: PathBuf,
}

impl Config {
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        let api_id = cli.api_id.ok_or_eyre("API ID not provided")?;
        let api_hash = cli.api_hash.clone().ok_or_eyre("API hash not provided")?;
        let phone_number = cli
            .phone_number
            .clone()
            .ok_or_eyre("Phone number not provided")?;

        Ok(Self {
            api_id,
            api_hash,
            phone_number,
            session_filename: DEFAULT_SESSION_FILENAME.into(),
        })
    }

    pub fn api_id(&self) -> i32 {
        self.api_id
    }

    pub fn api_hash(&self) -> &str {
        &self.api_hash
    }

    pub fn phone_number(&self) -> &str {
        &self.phone_number
    }

    pub fn session_filename(&self) -> &PathBuf {
        &self.session_filename
    }
}

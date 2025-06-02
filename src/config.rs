use std::{path::PathBuf, str::FromStr};

use color_eyre::{Result, eyre::OptionExt};
use kdl::{KdlDocument, KdlError};
use thiserror::Error;
use tracing::{error, info};

use crate::cli::Cli;

const DEFAULT_SESSION_FILENAME: &str = "shabby.session";

#[derive(Debug)]
pub struct Config {
    api_id: i32,
    api_hash: String,
    phone_number: String,
    session_filename: PathBuf,
}

#[derive(Debug, Default)]
struct ConfigFile {
    pub api_id: Option<i32>,
    pub api_hash: Option<String>,
    pub phone_number: Option<String>,
    pub session_filename: Option<PathBuf>,
}

#[derive(Debug, Error)]
enum ConfigFileError {
    #[error("Failed to read config file")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config file")]
    Parse(#[from] KdlError),

    #[error("Invalid value in config file")]
    InvalidValue,
}

impl Config {
    pub fn from_cli(cli: &Cli) -> Result<Self> {
        let mut config_file: Option<ConfigFile> = None;
        let mut api_id: Option<i32> = None;
        let mut api_hash: Option<String> = None;
        let mut phone_number: Option<String> = None;
        let mut session_filename: Option<PathBuf> = None;

        if let Some(config_path) = &cli.config {
            config_file = Some(ConfigFile::load_file(config_path)?);
        }

        if let Some(config_file) = config_file {
            api_id = config_file.api_id;
            api_hash = config_file.api_hash;
            phone_number = config_file.phone_number;
            session_filename = config_file.session_filename;
        }

        if let Some(cli_api_id) = cli.api_id {
            api_id = Some(cli_api_id);
        }

        if let Some(cli_api_hash) = &cli.api_hash {
            api_hash = Some(cli_api_hash.to_string());
        }

        if let Some(cli_phone_number) = &cli.phone_number {
            phone_number = Some(cli_phone_number.to_string());
        }

        Ok(Self {
            api_id: api_id.ok_or_eyre("API ID not provided")?,
            api_hash: api_hash.ok_or_eyre("API hash not provided")?,
            phone_number: phone_number.ok_or_eyre("Phone number not provided")?,
            session_filename: session_filename
                .unwrap_or_else(|| PathBuf::from(DEFAULT_SESSION_FILENAME)),
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

impl ConfigFile {
    fn load_file(path: &PathBuf) -> Result<Self, ConfigFileError> {
        info!(path = %path.display(), "Loading configuration from file");
        let content = std::fs::read_to_string(path)?;
        content.parse()
    }
}

impl FromStr for ConfigFile {
    type Err = ConfigFileError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut config = ConfigFile::default();
        let doc: KdlDocument = s.parse()?;

        if let Some(telegram) = doc.get("telegram") {
            if let Some(children) = telegram.children() {
                if let Some(api_id) = children.get_arg("api_id") {
                    match api_id.as_integer() {
                        Some(id) if id >= i32::MIN as i128 && id <= i32::MAX as i128 => {
                            info!("Parsed valid API ID from config file");
                            config.api_id = Some(id as i32);
                        }
                        _ => {
                            error!("API ID key present in config but value is missing or invalid");
                            return Err(ConfigFileError::InvalidValue);
                        }
                    }
                }

                if let Some(api_hash) = children.get_arg("api_hash") {
                    if let Some(hash) = api_hash.as_string() {
                        info!("Parsed API hash from config file");
                        config.api_hash = Some(hash.to_string());
                    } else {
                        error!("API hash key present in config but value is missing or invalid");
                        return Err(ConfigFileError::InvalidValue);
                    }
                }

                if let Some(phone_number) = children.get_arg("phone_number") {
                    if let Some(phone) = phone_number.as_string() {
                        info!("Parsed phone number from config file");
                        config.phone_number = Some(phone.to_string());
                    } else {
                        error!(
                            "Phone number key present in config but value is missing or invalid"
                        );
                        return Err(ConfigFileError::InvalidValue);
                    }
                }

                if let Some(session_filename) = children.get_arg("session_filename") {
                    if let Some(filename) = session_filename.as_string() {
                        info!("Parsed session filename from config file");
                        config.session_filename = Some(PathBuf::from(filename));
                    } else {
                        error!(
                            "Session filename key present in config but value is missing or invalid"
                        );
                        return Err(ConfigFileError::InvalidValue);
                    }
                }
            }
        }

        Ok(config)
    }
}

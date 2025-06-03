use std::path::PathBuf;

use color_eyre::{Result, eyre::Context};
use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};

const APP_TLD: &str = "com";
const APP_AUTHOR: &str = "Sharparam";
const APP_NAME: &str = "shabby";

fn strategy() -> Result<impl AppStrategy> {
    choose_app_strategy(AppStrategyArgs {
        top_level_domain: APP_TLD.to_string(),
        author: APP_AUTHOR.to_string(),
        app_name: APP_NAME.to_string(),
    })
    .wrap_err("Failed to create app strategy")
}

pub fn config() -> Result<PathBuf> {
    Ok(strategy()?.config_dir())
}

pub fn state() -> Result<PathBuf> {
    let strategy = strategy()?;
    match strategy.state_dir() {
        Some(path) => Ok(path),
        None => Ok(strategy.data_dir()),
    }
}

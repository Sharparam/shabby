use std::str::FromStr;

use clap::Args;

use super::{ActionResult, BotCommandError};

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
    pub fn handle(&self, text: Option<&str>) -> Result<ActionResult, BotCommandError> {
        let text = text
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.text.join(" "));
        let transformed_text = self.mode.transform(&text);
        Ok(ActionResult::edit(transformed_text.into()))
    }
}

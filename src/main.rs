use std::{io, process::ExitCode};

use color_eyre::Result;

use shabby::run;

#[allow(
    unused_imports,
    reason = "these will just constantly be added and removed otherwise"
)]
use tracing::{debug, error, info, warn};

/// The entry point for shabby.
///
/// Great things will eventually happen here.
#[tokio::main]
async fn main() -> Result<ExitCode> {
    color_eyre::install()?;

    if let Err(err) = run().await {
        if let Some(clap_err) = err.root_cause().downcast_ref::<clap::Error>() {
            clap_err.print().unwrap();
            return match clap_err.kind() {
                clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion => {
                    Ok(ExitCode::SUCCESS)
                }
                _ => Ok(ExitCode::from(64)),
            };
        }

        eprintln!("Error: {:?}", err);

        for cause in err.chain() {
            if cause.downcast_ref::<io::Error>().is_some() {
                return Ok(ExitCode::from(66));
            }
        }

        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

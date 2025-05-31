use std::io;

use color_eyre::{Result, eyre::WrapErr};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{Layer, Registry, layer::SubscriberExt, reload, util::SubscriberInitExt};

pub use self::level::LogLevel;

mod compat;
mod level;

#[must_use]
pub struct LogState {
    level_filter_reload_handle: reload::Handle<LevelFilter, Registry>,
    // level_filter_others_reload_handle:
    //     reload::Handle<LevelFilter, Layered<Box<dyn Layer<Registry> + Send + Sync>, Registry>>,
}

impl LogState {
    pub fn set_level_filter<L>(&self, level: L) -> Result<()>
    where
        L: Into<LogLevel>,
    {
        let log_level = level.into();
        let level_filter = LevelFilter::from(log_level);
        // let level_filter_others = LevelFilter::from(map_other_log_level(log_level));
        self.level_filter_reload_handle
            .modify(|f| *f = level_filter)
            .wrap_err("Failed to modify log level filter")
        // self.level_filter_others_reload_handle
        //     .modify(|f| *f = level_filter_others)
        //     .wrap_err("Failed to modify other log level filter")
    }
}

pub fn init<L>(level: L) -> Result<LogState>
where
    L: Into<LogLevel>,
{
    let log_level = level.into();
    let log_level_others = map_other_log_level(log_level);

    let level_filter = LevelFilter::from(log_level);
    let (level_filter, level_filter_reload_handle) = reload::Layer::new(level_filter);

    let level_filter_others = LevelFilter::from(log_level_others);
    // let (level_filter_others, level_filter_others_reload_handle) =
    //     reload::Layer::new(level_filter_others);

    let others_layer = tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .without_time()
        .with_filter(level_filter_others)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            !metadata.target().starts_with("shabby")
        }))
        .boxed();

    #[cfg(debug_assertions)]
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .pretty()
        .without_time()
        .with_filter(level_filter)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.target().starts_with("shabby")
        }))
        .boxed();

    #[cfg(not(debug_assertions))]
    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(io::stderr)
        .without_time()
        .with_filter(level_filter)
        .with_filter(tracing_subscriber::filter::filter_fn(|metadata| {
            metadata.target().starts_with("shabby")
        }));

    tracing_subscriber::registry()
        .with(stderr_layer)
        .with(others_layer)
        .try_init()
        .wrap_err("Failed to set default logger")?;

    Ok(LogState {
        level_filter_reload_handle,
        // level_filter_others_reload_handle,
    })
}

fn map_other_log_level(level: LogLevel) -> LogLevel {
    match level {
        #[cfg(debug_assertions)]
        LogLevel::Trace | LogLevel::Debug | LogLevel::Info => LogLevel::Warn,
        #[cfg(not(debug_assertions))]
        LogLevel::Trace | LogLevel::Debug | LogLevel::Info | LogLevel::Warn => LogLevel::Error,
        _ => level,
    }
}

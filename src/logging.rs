use std::{fmt::Display, io, str::FromStr};

use color_eyre::{
    Result,
    eyre::{WrapErr, eyre},
};
use tracing::metadata::LevelFilter;
use tracing_subscriber::{
    Layer, Registry,
    layer::{Layered, SubscriberExt},
    reload,
    util::SubscriberInitExt,
};

mod compat;

#[derive(Default, Copy, Clone, Debug)]
pub enum LogLevel {
    Off,

    Error,

    Warn,

    #[cfg_attr(not(debug_assertions), default)]
    Info,

    #[cfg_attr(debug_assertions, default)]
    Debug,

    Trace,
}

#[must_use]
pub struct LogState {
    level_filter_reload_handle: reload::Handle<LevelFilter, Registry>,

    level_filter_others_reload_handle:
        reload::Handle<LevelFilter, Layered<Box<dyn Layer<Registry> + Send + Sync>, Registry>>,
}

impl LogState {
    pub fn set_level_filter<L>(&self, level: L) -> Result<()>
    where
        L: Into<LogLevel>,
    {
        let log_level = level.into();
        let level_filter = LevelFilter::from(log_level);
        let level_filter_others = LevelFilter::from(map_other_log_level(log_level));
        self.level_filter_reload_handle
            .modify(|f| *f = level_filter)
            .wrap_err("Failed to modify log level filter")?;
        self.level_filter_others_reload_handle
            .modify(|f| *f = level_filter_others)
            .wrap_err("Failed to modify other log level filter")
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
    let (level_filter_others, level_filter_others_reload_handle) =
        reload::Layer::new(level_filter_others);

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
        level_filter_others_reload_handle,
    })
}

fn map_other_log_level(level: LogLevel) -> LogLevel {
    match level {
        LogLevel::Trace | LogLevel::Debug | LogLevel::Info => LogLevel::Warn,
        _ => level,
    }
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use LogLevel::*;

        let level = match self {
            Off => "OFF",
            Error => "ERROR",
            Warn => "WARN",
            Info => "INFO",
            Debug => "DEBUG",
            Trace => "TRACE",
        };

        f.write_str(level)
    }
}

impl FromStr for LogLevel {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LogLevel::*;

        s.parse::<usize>()
            .ok()
            .and_then(|n| match n {
                0 => Some(Off),
                1 => Some(Error),
                2 => Some(Warn),
                3 => Some(Info),
                4 => Some(Debug),
                5 => Some(Trace),
                _ => None,
            })
            .or_else(|| match s {
                "" => Some(Default::default()),
                s if s.eq_ignore_ascii_case("e") => Some(Error),
                s if s.eq_ignore_ascii_case("err") => Some(Error),
                s if s.eq_ignore_ascii_case("error") => Some(Error),
                s if s.eq_ignore_ascii_case("w") => Some(Warn),
                s if s.eq_ignore_ascii_case("warn") => Some(Warn),
                s if s.eq_ignore_ascii_case("warning") => Some(Warn),
                s if s.eq_ignore_ascii_case("i") => Some(Info),
                s if s.eq_ignore_ascii_case("inf") => Some(Info),
                s if s.eq_ignore_ascii_case("info") => Some(Info),
                s if s.eq_ignore_ascii_case("information") => Some(Info),
                s if s.eq_ignore_ascii_case("d") => Some(Debug),
                s if s.eq_ignore_ascii_case("dbg") => Some(Debug),
                s if s.eq_ignore_ascii_case("debug") => Some(Debug),
                s if s.eq_ignore_ascii_case("t") => Some(Trace),
                s if s.eq_ignore_ascii_case("trace") => Some(Trace),
                s if s.eq_ignore_ascii_case("v") => Some(Trace),
                s if s.eq_ignore_ascii_case("verbose") => Some(Trace),
                s if s.eq_ignore_ascii_case("o") => Some(Off),
                s if s.eq_ignore_ascii_case("off") => Some(Off),
                s if s.eq_ignore_ascii_case("disable") => Some(Off),
                s if s.eq_ignore_ascii_case("disabled") => Some(Off),
                s if s.eq_ignore_ascii_case("no") => Some(Off),
                s if s.eq_ignore_ascii_case("none") => Some(Off),
                _ => None,
            })
            .ok_or(eyre!("Invalid log level: {}", s))
    }
}

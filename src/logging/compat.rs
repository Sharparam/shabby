use tracing::level_filters::LevelFilter;

use super::LogLevel;

impl From<LogLevel> for LevelFilter {
    fn from(value: LogLevel) -> Self {
        use LogLevel::*;
        match value {
            Off => LevelFilter::OFF,
            Error => LevelFilter::ERROR,
            Warn => LevelFilter::WARN,
            Info => LevelFilter::INFO,
            Debug => LevelFilter::DEBUG,
            Trace => LevelFilter::TRACE,
        }
    }
}

impl From<LevelFilter> for LogLevel {
    fn from(value: LevelFilter) -> Self {
        use LogLevel::*;
        match value {
            LevelFilter::OFF => Off,
            LevelFilter::ERROR => Error,
            LevelFilter::WARN => Warn,
            LevelFilter::INFO => Info,
            LevelFilter::DEBUG => Debug,
            LevelFilter::TRACE => Trace,
        }
    }
}

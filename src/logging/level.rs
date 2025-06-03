use std::{fmt::Display, str::FromStr};

use color_eyre::eyre::eyre;

#[derive(Default, Copy, Clone, Debug, PartialEq)]
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

impl LogLevel {
    pub fn with_offset(&self, offset: i16) -> LogLevel {
        let value: i16 = (*self).into();
        value.saturating_add(offset).into()
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

impl From<i8> for LogLevel {
    fn from(value: i8) -> Self {
        let value = value as i16;
        value.into()
    }
}

impl From<i16> for LogLevel {
    fn from(value: i16) -> Self {
        let value = value as i128;
        value.into()
    }
}

impl From<i128> for LogLevel {
    fn from(value: i128) -> Self {
        use LogLevel::*;

        match value {
            i128::MIN..=0 => Off,
            1 => Error,
            2 => Warn,
            3 => Info,
            4 => Debug,
            5..=i128::MAX => Trace,
        }
    }
}

impl From<LogLevel> for i16 {
    fn from(value: LogLevel) -> Self {
        use LogLevel::*;

        match value {
            Off => 0,
            Error => 1,
            Warn => 2,
            Info => 3,
            Debug => 4,
            Trace => 5,
        }
    }
}

impl FromStr for LogLevel {
    type Err = color_eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use LogLevel::*;

        s.parse::<i128>()
            .ok()
            .map(|n| n.into())
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

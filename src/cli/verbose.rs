use clap::{ArgAction, Args, arg};

use indoc::indoc;

use crate::logging::LogLevel;

#[cfg(not(debug_assertions))]
type DefaultLevel = InfoLevel;

#[cfg(debug_assertions)]
type DefaultLevel = DebugLevel;

#[derive(Args, Debug, Clone)]
pub struct Verbosity<L: VerbosityLevel = DefaultLevel> {
    #[arg(
        short = 'v',
        long,
        action = ArgAction::Count,
        global = true,
        group = "verbosity",
        help = L::verbose_help(),
        long_help = L::verbose_long_help()
    )]
    verbose: u8,

    #[arg(
        short = 'q',
        long,
        action = ArgAction::Count,
        global = true,
        group = "verbosity",
        help = L::quiet_help(),
        long_help = L::quiet_long_help(),
    )]
    quiet: u8,

    #[arg(skip)]
    phantom: std::marker::PhantomData<L>,
}

impl<L: VerbosityLevel> Verbosity<L> {
    pub fn new(verbose: u8, quiet: u8) -> Self {
        Self {
            verbose,
            quiet,
            phantom: std::marker::PhantomData,
        }
    }

    pub fn is_present(&self) -> bool {
        self.verbose != 0 || self.quiet != 0
    }

    pub fn is_silent(&self) -> bool {
        self.level() == LogLevel::Off
    }

    pub fn level(&self) -> LogLevel {
        let offset = self.verbose as i16 - self.quiet as i16;
        L::default_level().with_offset(offset)
    }
}

pub trait VerbosityLevel {
    fn default_level() -> LogLevel;

    fn verbose_help() -> Option<&'static str> {
        Some("Increase logging verbosity")
    }

    fn verbose_long_help() -> Option<&'static str> {
        None
    }

    fn quiet_help() -> Option<&'static str> {
        Some("Decrease logging verbosity")
    }

    fn quiet_long_help() -> Option<&'static str> {
        None
    }
}

#[derive(Copy, Clone, Debug)]
pub struct InfoLevel;

impl VerbosityLevel for InfoLevel {
    fn default_level() -> LogLevel {
        LogLevel::Info
    }

    fn verbose_long_help() -> Option<&'static str> {
        Some(indoc! {"
            Increase logging verbosity.

            The more times this optioni is specified, the more verbose the
            output will become.

            Specifically, they relate as follows:
                -v     Show debug
                -vv    Show trace

            Specifying this option more than two times has no further effect.
        "})
    }

    fn quiet_long_help() -> Option<&'static str> {
        Some(indoc! {"
            Decrease logging verbosity.

            The more times this option is specified, the less verbose the
            output will become.

            Specifically, they relate as follows:
                -q      Show warning
                -qq     Show error
                -qqq    Show nothing (turns off logging)

            Specifying this option more than three times has no further effect.
        "})
    }
}

#[derive(Copy, Clone, Debug)]
pub struct DebugLevel;

impl VerbosityLevel for DebugLevel {
    fn default_level() -> LogLevel {
        LogLevel::Debug
    }

    fn verbose_long_help() -> Option<&'static str> {
        Some(indoc! {"
            Increase logging verbosity.

            The more times this option is specified, the more verbose the
            output will become.

            Specifically, they relate as follows:
                -v     Show trace

            Specifying this option more than once has no further effect.

            The default verbosity level is debug.
        "})
    }

    fn quiet_long_help() -> Option<&'static str> {
        Some(indoc! {"
            Decrease logging verbosity.

            The more times this option is specified, the less verbose the
            output will become.

            Specifically, they relate as follows:
                -q      Show info
                -qq     Show warning
                -qqq    Show error
                -qqqq   Show nothing (turns off logging)

            Specifying this option more than four times has no further effect.

            The default verbosity level is debug.
        "})
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_app() {
        #[derive(Debug, clap::Parser)]
        struct App {
            #[command(flatten)]
            verbose: Verbosity,
        }

        use clap::CommandFactory;
        App::command().debug_assert();
    }

    macro_rules! test_level {
        ($testname:ident, $type:ty, $verbose:expr, $quiet:expr, $expected:expr) => {
            #[test]
            fn $testname() {
                let verbosity = Verbosity::<$type>::new($verbose, $quiet);
                let level = verbosity.level();
                assert_eq!(level, $expected);
            }
        };
    }

    test_level!(verbosity_info_0_0_info, InfoLevel, 0, 0, LogLevel::Info);
    test_level!(verbosity_info_1_0_debug, InfoLevel, 1, 0, LogLevel::Debug);
    test_level!(verbosity_info_2_0_trace, InfoLevel, 2, 0, LogLevel::Trace);
    test_level!(verbosity_info_3_0_trace, InfoLevel, 3, 0, LogLevel::Trace);
    test_level!(
        verbosity_info_255_0_trace,
        InfoLevel,
        255,
        0,
        LogLevel::Trace
    );
    test_level!(verbosity_info_0_1_warn, InfoLevel, 0, 1, LogLevel::Warn);
    test_level!(verbosity_info_0_2_error, InfoLevel, 0, 2, LogLevel::Error);
    test_level!(verbosity_info_0_3_off, InfoLevel, 0, 3, LogLevel::Off);
    test_level!(verbosity_info_0_4_off, InfoLevel, 0, 4, LogLevel::Off);
    test_level!(verbosity_info_0_255_off, InfoLevel, 0, 255, LogLevel::Off);
    test_level!(
        verbosity_info_255_255_info,
        InfoLevel,
        255,
        255,
        LogLevel::Info
    );

    test_level!(verbosity_debug_0_0_debug, DebugLevel, 0, 0, LogLevel::Debug);
    test_level!(verbosity_debug_1_0_trace, DebugLevel, 1, 0, LogLevel::Trace);
    test_level!(verbosity_debug_2_0_trace, DebugLevel, 2, 0, LogLevel::Trace);
    test_level!(
        verbosity_debug_255_0_trace,
        DebugLevel,
        255,
        0,
        LogLevel::Trace
    );
    test_level!(verbosity_debug_0_1_info, DebugLevel, 0, 1, LogLevel::Info);
    test_level!(verbosity_debug_0_2_warn, DebugLevel, 0, 2, LogLevel::Warn);
    test_level!(verbosity_debug_0_3_error, DebugLevel, 0, 3, LogLevel::Error);
    test_level!(verbosity_debug_0_4_off, DebugLevel, 0, 4, LogLevel::Off);
    test_level!(verbosity_debug_0_5_off, DebugLevel, 0, 5, LogLevel::Off);
    test_level!(verbosity_debug_0_255_off, DebugLevel, 0, 255, LogLevel::Off);
    test_level!(
        verbosity_debug_255_255_debug,
        DebugLevel,
        255,
        255,
        LogLevel::Debug
    );
}

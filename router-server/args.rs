use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum LogColored {
    Never,
    Auto,
    Always,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "lo")]
    pub(crate) iface: String,

    #[arg(short, long, default_value = "127.0.0.1:6707")]
    pub(crate) bind: String,

    #[arg(short, long, default_value_t = LogLevel::Trace, value_enum)]
    pub(crate) log_level: LogLevel,

    #[arg(long, default_value_t = LogColored::Always, value_enum)]
    pub(crate) log_colored: LogColored,
}

pub fn parse() -> Args {
    Args::parse()
}

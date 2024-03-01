use crate::args::{LogColored, LogLevel};
use flexi_logger::{style, AdaptiveFormat, DeferredNow, Logger as FlexiLogger, LoggerHandle, WriteMode};
use log::{Level, Record};
use std::{sync::Arc, time::Duration};
use tokio::{sync::Mutex, time};

type Result<T> = std::result::Result<T, anyhow::Error>;

const LOG_TIMESTAMP: &str = "%Y-%m-%d %H:%M:%S%.3f";

#[derive(Clone)]
pub struct Logger(Arc<Mutex<LoggerHandle>>);

impl Logger {
    pub fn build(level: &LogLevel, colored: &LogColored) -> Result<Self> {
        let level = match level {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        };

        let spec = format!("info, router = {level}");

        let fmt_uncolored = match colored {
            LogColored::Never => log_format_uncolored,
            LogColored::Auto => log_format_uncolored,
            LogColored::Always => log_format_colored,
        };

        let fmt_colored = match colored {
            LogColored::Never => log_format_uncolored,
            LogColored::Auto => log_format_colored,
            LogColored::Always => log_format_colored,
        };

        Ok(Self(Arc::new(Mutex::new(
            FlexiLogger::try_with_str(spec)?
                .log_to_stdout()
                .adaptive_format_for_stdout(AdaptiveFormat::Custom(fmt_uncolored, fmt_colored))
                .set_palette("196;190;-;33;8".to_owned())
                .write_mode(WriteMode::Async)
                .start()?,
        ))))
    }

    pub async fn close(self) {
        time::sleep(Duration::from_millis(200)).await;
        self.0.lock().await.shutdown();
    }
}

fn log_format_uncolored(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> std::result::Result<(), std::io::Error> {
    write!(w, "{} [{:.1}] {}", now.format(LOG_TIMESTAMP), record.level(), record.args())
}

fn log_format_colored(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> std::result::Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "{} {}",
        style(Level::Trace).paint(now.format(LOG_TIMESTAMP).to_string()),
        style(level).paint(format!("[{:.1}] {}", level, record.args())),
    )
}

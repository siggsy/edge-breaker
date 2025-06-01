use colored::Colorize;
use log::{Level, Metadata, Record};

pub struct Logger;

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let level = match record.level() {
            Level::Error => "ERRO".red(),
            Level::Warn => "WARN".yellow(),
            Level::Info => "INFO".cyan(),
            Level::Debug => "DEBG".blue(),
            Level::Trace => "TRAC".purple(),
        }
        .bold();

        eprintln!(
            "[{}:{}:{}] {}",
            level,
            record.file().unwrap(),
            record.line().unwrap(),
            record.args()
        );
    }

    fn flush(&self) {}
}

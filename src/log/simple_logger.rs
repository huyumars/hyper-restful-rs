use chrono::Local;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct SimpleLogger;

#[allow(dead_code)]
static LOGGER: SimpleLogger = SimpleLogger;

#[allow(dead_code)]
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} {} - {}", Local::now(), record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

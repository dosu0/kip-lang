use log::{Level, LevelFilter, Log, Record, SetLoggerError};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

static LOGGER: Logger = Logger;
struct Logger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    Ok(log::set_max_level(LevelFilter::Info))
}

static LEVEL_NAMES: [&str; 6] = ["off", "error", "warning", "info", "debug", "trace"];

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut buffer = StandardStream::stderr(ColorChoice::Auto);
            let mut color_spec = ColorSpec::new();
            color_spec.set_bold(true);

            match record.level() {
                Level::Error => color_spec.set_fg(Some(Color::Red)),
                Level::Warn => color_spec.set_fg(Some(Color::Yellow)),
                Level::Info => color_spec.set_fg(Some(Color::Blue)),
                Level::Debug => color_spec.set_fg(Some(Color::Green)),
                Level::Trace => color_spec.set_fg(Some(Color::Black)),
            };

            buffer.set_color(&color_spec).unwrap_or(());

            writeln!(
                &mut buffer,
                "[{}] {}: {}",
                record.target(),
                LEVEL_NAMES[record.level() as usize],
                record.args()
            )
            .unwrap();

            // reset color
            color_spec.clear();
            buffer.set_color(&color_spec).unwrap();
        }
    }

    fn flush(&self) {}
}

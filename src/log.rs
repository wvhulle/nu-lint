use std::{env::current_dir, io, io::Write};

use env_logger::fmt::Formatter;

/// Initialize the logger with an easy to read format for stdout terminal
/// output. Use it to debug tests with print debugging.
#[cfg(test)]
pub fn init_env_log() {
    env_logger::builder()
        .format(format_log_record)
        .is_test(true)
        .try_init()
        .ok();
}

pub fn init_log() {
    env_logger::builder()
        .format(format_log_record)
        .filter_level(log::LevelFilter::Debug)
        .try_init()
        .ok();
}

fn format_log_record(buf: &mut Formatter, record: &log::Record) -> io::Result<()> {
    let relative_file = get_relative_file_path(record);
    let (color_start, color_end) = get_level_colors(record.level());

    writeln!(
        buf,
        "{}{}:{} {}{}",
        color_start,
        relative_file,
        record.line().unwrap_or(0),
        record.args(),
        color_end
    )
}

fn get_relative_file_path<'a>(record: &'a log::Record) -> &'a str {
    let file = record.file().unwrap_or("unknown");
    current_dir()
        .ok()
        .and_then(|cwd| file.strip_prefix(&*cwd.to_string_lossy()))
        .unwrap_or(file)
        .trim_start_matches('/')
}

const fn get_level_colors(level: log::Level) -> (&'static str, &'static str) {
    match level {
        log::Level::Error => ("\x1b[91m", "\x1b[0m"), // Red
        log::Level::Warn => ("\x1b[93m", "\x1b[0m"),  // Yellow
        log::Level::Info => ("\x1b[34m", "\x1b[0m"),  // Dark blue
        log::Level::Debug => ("\x1b[96m", "\x1b[0m"), // Cyan
        log::Level::Trace => ("\x1b[90m", "\x1b[0m"), // Gray
    }
}

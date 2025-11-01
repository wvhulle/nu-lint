// Custom logging setup with colored output and relative file paths
pub fn instrument() {
    env_logger::builder()
        .format(format_log_record)
        .try_init()
        .ok();
}

fn format_log_record(
    buf: &mut env_logger::fmt::Formatter,
    record: &log::Record,
) -> std::io::Result<()> {
    use std::io::Write;

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
    std::env::current_dir()
        .ok()
        .and_then(|cwd| file.strip_prefix(&*cwd.to_string_lossy()))
        .unwrap_or(file)
        .trim_start_matches('/')
}

fn get_level_colors(level: log::Level) -> (&'static str, &'static str) {
    match level {
        log::Level::Error => ("\x1b[91m", "\x1b[0m"), // Red
        log::Level::Warn => ("\x1b[93m", "\x1b[0m"),  // Yellow
        log::Level::Info => ("\x1b[34m", "\x1b[0m"),  // Dark blue
        log::Level::Debug => ("\x1b[96m", "\x1b[0m"), // Cyan
        log::Level::Trace => ("\x1b[90m", "\x1b[0m"), // Gray
    }
}

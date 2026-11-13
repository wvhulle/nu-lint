use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use tracing::{Subscriber, level_filters::LevelFilter, subscriber::set_global_default};
use tracing_appender::{non_blocking, non_blocking::WorkerGuard};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, registry::LookupSpan};

const NU_LINT_CRATES: [&str; 3] = ["nu_lint", "nu_lint_analysis", "nu_lint_rules"];

pub enum LspLogDestination {
    Stderr,
    StderrAndFile(PathBuf),
}

#[must_use]
pub enum LspLogSession {
    StderrOnly,
    MirroredToFile { _flush_file_on_drop: WorkerGuard },
}

fn nu_lint_crates_filter(log_level: LevelFilter) -> EnvFilter {
    let directives = NU_LINT_CRATES
        .map(|crate_name| format!("{crate_name}={log_level}"))
        .join(",");
    EnvFilter::new(directives)
}

fn stderr_layer<S>(log_level: LevelFilter) -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fmt::layer()
        .with_writer(io::stderr)
        .with_ansi(false)
        .with_target(false)
        .with_filter(nu_lint_crates_filter(log_level))
}

fn open_log_file(log_file: &Path) -> io::Result<File> {
    if let Some(log_directory) = log_file
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(log_directory)?;
    }
    OpenOptions::new().create(true).append(true).open(log_file)
}

fn log_to_stderr(log_level: LevelFilter) -> LspLogSession {
    set_global_default(tracing_subscriber::registry().with(stderr_layer(log_level))).ok();
    LspLogSession::StderrOnly
}

fn log_to_stderr_and_file(log_level: LevelFilter, log_file: &Path) -> LspLogSession {
    let opened_log_file = match open_log_file(log_file) {
        Ok(opened_log_file) => opened_log_file,
        Err(open_error) => {
            let stderr_session = log_to_stderr(log_level);
            tracing::error!(
                "Cannot write LSP log to {}: {open_error}",
                log_file.display()
            );
            return stderr_session;
        }
    };
    let (file_writer, file_writer_guard) = non_blocking(opened_log_file);
    let file_layer = fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_file(true)
        .with_line_number(true)
        .with_target(false)
        .with_filter(nu_lint_crates_filter(log_level));

    set_global_default(
        tracing_subscriber::registry()
            .with(stderr_layer(log_level))
            .with(file_layer),
    )
    .ok();
    LspLogSession::MirroredToFile {
        _flush_file_on_drop: file_writer_guard,
    }
}

pub fn init_lsp_log(log_level: LevelFilter, log_destination: &LspLogDestination) -> LspLogSession {
    tracing_log::LogTracer::init().ok();
    match log_destination {
        LspLogDestination::Stderr => log_to_stderr(log_level),
        LspLogDestination::StderrAndFile(log_file) => log_to_stderr_and_file(log_level, log_file),
    }
}

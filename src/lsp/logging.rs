use nu_lint_analysis::log::NU_LINT_DEBUG_FILTER;
use tracing::subscriber::set_global_default;
use tracing_appender::{
    non_blocking,
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};

#[must_use]
pub fn init_lsp_log() -> Option<WorkerGuard> {
    tracing_log::LogTracer::init().ok();

    let log_dir = dirs::cache_dir().map(|cache_dir| cache_dir.join("nu-lint"))?;
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("lsp.log")
        .build(log_dir)
        .ok()?;
    let (non_blocking_writer, guard) = non_blocking(file_appender);

    let filter = EnvFilter::new(NU_LINT_DEBUG_FILTER);
    let subscriber = tracing_subscriber::registry().with(filter).with(
        fmt::layer()
            .with_writer(non_blocking_writer)
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(false),
    );

    set_global_default(subscriber).ok();
    Some(guard)
}

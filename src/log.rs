/// Initialize logger for tests. Output is captured and only shown on failure.
/// Only shows debug+ messages from nu-lint code.
pub fn init_test_log() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    // Set up log -> tracing bridge
    tracing_log::LogTracer::init().ok();

    let filter = EnvFilter::new("nu_lint=debug");

    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_test_writer().with_target(false))
        .try_init();
}

#[cfg(feature = "lsp")]
pub use tracing_appender::non_blocking::WorkerGuard as LogGuard;

/// Initialize file-based logger for LSP mode with daily rotation.
/// Logs are written to `~/.cache/nu-lint/lsp.YYYY-MM-DD.log`.
/// Returns a guard that must be kept alive for the duration of logging.
#[cfg(feature = "lsp")]
#[must_use]
pub fn init_lsp_log() -> Option<LogGuard> {
    use tracing::subscriber::set_global_default;
    use tracing_appender::{non_blocking, rolling};
    use tracing_subscriber::{fmt, layer::SubscriberExt};

    // Set up log -> tracing bridge
    tracing_log::LogTracer::init().ok();

    let log_dir = dirs::cache_dir().map(|d| d.join("nu-lint"))?;
    let file_appender = rolling::daily(log_dir, "lsp.log");
    let (non_blocking, guard) = non_blocking(file_appender);

    let subscriber = tracing_subscriber::registry().with(
        fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(false),
    );

    set_global_default(subscriber).ok();
    Some(guard)
}

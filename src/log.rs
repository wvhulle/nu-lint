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

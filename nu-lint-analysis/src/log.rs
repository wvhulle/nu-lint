pub const NU_LINT_DEBUG_FILTER: &str = "nu_lint=debug,nu_lint_analysis=debug,nu_lint_rules=debug";

pub fn init_test_log() {
    use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

    tracing_log::LogTracer::init().ok();

    let _ = tracing_subscriber::registry()
        .with(EnvFilter::new(NU_LINT_DEBUG_FILTER))
        .with(fmt::layer().with_test_writer().with_target(false))
        .try_init();
}

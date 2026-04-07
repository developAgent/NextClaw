use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize logging for the application
///
/// This sets up structured logging with:
/// - Console output in development
/// - File output in production
/// - Environment-based log level filtering
pub fn setup_logging() -> Option<WorkerGuard> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_target(true)
        .with_thread_ids(true)
        .with_thread_names(true);

    #[cfg(debug_assertions)]
    {
        // Development: Console only
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .init();
        None
    }

    #[cfg(not(debug_assertions))]
    {
        // Production: File output
        let log_dir = directories::ProjectDirs::from("com", "ceoclaw", "CEOClaw")
            .expect("Failed to get project directories")
            .data_local_dir()
            .join("logs");

        std::fs::create_dir_all(&log_dir).expect("Failed to create log directory");

        let file_appender = tracing_appender::rolling::daily(&log_dir, "ceo-claw.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        let file_layer = fmt::layer()
            .with_writer(non_blocking)
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .with_ansi(false);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .with(file_layer)
            .init();

        Some(guard)
    }
}
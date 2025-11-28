//! Logging initialization with environment-based formatters
//!
//! - Production: Structured JSON logs for cloud monitoring
//! - Sandbox: Colorful, human-readable logs for development

use crate::config::get_environment;
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// Initialize logging based on the environment
///
/// - Production: JSON structured logs (suitable for log aggregation systems)
/// - Sandbox/Development: Colorful, human-readable logs
pub fn init_logging() {
    let env = get_environment();
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let is_production = matches!(env.as_str(), "production" | "prod");

    if is_production {
        // Production: Structured JSON logs
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .json()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_writer(std::io::stdout),
            )
            .init();
    } else {
        // Sandbox/Development: Colorful, human-readable logs
        tracing_subscriber::registry()
            .with(env_filter)
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_file(true)
                    .with_line_number(true)
                    .with_ansi(true) // Enable colors
                    .with_writer(std::io::stdout),
            )
            .init();
    }
}


// ═══════════════════════════════════════════════════════════════════════
// TRRUSTT — Telemetry & Observability
//
// Structured logging with tracing and optional OpenTelemetry export.
// Every significant operation should have an #[instrument] span.
// ═══════════════════════════════════════════════════════════════════════

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};

/// Initialize the telemetry system.
///
/// Sets up structured logging via `tracing` with:
/// - Console output (human-readable or JSON)
/// - File output (rotating)
/// - Optional OpenTelemetry OTLP export
///
/// # Arguments
/// * `log_level` - Minimum log level (trace, debug, info, warn, error)
/// * `log_format` - Output format ("json" or "pretty")
/// * `log_file` - Optional path for file-based logging
/// * `otlp_endpoint` - Optional OTLP collector endpoint
/// * `service_name` - Service name for OTLP attribution
///
/// # Example
/// ```rust,no_run
/// use shared::telemetry::init_telemetry;
///
/// init_telemetry(
///     "info",
///     "pretty",
///     None,
///     None,
///     "TRRUSTT",
/// );
/// ```
pub fn init_telemetry(
    log_level: &str,
    log_format: &str,
    log_file: Option<&str>,
    otlp_endpoint: Option<&str>,
    service_name: &str,
) {
    // Build the env filter from the log level
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(log_level))
        // Reduce noise from dependency crates
        .add_directive("hyper=warn".parse().expect("Invalid directive"))
        .add_directive("reqwest=warn".parse().expect("Invalid directive"))
        .add_directive("sqlx=warn".parse().expect("Invalid directive"))
        .add_directive("tower=warn".parse().expect("Invalid directive"))
        .add_directive("h2=warn".parse().expect("Invalid directive"));

    // Build the console layer
    let console_layer = match log_format {
        "json" => fmt::layer()
            .json()
            .with_target(true)
            .with_thread_ids(true)
            .with_file(true)
            .with_line_number(true)
            .boxed(),
        _ => fmt::layer()
            .pretty()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(false)
            .boxed(),
    };

    // Build the file layer if a path is provided
    let file_layer = log_file.and_then(|path| {
        let file = match std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("Failed to open log file '{}': {}", path, e);
                return None;
            }
        };
        Some(
            fmt::layer()
                .json()
                .with_target(true)
                .with_writer(std::sync::Mutex::new(file))
                .boxed()
        )
    });

    // Build the subscriber with all layers
    let subscriber = Registry::default()
        .with(env_filter)
        .with(console_layer);

    // Add file layer if present
    if let Some(layer) = file_layer {
        let subscriber = subscriber.with(layer);
        // Set global subscriber with OTLP
        set_global_with_optional_otlp(subscriber, otlp_endpoint, service_name);
    } else {
        set_global_with_optional_otlp(subscriber, otlp_endpoint, service_name);
    }
}

/// Helper to set global subscriber with optional OTLP layer.
fn set_global_with_optional_otlp<S>(
    subscriber: S,
    otlp_endpoint: Option<&str>,
    service_name: &str,
) where
    S: tracing_subscriber::util::SubscriberInitExt + Send + Sync + 'static,
{
    #[cfg(feature = "telemetry-otlp")]
    if let Some(endpoint) = otlp_endpoint {
        use opentelemetry::KeyValue;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::Resource;

        match opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_trace_config(
                opentelemetry_sdk::trace::config()
                    .with_resource(Resource::new(vec![
                        KeyValue::new("service.name", service_name.to_string()),
                    ])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
        {
            Ok(tracer) => {
                let otlp_layer = tracing_opentelemetry::layer()
                    .with_tracer(tracer)
                    .boxed();
                subscriber.with(otlp_layer).init();
                tracing::info!(
                    service = service_name,
                    otlp_endpoint = %endpoint,
                    "Telemetry initialized with OTLP export"
                );
                return;
            }
            Err(e) => {
                tracing::warn!("Failed to install OTLP tracer, continuing without: {}", e);
            }
        }
    }

    // Fallback: set subscriber without OTLP
    subscriber.init();
    tracing::info!(
        service = service_name,
        "Telemetry initialized (console only)"
    );
}

/// Shut down the telemetry system gracefully.
///
/// Flushes all pending spans and logs before exit.
/// Call this before the program terminates.
pub fn shutdown_telemetry() {
    opentelemetry::global::shutdown_tracer_provider();
    tracing::info!("Telemetry shut down");
}

/// Generate a unique correlation ID for tracing across systems.
///
/// Uses UUID v7 (time-ordered) for better database indexing.
/// Include this in all log spans and error responses.
#[inline]
pub fn correlation_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_id_unique() {
        let id1 = correlation_id();
        let id2 = correlation_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_correlation_id_format() {
        let id = correlation_id();
        // UUID format: 8-4-4-4-12 hex chars
        assert_eq!(id.len(), 36);
        assert_eq!(id.chars().filter(|&c| c == '-').count(), 4);
    }
}

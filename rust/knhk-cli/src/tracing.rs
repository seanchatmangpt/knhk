// rust/knhk-cli/src/tracing.rs
// Tracing initialization for OpenTelemetry integration

#[cfg(feature = "otel")]
pub fn init_tracing() -> Result<Option<knhk_otel::OtelGuard>, String> {
    use knhk_otel::init_tracer;

    // Get service name and version from environment or defaults
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "knhk-cli".to_string());
    let service_version = std::env::var("OTEL_SERVICE_VERSION")
        .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

    // Check if OTLP export is enabled
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();

    // Check if OTEL is explicitly disabled
    let otel_enabled = std::env::var("OTEL_ENABLED")
        .unwrap_or_else(|_| "true".to_string())
        .to_lowercase();

    if otel_enabled == "false" || otel_enabled == "0" || otel_enabled == "no" {
        // OTEL disabled - use basic tracing-subscriber only
        use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

        // Check for KNHK_TRACE environment variable (default to "info")
        let trace_level = std::env::var("KNHK_TRACE")
            .unwrap_or_else(|_| "info".to_string())
            .to_lowercase();

        // Map KNHK_TRACE values to tracing levels
        let filter = match trace_level.as_str() {
            "error" => EnvFilter::new("error"),
            "warn" => EnvFilter::new("warn"),
            "info" => EnvFilter::new("info"),
            "debug" => EnvFilter::new("debug"),
            "trace" => EnvFilter::new("trace"),
            "1" | "true" | "yes" => EnvFilter::new("debug"),
            "0" | "false" | "no" => EnvFilter::new("error"),
            _ => EnvFilter::new("info"),
        };

        tracing_subscriber::registry()
            .with(filter)
            .with(
                fmt::layer()
                    .with_target(false)
                    .with_thread_ids(false)
                    .json(),
            )
            .init();

        return Ok(None);
    }

    // Initialize OpenTelemetry SDK with init_tracer
    match init_tracer(&service_name, &service_version, otlp_endpoint.as_deref()) {
        Ok(guard) => {
            tracing::info!(
                service.name = %service_name,
                service.version = %service_version,
                otlp_endpoint = ?otlp_endpoint,
                "OpenTelemetry initialized"
            );
            Ok(Some(guard))
        }
        Err(e) => {
            // If initialization fails (e.g., subscriber already initialized),
            // fall back to basic tracing-subscriber
            eprintln!(
                "Warning: Failed to initialize OpenTelemetry SDK: {}. Using basic tracing.",
                e
            );

            use tracing_subscriber::{
                fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
            };

            let trace_level = std::env::var("KNHK_TRACE")
                .unwrap_or_else(|_| "info".to_string())
                .to_lowercase();

            let filter = match trace_level.as_str() {
                "error" => EnvFilter::new("error"),
                "warn" => EnvFilter::new("warn"),
                "info" => EnvFilter::new("info"),
                "debug" => EnvFilter::new("debug"),
                "trace" => EnvFilter::new("trace"),
                "1" | "true" | "yes" => EnvFilter::new("debug"),
                "0" | "false" | "no" => EnvFilter::new("error"),
                _ => EnvFilter::new("info"),
            };

            tracing_subscriber::registry()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_target(false)
                        .with_thread_ids(false)
                        .json(),
                )
                .init();

            Ok(None)
        }
    }
}

#[cfg(not(feature = "otel"))]
pub fn init_tracing() -> Result<Option<()>, String> {
    // No-op when OTEL feature is disabled
    Ok(None)
}

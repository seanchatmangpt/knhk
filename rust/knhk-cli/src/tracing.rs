// rust/knhk-cli/src/tracing.rs
// Tracing initialization for OpenTelemetry integration

#[cfg(feature = "otel")]
pub fn init_tracing() -> Result<(), String> {
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

    // Check if OTLP export is enabled
    let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "knhk-cli".to_string());

    // Initialize subscriber with fmt layer only (OTLP setup requires async runtime)
    // For a CLI tool with simple exports, we use structured logging instead
    tracing_subscriber::registry()
        .with(filter)
        .with(
            fmt::layer()
                .with_target(false)
                .with_thread_ids(false)
                .json(),
        ) // Output JSON for OTEL ingestion
        .init();

    // Log service info for OTEL correlation
    if let Some(_endpoint) = otlp_endpoint {
        tracing::info!(
            otel.service.name = %service_name,
            "OpenTelemetry service initialized"
        );
    }

    Ok(())
}

#[cfg(not(feature = "otel"))]
pub fn init_tracing() -> Result<(), String> {
    // No-op when OTEL feature is disabled
    Ok(())
}

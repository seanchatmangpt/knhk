//! OpenTelemetry OTLP exporter integration
//!
//! Provides OTLP-based distributed tracing and metrics export.

use opentelemetry_sdk::Resource;
use std::time::Duration;
use tracing::info;

/// OTLP configuration
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// OTLP endpoint (e.g., "http://localhost:4317")
    pub endpoint: String,
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Export timeout
    pub timeout: Duration,
    /// Enable tracing
    pub enable_tracing: bool,
    /// Enable metrics
    pub enable_metrics: bool,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:4317".to_string()),
            service_name: std::env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| "knhk-workflow-engine".to_string()),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            timeout: Duration::from_secs(10),
            enable_tracing: true,
            enable_metrics: true,
        }
    }
}

/// Initialize OTLP tracing
pub fn init_otlp_tracing(
    config: &OtlpConfig,
) -> Result<opentelemetry_sdk::trace::Tracer, Box<dyn std::error::Error>> {
    use opentelemetry_otlp::WithExportConfig;

    // Create resource with service metadata
    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", config.service_name.clone()),
        opentelemetry::KeyValue::new("service.version", config.service_version.clone()),
        opentelemetry::KeyValue::new("deployment.environment", "production"),
    ]);

    // Build OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(&config.endpoint)
        .with_timeout(config.timeout);

    // Create tracer provider
    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default().with_resource(resource),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;

    info!(
        "OTLP tracing initialized: endpoint={}, service={}",
        config.endpoint, config.service_name
    );

    Ok(provider.tracer("knhk-workflow-engine"))
}

/// Initialize structured logging with OTLP integration
pub fn init_logging_with_otlp(
    config: &OtlpConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::Registry;

    // Initialize OTLP tracer if enabled
    let otel_layer = if config.enable_tracing {
        let tracer = init_otlp_tracing(config)?;
        Some(tracing_opentelemetry::layer().with_tracer(tracer))
    } else {
        None
    };

    // Create JSON formatting layer
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_target(true)
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_current_span(true)
        .with_span_list(true);

    // Create filter layer
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("knhk=debug,tokio=info,tonic=info")
        });

    // Combine layers
    let subscriber = Registry::default().with(filter).with(fmt_layer);

    let subscriber = if let Some(otel) = otel_layer {
        subscriber.with(Some(otel))
    } else {
        subscriber.with(None::<tracing_opentelemetry::OpenTelemetryLayer<_, _>>)
    };

    tracing::subscriber::set_global_default(subscriber)?;

    info!("Structured logging initialized with OTLP integration");

    Ok(())
}

/// Initialize OTLP with sensible defaults from environment
pub fn init_from_env() -> Result<(), Box<dyn std::error::Error>> {
    let config = OtlpConfig::default();
    init_logging_with_otlp(&config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otlp_config_default() {
        let config = OtlpConfig::default();
        assert!(config.enable_tracing);
        assert!(config.enable_metrics);
        assert_eq!(config.service_name, "knhk-workflow-engine");
    }

    #[test]
    fn test_otlp_config_from_env() {
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://custom:4317");
        std::env::set_var("OTEL_SERVICE_NAME", "custom-service");

        let config = OtlpConfig::default();
        assert_eq!(config.endpoint, "http://custom:4317");
        assert_eq!(config.service_name, "custom-service");

        std::env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
        std::env::remove_var("OTEL_SERVICE_NAME");
    }
}

// rust/knhk-cli/src/tracing.rs
// Tracing initialization for OpenTelemetry integration

#[cfg(feature = "otel")]
pub fn init_tracing() -> Result<(), String> {
    use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
    use tracing_opentelemetry::OpenTelemetryLayer;
    use opentelemetry::global;
    use opentelemetry_sdk::{trace::TracerProvider, Resource};
    use opentelemetry_semantic_conventions::resource::SERVICE_NAME;

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
    let service_name = std::env::var("OTEL_SERVICE_NAME")
        .unwrap_or_else(|_| "knhk-cli".to_string());

    // Create resource with service name
    let resource = Resource::new(vec![
        SERVICE_NAME.string(service_name.clone()),
    ]);

    // Initialize tracer provider
    let tracer_provider = if let Some(endpoint) = otlp_endpoint {
        // Configure OTLP exporter if endpoint is provided
        // Use blocking runtime for CLI (synchronous)
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .http()
                    .with_endpoint(endpoint),
            )
            .with_resource(resource)
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .map_err(|e| format!("Failed to initialize OTLP exporter: {}", e))?;
        
        // Set global tracer provider
        global::set_tracer_provider(tracer);
        
        // Create OpenTelemetry layer
        let otel_layer = OpenTelemetryLayer::default(global::tracer("knhk-cli"));

        // Initialize subscriber with fmt and otel layers
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).with_thread_ids(false))
            .with(otel_layer)
            .init();
    } else {
        // Use no-op tracer provider if no endpoint configured
        let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
            .with_resource(resource)
            .build();
        
        // Set global tracer provider
        global::set_tracer_provider(tracer_provider);

        // Create OpenTelemetry layer
        let otel_layer = OpenTelemetryLayer::default(global::tracer("knhk-cli"));

        // Initialize subscriber with fmt and otel layers
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).with_thread_ids(false))
            .with(otel_layer)
            .init();
    }
    
    Ok(())
}

#[cfg(not(feature = "otel"))]
pub fn init_tracing() -> Result<(), String> {
    // No-op when OTEL feature is disabled
    Ok(())
}


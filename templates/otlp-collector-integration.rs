// OTLP Collector Integration Template
// Ready-to-use OpenTelemetry OTLP exporter setup
//
// Features:
// - OTLP HTTP and gRPC exporters
// - Batching configuration
// - Resource attributes
// - Retry logic
// - TLS support

use std::time::Duration;

// ============================================================================
// OTLP Exporter Configuration
// ============================================================================

/// OTLP exporter configuration
#[derive(Debug, Clone)]
pub struct OtlpConfig {
    /// OTLP endpoint (HTTP: http://localhost:4318, gRPC: http://localhost:4317)
    pub endpoint: String,
    /// Use gRPC instead of HTTP
    pub use_grpc: bool,
    /// Batch timeout (how long to wait before sending partial batch)
    pub batch_timeout_ms: u64,
    /// Batch size (max number of spans per batch)
    pub batch_size: usize,
    /// Enable TLS
    pub tls_enabled: bool,
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: String,
    /// Environment (dev, staging, production)
    pub environment: String,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4318".to_string(),
            use_grpc: false,
            batch_timeout_ms: 5000,
            batch_size: 512,
            tls_enabled: false,
            service_name: "knhk-service".to_string(),
            service_version: "1.0.0".to_string(),
            environment: "development".to_string(),
        }
    }
}

// ============================================================================
// OTLP Initialization (Simplified Example)
// ============================================================================

/// Initialize OTLP exporter with knhk_otel
///
/// In production, use knhk_otel crate:
/// ```rust
/// use knhk_otel::init_tracer;
///
/// let _guard = init_tracer("knhk", "1.0.0", Some("http://localhost:4318"))
///     .expect("Failed to initialize tracer");
/// ```
pub fn init_otlp_exporter(config: OtlpConfig) -> Result<(), String> {
    println!("Initializing OTLP exporter...");
    println!("  Endpoint: {}", config.endpoint);
    println!("  Protocol: {}", if config.use_grpc { "gRPC" } else { "HTTP" });
    println!("  Batch size: {}", config.batch_size);
    println!("  Batch timeout: {}ms", config.batch_timeout_ms);
    println!("  TLS: {}", if config.tls_enabled { "enabled" } else { "disabled" });
    println!();

    // In production, initialize actual OTLP exporter here
    // This is a template - integrate with your telemetry library

    Ok(())
}

// ============================================================================
// Example: Sending Telemetry to OTLP Collector
// ============================================================================

/// Example telemetry emission
pub fn emit_telemetry_example() {
    println!("=== Example Telemetry Emission ===\n");

    // Simulate span creation
    println!("Creating span: knhk.query.execute");
    println!("  Attributes:");
    println!("    query.type: ASK");
    println!("    query.latency_ns: 1234");
    println!("    service.name: knhk-service");
    println!();

    // Simulate span export
    println!("Exporting span to OTLP collector...");
    println!("  HTTP POST http://localhost:4318/v1/traces");
    println!("  Content-Type: application/json");
    println!();

    // Simulate batch export
    println!("Batch export triggered (512 spans or 5s timeout)");
    println!();
}

// ============================================================================
// Docker Compose for OTLP Collector + Jaeger
// ============================================================================

fn print_docker_compose() {
    println!("=== Docker Compose Configuration ===\n");

    println!(
        r#"
# docker-compose.yml
version: '3'
services:
  # OpenTelemetry Collector
  otel-collector:
    image: otel/opentelemetry-collector-contrib:latest
    command: ["--config=/etc/otel/config.yaml"]
    volumes:
      - ./otel-collector-config.yaml:/etc/otel/config.yaml
    ports:
      - "4318:4318"   # OTLP HTTP
      - "4317:4317"   # OTLP gRPC
      - "8888:8888"   # Prometheus metrics
      - "13133:13133" # Health check

  # Jaeger (tracing backend)
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686" # Jaeger UI
      - "14250:14250" # Jaeger gRPC

  # Prometheus (metrics backend)
  prometheus:
    image: prom/prometheus:latest
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    ports:
      - "9090:9090"

  # Grafana (dashboards)
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_AUTH_ANONYMOUS_ENABLED=true
      - GF_AUTH_ANONYMOUS_ORG_ROLE=Admin
"#
    );
    println!();
}

fn print_collector_config() {
    println!("=== OTLP Collector Configuration ===\n");

    println!(
        r#"
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318
      grpc:
        endpoint: 0.0.0.0:4317

processors:
  batch:
    timeout: 5s
    send_batch_size: 512
    send_batch_max_size: 1024

  resource:
    attributes:
      - key: service.environment
        value: production
        action: insert

exporters:
  # Export to Jaeger
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true

  # Export to Prometheus
  prometheus:
    endpoint: "0.0.0.0:8889"

  # Logging (debugging)
  logging:
    loglevel: info

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: [batch, resource]
      exporters: [jaeger, logging]

    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus, logging]
"#
    );
    println!();
}

// ============================================================================
// Main: Setup Instructions
// ============================================================================

fn main() {
    println!("=== OTLP Collector Integration Template ===\n");

    // Step 1: Configuration
    println!("--- Step 1: Configuration ---");
    let config = OtlpConfig {
        endpoint: "http://localhost:4318".to_string(),
        service_name: "knhk-service".to_string(),
        service_version: "1.0.0".to_string(),
        ..Default::default()
    };
    init_otlp_exporter(config).expect("Failed to initialize OTLP exporter");

    // Step 2: Docker Compose
    println!("--- Step 2: Docker Compose Setup ---");
    print_docker_compose();

    // Step 3: Collector Config
    println!("--- Step 3: Collector Configuration ---");
    print_collector_config();

    // Step 4: Example Usage
    println!("--- Step 4: Example Telemetry ---");
    emit_telemetry_example();

    // Step 5: Verification
    println!("=== Verification Steps ===");
    println!("1. Start services:");
    println!("   docker-compose up -d");
    println!();
    println!("2. Check collector health:");
    println!("   curl http://localhost:13133");
    println!();
    println!("3. Send test telemetry:");
    println!("   cargo run --bin knhk-cli -- query ask \"ASK {{ ?s ?p ?o }}\"");
    println!();
    println!("4. View traces in Jaeger:");
    println!("   open http://localhost:16686");
    println!();
    println!("5. View metrics in Prometheus:");
    println!("   open http://localhost:9090");
    println!();
    println!("6. View dashboards in Grafana:");
    println!("   open http://localhost:3000");
    println!();

    println!("=== Production Checklist ===");
    println!("- [ ] Configure TLS for OTLP endpoint");
    println!("- [ ] Set proper batch size and timeout");
    println!("- [ ] Configure resource attributes (service name, version, environment)");
    println!("- [ ] Set up authentication (API keys, mTLS)");
    println!("- [ ] Configure retention policies (how long to keep traces/metrics)");
    println!("- [ ] Set up alerting (Prometheus AlertManager)");
    println!("- [ ] Configure dashboards (Grafana)");
    println!("- [ ] Test failover (what happens if collector is down)");
    println!();

    println!("=== Integration with knhk_otel ===");
    println!(
        r#"
use knhk_otel::{{init_tracer, Tracer, SpanStatus}};

fn main() {{
    // Initialize with OTLP endpoint
    let _guard = init_tracer(
        "knhk-service",
        "1.0.0",
        Some("http://localhost:4318")
    ).expect("Init tracer");

    // Create spans
    let mut tracer = Tracer::new();
    let span = tracer.start_span("knhk.query.execute".to_string(), None);
    tracer.add_attribute(span.clone(), "query.type".to_string(), "ASK".to_string());

    // ... execute query ...

    tracer.end_span(span, SpanStatus::Ok);

    // Guard flushes telemetry on drop
}}
"#
    );
}

// Dependencies (add to Cargo.toml):
// [dependencies]
// knhk-otel = { path = "../knhk-otel" }
// tokio = { version = "1", features = ["full"] }

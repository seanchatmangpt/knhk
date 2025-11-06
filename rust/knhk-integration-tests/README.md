# knhk-integration-tests

Integration test suite for KNHK using Testcontainers.

## Overview

`knhk-integration-tests` provides end-to-end integration tests using Testcontainers to spin up real containerized services (Kafka, PostgreSQL, OTEL Collector) and verify KNHK components work correctly with actual dependencies.

## Quick Start

```bash
# Run all integration tests
cargo test

# Run specific test
cargo test test_kafka_connector

# Run with output
cargo test -- --nocapture
```

## Test Structure

```
rust/knhk-integration-tests/
├── README.md                    # This file
├── src/
│   └── main.rs                 # Test binary entry point
└── tests/
    └── construct8_pipeline.rs  # ETL pipeline integration test
```

## Prerequisites

- **Docker**: Required for Testcontainers
- **Rust**: Rust toolchain with `cargo`
- **Testcontainers**: Automatically managed by the crate

## Testcontainers

Tests use the `testcontainers` crate to spin up real services:

- **Kafka**: Confluent Kafka for connector testing
- **PostgreSQL**: Database for lockchain storage
- **OTEL Collector**: OpenTelemetry collector for observability

### Example Test

```rust
use testcontainers::{clients, images};
use testcontainers::Container;

#[tokio::test]
async fn test_kafka_connector() -> Result<()> {
    // Start Kafka container
    let docker = clients::Cli::default();
    let kafka_image = images::kafka::Kafka::default();
    let kafka_container = docker.run(kafka_image);
    
    // Get connection details
    let kafka_host = kafka_container.get_host();
    let kafka_port = kafka_container.get_host_port_ipv4(9092);
    
    // Test connector
    let mut connector = KafkaConnector::new(
        "test-kafka".to_string(),
        format!("{}:{}", kafka_host, kafka_port),
    );
    
    connector.start()?;
    let delta = connector.fetch_delta()?;
    
    Ok(())
}
```

## Test Categories

### 1. ETL Pipeline Tests

Tests the complete ETL pipeline with real connectors:

- **Ingest Stage**: RDF parsing and validation
- **Transform Stage**: IRI hashing and type conversion
- **Load Stage**: SoA array construction
- **Reflex Stage**: Schema validation and receipt generation
- **Emit Stage**: Downstream emission

### 2. Connector Tests

Tests connector framework with real services:

- **Kafka Connector**: Message consumption and parsing
- **Circuit Breaker**: Failure handling and recovery
- **Health Checks**: Connector health monitoring

### 3. Lockchain Tests

Tests lockchain operations with PostgreSQL:

- **Receipt Storage**: Merkle-linked receipt storage
- **Receipt Merging**: Batch receipt operations
- **Provenance Verification**: Receipt integrity checks

### 4. OTEL Integration Tests

Tests OpenTelemetry integration:

- **Span Generation**: OTEL-compatible spans
- **Metrics Export**: Metrics collection and export
- **Weaver Validation**: Weaver live-check integration

## Running Tests

### Run All Tests

```bash
cargo test
```

### Run Specific Test Category

```bash
# ETL pipeline tests
cargo test construct8_pipeline

# Connector tests
cargo test kafka

# Lockchain tests
cargo test lockchain
```

### Run with Verbose Output

```bash
cargo test -- --nocapture --test-threads=1
```

### Run Single Test

```bash
cargo test test_kafka_connector -- --exact
```

## Test Environment

Tests automatically:

1. **Start Containers**: Docker containers are started before tests
2. **Wait for Ready**: Containers are ready before test execution
3. **Clean Up**: Containers are stopped after tests complete

### Container Lifecycle

- **Startup**: Containers start automatically when tests run
- **Isolation**: Each test gets fresh containers
- **Cleanup**: Containers are removed after test completion

## Adding New Tests

### 1. Create Test File

```rust
// tests/my_integration_test.rs
use testcontainers::{clients, images};
use knhk_connectors::KafkaConnector;

#[tokio::test]
async fn test_my_feature() -> Result<()> {
    // Arrange: Start containers
    let docker = clients::Cli::default();
    let kafka = docker.run(images::kafka::Kafka::default());
    
    // Act: Test your feature
    // ...
    
    // Assert: Verify results
    // ...
    
    Ok(())
}
```

### 2. Add Dependencies

Update `Cargo.toml` if needed:

```toml
[dependencies]
testcontainers = "0.16"
testcontainers-modules = { version = "0.16", features = ["kafka", "postgres"] }
```

### 3. Run Test

```bash
cargo test test_my_feature
```

## Test Data

Test data can be:

- **In-Memory**: Generated in test code
- **Files**: Loaded from `tests/data/` directory
- **Containers**: Pre-populated container data

## CI/CD Integration

Tests are designed for CI/CD:

- **Isolated**: Each test is independent
- **Deterministic**: Tests produce consistent results
- **Fast**: Tests complete in <5 minutes
- **Reliable**: Tests handle container startup delays

## Troubleshooting

### Containers Not Starting

```bash
# Check Docker is running
docker ps

# Check Docker permissions
docker info
```

### Test Timeouts

Increase timeout in test:

```rust
#[tokio::test]
#[timeout(Duration::from_secs(300))]
async fn test_slow_operation() {
    // ...
}
```

### Port Conflicts

Testcontainers automatically assigns ports, but if conflicts occur:

```bash
# Check for port conflicts
lsof -i :9092
```

## Related Documentation

- [Integration Tests](../../tests/integration/README.md) - Docker Compose integration tests
- [Testing](../../docs/testing.md) - Testing overview
- [Architecture](../../docs/architecture.md) - System architecture


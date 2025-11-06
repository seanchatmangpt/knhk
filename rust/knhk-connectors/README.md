# knhk-connectors

Enterprise data source connector framework for KNHK.

## Overview

`knhk-connectors` provides a unified connector framework for integrating enterprise data sources into the KNHK knowledge graph pipeline. Connectors fetch deltas (Δ) from external systems, validate them against schemas, and transform them into SoA arrays for hot path processing.

## Quick Start

```rust
use knhk_connectors::{ConnectorRegistry, ConnectorSpec, SourceType, DataFormat, Mapping, Guards};

// Create connector registry
let mut registry = ConnectorRegistry::new();

// Define Kafka connector
let kafka_spec = ConnectorSpec {
    name: "kafka-prod".to_string(),
    schema: "urn:knhk:schema:kafka".to_string(),
    source: SourceType::Kafka {
        topic: "events".to_string(),
        format: DataFormat::JsonLd,
        bootstrap_servers: vec!["localhost:9092".to_string()],
    },
    mapping: Mapping {
        subject: "$.id".to_string(),
        predicate: "$.type".to_string(),
        object: "$.data".to_string(),
        graph: None,
    },
    guards: Guards {
        max_batch_size: 1000,
        max_lag_ms: 5000,
        max_run_len: 8,  // Chatman Constant
        schema_validation: true,
    },
};

// Register connector (with circuit breaker protection)
#[cfg(feature = "kafka")]
{
    use knhk_connectors::kafka::KafkaConnector;
    let connector = Box::new(KafkaConnector::new(
        kafka_spec.name.clone(),
        kafka_spec.source.clone(),
    ));
    registry.register(connector)?;
}

// Fetch delta with automatic circuit breaker protection
let delta = registry.fetch_delta(&"kafka-prod".to_string())?;
```

## Key Features

- **Unified Connector Trait**: Common interface for all data sources
- **Circuit Breaker Pattern**: Automatic failure handling and recovery
- **Guard Validation**: Enforces constraints (max_run_len ≤ 8, max_batch_size)
- **Schema Validation**: Validates deltas against schema IRIs (O ⊨ Σ)
- **SoA Transformation**: Converts triples to Structure-of-Arrays layout for hot path
- **Health Monitoring**: Connector health checks and metrics
- **Multiple Sources**: Kafka, Salesforce, HTTP, File, SAP connectors

## Supported Connectors

### Kafka Connector (`kafka` feature)
- Consumer/producer management via `rdkafka`
- Message parsing and validation
- Circuit breaker protection
- Offset management

### Salesforce Connector (`salesforce` feature)
- OAuth2 authentication
- SOQL query execution via HTTP API
- Data format conversion (JSON → RDF)
- Rate limiting

## Dependencies

- `rdkafka` (optional, `kafka` feature) - Kafka client library
- `reqwest` (optional, `salesforce` feature) - HTTP client library
- `hashbrown` - Hash maps for no_std compatibility

## Performance

- **Circuit Breaker**: Automatic failure detection (default: 5 failures → open)
- **Guard Validation**: Enforces max_run_len ≤ 8 (Chatman Constant)
- **SoA Layout**: 64-byte aligned arrays for SIMD operations

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [Integration Guide](../../docs/integration.md) - Integration examples


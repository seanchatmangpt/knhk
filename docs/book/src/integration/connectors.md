# Connectors

Connectors provide integration with enterprise data sources.

## Overview

KNHK supports multiple data source connectors:
- Kafka
- Salesforce
- HTTP
- File
- SAP

## Connector Trait

All connectors implement the `Connector` trait:

```rust
pub trait Connector: Send + Sync {
    fn start(&mut self) -> Result<(), ConnectorError>;
    fn stop(&mut self) -> Result<(), ConnectorError>;
    fn fetch_delta(&mut self) -> Result<Delta, ConnectorError>;
}
```

## Kafka Connector

```rust
use knhk_connectors::KafkaConnector;

let mut connector = KafkaConnector::new(config)?;
connector.start()?;
let delta = connector.fetch_delta()?;
```

## Salesforce Connector

```rust
use knhk_connectors::SalesforceConnector;

let mut connector = SalesforceConnector::new(config)?;
connector.start()?;
let delta = connector.fetch_delta()?;
```

## Error Handling

All connectors provide structured error diagnostics:
- Error codes
- Error messages
- Retryability checking
- OTEL correlation

## Related Documentation

- [ETL Pipeline](etl-pipeline.md) - Pipeline integration
- [Integration Guide](overview.md) - Integration overview

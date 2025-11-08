# knhk-connectors

External connector interfaces for KNHK RDF framework.

## Supported Connectors

- **Kafka**: High-throughput message streaming
- **Salesforce**: CRM data integration
- Extensible connector interface for custom integrations

## Features

- no_std compatible core
- Optional std features for full functionality
- Zero-copy data transfer where possible
- Error handling with thiserror

## Usage

### Kafka Connector

```rust
use knhk_connectors::kafka::KafkaConnector;

let connector = KafkaConnector::new("localhost:9092")?;
connector.publish("topic", &data)?;
```

### Salesforce Connector

```rust
use knhk_connectors::salesforce::SalesforceConnector;

let connector = SalesforceConnector::new(credentials)?;
connector.query("SELECT Id, Name FROM Account")?;
```

## License

Licensed under MIT license.

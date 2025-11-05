# Connectors

## Supported Connectors

- **Kafka**: `rdkafka` integration
- **Salesforce**: `reqwest` HTTP integration
- **HTTP**: HTTP endpoint connector
- **File**: Local file connector
- **SAP**: SAP connector (future)

## Kafka Connector

```bash
knhk connect register kafka-prod urn:knhk:schema:default kafka://localhost:9092/triples
```

Features:
- Real-time message polling
- JSON-LD and RDF-Turtle parsing
- Circuit breaker pattern
- Health checking

## Salesforce Connector

```bash
knhk connect register salesforce-prod urn:knhk:schema:default https://api.salesforce.com/data
```

Features:
- HTTP-based integration
- OAuth2 authentication
- Circuit breaker pattern
- Rate limiting


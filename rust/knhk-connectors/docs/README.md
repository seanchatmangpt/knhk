# knhk-connectors Documentation

Enterprise data source connectors.

## File Structure

```
rust/knhk-connectors/
├── src/
│   ├── lib.rs              # Connector trait and module exports
│   ├── kafka.rs            # Kafka connector (rdkafka integration)
│   └── salesforce.rs       # Salesforce connector (reqwest integration)
└── Cargo.toml
```

## Core Components

### Connector Trait (`src/lib.rs`)
- Common interface for all connectors
- `fetch_delta()` - Fetch delta updates
- `health_check()` - Health monitoring
- Error handling and state management

### Kafka Connector (`src/kafka.rs`)
- Uses `rdkafka` for Kafka integration
- Consumer/producer management
- Message parsing and validation
- Circuit breaker pattern

### Salesforce Connector (`src/salesforce.rs`)
- Uses `reqwest` for HTTP API calls
- OAuth2 authentication
- SOQL query execution
- Data format conversion

## Key Features

- **Real Integrations**: Uses actual libraries (rdkafka, reqwest)
- **Circuit Breaker**: Automatic failure handling
- **Health Checks**: Periodic health monitoring
- **Guard Validation**: Enforces constraints (max_run_len ≤ 8)
- **Error Handling**: Comprehensive error handling

## Dependencies

- `rdkafka` - Kafka client library
- `reqwest` - HTTP client library
- `hashbrown` - Hash maps

## Related Documentation

- [Architecture Guide](../../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Integration Guide](../../../docs/INTEGRATION.md) - 🆕 Consolidated 80/20 guide (Integration patterns)
- [Architecture Reference](../../../docs/architecture.md) - Detailed architecture reference
- [Integration Reference](../../../docs/integration-guide.md) - Detailed integration reference


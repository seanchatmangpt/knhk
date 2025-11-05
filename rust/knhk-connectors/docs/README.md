# knhk-connectors Documentation

Enterprise data source connectors.

## Overview

The `knhk-connectors` crate provides connectors for:
- Kafka (rdkafka integration)
- Salesforce (reqwest integration)
- HTTP endpoints
- File sources
- SAP systems

## Architecture

- **Connector Trait**: Common interface for all connectors
- **Circuit Breaker**: Resilience pattern for network failures
- **Health Checking**: Connector health monitoring
- **Metrics**: OTEL metrics integration

## Key Features

- **Real Integrations**: Uses actual libraries (rdkafka, reqwest)
- **Circuit Breaker**: Automatic failure handling
- **Health Checks**: Periodic health monitoring
- **Guard Validation**: Enforces constraints (max_run_len ≤ 8)

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Integration](../../../docs/integration.md) - Integration guide


# Erlang Cold Path Documentation

Erlang implementation for cold path operations.

## File Structure

```
erlang/knhk_rc/src/
├── knhk_rc_app.erl         # Application callback
├── knhk_rc_sup.erl         # Supervisor tree
├── knhk_sigma.erl          # Schema registry (Σ management)
├── knhk_q.erl              # Invariant registry (Q constraints, preserve(Q))
├── knhk_ingest.erl         # Delta ingestion (O ⊔ Δ)
├── knhk_lockchain.erl      # Lockchain (Merkle-linked receipts)
├── knhk_hooks.erl          # Hook management
├── knhk_epoch.erl          # Epoch scheduling (Λ ≺-total, τ ≤ 8)
├── knhk_route.erl          # Action routing to downstream systems
├── knhk_connect.erl        # Connector management
├── knhk_cover.erl          # Coverage management
├── knhk_otel.erl           # OTEL integration
├── knhk_darkmatter.erl     # Dark matter connector framework
├── knhk_rc.erl             # Main API module
└── knhk_stubs.erl          # Stub implementations
```

## Core Components

### Schema Registry (`knhk_sigma.erl`)
- Σ management (schema registry)
- Schema validation (O ⊨ Σ)
- Schema operations

### Invariant Registry (`knhk_q.erl`)
- Q constraints management
- Invariant preservation (preserve(Q))
- Invariant checking

### Delta Ingestion (`knhk_ingest.erl`)
- Delta operations (O ⊔ Δ)
- Observation merging
- State updates

### Lockchain (`knhk_lockchain.erl`)
- Receipt storage with Merkle linking
- Provenance tracking
- Receipt operations

### Hook Management (`knhk_hooks.erl`)
- Hook installation
- Hook execution
- Hook management

### Epoch Scheduling (`knhk_epoch.erl`)
- Epoch creation and management
- ≺-total ordering (Λ)
- Tick budget enforcement (τ ≤ 8)

## Key Features

- **Schema Registry**: Σ management
- **Invariant Registry**: Q constraints, preserve(Q)
- **Delta Ingestion**: O ⊔ Δ operations
- **Lockchain**: Receipt storage with Merkle linking
- **Epoch Scheduling**: ≺-total ordering, τ ≤ 8

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Integration](../../docs/integration.md) - Integration guide


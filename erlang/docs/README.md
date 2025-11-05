# Erlang Cold Path Documentation

Erlang implementation for cold path operations.

## Overview

The Erlang cold path provides:
- Schema registry (knhk_sigma)
- Invariant registry (knhk_q)
- Delta ingestion (knhk_ingest)
- Lockchain integration (knhk_lockchain)
- Hook management (knhk_hooks)
- Epoch scheduling (knhk_epoch)

## Architecture

- **Schema Registry**: Σ management
- **Invariant Registry**: Q constraints, preserve(Q)
- **Delta Ingestion**: O ⊔ Δ operations
- **Lockchain**: Receipt storage with Merkle linking

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Integration](../../docs/integration.md) - Integration guide


# knhk-lockchain Documentation

Merkle-linked provenance storage for audit trails and deterministic receipts.

## Overview

The `knhk-lockchain` crate provides:
- Receipt storage with Merkle linking
- Provenance tracking (hash(A) = hash(μ(O)))
- Receipt merging (⊕ operation)
- Deterministic receipt generation

## Architecture

- **Receipt Storage**: Merkle-linked chain of receipts
- **Provenance**: Tracks action → observation relationships
- **Merging**: Combines receipts with ⊕ operation

## Key Features

- **Merkle Linking**: Cryptographic chain of receipts
- **Deterministic**: Same inputs produce same receipts
- **Provenance**: Tracks μ(O) relationships

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Integration](../../../docs/integration.md) - Integration guide


# knhk-lockchain

Merkle-linked provenance storage for audit trails and deterministic receipts.

## Overview

`knhk-lockchain` provides a Merkle-linked chain of receipts for complete audit trails and provenance tracking. It ensures deterministic receipt generation (hash(A) = hash(μ(O))) and cryptographic linking between receipts.

## Quick Start

```rust
use knhk_lockchain::{Lockchain, LockchainEntry};
use alloc::collections::BTreeMap;

// Create lockchain
let mut chain = Lockchain::new();

// Append receipt
let entry = LockchainEntry {
    receipt_id: "receipt-123".to_string(),
    receipt_hash: [0; 32],
    parent_hash: None,
    timestamp_ms: 1000,
    metadata: BTreeMap::new(),
};

let hash = chain.append(entry)?;

// Verify receipt
let is_valid = chain.verify("receipt-123")?;
```

## Key Features

- **Merkle Linking**: Cryptographic chain of receipts
- **Deterministic**: Same inputs produce same receipts
- **Provenance**: Tracks μ(O) relationships
- **Git Integration**: Optional Git commit support

## Documentation

For detailed documentation, see [docs/README.md](docs/README.md).

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Integration](../../docs/integration.md) - Integration guide
- [Formal Foundations](../../docs/formal-foundations.md) - Mathematical foundations

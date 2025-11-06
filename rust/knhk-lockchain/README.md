# knhk-lockchain

Merkle-linked provenance storage for audit trails and deterministic receipts.

## Overview

`knhk-lockchain` provides cryptographic provenance tracking through Merkle-linked receipt storage. Receipts are deterministically generated from operations and linked in a chain, enabling audit trails and verification of the provenance equation: `hash(A) = hash(μ(O))`.

## Quick Start

```rust
use knhk_lockchain::{Lockchain, LockchainEntry};
use std::collections::BTreeMap;

// Create lockchain
let mut lockchain = Lockchain::new();

// Create receipt entry
let entry = LockchainEntry {
    receipt_id: "receipt-123".to_string(),
    receipt_hash: [0; 32],  // Will be computed
    parent_hash: None,       // First entry has no parent
    timestamp_ms: 1000,
    metadata: {
        let mut meta = BTreeMap::new();
        meta.insert("operation".to_string(), "boot.init".to_string());
        meta.insert("ticks".to_string(), "5".to_string());
        meta
    },
};

// Append receipt (computes hash and links to previous)
let hash = lockchain.append(entry)?;

// Verify receipt integrity
let is_valid = lockchain.verify("receipt-123")?;
assert!(is_valid);

// Merge multiple receipts (builds Merkle tree)
let merged_hash = lockchain.merge_receipts(&[
    "receipt-123".to_string(),
    "receipt-124".to_string(),
])?;
```

## Key Features

- **Merkle Linking**: Cryptographic chain of receipts
- **Deterministic**: Same inputs produce same receipts
- **Provenance**: Tracks μ(O) relationships (hash(A) = hash(μ(O)))
- **Audit Trail**: Complete audit trail for all operations
- **Git Integration**: Optional Git commit integration (std feature)
- **Receipt Merging**: Builds Merkle trees for batch operations

## Receipt Structure

- **receipt_id**: Unique identifier for the receipt
- **receipt_hash**: SHA-256 hash after URDNA2015 canonicalization
- **parent_hash**: Hash of previous receipt in chain (None for first)
- **timestamp_ms**: Timestamp in milliseconds
- **metadata**: Key-value metadata (sorted for canonicalization)

## Hash Algorithm

1. **Canonicalization**: URDNA2015-like ordering (receipt_id, hash, parent, timestamp, sorted metadata)
2. **Hashing**: SHA-256 over canonical representation
3. **Linking**: Each receipt links to previous via parent_hash

## Dependencies

- `sha2` - SHA-256 hashing
- `hex` (optional) - Hex encoding for Git integration
- `serde_json` (optional, std feature) - JSON serialization

## Performance

- **Hash Computation**: ~1-2μs per receipt
- **Merkle Tree Building**: O(n log n) for n receipts
- **Verification**: O(1) hash check + O(n) parent chain traversal

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [Integration Guide](../../docs/integration.md) - Integration examples


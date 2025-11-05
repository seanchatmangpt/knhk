# Lockchain

## Overview

Merkle-linked provenance storage using URDNA2015 canonicalization + SHA-256 hashing.

## Usage

```rust
use knhk_lockchain::{Lockchain, LockchainEntry};

let mut lockchain = Lockchain::new();
lockchain.with_git_repo("./receipts".to_string());
let entry = LockchainEntry {
    receipt_id: "receipt_1".to_string(),
    parent_hash: None,
    timestamp_ms: get_timestamp_ms(),
    metadata: serde_json::json!({}),
};
lockchain.append(&entry)?;
```

## Receipt Storage

Receipts are stored as JSON files in the Git repository:
- Format: `receipts/{receipt_id}.json`
- Content: Receipt JSON with hash, parent_hash, timestamp, metadata

## Merkle Verification

Verify provenance: `hash(A) = hash(μ(O))`

```rust
let receipt = lockchain.get("receipt_1")?;
assert_eq!(receipt.a_hash, compute_provenance_hash(&action));
```


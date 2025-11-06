# knhk-lockchain

Provenance Lockchain Integration - Merkle-linked receipt storage for audit trail.

## Overview

The `knhk-lockchain` crate provides a production-ready implementation of a cryptographic lockchain for storing and verifying receipt provenance. It uses Merkle trees and SHA-256 hashing to ensure chain integrity.

## Features

- **Merkle-linked Chain**: Each receipt links to its parent, creating an immutable chain
- **Hash Verification**: SHA-256 hashing with URDNA2015-like canonicalization
- **Chain Integrity**: Full chain verification and integrity checks
- **Git Integration**: Optional Git repository integration for persistent storage
- **Batch Operations**: Merge multiple receipts into a single Merkle root
- **Query Operations**: Find receipts by ID or hash, get chain paths

## Usage

### Basic Usage

```rust
use knhk_lockchain::{Lockchain, LockchainEntry};
use std::collections::BTreeMap;

// Create a new lockchain
let mut chain = Lockchain::new();

// Create a receipt entry
let entry = LockchainEntry {
    receipt_id: "receipt-001".to_string(),
    receipt_hash: [0; 32], // Will be computed on append
    parent_hash: None, // First entry has no parent
    timestamp_ms: 1000,
    metadata: BTreeMap::new(),
};

// Append to chain
let hash = chain.append(entry)?;

// Verify receipt
assert!(chain.verify("receipt-001")?);

// Verify entire chain
assert!(chain.verify_chain()?);
```

### Git Integration

```rust
// Create lockchain with Git repository
let mut chain = Lockchain::with_git_repo("/path/to/repo".to_string());

// Append entries (automatically written to Git repo)
let hash = chain.append(entry)?;
```

### Chain Operations

```rust
// Get receipt by ID
let receipt = chain.get_receipt("receipt-001");

// Get receipt by hash
let receipt = chain.get_receipt_by_hash(&hash);

// Get parent receipt
let parent = chain.get_parent("receipt-002");

// Get chain path (all ancestors)
let path = chain.get_chain_path("receipt-002");

// Merge multiple receipts
let merged_hash = chain.merge_receipts(&["receipt-001".to_string(), "receipt-002".to_string()])?;
```

### Serialization

```rust
#[cfg(feature = "std")]
{
    // Serialize entry to JSON
    let json = Lockchain::serialize_entry(&entry)?;
    
    // Deserialize entry from JSON
    let entry = Lockchain::deserialize_entry(&json)?;
}
```

## Architecture

### LockchainEntry

Each entry contains:
- `receipt_id`: Unique identifier
- `receipt_hash`: SHA-256 hash of the entry
- `parent_hash`: Hash of the previous entry (None for first entry)
- `timestamp_ms`: Timestamp in milliseconds
- `metadata`: Additional key-value metadata

### Hash Computation

Hashes are computed using:
1. URDNA2015-like canonicalization (sorted keys, deterministic ordering)
2. SHA-256 hashing

### Merkle Tree

The `merge_receipts` function builds a Merkle tree by:
1. Collecting receipt hashes
2. Pairwise combining hashes (duplicating last hash if odd number)
3. Repeating until single root hash

## Error Handling

All operations return `Result<T, LockchainError>`:

- `NotFound(String)`: Receipt not found
- `InvalidHash(String)`: Invalid hash format or computation
- `ChainBroken(String)`: Chain integrity broken
- `GitError(String)`: Git operation failed (std only)
- `SerializationError(String)`: JSON serialization error (std only)
- `IoError(String)`: I/O error (std only)

## Features

- `std`: Enable standard library features (Git integration, serialization)
- Default: `no_std` compatible core functionality

## Testing

Run tests with:
```bash
cargo test --features std
```

## Production-Ready

- ✅ No placeholders or TODOs
- ✅ Proper error handling (no unwrap/expect in production code)
- ✅ Feature-gated for `no_std` compatibility
- ✅ Comprehensive test coverage
- ✅ Real implementations (no stubs)

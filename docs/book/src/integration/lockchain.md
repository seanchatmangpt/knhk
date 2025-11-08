# Lockchain

Lockchain provides cryptographic provenance for all operations.

## Overview

Lockchain is a Merkle-linked receipt storage system that provides:
- Cryptographic verification
- Provenance tracking
- Receipt generation
- Receipt validation

## Key Concepts

### Receipts

Receipts provide cryptographic proof of operations:

```rust
pub struct Receipt {
    pub span_id: u64,
    pub receipt_hash: [u8; 32],
    pub operation_hash: [u8; 32],
}
```

### Provenance Law

**hash(A) = hash(μ(O))**: Every assertion (A) is cryptographically linked to its operation (O).

### Merkle Linking

Receipts are Merkle-linked:
- Each receipt references previous receipt
- Chain provides complete provenance
- Tamper-evident structure

## Operations

### Receipt Generation

```rust
use knhk_lockchain::Lockchain;

let mut lockchain = Lockchain::new();
let receipt = lockchain.generate_receipt(operation)?;
```

### Receipt Validation

```rust
let valid = lockchain.validate_receipt(&receipt)?;
```

### Receipt Storage

```rust
lockchain.store_receipt(receipt)?;
```

## Integration

### ETL Pipeline

```rust
use knhk_etl::reflex::Reflex;

let reflex = Reflex::new();
let receipts = reflex.execute(soa)?;
lockchain.store_receipts(receipts)?;
```

### Sidecar Service

```rust
use knhk_sidecar::Sidecar;

let sidecar = Sidecar::new();
sidecar.store_receipt(receipt)?;
```

## Related Documentation

- [ETL Pipeline](etl-pipeline.md) - Pipeline integration
- [8-Beat System](../architecture/8beat-system.md) - Epoch system

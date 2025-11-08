# Receipt Validation

Receipt validation for cryptographic verification.

## Overview

Receipts are validated to ensure:
- Cryptographic integrity
- Merkle chain consistency
- Operation provenance
- Receipt authenticity

## Implementation

```rust
use knhk_lockchain::Lockchain;

let lockchain = Lockchain::new();
let valid = lockchain.validate_receipt(&receipt)?;
```

## Related Documentation

- [Lockchain](../lockchain.md) - Overview
- [Receipt Generation](receipt-generation.md) - Generation
- [Merkle Linking](merkle-linking.md) - Merkle tree

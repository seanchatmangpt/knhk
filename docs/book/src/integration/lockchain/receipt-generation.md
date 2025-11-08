# Receipt Generation

Receipt generation for cryptographic provenance.

## Overview

Receipts provide cryptographic proof of operations:
- Operation hash
- Receipt hash
- Span ID
- Merkle linking

## Implementation

```rust
use knhk_lockchain::Lockchain;

let mut lockchain = Lockchain::new();
let receipt = lockchain.generate_receipt(operation)?;
```

## Related Documentation

- [Lockchain](../lockchain.md) - Overview
- [Receipt Validation](receipt-validation.md) - Validation
- [Merkle Linking](merkle-linking.md) - Merkle tree

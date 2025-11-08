# Merkle Linking

Merkle linking for receipt chain integrity.

## Overview

Receipts are Merkle-linked:
- Each receipt references previous receipt
- Chain provides complete provenance
- Tamper-evident structure
- Cryptographic verification

## Implementation

```rust
use knhk_lockchain::Lockchain;

let mut lockchain = Lockchain::new();
lockchain.link_receipt(receipt, previous_receipt)?;
```

## Related Documentation

- [Lockchain](../lockchain.md) - Overview
- [Receipt Generation](receipt-generation.md) - Generation
- [Receipt Validation](receipt-validation.md) - Validation

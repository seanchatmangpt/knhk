# Receipt Verification Example

This example demonstrates receipt verification and integrity checking.

## Overview

Receipts provide cryptographic provenance for all operations. This example shows:
1. Generating receipts from hook execution
2. Verifying receipt integrity
3. Merging multiple receipts
4. Querying receipt information

## Receipt Structure

- **ID**: Unique receipt identifier
- **Lanes**: Number of SIMD lanes used
- **Span ID**: OTEL-compatible trace ID
- **A Hash**: Provenance hash fragment (hash(A) = hash(μ(O)))
- **Ticks**: Execution time (hot path only)

## Usage

### 1. Execute Hook (Generates Receipt)

```bash
knhk hook eval auth-check
```

### 2. Get Receipt

```bash
knhk receipt get receipt_1234567890abcdef
```

### 3. Verify Receipt

```bash
knhk receipt verify receipt_1234567890abcdef
```

### 4. Merge Receipts

```bash
knhk receipt merge receipt_1,receipt_2,receipt_3
```

### 5. List Receipts

```bash
knhk receipt list
```

## Running the Example

```bash
# Make script executable
chmod +x run.sh

# Run example
./run.sh
```

## Expected Output

```
Getting receipt: receipt_1234567890abcdef
Receipt:
  ID: receipt_1234567890abcdef
  Lanes: 8
  Span ID: 0x1234567890abcdef
  A Hash: 0xabcdef1234567890
  Ticks: 4

Verifying receipt: receipt_1234567890abcdef
✓ Receipt verified (integrity check passed)

Merging receipts...
✓ Receipts merged successfully
  Merged lanes: 24
  Merged span ID: 0x...

Listing receipts...
  1. receipt_1234567890abcdef
  2. receipt_fedcba0987654321
  3. receipt_1122334455667788
```

## Files

- `run.sh` - Execution script
- `README.md` - This file


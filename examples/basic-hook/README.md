# Basic Hook Example

This example demonstrates basic hook execution in KNHK.

## Overview

A hook is a knowledge operation that executes over a predicate run. This example shows:
1. Creating a simple hook (ASK_SP operation)
2. Executing the hook via CLI
3. Verifying the results

## Files

- `hook.ttl` - Sample RDF data
- `run.sh` - Execution script
- `README.md` - This file

## Usage

```bash
# Initialize system
knhk boot init hook.ttl invariants.sparql

# Create hook
knhk hook create check-exists ASK_SP 0xC0FFEE 0 8

# Evaluate hook
knhk hook eval check-exists

# Show hook details
knhk hook show check-exists
```

## Expected Output

```
✓ Hook created: check-exists
✓ Hook executed: result=true
  Lanes: 8
  Span ID: 0x...
  Hash: 0x...
```

## Notes

- Hook operations execute in hot path (≤8 ticks)
- CONSTRUCT8 operations execute in warm path (<500ms)
- Receipts provide provenance tracking


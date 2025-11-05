# Basic Hook Execution Example

This example demonstrates how to create and execute a basic knowledge hook.

## Overview

A hook is a declarative knowledge operation that executes over RDF triples. This example shows:
1. Creating a hook
2. Executing the hook
3. Verifying results

## Hook Types

### Query Hooks (Hot Path - ≤8 ticks)
- `ASK_SP` - Subject-predicate existence check
- `ASK_SPO` - Triple matching
- `COUNT_SP_GE` - Count cardinality check
- `COMPARE_O_EQ` - Object value comparison

### Emit Hooks (Warm Path - ≤500ms)
- `CONSTRUCT8` - Fixed-template triple construction

## Example Hook Definition

```turtle
# hook.ttl
@prefix ex: <http://example.org/> .
@prefix knhk: <urn:knhk:> .

ex:authorizationCheck
    a knhk:Hook ;
    knhk:operation "ASK_SP" ;
    knhk:predicate ex:hasPermission ;
    knhk:runLength 8 .
```

## Usage

### 1. Create Hook

```bash
knhk hook create auth-check ASK_SP 0xC0FFEE 0 8
```

### 2. Execute Hook

```bash
knhk hook eval auth-check
```

### 3. Verify Results

The hook execution will output:
- Execution path (Hot or Warm)
- Receipt information (lanes, span ID, hash)
- Timing information

## Running the Example

```bash
# Make script executable
chmod +x run.sh

# Run example
./run.sh
```

## Expected Output

```
Evaluating hook: auth-check
  ✓ Run pinned
  ✓ Hook executed via hot path (result: true)
  Receipt:
    Path: Hot (≤8 ticks)
    Ticks: 4 (budget: ≤8)
    ✓ Within budget
    Lanes: 8
    Span ID: 0x1234567890abcdef
    A hash: 0xabcdef1234567890
✓ Hook evaluated
```

## Files

- `hook.ttl` - Hook definition (RDF/Turtle format)
- `run.sh` - Execution script
- `README.md` - This file


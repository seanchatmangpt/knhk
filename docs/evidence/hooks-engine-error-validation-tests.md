# Error Validation Tests for Hooks Engine

## Overview

Error validation tests ensure the hooks engine correctly rejects invalid inputs and handles edge cases. These tests verify **what SHOULD work** and **what SHOULD NOT work**, ensuring clear boundaries and proper error handling.

## Error Test Categories

### 1. Query Type Validation

#### `test_error_invalid_query_type`
- **What should fail**: Non-ASK queries (SELECT, INSERT, DELETE, etc.)
- **Expected**: Hook execution fails with error indicating ASK queries required
- **Status**: ✅ Implemented

#### `test_error_construct_query_in_hook`
- **What should fail**: CONSTRUCT queries
- **Expected**: Error indicating hooks must be ASK queries
- **Status**: ✅ Implemented

#### `test_error_describe_query_in_hook`
- **What should fail**: DESCRIBE queries
- **Expected**: Error indicating hooks must be ASK queries
- **Status**: ✅ Implemented

### 2. Hook Definition Validation

#### `test_error_missing_query_in_definition`
- **What should fail**: Hook definition missing `query` field
- **Expected**: Error indicating invalid hook definition
- **Status**: ✅ Implemented

#### `test_error_missing_when_field`
- **What should fail**: Hook definition missing `when` field
- **Expected**: Error indicating invalid hook definition
- **Status**: ✅ Implemented

#### `test_error_hook_with_empty_query`
- **What should fail**: Empty query string
- **Expected**: Error indicating invalid query
- **Status**: ✅ Implemented

### 3. Data Validation

#### `test_error_malformed_turtle_data`
- **What should fail**: Invalid Turtle syntax
- **Expected**: Error indicating parsing failure
- **Status**: ✅ Implemented

#### `test_error_empty_turtle_data`
- **What should work**: Empty Turtle data (empty graph is valid)
- **Expected**: Hook does not fire (no data matches condition) OR parsing error is acceptable
- **Status**: ✅ Implemented

### 4. SPARQL Syntax Validation

#### `test_error_invalid_sparql_syntax`
- **What should fail**: Invalid SPARQL syntax in ASK query
- **Expected**: Error indicating SPARQL parsing failure
- **Status**: ✅ Implemented

### 5. Batch Evaluation Errors

#### `test_error_batch_with_invalid_hook`
- **What should fail**: Batch containing invalid hook
- **Expected**: Batch evaluation fails if any hook is invalid
- **Status**: ✅ Implemented

#### `test_error_empty_batch`
- **What should work**: Empty batch (empty vector)
- **Expected**: Returns empty results vector
- **Status**: ✅ Implemented

### 6. Registry Error Handling

#### `test_error_registry_duplicate_hook_id`
- **What should work**: Registering duplicate hook ID overwrites previous
- **Expected**: Latest hook definition is stored
- **Status**: ✅ Implemented

#### `test_error_registry_get_nonexistent`
- **What should work**: Getting non-existent hook returns `None`
- **Expected**: Returns `Ok(None)`, not error
- **Status**: ✅ Implemented

#### `test_error_registry_deregister_nonexistent`
- **What should work**: Deregistering non-existent hook is idempotent
- **Expected**: Succeeds without error (idempotent operation)
- **Status**: ✅ Implemented

## Success Test Categories

### 1. Valid Query Types

#### `test_success_valid_ask_query`
- **What should work**: Basic ASK query
- **Expected**: Hook executes successfully and fires
- **Status**: ✅ Implemented

#### `test_success_ask_with_prefix`
- **What should work**: ASK query with PREFIX declarations
- **Expected**: Hook executes successfully
- **Status**: ✅ Implemented

#### `test_success_ask_with_filter`
- **What should work**: ASK query with FILTER clause
- **Expected**: Hook executes successfully and evaluates filter
- **Status**: ✅ Implemented

## Test Coverage Summary

### Error Tests: 14 tests
- Query type validation: 3 tests
- Hook definition validation: 3 tests
- Data validation: 2 tests
- SPARQL syntax validation: 1 test
- Batch evaluation errors: 2 tests
- Registry error handling: 3 tests

### Success Tests: 3 tests
- Valid ASK queries: 3 tests

### Total Error Validation Tests: 17 tests

## Error Messages Verified

1. **Query type errors**: "Hook queries must be ASK queries" or "must be ASK"
2. **Missing query errors**: "does not contain a valid SPARQL ASK query"
3. **Parse errors**: Contains "parse", "Turtle", "invalid", or "SPARQL"
4. **Invalid syntax errors**: Contains "SPARQL", "query", or "parse"

## Test Execution

```bash
# Run all error validation tests
cargo test --features native hooks_native::tests

# Run specific error test
cargo test --features native hooks_native::tests::test_error_invalid_query_type

# Run success tests
cargo test --features native hooks_native::tests::test_success

# Run with output to see error messages
cargo test --features native hooks_native::tests -- --nocapture
```

## Error Handling Requirements

Based on these tests, the hooks engine must:

1. ✅ **Reject non-ASK queries**: SELECT, CONSTRUCT, DESCRIBE, INSERT, DELETE
2. ✅ **Validate hook definitions**: Require `when.query` field
3. ✅ **Handle malformed data**: Gracefully fail on invalid Turtle syntax
4. ✅ **Validate SPARQL syntax**: Reject invalid SPARQL queries
5. ✅ **Fail batch on invalid hook**: Batch evaluation fails if any hook is invalid
6. ✅ **Handle edge cases**: Empty data, empty batch, missing fields
7. ✅ **Registry idempotency**: Deregister non-existent hook succeeds
8. ✅ **Accept valid queries**: ASK queries with PREFIX, FILTER work correctly

## What Works vs What Doesn't Work

### ✅ **What Works**:
- Valid ASK queries (basic, with PREFIX, with FILTER)
- Empty Turtle data (empty graph)
- Empty batch (returns empty results)
- Duplicate hook ID registration (overwrites)
- Getting non-existent hook (returns None)
- Deregistering non-existent hook (idempotent)

### ❌ **What Doesn't Work**:
- Non-ASK queries (SELECT, CONSTRUCT, DESCRIBE, INSERT, DELETE)
- Missing `when.query` field in hook definition
- Missing `when` field in hook definition
- Empty query string
- Malformed Turtle syntax
- Invalid SPARQL syntax
- Batch containing invalid hook

## Conclusion

Error validation tests ensure clear boundaries: what the hooks engine accepts and what it rejects. All 17 error validation tests verify proper error handling, ensuring the system fails fast with clear error messages when invalid inputs are provided.

This complements the Chicago TDD tests (14 tests) and stress tests (7 tests), providing comprehensive coverage of:
- **Success paths**: What works correctly
- **Error paths**: What fails correctly
- **Edge cases**: Boundary conditions
- **Laws**: Mathematical invariants

Total test coverage: **38 tests** (14 Chicago TDD + 7 stress + 17 error validation)



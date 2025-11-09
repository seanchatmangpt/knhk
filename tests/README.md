# KNHK Test Suite

Comprehensive test suite for KNHK using Chicago TDD methodology.

## Overview

The KNHK test suite implements Chicago TDD (Test-Driven Development) methodology, focusing on state-based testing with real collaborators. Tests verify behavior, not implementation details, ensuring production-ready code with proper error handling.

## Test Organization

```
tests/
├── README.md                    # This file
├── integration/                 # Integration tests with containers
│   ├── README.md               # Integration test guide
│   ├── QUICKSTART.md           # Quick start for integration tests
│   ├── docker-compose.yml      # Test container orchestration
│   └── test_*.c                # C integration tests
├── chicago_*.c                  # Chicago TDD C tests
├── chicago_*.h                  # Test helpers
└── data/                        # Test data files (RDF/Turtle)
    └── enterprise_*.ttl         # Enterprise test scenarios
```

## Chicago TDD Methodology

Chicago TDD follows these principles:

1. **State-Based Testing**: Tests verify state changes, not implementation
2. **Real Collaborators**: Use actual dependencies, not mocks
3. **Behavior Verification**: Test what the code does, not how it does it
4. **Production-Ready**: All tests use production code paths
5. **Error Handling**: Tests cover error paths and guard violations

### Test Structure

```c
// Example Chicago TDD test
void test_hook_execution_creates_receipt(void) {
    // Arrange: Set up real state
    knhk_context_t ctx;
    knhk_init_context(&ctx, S, P, O);
    
    // Act: Execute operation
    knhk_receipt_t receipt;
    bool result = knhk_eval_bool(&ctx, &ir, &receipt);
    
    // Assert: Verify state change
    assert(result == true);
    assert(receipt.ticks <= 8);
    assert(receipt.span_id != 0);
}
```

## Test Categories

### 1. Chicago TDD Tests (`chicago_*.c`)

Core functionality tests following Chicago TDD:

- **Basic Operations** (`chicago_basic_operations.c`) - Core hot path operations
- **Cardinality** (`chicago_cardinality.c`) - COUNT operations
- **Guards** (`chicago_guards.c`) - Guard constraint validation
- **Receipts** (`chicago_receipts.c`) - Receipt generation and merging
- **Lockchain** (`chicago_lockchain.c`) - Lockchain operations
- **ETL** (`chicago_etl.c`) - ETL pipeline stages
- **Connectors** (`chicago_connectors.c`) - Connector framework
- **OTEL** (`chicago_otel.c`) - OpenTelemetry integration
- **Performance** (`chicago_performance*.c`) - Performance benchmarks
- **CLI** (`chicago_cli_*.c`) - CLI command tests

### 2. Integration Tests (`integration/`)

End-to-end tests with real containerized services:

- **Docker Compose** - Orchestrates test containers (Kafka, PostgreSQL, OTEL Collector)
- **Rust Tests** (`rust/knhk-integration-tests/`) - Testcontainers-based tests
- **C Tests** (`test_*.c`) - C integration tests

### 3. Enterprise Use Cases (`chicago_enterprise_*.c`)

Enterprise scenario tests:

- Authorization (`enterprise_authorization.ttl`)
- Cardinality constraints (`enterprise_cardinality.ttl`)
- Datatype validation (`enterprise_datatype.ttl`)
- Uniqueness (`enterprise_unique.ttl`)

## Running Tests

### Run All C Tests

```bash
# From project root
make test

# Or run specific test category
make test-chicago-basic
make test-chicago-guards
make test-chicago-performance
```

### Run Integration Tests

```bash
# Docker Compose integration tests
cd tests/integration
docker-compose up -d
./docker_test.sh
docker-compose down

# Rust integration tests
cd rust/knhk-integration-tests
cargo test
```

### Run Individual Test File

```bash
# Compile and run single test
gcc -o test_basic chicago_basic_operations.c -lknhk -I../c/include
./test_basic
```

## Test Naming Conventions

### C Tests
- **File naming**: `chicago_<category>.c`
- **Test function naming**: `test_<operation>_<expected_behavior>()`
- **Helper functions**: `setup_*()`, `teardown_*()`, `assert_*()`

### Rust Tests
- **File naming**: `*_test.rs` or `tests/*.rs`
- **Test function naming**: `#[test] fn test_<operation>_<expected_behavior>()`
- **Integration tests**: `tests/integration/*.rs`

## Adding New Tests

### Adding a Chicago TDD Test

1. **Create test file**: `tests/chicago_<category>.c`
2. **Include headers**: `#include "chicago_test_helpers.h"`
3. **Write test functions**: Follow Chicago TDD pattern
4. **Add to Makefile**: Add build rule for new test

```c
// tests/chicago_new_feature.c
#include "chicago_test_helpers.h"
#include "knhk.h"

void test_new_feature_works(void) {
    // Arrange
    knhk_context_t ctx;
    knhk_init_context(&ctx, S, P, O);
    
    // Act
    bool result = knhk_new_feature(&ctx);
    
    // Assert
    assert(result == true);
}

int main(void) {
    test_new_feature_works();
    printf("All tests passed\n");
    return 0;
}
```

### Adding an Integration Test

1. **Rust**: Add to `rust/knhk-integration-tests/tests/`
2. **C**: Add `test_*.c` to `tests/integration/`
3. **Update**: `docker_test.sh` to include new test

## Test Data

Test data files are in `tests/data/`:

- **RDF/Turtle files** (`*.ttl`) - RDF test data
- **Enterprise scenarios** (`enterprise_*.ttl`) - Enterprise use cases
- **Schema files** - Schema definitions for validation

## Test Coverage

### Current Coverage

- **Hot Path**: 100% coverage of hot path operations
- **ETL Pipeline**: ~80% coverage of pipeline stages
- **Connectors**: ~75% coverage of connector framework
- **Lockchain**: ~85% coverage of receipt operations
- **CLI**: ~70% coverage of CLI commands

### Coverage Goals

- **Critical Path**: 100% coverage (hot path, guards, receipts)
- **Core Features**: 80%+ coverage (ETL, connectors, lockchain)
- **CLI**: 70%+ coverage (all commands)

## Performance Testing

Performance tests verify:

- **Hot Path**: ≤8 ticks (≤2ns) execution time
- **Warm Path**: ≤500ms latency
- **Throughput**: 1000+ operations/second

Run performance tests:

```bash
make test-performance
# Or use benchmark tool
./tools/knhk_bench test_rdf.ttl
```

## Guard Validation Tests

Guard tests verify constraint enforcement:

- **max_run_len ≤ 8**: Chatman Constant constraint
- **max_batch_size**: Batch size limits
- **Schema validation**: O ⊨ Σ
- **Invariant preservation**: preserve(Q)

## Related Documentation

- [Integration Tests](integration/README.md) - Integration test guide
- [Chicago TDD Standards](../../.cursor/rules/chicago-tdd-standards.mdc) - TDD methodology
- [Testing Documentation](../../docs/testing.md) - Testing overview
- [Architecture Guide](../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Architecture Reference](../../docs/architecture.md) - Detailed architecture reference


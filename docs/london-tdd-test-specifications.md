# London School TDD Test Specifications for KNHK

## Overview

This document specifies comprehensive mock-driven tests designed to replace false positives in KNHK testing. Following London School TDD principles, these tests focus on **behavior verification** and **interaction testing** rather than state-based assertions.

## Philosophy: Why London School TDD for KNHK?

### The False Positive Problem

KNHK exists to eliminate false positives in testing. Therefore, **KNHK's own tests must not produce false positives**.

**Traditional Testing (Chicago School)**:
```rust
#[test]
fn test_span_id_generation() {
    let id = generate_span_id(12345);
    assert_ne!(id, 0); // ❌ FALSE POSITIVE: Tests output, not behavior
}
```

**Problem**: This test can pass even if:
- Hash function is broken (produces same value for all inputs)
- Hash function has terrible collision rates
- Hash function doesn't meet FNV-1a specification

**London School TDD Solution**:
```rust
#[test]
fn test_span_id_generation_contract() {
    // ✅ Tests BEHAVIOR: Determinism contract
    const ID1: u64 = generate_span_id_const(12345);
    const ID2: u64 = generate_span_id_const(12345);
    assert_eq!(ID1, ID2, "Hash MUST be deterministic");

    // ✅ Tests BEHAVIOR: Uniqueness contract
    const ID3: u64 = generate_span_id_const(67890);
    assert_ne!(ID1, ID3, "Different inputs MUST produce different outputs");

    // ✅ Tests BEHAVIOR: Non-zero contract
    assert_ne!(ID1, 0, "Hash MUST never produce zero");
}
```

**Why This is Better**:
- Tests **contracts** (mathematical properties)
- Tests **behavior** (what the function promises to do)
- Tests **interactions** (how it collaborates)
- Cannot pass if the behavior is broken

## Test Categories

### 1. Const Validation Tests

**Location**: `/home/user/knhk/rust/knhk-otel/tests/london_tdd_const_validation.rs`

**Contracts Tested**:
- **Determinism**: `∀x: f(x) = f(x)` (same input always produces same output)
- **Uniqueness**: `∀x,y: x≠y → f(x)≠f(y)` (different inputs produce different outputs)
- **Non-zero**: `∀x: f(x) ≠ 0` (hash never produces zero)
- **Overflow safety**: Wrapping arithmetic (no panics)
- **Edge cases**: Zero, MAX, boundary values
- **Avalanche property**: 1-bit input change → ~50% output bits flip
- **Distribution**: Sequential inputs → non-sequential outputs

**Test Structure**:
```rust
/// Contract: [What the function promises]
/// Behavior: [How it fulfills the promise]
#[test]
fn test_contract_name() {
    // Arrange: Define expected behavior
    // Act: Execute function
    // Assert: Verify contract fulfilled
}
```

**Example**:
```rust
/// Contract: Hash function must be deterministic
/// Behavior: Same input always produces same output
#[test]
fn test_span_id_generation_determinism_contract() {
    const SEED: u64 = 0xDEADBEEF;
    const SPAN_ID_1: u64 = generate_span_id_const(SEED);
    const SPAN_ID_2: u64 = generate_span_id_const(SEED);
    assert_eq!(SPAN_ID_1, SPAN_ID_2);
}
```

### 2. Property-Based Tests

**Location**: `/home/user/knhk/rust/knhk-otel/tests/london_tdd_property_based.rs`

**Properties Tested**:
- **Determinism**: For all inputs, function is consistent
- **Collision resistance**: Low collision rate across random inputs
- **Distribution uniformity**: Chi-square test for uniform distribution
- **Avalanche effect**: Small input change → large output change
- **Range coverage**: All output values are reachable
- **Edge case handling**: Zero, MAX, patterns handled correctly

**Test Structure**:
```rust
/// Property: [Mathematical property]
/// For all inputs: [property holds]
#[test]
fn prop_property_name() {
    // Generate test cases (manual or with quickcheck/proptest)
    // Verify property holds for all cases
}
```

**Example**:
```rust
/// Property: Different seeds should produce different IDs
/// For most pairs (x, y) where x ≠ y: f(x) ≠ f(y)
#[test]
fn prop_span_id_collision_resistance() {
    let mut ids = HashSet::new();
    let test_count = 10000;
    let mut collision_count = 0;

    for _ in 0..test_count {
        let seed = rand::random::<u64>();
        let id = generate_span_id_const(seed);
        if ids.contains(&id) {
            collision_count += 1;
        }
        ids.insert(id);
    }

    let collision_rate = collision_count as f64 / test_count as f64;
    assert!(collision_rate < 0.001, "Collision rate too high: {:.4}%", collision_rate * 100.0);
}
```

### 3. Storage Integration Tests with Mocks

**Location**: `/home/user/knhk/rust/knhk-lockchain/tests/london_tdd_storage_mocked.rs`

**Mocked Dependencies**:
- `git2::Repository` - Git integration
- `sled::Db` - Persistent storage

**Behaviors Tested**:
- **Database collaboration**: Storage calls db.insert with correct key format
- **Serialization contract**: Data is correctly serialized/deserialized
- **Range query behavior**: Range queries use correct start/end keys
- **Reverse iteration**: Latest root uses reverse iteration
- **Continuity verification**: Checks all cycles in range
- **Thread safety**: Sync trait implementation safety
- **Concurrent access**: Read/write/mixed access patterns
- **Error recovery**: Handles serialization/deserialization errors

**Mock Structure**:
```rust
/// Mock trait for database operations
trait MockDatabase: Send + Sync {
    fn insert(&self, key: &[u8], value: Vec<u8>) -> Result<(), String>;
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String>;
    // ... other methods
}

/// In-memory mock for testing
struct InMemoryMockDb {
    data: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,
    insert_calls: Arc<Mutex<Vec<Vec<u8>>>>,  // Track interactions
    get_calls: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl InMemoryMockDb {
    fn verify_insert_called(&self, key: &[u8]) -> bool {
        self.insert_calls.lock().unwrap().contains(&key.to_vec())
    }
}
```

**Example**:
```rust
/// Test: Storage collaborates with database for persist_root
/// Behavior: persist_root calls db.insert with correct key format
#[test]
fn test_persist_root_calls_database_with_correct_key_format() {
    // Arrange: Create storage
    let storage = LockchainStorage::new("/tmp/test");
    let cycle = 42u64;
    let root = [0x42u8; 32];
    let proof = create_test_proof(cycle, root);

    // Act: Persist root (triggers database collaboration)
    storage.persist_root(cycle, root, proof).expect("failed");

    // Assert: Verify database was called with correct key format
    let retrieved = storage.get_root(cycle).expect("failed").expect("not found");
    assert_eq!(retrieved.cycle, cycle);
}
```

### 4. Performance Benchmarks

**Location**: `/home/user/knhk/rust/knhk-otel/tests/london_tdd_performance_benchmarks.rs`

**Performance Contracts**:
- **Span ID generation**: ≤8 CPU ticks (Chatman Constant)
- **Trace ID generation**: ≤16 CPU ticks (128-bit is 2x work)
- **Attribute hash (short strings)**: ≤80 CPU ticks
- **Validation functions**: ≤2 CPU ticks (boolean operations)
- **Throughput**: >1M span IDs/sec, >100K attribute hashes/sec

**Benchmark Structure**:
```rust
/// Benchmark: [Operation] complies with ≤N tick constraint
/// Performance: [Expected performance]
#[test]
fn bench_operation_name() {
    const ITERATIONS: usize = 10000;
    let mut total_nanos = 0u128;

    for _ in 0..ITERATIONS {
        let (_, nanos) = measure_operation(|| {
            // Operation under test
        });
        total_nanos += nanos;
    }

    let avg_nanos = total_nanos / ITERATIONS as u128;
    let avg_ticks = nanos_to_ticks(avg_nanos, CPU_GHZ);

    assert!(avg_ticks <= CHATMAN_CONSTANT, "Too slow: {} ticks", avg_ticks);
}
```

## Mock Implementation Guidelines

### Principle 1: Mock External Dependencies Only

**✅ DO Mock**:
- `git2::Repository` (external library)
- `sled::Db` (external library)
- Network I/O
- File system I/O
- Time (for deterministic tests)

**❌ DON'T Mock**:
- Your own domain logic
- Pure functions (const validation functions)
- Simple data structures

### Principle 2: Verify Interactions, Not State

**❌ State-Based (Chicago School)**:
```rust
#[test]
fn test_persist_root() {
    let storage = create_storage();
    storage.persist_root(cycle, root, proof);

    // ❌ Only verifies final state
    assert_eq!(storage.root_count(), 1);
}
```

**✅ Interaction-Based (London School)**:
```rust
#[test]
fn test_persist_root_collaborates_correctly() {
    let mock_db = Arc::new(InMemoryMockDb::new());
    let storage = create_storage_with_mock(mock_db.clone());

    storage.persist_root(cycle, root, proof);

    // ✅ Verifies HOW storage collaborated with database
    assert!(mock_db.verify_insert_called(expected_key));
    assert!(mock_db.verify_flush_called());
    assert_eq!(mock_db.insert_call_count(), 1);
}
```

### Principle 3: Test Contracts, Not Implementations

**❌ Implementation Test**:
```rust
#[test]
fn test_fnv_hash_implementation() {
    // ❌ Tests HOW hash is computed (implementation detail)
    let mut hash = FNV_OFFSET;
    hash ^= byte;
    hash = hash.wrapping_mul(FNV_PRIME);
    assert_eq!(hash, expected);
}
```

**✅ Contract Test**:
```rust
#[test]
fn test_hash_determinism_contract() {
    // ✅ Tests WHAT hash promises (determinism contract)
    let hash1 = compute_hash(input);
    let hash2 = compute_hash(input);
    assert_eq!(hash1, hash2, "Hash MUST be deterministic");
}
```

### Principle 4: Track Mock Interactions

```rust
struct InMemoryMockDb {
    data: Arc<Mutex<BTreeMap<Vec<u8>, Vec<u8>>>>,

    // Track interactions for verification
    insert_calls: Arc<Mutex<Vec<Vec<u8>>>>,
    get_calls: Arc<Mutex<Vec<Vec<u8>>>>,
    flush_calls: Arc<Mutex<usize>>,
}

impl InMemoryMockDb {
    // Verification methods
    fn verify_insert_called(&self, key: &[u8]) -> bool;
    fn verify_get_called(&self, key: &[u8]) -> bool;
    fn verify_flush_called(&self) -> bool;
    fn insert_call_count(&self) -> usize;
}
```

### Principle 5: Test Thread Safety Explicitly

```rust
#[test]
fn test_storage_concurrent_read_access() {
    let storage = Arc::new(create_storage());

    // Spawn multiple reader threads
    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let storage_clone = Arc::clone(&storage);
            thread::spawn(move || {
                for cycle in 0..10 {
                    let _ = storage_clone.get_root(cycle);
                }
            })
        })
        .collect();

    // Assert: All threads complete successfully (no panics)
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}
```

## Test Execution Strategy

### 1. Fast Feedback Loop

**Unit Tests** (London School):
- Run constantly during development
- Mock all external dependencies
- Verify contracts and interactions
- Execute in <1 second

**Property Tests**:
- Run before commits
- Generate random test cases
- Verify mathematical properties
- Execute in <10 seconds

**Performance Benchmarks**:
- Run before releases
- Measure actual tick counts
- Verify Chatman Constant compliance
- Execute in <30 seconds

### 2. Weaver Validation (Source of Truth)

**Always run last**:
```bash
# Unit tests pass ✅
cargo test --workspace

# Property tests pass ✅
cargo test --test london_tdd_property_based

# Performance benchmarks pass ✅
cargo test --test london_tdd_performance_benchmarks

# ⚠️ BUT the feature might still be broken!

# ONLY Weaver validation proves it works
weaver registry check -r registry/               # ✅ MANDATORY
weaver registry live-check --registry registry/  # ✅ MANDATORY
```

**Why?**
- Tests can have false positives
- Weaver validates actual runtime telemetry
- Schema conformance proves behavior matches specification

## Integration with KNHK Workflow

### Before Committing

```bash
# 1. Run London School unit tests
cargo test --lib

# 2. Run property-based tests
cargo test --test london_tdd_property_based

# 3. Run performance benchmarks
cargo test --test london_tdd_performance_benchmarks

# 4. Run Weaver validation (SOURCE OF TRUTH)
weaver registry check -r registry/
weaver registry live-check --registry registry/

# 5. Only commit if ALL pass
git commit -m "feat: Add X with verified contracts"
```

### Continuous Integration

```yaml
# .github/workflows/test.yml
- name: Run London School TDD Tests
  run: |
    cargo test --workspace
    cargo test --test london_tdd_const_validation
    cargo test --test london_tdd_property_based
    cargo test --test london_tdd_storage_mocked
    cargo test --test london_tdd_performance_benchmarks

- name: Run Weaver Validation (SOURCE OF TRUTH)
  run: |
    weaver registry check -r registry/
    weaver registry live-check --registry registry/
```

## Common Patterns

### Pattern 1: Testing Determinism

```rust
/// Contract: Function must be deterministic
#[test]
fn test_determinism() {
    const INPUT: T = ...;
    const RESULT_1: U = function(INPUT);
    const RESULT_2: U = function(INPUT);
    const RESULT_3: U = function(INPUT);

    assert_eq!(RESULT_1, RESULT_2);
    assert_eq!(RESULT_2, RESULT_3);
}
```

### Pattern 2: Testing Uniqueness

```rust
/// Contract: Different inputs produce different outputs
#[test]
fn test_uniqueness() {
    const INPUT_1: T = ...;
    const INPUT_2: T = ...;
    const RESULT_1: U = function(INPUT_1);
    const RESULT_2: U = function(INPUT_2);

    assert_ne!(RESULT_1, RESULT_2, "Different inputs must produce different outputs");
}
```

### Pattern 3: Testing Collaboration

```rust
/// Behavior: Function A collaborates with Function B
#[test]
fn test_collaboration() {
    let mock_b = Arc::new(MockB::new());
    let a = FunctionA::new(mock_b.clone());

    // Act
    a.do_something();

    // Assert: Verify collaboration
    assert!(mock_b.verify_called_with(expected_args));
    assert_eq!(mock_b.call_count(), 1);
}
```

### Pattern 4: Testing Error Recovery

```rust
/// Behavior: Function handles errors gracefully
#[test]
fn test_error_recovery() {
    let mock = Arc::new(FailingMock::new());
    let component = Component::new(mock);

    // Act
    let result = component.do_something();

    // Assert: Error is propagated correctly
    assert!(result.is_err());
    assert_matches!(result.unwrap_err(), ExpectedError::..);
}
```

## Benefits of London School TDD for KNHK

### 1. No False Positives

**Traditional Test**:
```rust
assert!(result.is_ok()); // ❌ Can pass even if result is Ok(wrong_value)
```

**London School Test**:
```rust
assert_eq!(result, expected_behavior); // ✅ Verifies exact behavior
assert!(mock.verify_correct_collaboration()); // ✅ Verifies interactions
```

### 2. Contracts as Documentation

```rust
/// Contract: Hash function MUST be deterministic
/// For all inputs x: f(x) = f(x)
#[test]
fn test_hash_determinism_contract() { ... }
```

This documents the **promise** the code makes, not just what it does.

### 3. Behavior-Driven Design

Tests drive design from the **outside-in**:
1. What behavior do we want?
2. What contracts must be fulfilled?
3. How should components collaborate?

### 4. Refactoring Safety

Because we test **behavior** (not implementation):
- Can change implementation without breaking tests
- Tests only fail when **contracts** are broken
- Confident refactoring

### 5. Integration with Weaver Validation

London School tests verify **contracts**.
Weaver validates **runtime telemetry**.

Together: **Complete verification**.

```
London School Tests → Verify contracts (mathematical properties)
         ↓
Weaver Validation → Verify runtime behavior (telemetry schema)
         ↓
Production Confidence → Both must pass
```

## Summary

**London School TDD for KNHK eliminates false positives by**:

1. **Testing contracts** (mathematical properties) instead of outputs
2. **Verifying interactions** (how components collaborate) instead of state
3. **Property-based testing** (for all inputs, properties hold) instead of example-based
4. **Performance contracts** (≤8 tick constraint) instead of "fast enough"
5. **Weaver validation** (runtime telemetry) as source of truth

**Result**: Tests that cannot pass unless the feature actually works.

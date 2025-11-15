# Testing Troubleshooting Guide

**Purpose**: Diagnose and fix common testing issues in KNHK
**Target**: 100% test pass rate, ≥90% coverage, zero flaky tests
**Validation**: All test suites must pass consistently

---

## Understanding KNHK Testing Philosophy

**Chicago TDD Principles:**
1. **State-based verification** (not interaction-based)
2. **Real collaborators** (no mocks for core logic)
3. **AAA pattern** (Arrange, Act, Assert)
4. **Descriptive names** (`test_component_scenario_expected`)

**Test Hierarchy:**
1. Weaver validation (source of truth for telemetry)
2. Performance tests (proves hot path ≤8 ticks)
3. Integration tests (proves components work together)
4. Unit tests (proves individual components work)

---

## Issue 1: Tests Passing But Feature Broken

### Symptom
```bash
$ cargo test
test result: ok. 150 passed

$ cargo run --bin knhk-cli -- query ask "ASK { ?s ?p ?o }"
Error: unimplemented!()
```

### Cause
**The False Positive Paradox**: Tests validate test code, not production behavior.

**Common patterns:**
- Tests mock the actual implementation
- Tests call stub/placeholder code
- Tests don't exercise real code path
- Tests check wrong assertions

### Diagnosis

**Step 1: Check what code is actually tested:**
```bash
# Generate coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Open coverage/index.html
# Look for:
# - Red lines (untested code)
# - Functions with 0% coverage
# - Entire modules untested
```

**Step 2: Verify tests call production code:**
```rust
// ❌ Wrong: Test calls mock, not real code
#[test]
fn test_query_execute() {
    let executor = MockExecutor::new();  // Mock!
    let result = executor.execute("ASK { ?s ?p ?o }");
    assert!(result.is_ok());  // Test passes, but real code never executed!
}

// ✅ Correct: Test calls real code
#[test]
fn test_query_execute() {
    let executor = WarmPathExecutor::new();  // Real!
    let query = Query::new_ask("ASK { ?s ?p ?o }");
    let result = executor.execute_ask(query);  // Real code executed!
    assert!(result.is_ok());
}
```

**Step 3: Verify Weaver validation:**
```bash
# Weaver live-check is the only source of truth for telemetry
weaver registry check -r /home/user/knhk/registry/
weaver registry live-check --registry /home/user/knhk/registry/

# If this fails, telemetry is broken, regardless of unit tests
```

### Solution

**1. Use real collaborators (Chicago TDD):**
```rust
// ❌ Wrong: Mock everything
#[test]
fn test_workflow_execute() {
    let mock_db = MockDatabase::new();
    let mock_executor = MockExecutor::new();
    let workflow = Workflow::new(mock_db, mock_executor);
    // Test passes but doesn't prove real code works!
}

// ✅ Correct: Use real collaborators
#[test]
fn test_workflow_execute() {
    let db = Database::in_memory();  // Real, but isolated
    let executor = WarmPathExecutor::new();  // Real
    let workflow = Workflow::new(db, executor);
    // Test proves real code works!
}
```

**2. Validate with Weaver:**
```rust
#[test]
fn test_telemetry_emitted() {
    // Arrange: Initialize tracer
    let _guard = init_tracer("test", "1.0.0", Some("http://localhost:4318"))
        .expect("Init tracer");

    // Act: Execute operation that should emit telemetry
    execute_query("ASK { ?s ?p ?o }");

    // Assert: Weaver validates telemetry
    // Run: weaver registry live-check --registry registry/
}
```

**3. Actually execute the command:**
```rust
// ❌ Wrong: Only test --help
#[test]
fn test_cli_query_command() {
    let output = Command::new("knhk")
        .arg("--help")
        .output()
        .expect("Failed to run");
    assert!(output.status.success());  // Proves nothing!
}

// ✅ Correct: Execute with real arguments
#[test]
fn test_cli_query_command() {
    let output = Command::new("knhk")
        .arg("query")
        .arg("ask")
        .arg("ASK { ?s ?p ?o }")
        .output()
        .expect("Failed to run");
    assert!(output.status.success());  // Proves command works!
    assert!(output.stdout.len() > 0);  // Proves output generated!
}
```

### Prevention
- Run Weaver live-check in CI
- Track code coverage (≥90% target)
- Actually execute commands, don't just test help text
- Use real collaborators, not mocks

---

## Issue 2: Flaky Tests (Random Failures)

### Symptom
```bash
$ cargo test
test result: ok. 150 passed

$ cargo test
test result: FAILED. 148 passed; 2 failed
test test_concurrent_execution ... FAILED
test test_timing_sensitive ... FAILED
```

### Diagnosis

**Step 1: Identify flaky tests:**
```bash
# Run tests 100 times
for i in {1..100}; do
    cargo test --quiet 2>&1 | tee -a test-results.txt
done

# Find tests that failed at least once
grep FAILED test-results.txt | sort | uniq -c
```

**Step 2: Categorize flakiness:**
- **Timing-dependent**: Races, timeouts, sleep()
- **Order-dependent**: Tests affect each other's state
- **Non-deterministic**: Random data, current time, network
- **Resource contention**: File locks, ports, CPU

### Common Causes & Solutions

**Cause 1: Race Conditions**
```rust
// ❌ Wrong: Race between threads
#[test]
fn test_concurrent_writes() {
    let data = Arc::new(Mutex::new(Vec::new()));

    let d1 = data.clone();
    thread::spawn(move || {
        d1.lock().unwrap().push(1);
    });

    // Race! Thread may not finish before assertion
    assert_eq!(data.lock().unwrap().len(), 1);  // Flaky!
}

// ✅ Correct: Wait for completion
#[test]
fn test_concurrent_writes() {
    let data = Arc::new(Mutex::new(Vec::new()));

    let d1 = data.clone();
    let handle = thread::spawn(move || {
        d1.lock().unwrap().push(1);
    });

    handle.join().unwrap();  // Wait for thread
    assert_eq!(data.lock().unwrap().len(), 1);  // Deterministic!
}
```

**Cause 2: Hardcoded Timeouts**
```rust
// ❌ Wrong: Arbitrary sleep
#[test]
fn test_async_operation() {
    start_async_operation();
    thread::sleep(Duration::from_millis(100));  // Flaky! May not be enough
    assert!(is_operation_complete());
}

// ✅ Correct: Wait with timeout
#[test]
fn test_async_operation() {
    start_async_operation();

    let timeout = Duration::from_secs(5);
    let start = Instant::now();
    while !is_operation_complete() {
        if start.elapsed() > timeout {
            panic!("Operation timed out");
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(is_operation_complete());  // Deterministic!
}
```

**Cause 3: Shared State Between Tests**
```rust
// ❌ Wrong: Tests share global state
static mut COUNTER: u32 = 0;

#[test]
fn test_increment() {
    unsafe { COUNTER += 1; }
    unsafe { assert_eq!(COUNTER, 1); }  // Flaky! Other tests modify COUNTER
}

// ✅ Correct: Isolated state per test
#[test]
fn test_increment() {
    let mut counter = 0;
    counter += 1;
    assert_eq!(counter, 1);  // Deterministic!
}
```

**Cause 4: Non-Deterministic Data**
```rust
// ❌ Wrong: Random data in assertions
use rand::Rng;

#[test]
fn test_sort() {
    let mut data: Vec<u32> = (0..100).map(|_| rand::thread_rng().gen()).collect();
    data.sort();
    assert!(is_sorted(&data));  // Assertion depends on random data!
}

// ✅ Correct: Fixed test data
#[test]
fn test_sort() {
    let mut data = vec![5, 2, 8, 1, 9];  // Known data
    data.sort();
    assert_eq!(data, vec![1, 2, 5, 8, 9]);  // Deterministic!
}
```

### Prevention
- Avoid `sleep()` in tests (use proper synchronization)
- Use thread::join() to wait for threads
- Avoid global mutable state
- Use fixed test data (not random)
- Run tests in isolation (`cargo test -- --test-threads=1`)

---

## Issue 3: Integration Tests Fail But Unit Tests Pass

### Symptom
```bash
$ cargo test --lib
test result: ok. 100 passed  # Unit tests pass

$ cargo test --test integration_complete
test result: FAILED. 5 passed; 3 failed  # Integration fails
```

### Diagnosis

**Common causes:**
- Components work alone but not together
- Real dependencies missing (database, network)
- Configuration differences (env vars, files)
- Initialization order issues

### Solution

**1. Test with real dependencies:**
```rust
// ❌ Wrong: Mock database in integration test
#[test]
fn test_workflow_integration() {
    let db = MockDatabase::new();  // Not real!
    let workflow = Workflow::new(db);
    // Doesn't prove real database integration works
}

// ✅ Correct: Use real (but isolated) database
#[test]
fn test_workflow_integration() {
    // Use real database with test schema
    let db = Database::connect("postgres://localhost/knhk_test")
        .expect("Test database");

    // Clean state before test
    db.execute("TRUNCATE TABLE workflows").expect("Clean DB");

    let workflow = Workflow::new(db);
    // Proves real database integration works!
}
```

**2. Verify initialization:**
```rust
// ❌ Wrong: Assume components initialized
#[test]
fn test_full_pipeline() {
    let engine = WorkflowEngine::new(state_store);  // May not be initialized!
    engine.execute_workflow(spec).await;  // Fails!
}

// ✅ Correct: Explicit initialization
#[test]
fn test_full_pipeline() {
    // Initialize all components
    let state_store = StateStore::new("./test_db").expect("State store");
    let engine = WorkflowEngine::new(state_store);
    engine.initialize().await.expect("Engine init");

    // Now test
    let result = engine.execute_workflow(spec).await;
    assert!(result.is_ok());
}
```

**3. Check configuration:**
```rust
// ❌ Wrong: Missing environment setup
#[test]
fn test_with_config() {
    let config = Config::load();  // May fail if env vars not set
    // ...
}

// ✅ Correct: Set test environment
#[test]
fn test_with_config() {
    // Set test environment variables
    env::set_var("KNHK_DB_URL", "postgres://localhost/knhk_test");
    env::set_var("KNHK_OTLP_ENDPOINT", "http://localhost:4318");

    let config = Config::load().expect("Config");
    // ...

    // Clean up
    env::remove_var("KNHK_DB_URL");
    env::remove_var("KNHK_OTLP_ENDPOINT");
}
```

### Prevention
- Use Docker Compose for test dependencies
- Document required test setup
- Use test fixtures (setup/teardown)
- Test initialization explicitly

---

## Issue 4: Performance Tests Fail (Exceeds 8 Ticks)

### Symptom
```bash
$ make test-performance-v04
FAIL: ASK query took 12 ticks (expected ≤8)
```

### Diagnosis

**Step 1: Identify slow operations:**
```rust
#[test]
fn test_hot_path_ask_query_within_budget() {
    let executor = WarmPathExecutor::new();
    let query = Query::new_ask("ASK { ?s ?p ?o }");

    let start = unsafe { std::arch::x86_64::_rdtsc() };
    let result = executor.execute_ask(query);
    let end = unsafe { std::arch::x86_64::_rdtsc() };
    let ticks = end - start;

    println!("Ticks: {}", ticks);  // Debug output
    assert!(ticks <= 8, "Expected ≤8 ticks, got {}", ticks);
}
```

**Step 2: Profile:**
```bash
# CPU profile
perf record -g cargo test test_hot_path_ask_query_within_budget
perf report --stdio | head -50

# Look for:
# - malloc/free calls (heap allocations)
# - Function calls (should be inlined)
# - Branch mispredicts
```

### Common Causes & Solutions

**Cause 1: Debug Build**
```bash
# ❌ Wrong: Performance test in debug mode
cargo test test_hot_path_ask_query_within_budget

# ✅ Correct: Use release mode
cargo test --release test_hot_path_ask_query_within_budget
```

**Cause 2: Heap Allocations**
```rust
// ❌ Wrong: Allocates in hot path
fn execute_ask(query: &str) -> bool {
    let data = Vec::new();  // Heap allocation! (~50 ticks)
    // ...
}

// ✅ Correct: Stack-only
fn execute_ask(query: &str) -> bool {
    let mut data: [u64; 8] = [0; 8];  // Stack (0 extra ticks)
    // ...
}
```

**Cause 3: Branching**
```rust
// ❌ Wrong: Branch in hot path
fn compare(a: u64, b: u64) -> bool {
    if a == b {  // Branch misprediction (~10 ticks)
        true
    } else {
        false
    }
}

// ✅ Correct: Branchless
fn compare(a: u64, b: u64) -> bool {
    a == b  // Compiles to CMOV (~1 tick)
}
```

See [Performance Troubleshooting](/home/user/knhk/docs/troubleshooting/PERFORMANCE_TROUBLESHOOTING.md) for more details.

---

## Issue 5: Chicago TDD Tests Too Coupled to Implementation

### Symptom
```rust
// Refactor implementation → All tests fail!
```

### Diagnosis
Tests are too tightly coupled to implementation details instead of behavior.

### Solution

**Test behavior, not implementation:**

```rust
// ❌ Wrong: Tests internal implementation
#[test]
fn test_workflow_internal_state() {
    let workflow = Workflow::new();
    workflow.execute_step();

    // Testing internal state (implementation detail)
    assert_eq!(workflow.internal_counter, 1);
    assert_eq!(workflow.state_machine.current_state, State::Running);
}

// ✅ Correct: Tests observable behavior
#[test]
fn test_workflow_execution() {
    // Arrange: Create workflow with known input
    let workflow = Workflow::new();

    // Act: Execute operation
    let result = workflow.execute_step();

    // Assert: Verify observable outcome
    assert!(result.is_ok());
    assert_eq!(workflow.status(), WorkflowStatus::Running);
}
```

**Test state changes, not method calls:**

```rust
// ❌ Wrong: Verify method calls (interaction testing)
#[test]
fn test_workflow_calls_database() {
    let mut mock_db = MockDatabase::new();
    mock_db.expect_save()
        .times(1)
        .return_const(Ok(()));  // Verifying interaction!

    let workflow = Workflow::new(mock_db);
    workflow.save();
}

// ✅ Correct: Verify state changes (Chicago TDD)
#[test]
fn test_workflow_saves_to_database() {
    let db = Database::in_memory();
    let workflow = Workflow::new(db.clone());

    workflow.save().expect("Save workflow");

    // Verify state change in database
    let saved = db.load_workflow(workflow.id()).expect("Load workflow");
    assert_eq!(saved, workflow);  // Verifying outcome!
}
```

### Prevention
- Test public APIs, not private internals
- Test outcomes, not interactions
- Use real collaborators (Chicago TDD)
- Refactor without breaking tests

---

## Issue 6: Test Coverage Low (<90%)

### Symptom
```bash
$ cargo tarpaulin --workspace
Test coverage: 67.2%  # Below 90% target
```

### Diagnosis

**Step 1: Identify untested code:**
```bash
# Generate HTML coverage report
cargo tarpaulin --workspace --out Html --output-dir coverage/

# Open coverage/index.html
# Red lines = untested code
```

**Step 2: Categorize untested code:**
- Error paths (Result::Err branches)
- Edge cases (empty inputs, large inputs)
- Initialization code
- Cleanup code
- Error handling

### Solution

**1. Test error paths:**
```rust
// Function
fn parse_query(sparql: &str) -> Result<Query, ParseError> {
    if sparql.is_empty() {
        return Err(ParseError::EmptyQuery);  // Error path
    }
    // ... parsing logic
}

// ❌ Wrong: Only test success path
#[test]
fn test_parse_query_success() {
    let result = parse_query("ASK { ?s ?p ?o }");
    assert!(result.is_ok());
}
// Coverage: 50% (error path untested)

// ✅ Correct: Test both paths
#[test]
fn test_parse_query_success() {
    let result = parse_query("ASK { ?s ?p ?o }");
    assert!(result.is_ok());
}

#[test]
fn test_parse_query_empty_input() {
    let result = parse_query("");
    assert!(matches!(result, Err(ParseError::EmptyQuery)));
}
// Coverage: 100%
```

**2. Test edge cases:**
```rust
#[test]
fn test_query_empty_graph() {
    let executor = WarmPathExecutor::new();  // Empty graph
    let result = executor.execute_ask("ASK { ?s ?p ?o }");
    assert_eq!(result, Ok(false));  // No triples = false
}

#[test]
fn test_query_large_graph() {
    let mut executor = WarmPathExecutor::new();
    // Add 10,000 triples
    for i in 0..10000 {
        executor.add_triple(format!("s{}", i), "p".to_string(), format!("o{}", i));
    }
    let result = executor.execute_ask("ASK { ?s ?p ?o }");
    assert_eq!(result, Ok(true));  // Has triples = true
}
```

**3. Test initialization/cleanup:**
```rust
#[test]
fn test_engine_initialization() {
    let state_store = StateStore::new("./test_init_db").expect("State store");
    let engine = WorkflowEngine::new(state_store);

    // Test initialization
    let result = engine.initialize().await;
    assert!(result.is_ok());

    // Cleanup
    fs::remove_dir_all("./test_init_db").ok();
}
```

### Prevention
- Set coverage target in CI (≥90%)
- Review coverage reports in PRs
- Write tests before implementation (TDD)
- Test all code paths (success, error, edge cases)

---

## Issue 7: Test Execution Slow

### Symptom
```bash
$ cargo test
Finished test [unoptimized + debuginfo] target(s) in 0.42s
Running unittests src/lib.rs (target/debug/deps/knhk_warm-...)
running 150 tests
...
test result: ok. 150 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.23s
# 45 seconds! Too slow!
```

### Diagnosis

**Step 1: Find slow tests:**
```bash
# Run with timing
cargo test -- --nocapture --test-threads=1 | grep "test result"
```

**Step 2: Profile test execution:**
```bash
# Profile tests
cargo test --no-fail-fast -- --nocapture --show-output
```

### Common Causes & Solutions

**Cause 1: Synchronous I/O**
```rust
// ❌ Wrong: Blocking I/O in tests
#[test]
fn test_fetch_data() {
    let data = reqwest::blocking::get("http://example.com/data")
        .unwrap()
        .text()
        .unwrap();  // Blocks for ~1s per test!
}

// ✅ Correct: Mock external I/O
#[test]
fn test_fetch_data() {
    let mock_server = mockito::Server::new();
    let _m = mock_server.mock("GET", "/data")
        .with_body("test data")
        .create();

    let data = fetch_data(&mock_server.url()).unwrap();  // <1ms
}
```

**Cause 2: Large Test Data**
```rust
// ❌ Wrong: Generate large data in every test
#[test]
fn test_process_large_dataset() {
    let data: Vec<_> = (0..1_000_000).collect();  // 1M items, slow!
    let result = process(data);
    // ...
}

// ✅ Correct: Use smaller representative data
#[test]
fn test_process_dataset() {
    let data = vec![1, 2, 3, 4, 5];  // Small but representative
    let result = process(data);
    // ...
}
```

**Cause 3: Serial Execution**
```bash
# ❌ Wrong: Single-threaded
cargo test -- --test-threads=1  # Very slow!

# ✅ Correct: Parallel execution (default)
cargo test  # Uses all cores
```

### Prevention
- Keep unit tests fast (<100ms each)
- Use mocks for slow external dependencies
- Run slow tests separately (`#[ignore]` + `cargo test -- --ignored`)
- Use minimal test data

---

## Quick Test Diagnostic Commands

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test test_hot_path_ask_query_within_budget

# Run with output
cargo test -- --nocapture

# Run performance tests
make test-performance-v04

# Run integration tests
make test-integration-v2

# Check coverage
cargo tarpaulin --workspace --out Html

# Run Weaver validation
weaver registry check -r /home/user/knhk/registry/
weaver registry live-check --registry /home/user/knhk/registry/

# Run tests 100 times (find flaky tests)
for i in {1..100}; do cargo test --quiet || echo "FAILED"; done
```

---

## See Also

- [Testing Guide](/home/user/knhk/docs/TESTING.md)
- [Testing Checklist](/home/user/knhk/docs/reference/cards/TESTING_CHECKLIST.md)
- [Weaver Validation Troubleshooting](/home/user/knhk/docs/troubleshooting/WEAVER_VALIDATION_TROUBLESHOOTING.md)

# Testing Patterns Reference

Common test patterns and approaches for KNHK applications.

---

## 1. Chicago TDD Testing Pattern

### AAA Pattern (Arrange-Act-Assert)

```rust
#[test]
fn test_process_payment() {
    // ARRANGE - Set up test data
    let payment = Payment::new(100.0, "USD").unwrap();
    let processor = MockProcessor::new();
    
    // ACT - Execute the code under test
    let result = processor.process(&payment).unwrap();
    
    // ASSERT - Verify results
    assert_eq!(result.status, PaymentStatus::Approved);
    assert_eq!(result.amount, 100.0);
}
```

**Principles**:
- One assertion per logical outcome
- Clear separation of setup/execution/verification
- Test names describe what is being tested
- No test interdependencies

---

## 2. Type-Safe Testing

### Use Types to Prevent Invalid Test States

```rust
#[test]
fn test_case_id_cannot_be_zero() {
    // This won't compile - type prevents invalid state
    // let invalid = CaseID::new(0); // Compile error!
    
    // Only valid IDs can be created
    let valid = CaseID::new(1).unwrap();
    assert_eq!(valid.as_u64(), 1);
}
```

**Benefits**:
- Compiler prevents invalid test setups
- Tests focus on business logic, not validation
- No need for separate validation tests

---

## 3. Property-Based Testing

### Using PropTest for Generative Testing

```rust
proptest! {
    #[test]
    fn test_count_stays_positive(n in 1..u32::MAX) {
        let count = Count::new(n).unwrap();
        prop_assert!(count.value() > 0);
    }
}
```

**When to use**:
- Testing invariants that should hold for all valid inputs
- Finding edge cases
- Regression testing

---

## 4. Snapshot Testing

### Compare Against Known Good Output

```rust
#[test]
fn test_process_discovery_output() {
    let graph = discover_process(&event_log).unwrap();
    insta::assert_snapshot!(graph);
}
```

**Benefits**:
- Detect unintended changes
- Document expected output
- Approve changes explicitly

---

## 5. Table-Driven Tests

### Test Multiple Cases in One Test

```rust
#[test]
fn test_state_transitions() {
    let test_cases = vec![
        (Initial, Running, true),
        (Running, Paused, true),
        (Completed, Running, false),  // Invalid
    ];
    
    for (from, to, should_succeed) in test_cases {
        let result = can_transition(from, to);
        assert_eq!(result, should_succeed);
    }
}
```

**Advantages**:
- Compact test coverage
- Easy to add new cases
- All cases tested with same logic

---

## 6. Mock vs. Stub vs. Spy

### When to Use Each

| Pattern | Use Case | Example |
|---------|----------|---------|
| **Mock** | Verify interactions | `processor.expect_process().times(1)` |
| **Stub** | Return fake data | `MockDB.return_user(user)` |
| **Spy** | Record calls, return real | `spy_on_database()` |

**Chicago TDD approach**: Use real objects unless external (network, file, database)

---

## 7. Integration Testing

### Test Multiple Components Together

```rust
#[tokio::test]
async fn test_workflow_integration() {
    let mut log = EventLog::new();
    
    // Add real events
    log.add_event(event1).unwrap();
    log.add_event(event2).unwrap();
    let closed = log.close();
    
    // Discover process
    let graph = discover_process(&closed).unwrap();
    
    // Verify integration worked end-to-end
    assert_eq!(graph.activities.len(), 2);
}
```

**Key points**:
- Test real interactions, not mocks
- Use in-memory databases for speed
- Verify end-to-end workflows

---

## 8. Performance Testing

### Verify ≤8 Tick Chatman Constant

```rust
#[test]
fn test_query_performance() {
    let query = build_hot_path_query();
    
    let start = std::time::Instant::now();
    let result = query.execute().unwrap();
    let elapsed = start.elapsed();
    
    // Verify ≤8 ticks (should be microseconds)
    assert!(elapsed.as_nanos() < 8000); // 8 microseconds = ~8 ticks
}
```

**Using Criterion for detailed benchmarks**:

```rust
fn bench_hot_path(c: &mut Criterion) {
    c.bench_function("hot_path_query", |b| {
        b.iter(|| query.execute())
    });
}
```

---

## 9. Error Path Testing

### Test Failures as Thoroughly as Success

```rust
#[test]
fn test_invalid_case_id_rejected() {
    let result = CaseID::new(0);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), InvalidIdError::ZeroId);
}

#[test]
fn test_empty_activity_rejected() {
    let result = ActivityName::new("");
    assert!(result.is_err());
}
```

**Principles**:
- Test all error paths
- Verify error messages
- Ensure graceful failure

---

## 10. Concurrent Testing

### Test Thread Safety

```rust
#[test]
fn test_concurrent_access() {
    let log = Arc::new(Mutex::new(EventLog::new()));
    
    let mut handles = vec![];
    for _ in 0..10 {
        let log = Arc::clone(&log);
        let h = std::thread::spawn(move || {
            let mut l = log.lock().unwrap();
            l.add_event(event).unwrap();
        });
        handles.push(h);
    }
    
    for h in handles {
        h.join().unwrap();
    }
    
    let l = log.lock().unwrap();
    assert_eq!(l.event_count(), 10);
}
```

---

## 11. Telemetry Testing

### Verify Instrumentation Correctness

```rust
#[test]
fn test_telemetry_emitted() {
    let mut spans = vec![];
    let tracer = MockTracer::new(&mut spans);
    
    // Execute instrumented code
    process_with_telemetry(&tracer).unwrap();
    
    // Verify telemetry
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0].name, "process_start");
    assert_eq!(spans[1].name, "process_complete");
}
```

---

## 12. Test Organization

### Structure for Maintainability

```
src/
├── lib.rs
├── module/
│   ├── mod.rs
│   ├── feature.rs
│   └── tests/
│       ├── mod.rs
│       ├── feature_tests.rs
│       └── integration_tests.rs

tests/
├── integration/
│   ├── full_workflow.rs
│   └── end_to_end.rs
```

**Naming**:
- Unit tests: `#[cfg(test)] mod tests { ... }`
- Integration tests: `tests/` directory

---

## Summary

| Pattern | Best For | Effort | Coverage |
|---------|----------|--------|----------|
| AAA | Unit testing | Low | High |
| Type-Safe | Compile-time checks | Low | Automatic |
| Property-Based | Edge cases | Medium | Very High |
| Snapshot | Output changes | Low | High |
| Table-Driven | Multiple cases | Low | High |
| Integration | Interactions | Medium | High |
| Performance | Speed validation | Medium | Targeted |
| Error | Failure paths | Medium | Critical |
| Concurrent | Thread safety | High | Essential |
| Telemetry | Instrumentation | Medium | Quality |

---

**Last Updated**: 2025-11-15
**Version**: v1.1.0
**Framework**: Testing Patterns Reference

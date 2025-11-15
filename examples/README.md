# Code Examples

Real, working code examples for common KNHK patterns. All examples are production-ready and copy-paste compatible.

---

## 📋 Available Examples

### [Hot Path Query](hot-path-query.rs)
**What it shows**: Fast SPARQL ASK query execution (≤8 ticks)
- Simple but complete example
- Shows proper telemetry instrumentation
- Performance measurement included
- Comments explain every step
- **Use this**: When learning hot path queries

**Key patterns**:
- Query execution with timing
- Telemetry span creation
- Result handling
- Error management

**Lines**: ~150 | **Time to understand**: 10-15 min

---

### [Warm Path Emit](warm-path-emit.rs)
**What it shows**: CONSTRUCT8 query for larger result sets (≤500ms)
- Complete working implementation
- Real error handling
- Shows telemetry for monitoring
- **Use this**: When learning warm path operations

**Key patterns**:
- CONSTRUCT query
- Result streaming
- Error recovery
- Performance telemetry

**Lines**: ~200 | **Time to understand**: 15-20 min

---

### [Chicago TDD User Registration](chicago-tdd-user-registration.rs)
**What it shows**: Test-driven development Chicago style
- Complete test suite with real objects (no mocks)
- AAA pattern (Arrange-Act-Assert) demonstrated
- Three test cases: success, duplicate, invalid email
- Shows how to test behavior, not implementation
- **Use this**: When learning Chicago TDD

**Key patterns**:
- Real object collaboration
- Assertion patterns
- Test organization
- No mocks (real services)

**Lines**: ~180 | **Time to understand**: 15-20 min

---

### [Telemetry Instrumentation](telemetry-instrumentation.rs)
**What it shows**: Complete OTEL instrumentation
- Spans with attributes
- Metrics (counter, histogram, gauge)
- Logs with context
- Strategic instrumentation pyramid
- Before/after comparison
- **Use this**: When adding telemetry to code

**Key patterns**:
- Span creation and attributes
- Metric recording
- Log context
- Performance considerations
- Sampling strategy

**Lines**: ~150 | **Time to understand**: 20-25 min

---

### [Performance Optimization: Before/After](performance-optimization-before-after.rs)
**What it shows**: Real optimization journey (15 ticks → 3 ticks)
- Shows slow version first
- Applies 4 optimization techniques
- Measures each improvement
- Explains why each works
- Final version hits ≤8 ticks
- **Use this**: When learning optimization

**Key patterns**:
- Profiling methodology
- Algorithm optimization
- Memory optimization
- Caching strategies
- Verification with benchmarks

**Lines**: ~200 | **Time to understand**: 25-30 min

---

### [Error Handling Pattern](error-handling-pattern.rs)
**What it shows**: Proper Rust error handling
- Result<T, E> usage throughout
- No unwrap/expect in production code
- Error telemetry and logging
- Recovery strategies
- Propagation vs handling decisions
- **Use this**: When implementing error handling

**Key patterns**:
- Result types
- Error propagation
- Custom errors
- Telemetry for errors
- Recovery options

**Lines**: ~120 | **Time to understand**: 15-20 min

---

### [Workflow State Machine](workflow-example.rs)
**What it shows**: Complete workflow with state machine
- Request-response pattern
- State transitions
- Error handling between states
- Telemetry at each step
- Clean state management
- **Use this**: When building workflows

**Key patterns**:
- Enum-based state machine
- State transitions
- Event handling
- Telemetry at milestones
- Error recovery

**Lines**: ~180 | **Time to understand**: 20-25 min

---

### [Integration Test](integration-test.rs)
**What it shows**: End-to-end feature testing
- Multiple components working together
- Real async/await patterns
- Database integration
- API testing
- Telemetry verification
- **Use this**: When writing integration tests

**Key patterns**:
- Test setup/teardown
- Async test handling
- Multiple component coordination
- Assertion strategies
- Resource cleanup

**Lines**: ~150 | **Time to understand**: 20-25 min

---

## 🎯 Quick Selection Guide

| Need | Example |
|------|---------|
| Learn hot path queries | [Hot Path Query](hot-path-query.rs) |
| Learn warm path | [Warm Path Emit](warm-path-emit.rs) |
| Learn TDD | [Chicago TDD User Registration](chicago-tdd-user-registration.rs) |
| Add telemetry | [Telemetry Instrumentation](telemetry-instrumentation.rs) |
| Optimize code | [Performance Optimization](performance-optimization-before-after.rs) |
| Handle errors | [Error Handling Pattern](error-handling-pattern.rs) |
| Build workflows | [Workflow State Machine](workflow-example.rs) |
| Write integration tests | [Integration Test](integration-test.rs) |

---

## 💡 How to Use These Examples

1. **Copy the code**: All examples are ready to copy-paste
2. **Read comments**: Each line is explained
3. **Run and experiment**: Try modifying values
4. **Reference patterns**: Use as templates for your code
5. **Learn from mistakes**: Comments explain why things work

---

## 🔗 Related Documentation

- **Tutorials**: [Learning-oriented guides](../papers/tutorials/)
- **How-to Guides**: [Task-oriented guides](../papers/how-to-guides/)
- **Templates**: [Ready-to-use code structures](templates/)
- **Troubleshooting**: [Common issues and fixes](../docs/troubleshooting/)

---

## 🚀 Learning Path

**Recommended order for learning**:

1. Start: [Hot Path Query](hot-path-query.rs) - Understand basic execution
2. Expand: [Warm Path Emit](warm-path-emit.rs) - Learn larger datasets
3. Test: [Chicago TDD User Registration](chicago-tdd-user-registration.rs) - Learn testing
4. Observe: [Telemetry Instrumentation](telemetry-instrumentation.rs) - Learn monitoring
5. Optimize: [Performance Optimization](performance-optimization-before-after.rs) - Learn optimization
6. Handle: [Error Handling Pattern](error-handling-pattern.rs) - Learn error management
7. Build: [Workflow State Machine](workflow-example.rs) - Learn complex patterns
8. Validate: [Integration Test](integration-test.rs) - Learn validation

---

## 📊 Examples Statistics

| Example | Lines | Time | Topic |
|---------|-------|------|-------|
| Hot Path Query | ~150 | 10-15 min | Queries |
| Warm Path Emit | ~200 | 15-20 min | Queries |
| Chicago TDD | ~180 | 15-20 min | Testing |
| Telemetry | ~150 | 20-25 min | Instrumentation |
| Performance | ~200 | 25-30 min | Optimization |
| Error Handling | ~120 | 15-20 min | Errors |
| Workflow | ~180 | 20-25 min | Workflows |
| Integration Test | ~150 | 20-25 min | Testing |

**Total lines**: ~1300
**Average example**: ~160 lines
**Format**: Runnable Rust code

---

**Last Updated**: 2025-11-15
**Version**: v1.1.0
**Framework**: Production Code Examples

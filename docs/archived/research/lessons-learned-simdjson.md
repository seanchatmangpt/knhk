# Lessons Learned from simdjson - Applied to KNHK

## Executive Summary

simdjson is a high-performance JSON parser that achieves gigabytes-per-second parsing speeds through SIMD instructions, microparallel algorithms, and careful engineering. This document extracts key lessons applicable to KNHK's hot path operations, performance-critical code, and development practices.

**Key Takeaway**: simdjson demonstrates that careful engineering, measurement-driven optimization, and pragmatic design choices can achieve order-of-magnitude performance improvements while maintaining code quality and usability.

---

## 1. Performance Optimization Techniques

### 1.1 Two-Stage Parsing Architecture

**Lesson**: Separate fast structural identification from slower semantic parsing.

**simdjson Approach**:
- **Stage 1 (Find marks)**: Fast SIMD-based identification of structural elements, strings, and UTF-8 validation
- **Stage 2 (Structure building)**: Slower semantic parsing (numbers, atoms, tree construction)

**Application to KNHK**:
- **Hot Path**: Fast structural validation (predicate matching, guard constraints) using SIMD
- **Warm Path**: Semantic operations (SPARQL queries, complex joins) using traditional algorithms
- **Benefit**: Aligns with KNHK's 80/20 philosophy - optimize the critical 20% (hot path) that provides 80% of value

**Implementation Pattern**:
```rust
// Stage 1: Fast structural validation (hot path, ≤8 ticks)
fn validate_structure(soa: &SoAArrays) -> Result<ValidationResult, Error> {
    // SIMD-based quick validation
    // Returns structural markers, not full semantic data
}

// Stage 2: Semantic parsing (warm path, can be slower)
fn parse_semantics(markers: &ValidationResult) -> Result<ParsedData, Error> {
    // Full semantic parsing only when needed
}
```

### 1.2 Pseudo-Structural Character Concept

**Lesson**: Redefine what constitutes "structural" to reduce branch misprediction.

**simdjson Approach**:
A character is pseudo-structural if:
1. Not enclosed in quotes, AND
2. Is a non-whitespace character, AND
3. Its preceding character is either:
   - a structural character, OR
   - whitespace, OR
   - the final quote in a string

**Application to KNHK**:
- **Triple Pattern Matching**: Redefine "structural" for RDF triples to reduce branches
- **Predicate Matching**: Use pseudo-structural concept for fast predicate identification
- **Guard Validation**: Apply structural concepts to guard constraint checking

**Benefit**: Fewer branches = better branch prediction = faster hot path execution

### 1.3 On-Demand Parsing (Lazy Evaluation)

**Lesson**: Parse only what you use, when you use it.

**simdjson Approach**:
- On-Demand API: Parse values only when accessed
- Type-specific parsing: Different parsers for `double`, `uint64_t`, `int64_t`
- Forward-only iteration: Single index maintained, no backtracking

**Application to KNHK**:
- **Lazy Triple Materialization**: Only materialize triples when accessed
- **Type-Specific Operations**: Different hot path kernels for ASK vs SELECT vs CONSTRUCT
- **Streaming Processing**: Process triples as they arrive, not after full materialization

**Implementation Pattern**:
```rust
// On-demand triple access
impl SoAArrays {
    fn get_triple(&self, index: usize) -> Option<Triple> {
        // Only materialize when accessed
        Some(Triple {
            s: self.s[index],
            p: self.p[index],
            o: self.o[index],
        })
    }
    
    fn iterate_triples(&self) -> TripleIterator {
        // Forward-only iterator, no backtracking
        TripleIterator::new(self)
    }
}
```

### 1.4 Runtime CPU Dispatching

**Lesson**: Compile multiple optimized kernels, select best at runtime.

**simdjson Approach**:
- Compile multiple implementations (icelake, haswell, westmere, arm64, fallback)
- Runtime CPU detection selects best implementation
- Compile for lowest common denominator, dispatch to optimized kernels

**Application to KNHK**:
- **Architecture-Specific Hot Paths**: Different C kernels for AVX2, NEON, SSE4.2
- **Runtime Detection**: Detect CPU features, select best hot path implementation
- **Fallback Support**: Always have a generic fallback for compatibility

**Implementation Pattern**:
```c
// Runtime dispatch to best implementation
typedef struct {
    int (*execute_hook)(SoAArrays*, PredRun*);
    const char* name;
} HotPathImplementation;

HotPathImplementation* detect_best_implementation(void) {
    if (cpu_supports_avx2()) return &haswell_impl;
    if (cpu_supports_sse42()) return &westmere_impl;
    return &fallback_impl;
}
```

### 1.5 Memory Reuse and Buffer Management

**Lesson**: Reuse buffers to keep memory hot in cache.

**simdjson Approach**:
- Parser objects retain internal buffers between parses
- Buffers grow as needed but don't shrink (for server loops)
- Can set max capacity to prevent unbounded growth

**Application to KNHK**:
- **SoA Buffer Reuse**: Reuse SoAArrays buffers across operations
- **Receipt Pool**: Pre-allocate receipt objects, reuse them
- **Ring Buffer Management**: Fixed-size ring buffers for hot path operations

**Implementation Pattern**:
```rust
// Reusable parser/buffer pattern
pub struct HotPathEngine {
    soa_buffers: SoAArrays,
    receipt_pool: Vec<Receipt>,
    max_capacity: usize,
}

impl HotPathEngine {
    fn execute(&mut self, triples: &[Triple]) -> Result<Vec<Receipt>> {
        // Reuse existing buffers, only allocate if needed
        if triples.len() > self.soa_buffers.capacity() {
            self.soa_buffers.grow(triples.len());
        }
        // Process using hot buffers
    }
}
```

### 1.6 Free Padding Optimization

**Lesson**: Exploit page boundaries to avoid extra allocations.

**simdjson Approach**:
- Requires `SIMDJSON_PADDING` bytes at end of buffer
- Can safely read beyond allocated buffer if within same page
- Most of the time, no allocation needed

**Application to KNHK**:
- **Triple Buffer Padding**: Add padding to SoAArrays for SIMD operations
- **Page-Aware Allocation**: Check page boundaries before allocating
- **Zero-Copy Operations**: Use padding to avoid copies when possible

**Benefit**: Reduces memory allocations in hot path, improves cache locality

---

## 2. Architecture and Design Patterns

### 2.1 Generic Code + Implementation-Specific Specialization

**Lesson**: Write generic code once, compile for each architecture.

**simdjson Approach**:
- `generic/*.h`: Generic code written once
- `haswell/*.h`, `arm64/*.h`: Implementation-specific functions
- Generic code includes implementation-specific headers
- Compiler generates optimized code for each architecture

**Application to KNHK**:
- **Generic Hot Path Logic**: Write hot path logic generically
- **Architecture-Specific Kernels**: C kernels for each architecture (AVX2, NEON, etc.)
- **Rust Wrapper**: Generic Rust wrapper calls architecture-specific C kernels

**Directory Structure**:
```
knhk-hot/
├── src/
│   ├── generic/          # Generic hot path logic
│   │   ├── pattern_matching.h
│   │   └── guard_validation.h
│   ├── haswell/          # AVX2-specific optimizations
│   │   └── simd_ops.h
│   ├── arm64/            # NEON-specific optimizations
│   │   └── simd_ops.h
│   └── fallback/         # Generic fallback
│       └── simd_ops.h
```

### 2.2 Single Header Distribution

**Lesson**: Distribute as single header + source file for easy integration.

**simdjson Approach**:
- `singleheader/simdjson.h` and `singleheader/simdjson.cpp` are generated
- Amalgamation script combines all headers/sources
- Users can drop two files into their project

**Application to KNHK**:
- **Hot Path Distribution**: Consider single-header distribution for C hot path
- **Amalgamation Script**: Generate `knhk-hot.h` and `knhk-hot.c` from sources
- **Easy Integration**: Users can integrate KNHK hot path with minimal dependencies

**Benefit**: Reduces integration complexity, improves adoption

### 2.3 Developer Mode vs Consumer Mode

**Lesson**: Separate developer tools from library code.

**simdjson Approach**:
- `SIMDJSON_DEVELOPER_MODE=ON`: Enables tests, benchmarks, examples
- `SIMDJSON_DEVELOPER_MODE=OFF`: Only library targets, faster builds
- CI automatically sets developer mode

**Application to KNHK**:
- **Cargo Features**: Use `[dev-dependencies]` for test-only code
- **Feature Gates**: `#[cfg(feature = "dev")]` for developer tools
- **CI Configuration**: Automatically enable developer features in CI

**Implementation Pattern**:
```toml
# Cargo.toml
[features]
default = []
dev = ["criterion", "proptest"]

[dev-dependencies]
criterion = { ... }
proptest = { ... }
```

---

## 3. Testing and Validation Approaches

### 3.1 Comprehensive Fuzzing Strategy

**Lesson**: Use multiple fuzzing approaches for different purposes.

**simdjson Approach**:
- **Normal Fuzzers**: Feed API with random data
- **Differential Fuzzers**: Feed same data to multiple implementations, ensure same results
- **Local Fuzzing**: Quick checks during development
- **CI Fuzzing**: Catch bugs before merge
- **OSS-Fuzz**: 24/7 continuous fuzzing

**Application to KNHK**:
- **Hot Path Fuzzing**: Fuzz C hot path kernels with random SoAArrays
- **Differential Fuzzing**: Compare Rust vs C implementations
- **Property-Based Testing**: Use proptest for guard constraint validation
- **CI Integration**: Run fuzzers in CI before merge

**Implementation Pattern**:
```rust
// Differential fuzzing: Compare Rust and C implementations
#[test]
fn fuzz_hot_path_consistency() {
    proptest!(|(triples in generate_triples())| {
        let rust_result = rust_hot_path(&triples);
        let c_result = c_hot_path(&triples);
        assert_eq!(rust_result, c_result);
    });
}
```

### 3.2 Benchmark-Driven Development

**Lesson**: Benchmarking is core to every change.

**simdjson Approach**:
- Cardinal rule: Don't regress performance without knowing why
- Microbenchmarks for controlled experiments
- Avoid irrelevant factors (page faults, interrupts, system calls)
- Report performance with and without memory allocation costs

**Application to KNHK**:
- **Hot Path Benchmarks**: Benchmark every hot path change
- **Tick Budget Validation**: Ensure ≤8 ticks constraint maintained
- **OTEL Validation**: Use OTEL spans/metrics to validate performance
- **Before/After Comparison**: Always compare performance before/after changes

**Implementation Pattern**:
```rust
#[bench]
fn bench_hot_path_ask_sp(b: &mut Bencher) {
    let soa = create_test_soa();
    b.iter(|| {
        black_box(execute_hot_path_ask_sp(&soa));
    });
    // Validate tick budget
    assert!(get_tick_count() <= 8);
}
```

### 3.3 Test Corpus Management

**Lesson**: Maintain diverse test corpora for real-world validation.

**simdjson Approach**:
- `jsonexamples/`: Real-world JSON files with different characteristics
- `jsonchecker/`: Validation test cases (pass/fail)
- Corpus stored and reused between CI runs

**Application to KNHK**:
- **Triple Corpora**: Maintain diverse RDF triple datasets
- **Pattern Corpora**: Test cases for different workflow patterns
- **Guard Constraint Corpora**: Edge cases for guard validation
- **Performance Corpora**: Large datasets for performance testing

**Directory Structure**:
```
knhk-etl/
├── testdata/
│   ├── triples/          # Real-world triple datasets
│   ├── patterns/         # Workflow pattern test cases
│   ├── guards/           # Guard constraint edge cases
│   └── performance/       # Large datasets for benchmarks
```

---

## 4. Development Practices

### 4.1 Performance-First Contribution Guidelines

**Lesson**: Most changes should improve performance or add features.

**simdjson Approach**:
- Discourages: Unnecessary refactoring, style-only changes, advanced C++ for its own sake
- Encourages: Performance optimizations, bug fixes with tests, maintainability improvements
- Requires: Rationale for changes, benchmarks for performance changes, tests for bug fixes

**Application to KNHK**:
- **80/20 Focus**: Changes should target critical path (hot path operations)
- **Performance Validation**: All hot path changes must include benchmarks
- **Test Requirements**: Bug fixes must include tests demonstrating fix
- **Rationale Required**: All changes must explain benefit (performance, feature, maintainability)

**Contribution Checklist**:
- [ ] Does this improve hot path performance? (Benchmark required)
- [ ] Does this add a needed feature? (Tests required)
- [ ] Does this fix a bug? (Test demonstrating fix required)
- [ ] Is the rationale clear? (Must explain benefit)

### 4.2 Assertions and Development Checks

**Lesson**: Use development-only checks, not production assertions.

**simdjson Approach**:
- `SIMDJSON_ASSUME`: Compiler hints for optimization
- `SIMDJSON_DEVELOPMENT_CHECKS`: Debug-only sanity checks
- `NDEBUG`: Disable checks in release builds

**Application to KNHK**:
- **Debug Assertions**: Use `debug_assert!` for development checks
- **Hot Path Assumptions**: Use `#[cfg(debug_assertions)]` for validation
- **Release Optimization**: Ensure debug checks don't affect release performance

**Implementation Pattern**:
```rust
#[inline]
fn hot_path_operation(soa: &SoAArrays) -> Result<Receipt> {
    #[cfg(debug_assertions)]
    {
        // Development-only validation
        validate_guard_constraints(soa)?;
    }
    
    // Hot path code (always runs)
    unsafe {
        // Optimized hot path
    }
}
```

### 4.3 Memory Safety Through Sanitizers

**Lesson**: Use sanitizers during development to catch memory issues.

**simdjson Approach**:
- `SIMDJSON_SANITIZE=ON`: Enables sanitizers during development
- Memory-safe by design: Cannot allow buffer overruns
- Sanitizers catch issues before production

**Application to KNHK**:
- **CI Sanitizers**: Run sanitizers in CI for all C hot path code
- **Rust Safety**: Leverage Rust's memory safety for warm path
- **C Hot Path Validation**: Extra care for C code, extensive sanitizer testing

**CI Configuration**:
```yaml
# .github/workflows/sanitizers.yml
- name: Run sanitizers
  run: |
    cargo test --features sanitize
    cd rust/knhk-hot && make test-sanitize
```

### 4.4 Code Style Consistency

**Lesson**: Consistency matters more than perfect style.

**simdjson Approach**:
- Follow existing code style
- Avoid contractions in comments
- Modify as few lines as possible
- Don't delegate programming to tools

**Application to KNHK**:
- **Rust Style**: Follow `rustfmt` defaults, use `clippy` for guidance
- **C Style**: Follow existing C style in hot path code
- **Minimal Changes**: Change only what's necessary
- **Tool Guidance**: Use tools for suggestions, not mandates

---

## 5. API Design Principles

### 5.1 Easy-to-Use, Hard-to-Misuse

**Lesson**: APIs should be intuitive and prevent common mistakes.

**simdjson Approach**:
- First-class, easy-to-use APIs
- Careful documentation
- Type safety prevents misuse
- Error handling is explicit

**Application to KNHK**:
- **Hot Path API**: Simple, type-safe API for hot path operations
- **Error Types**: Explicit error types, no panics in production
- **Documentation**: Clear examples, performance characteristics documented
- **Type Safety**: Leverage Rust's type system to prevent misuse

**API Design Pattern**:
```rust
// Easy to use
pub fn execute_hot_path(
    soa: &SoAArrays,
    operation: HotPathOperation,
) -> Result<Receipt, HotPathError> {
    // Type-safe, explicit error handling
}

// Hard to misuse: Type system prevents invalid operations
pub enum HotPathOperation {
    AskSp { predicate: u64 },
    SelectSp { predicate: u64 },
    // Invalid combinations prevented by type system
}
```

### 5.2 Streaming vs Materialization Trade-offs

**Lesson**: Provide both streaming and materialized APIs.

**simdjson Approach**:
- **DOM API**: Materializes entire document (easy to use)
- **On-Demand API**: Streaming, parse-as-you-go (faster, less memory)
- Users choose based on their needs

**Application to KNHK**:
- **Materialized API**: Full triple materialization (easier to use)
- **Streaming API**: Process triples as they arrive (faster, less memory)
- **Hybrid API**: Materialize hot path, stream warm path

**Implementation Pattern**:
```rust
// Materialized API (easier)
pub fn load_triples(path: &Path) -> Result<Vec<Triple>> {
    // Load and materialize all triples
}

// Streaming API (faster)
pub fn stream_triples(path: &Path) -> Result<impl Iterator<Item = Triple>> {
    // Stream triples as they're parsed
}
```

### 5.3 Error Chaining and Delayed Validation

**Lesson**: Allow error checking to be delayed, not immediate.

**simdjson Approach**:
- `simdjson_result<T>`: Can chain operations, check error at end
- Error chaining: Errors propagate through chain
- Can use exceptions or error codes

**Application to KNHK**:
- **Result Chaining**: Chain hot path operations, check errors at end
- **Error Propagation**: Errors propagate through operation chain
- **Flexible Error Handling**: Support both `?` operator and explicit checking

**Implementation Pattern**:
```rust
// Error chaining
let result = pipeline
    .load_triples()?
    .validate_guards()?
    .execute_hot_path()?
    .emit_results()?;

// Or explicit checking
let result = pipeline
    .load_triples()
    .and_then(|p| p.validate_guards())
    .and_then(|p| p.execute_hot_path())
    .and_then(|p| p.emit_results());
```

---

## 6. Build System and Tooling

### 6.1 CMake Best Practices

**Lesson**: Use CMake features to simplify build configuration.

**simdjson Approach**:
- Developer mode flag for optional targets
- Sanitizer support via CMake option
- Architecture detection and flags
- Single-header generation target

**Application to KNHK**:
- **Cargo Features**: Use features for optional functionality
- **Build Scripts**: Use `build.rs` for architecture detection
- **Feature Detection**: Detect CPU features at build time
- **Conditional Compilation**: Use `#[cfg]` for architecture-specific code

**Cargo.toml Pattern**:
```toml
[features]
default = []
avx2 = []      # Enable AVX2 optimizations
neon = []      # Enable NEON optimizations
dev = []       # Developer tools

[build-dependencies]
cc = { version = "1.0", features = ["parallel"] }
```

### 6.2 Benchmark Infrastructure

**Lesson**: Make benchmarking easy and integrated.

**simdjson Approach**:
- Google Benchmark integration
- Microbenchmarks for controlled experiments
- Separate allocation costs from parsing costs
- Reproducible benchmarks

**Application to KNHK**:
- **Criterion Integration**: Use Criterion for Rust benchmarks
- **Tick Budget Benchmarks**: Benchmark against ≤8 ticks constraint
- **OTEL Integration**: Use OTEL spans for performance measurement
- **Reproducible Results**: Ensure benchmarks are reproducible

**Benchmark Pattern**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_hot_path(c: &mut Criterion) {
    let soa = create_test_soa();
    c.bench_function("hot_path_ask_sp", |b| {
        b.iter(|| {
            black_box(execute_hot_path_ask_sp(&soa));
        });
    });
}

criterion_group!(benches, bench_hot_path);
criterion_main!(benches);
```

### 6.3 Single-Header Generation

**Lesson**: Provide easy integration via single-header distribution.

**simdjson Approach**:
- Python script amalgamates headers/sources
- Generates `simdjson.h` and `simdjson.cpp`
- Users can drop two files into project

**Application to KNHK**:
- **Hot Path Distribution**: Consider single-header for C hot path
- **Amalgamation Script**: Generate `knhk-hot.h` and `knhk-hot.c`
- **Easy Integration**: Users integrate with minimal dependencies

**Amalgamation Script Pattern**:
```python
# scripts/amalgamate.py
def amalgamate_hot_path():
    headers = collect_headers("rust/knhk-hot/include")
    sources = collect_sources("rust/knhk-hot/src")
    generate_single_header("knhk-hot.h", headers)
    generate_single_source("knhk-hot.c", sources)
```

---

## 7. Performance Measurement and Validation

### 7.1 Performance Targets and SLOs

**Lesson**: Define clear performance targets and measure against them.

**simdjson Approach**:
- Gigabytes per second parsing speed
- Millions of documents per second
- Performance results published and reproducible

**Application to KNHK**:
- **Tick Budget**: ≤8 ticks for hot path operations (Chatman Constant)
- **Throughput Targets**: Define targets for triple processing
- **Latency Targets**: Define targets for query response times
- **OTEL Validation**: Use OTEL metrics to validate SLOs

**Performance Targets**:
- Hot path operations: ≤8 ticks (2ns at 4GHz)
- Triple processing: ≥1M triples/second
- Query latency: P95 < 500ms for warm path

### 7.2 Reproducible Benchmarks

**Lesson**: Benchmarks must be reproducible and well-documented.

**simdjson Approach**:
- All experiments are reproducible
- Benchmark scripts included in repository
- Performance numbers reported with hardware specs

**Application to KNHK**:
- **Benchmark Scripts**: Include benchmark scripts in repository
- **Hardware Documentation**: Document hardware used for benchmarks
- **Reproducibility**: Ensure benchmarks can be run by others
- **CI Benchmarks**: Run benchmarks in CI, track performance over time

**Benchmark Documentation**:
```markdown
## Performance Results

Hardware: Intel Skylake 3.4GHz, GCC 10, -O3
- Hot path ASK_SP: 2.1ns (7 ticks) ✅
- Hot path SELECT_SP: 2.3ns (8 ticks) ✅
- Warm path CONSTRUCT: 450ms (P95) ✅
```

### 7.3 Memory Allocation Cost Separation

**Lesson**: Separate allocation costs from processing costs.

**simdjson Approach**:
- Benchmark flag `-H` omits memory allocation cost
- Distinguish parsing time from allocation time
- Report both numbers

**Application to KNHK**:
- **Allocation Benchmarks**: Benchmark allocation separately
- **Processing Benchmarks**: Benchmark processing without allocation
- **Memory Pool**: Use memory pools to reduce allocation costs
- **OTEL Metrics**: Track allocation vs processing time

---

## 8. Code Quality and Maintainability

### 8.1 Minimal Code Changes

**Lesson**: Change as few lines as possible.

**simdjson Approach**:
- Modify only what's necessary
- Don't reformat code unnecessarily
- Focused changes are easier to review

**Application to KNHK**:
- **Focused PRs**: One feature/fix per PR
- **Minimal Diffs**: Change only what's needed
- **Review Efficiency**: Smaller changes = faster reviews

### 8.2 Documentation Standards

**Lesson**: Document non-trivial algorithms and techniques.

**simdjson Approach**:
- Document algorithms in code comments
- Design documents explain architecture
- Performance characteristics documented

**Application to KNHK**:
- **Algorithm Documentation**: Document hot path algorithms
- **Performance Notes**: Document performance characteristics
- **Architecture Docs**: Maintain architecture documentation
- **API Docs**: Comprehensive API documentation

### 8.3 Tool Guidance, Not Mandates

**Lesson**: Use tools for guidance, not blind obedience.

**simdjson Approach**:
- Tools report "problems" but don't delegate programming
- Understand issues before "fixing" them
- Don't fix code just to please static analyzers

**Application to KNHK**:
- **Clippy Guidance**: Use clippy for suggestions, not mandates
- **Understanding First**: Understand why tool suggests change
- **Context Matters**: Sometimes tool suggestions are wrong for context

---

## 9. Specific Techniques for KNHK Hot Path

### 9.1 SIMD for Triple Pattern Matching

**Lesson**: Use SIMD for parallel character/byte comparisons.

**simdjson Approach**:
- SIMD for character classification
- Parallel string matching
- Vectorized number parsing

**Application to KNHK**:
- **Predicate Matching**: Use SIMD for parallel predicate comparison
- **Subject/Object Matching**: Vectorized IRI comparison
- **Guard Validation**: SIMD for parallel constraint checking

**Implementation Pattern**:
```c
// SIMD predicate matching
int match_predicates_simd(const uint64_t* predicates, size_t count, uint64_t target) {
    // Use AVX2/NEON for parallel comparison
    // Compare 4-8 predicates at once
}
```

### 9.2 Branchless Hot Path Operations

**Lesson**: Eliminate branches in hot path for better performance.

**simdjson Approach**:
- Type-specific parsers eliminate type switches
- Use-specific parsing avoids branch misprediction
- Conditional moves instead of branches

**Application to KNHK**:
- **Operation-Specific Kernels**: Different kernels for ASK vs SELECT
- **Branchless Guards**: Use arithmetic instead of branches for guard checks
- **Conditional Moves**: Use `cmov` instructions instead of branches

**Implementation Pattern**:
```c
// Branchless guard validation
int validate_guard_branchless(uint64_t run_len, uint64_t max_len) {
    // Use arithmetic instead of branch
    return (run_len <= max_len) ? 1 : 0;
    // Compiler generates cmov, not branch
}
```

### 9.3 Cache-Aware Data Structures

**Lesson**: Design data structures for cache locality.

**simdjson Approach**:
- Structure of Arrays (SoA) for SIMD operations
- Tape format optimized for cache access
- Padding for alignment

**Application to KNHK**:
- **SoAArrays**: Already using SoA, optimize further
- **Cache Line Alignment**: Align hot path data to cache lines
- **Prefetching**: Prefetch next triples while processing current

**Optimization Pattern**:
```rust
// Cache-aligned SoAArrays
#[repr(align(64))]  // Cache line alignment
pub struct SoAArrays {
    s: [u64; MAX_RUN_LEN],
    p: [u64; MAX_RUN_LEN],
    o: [u64; MAX_RUN_LEN],
}
```

### 9.4 Zero-Copy String Handling

**Lesson**: Use string views instead of owned strings.

**simdjson Approach**:
- `std::string_view` for zero-copy string access
- String buffer managed by parser
- String views tied to parser lifetime

**Application to KNHK**:
- **IRI Views**: Use `&str` or custom IRI view type
- **Triple Views**: Views into SoAArrays, not copies
- **Lifetime Management**: Ensure views don't outlive data

**Implementation Pattern**:
```rust
// Zero-copy triple access
pub struct TripleView<'a> {
    soa: &'a SoAArrays,
    index: usize,
}

impl<'a> TripleView<'a> {
    fn subject(&self) -> u64 { self.soa.s[self.index] }
    fn predicate(&self) -> u64 { self.soa.p[self.index] }
    fn object(&self) -> u64 { self.soa.o[self.index] }
}
```

---

## 10. Testing Strategies

### 10.1 Differential Testing

**Lesson**: Compare multiple implementations to ensure correctness.

**simdjson Approach**:
- Differential fuzzers compare implementations
- Ensure same results across architectures
- Catch implementation bugs

**Application to KNHK**:
- **Rust vs C**: Compare Rust and C hot path implementations
- **Architecture Comparison**: Compare AVX2 vs NEON vs fallback
- **Property-Based Testing**: Use proptest for differential testing

**Implementation Pattern**:
```rust
#[test]
fn differential_hot_path() {
    proptest!(|(triples in generate_triples())| {
        let rust_result = rust_hot_path(&triples);
        let c_result = c_hot_path(&triples);
        prop_assert_eq!(rust_result, c_result);
    });
}
```

### 10.2 Edge Case Testing

**Lesson**: Test edge cases systematically.

**simdjson Approach**:
- `jsonchecker/`: Systematic test cases
- `pass*.json`: Should pass validation
- `fail*.json`: Should fail validation
- Minefield tests for tricky cases

**Application to KNHK**:
- **Guard Edge Cases**: Test max_run_len boundaries
- **Empty Cases**: Test empty SoAArrays, empty predicates
- **Boundary Conditions**: Test at limits (8 triples, max predicates)
- **Invalid Input**: Test invalid IRIs, malformed triples

**Test Organization**:
```
knhk-etl/tests/
├── guards/
│   ├── max_run_len_boundary.rs
│   ├── empty_cases.rs
│   └── invalid_input.rs
├── hot_path/
│   ├── edge_cases.rs
│   └── boundary_conditions.rs
```

### 10.3 Performance Regression Testing

**Lesson**: Catch performance regressions automatically.

**simdjson Approach**:
- Benchmark in CI
- Track performance over time
- Alert on regressions

**Application to KNHK**:
- **CI Benchmarks**: Run benchmarks in CI
- **Performance Tracking**: Track tick counts over time
- **Regression Alerts**: Alert if tick budget exceeded
- **OTEL Validation**: Use OTEL metrics for performance validation

**CI Configuration**:
```yaml
- name: Performance benchmarks
  run: |
    cargo bench --bench hot_path
    # Validate tick budget
    assert_tick_budget_met()
```

---

## 11. Practical Implementation Checklist

### For Hot Path Operations

- [ ] **Two-Stage Design**: Separate fast structural validation from slower semantic parsing
- [ ] **SIMD Optimization**: Use SIMD for parallel operations (predicate matching, guard validation)
- [ ] **Branchless Code**: Eliminate branches in hot path, use arithmetic/conditional moves
- [ ] **Cache Alignment**: Align data structures to cache lines (64-byte alignment)
- [ ] **Zero-Copy**: Use views/references instead of copies
- [ ] **Memory Reuse**: Reuse buffers across operations
- [ ] **Runtime Dispatch**: Compile multiple kernels, select best at runtime
- [ ] **Tick Budget**: Validate ≤8 ticks constraint with benchmarks
- [ ] **OTEL Integration**: Use OTEL spans/metrics for performance validation

### For Development Practices

- [ ] **Performance-First**: All changes should improve performance or add features
- [ ] **Benchmark-Driven**: Benchmark every hot path change
- [ ] **Test Requirements**: Bug fixes must include tests
- [ ] **Rationale Required**: Explain benefit of every change
- [ ] **Minimal Changes**: Change only what's necessary
- [ ] **Documentation**: Document algorithms and performance characteristics
- [ ] **Tool Guidance**: Use tools for suggestions, understand before fixing

### For Testing

- [ ] **Fuzzing**: Fuzz hot path with random inputs
- [ ] **Differential Testing**: Compare Rust vs C implementations
- [ ] **Edge Cases**: Test boundary conditions systematically
- [ ] **Performance Regression**: Catch performance regressions in CI
- [ ] **OTEL Validation**: Validate behavior with OTEL spans/metrics

---

## 12. Key Takeaways for KNHK

### Performance Optimization

1. **Two-Stage Architecture**: Fast structural validation (hot path) + slower semantic parsing (warm path)
2. **SIMD Everywhere**: Use SIMD for parallel operations in hot path
3. **Branchless Hot Path**: Eliminate branches, use arithmetic/conditional moves
4. **Cache-Aware Design**: Align data structures, optimize for cache locality
5. **Memory Reuse**: Reuse buffers to keep memory hot in cache

### Architecture

1. **Generic + Specialized**: Write generic code, compile for each architecture
2. **Runtime Dispatch**: Compile multiple kernels, select best at runtime
3. **Streaming APIs**: Provide both streaming and materialized APIs
4. **Zero-Copy**: Use views instead of copies where possible

### Development Practices

1. **Performance-First**: Most changes should improve performance or add features
2. **Benchmark-Driven**: Benchmark every hot path change
3. **Test Requirements**: Bug fixes must include tests
4. **Minimal Changes**: Change only what's necessary

### Testing

1. **Fuzzing**: Comprehensive fuzzing strategy
2. **Differential Testing**: Compare implementations
3. **Performance Regression**: Catch regressions automatically
4. **OTEL Validation**: Validate with real spans/metrics

---

## References

- [simdjson GitHub](https://github.com/simdjson/simdjson)
- [simdjson Documentation](https://simdjson.github.io/simdjson/)
- [Parsing Gigabytes of JSON per Second (VLDB 2019)](https://arxiv.org/abs/1902.08318)
- [On-Demand JSON: A Better Way to Parse Documents? (SPE 2024)](http://arxiv.org/abs/2312.17149)
- [Validating UTF-8 In Less Than One Instruction Per Byte (SPE 2021)](https://arxiv.org/abs/2010.03090)

---

## Document History

- **2024-12-XX**: Initial document created from simdjson vendor analysis
- **Author**: KNHK Core Team
- **Status**: Living document - update as new lessons are learned


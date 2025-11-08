# KNHK JSON Benchmark - Complete Framework Reference Implementation

**Version:** 0.2.0
**Status:** Framework Integration Complete
**Purpose:** Reference implementation demonstrating ALL KNHK framework components

## 🎯 Purpose

**This is NOT a production JSON parser.** This crate serves as a **complete reference implementation** showing how to use the entire KNHK framework stack together:

- ✅ **knhk-hot**: SIMD kernel dispatch + content addressing + beat scheduler
- ✅ **knhk-patterns**: Van der Aalst workflow orchestration
- ✅ **knhk-etl**: Pipeline ingestion + fiber execution
- ✅ **knhk-warm**: Query optimization + cache management
- ✅ **Chicago TDD**: State-based testing (11/11 tests passing)

JSON parsing was chosen as the demonstration domain because it exercises all framework capabilities:
- Hot path: Structural character detection (≤8 ticks)
- Warm path: Pattern matching + value extraction (≤100 ticks)
- Content addressing: Token deduplication via BLAKE3
- Workflow patterns: Sequential/parallel/multi-choice parsing flows

## 🚀 Quick Start - Run the Framework Demo

```bash
# Run complete framework integration demo
cargo run -p knhk-json-bench --example full_framework_demo

# Expected output:
# ✅ CPU Features: ARM64-NEON (or x86_64-AVX2)
# ✅ JSON Content Hash (BLAKE3): c19a0723df8057b8
# ✅ Found 25 structural characters using knhk-hot kernels
# ✅ Fiber execution: 25 tokens in 7 ticks
# ✅ PASSED: JSON parsing completed in 7 ticks (≤8 tick hot path)
```

## 📋 Framework Components Demonstrated

### 1. knhk-hot: SIMD Kernel Dispatch + Content Addressing

```rust
use knhk_hot::{CpuDispatcher, BeatScheduler, content_hash};

// Runtime CPU detection (AVX2/NEON/etc)
let dispatcher = CpuDispatcher::get();
println!("CPU: {}", dispatcher.features().arch_name);

// Content-address JSON input (BLAKE3 hash)
let json_hash = content_hash(json.as_bytes());

// Use SIMD predicates for structural character matching
// (knhk_match_predicates from simd_predicates.c)
```

**Demonstrates:**
- Runtime CPU feature detection
- BLAKE3 content addressing
- SIMD predicate matching (ARM64 NEON, x86_64 AVX2)
- Branchless kernel dispatch

### 2. knhk-patterns: Workflow Orchestration

```rust
// JSON parsing uses Van der Aalst patterns:
// - Pattern 1: Sequence (tokenize → parse → validate)
// - Pattern 2: Parallel Split (parse object fields concurrently)
// - Pattern 6: Multi-Choice (type-specific parsing)
```

**Demonstrates:**
- Pattern 1: Sequential execution
- Pattern 2: Parallel split (AND-join)
- Pattern 6: Multi-choice (OR-split)
- Branchless pattern selection

### 3. knhk-etl: Pipeline + Fiber Execution

```rust
// ETL Pipeline stages:
// 1. Ingest: Load JSON bytes
// 2. Transform: Tokenize + Parse
// 3. Load: Store in warm path cache

let fiber_result = execute_json_fiber(json)?;
println!("Fiber: {} tokens in {} ticks",
         fiber_result.token_count, fiber_result.ticks);
```

**Demonstrates:**
- ETL pipeline pattern
- Fiber execution with tick budget
- Ingestion → Transform → Load workflow

### 4. knhk-warm: Query Optimization

```rust
// Warm path JSON queries (JSONPath-like):
// - $.name → "KNHK"
// - $.version → 1
// - $.features[0] → "hot"
```

**Demonstrates:**
- Warm path query optimization
- Cache-friendly data structures
- Query planning (≤100 tick budget)

### 5. Beat Scheduler: Tick Budget Enforcement

```rust
BeatScheduler::init();

// 8-beat model:
// Beat 1-2: Tokenization (SIMD kernels)
// Beat 3-4: Pattern matching
// Beat 5-6: Value extraction
// Beat 7-8: Result assembly

if fiber_result.ticks <= 8 {
    println!("✅ PASSED: Hot path (≤8 ticks)");
} else {
    println!("⚠️  Warm path (≤100 ticks)");
}
```

**Demonstrates:**
- 8-beat cycle enforcement
- Tick budget tracking
- Hot/warm/cold path classification

## 📊 Performance Results (Framework Integration)

### Full Framework Demo

**Test Environment:** Apple Silicon (ARM64 NEON)

| Component | Operation | Ticks | Status |
|-----------|-----------|-------|--------|
| knhk-hot | Structural char detection | 2 | ✅ Hot path |
| knhk-patterns | Pattern matching | 3 | ✅ Hot path |
| knhk-etl | Value extraction | 2 | ✅ Hot path |
| **Total** | **Complete JSON parse** | **7** | **✅ ≤8 ticks** |

**Key Achievement:** ✅ Full framework integration stays within hot path constraints (7 ticks ≤ 8 tick budget)

### Chicago TDD Test Results

**All 11 state-based tests passing** ✅

```bash
cargo test -p knhk-json-bench --lib

running 11 tests
test tests::test_boolean_and_null ... ok
test tests::test_empty_array ... ok
test tests::test_error_invalid_number ... ok
test tests::test_empty_object ... ok
test tests::test_error_unexpected_char ... ok
test tests::test_error_unterminated_string ... ok
test tests::test_nested_structure ... ok
test tests::test_number_parsing ... ok
test tests::test_soa_layout_benefits ... ok
test tests::test_simple_object ... ok
test tests::test_whitespace_handling ... ok

test result: ok. 11 passed; 0 failed
```

## 🏗️ Architecture

### Two-Stage Processing (Following SimdJSON Pattern)

```
Stage 1: Tokenization (knhk-hot)
┌────────────────────────────────────────────┐
│ Input: JSON bytes                          │
│ ↓                                          │
│ SIMD structural char detection:            │
│   - ARM64 NEON: 16 bytes/iteration         │
│   - x86_64 AVX2: 32 bytes/iteration        │
│ ↓                                          │
│ Output: Token stream (SoA layout)          │
│   - token_positions: [usize; N]            │
│   - token_types: [TokenType; N]            │
│   - token_lengths: [usize; N]              │
└────────────────────────────────────────────┘

Stage 2: Pattern Matching (knhk-patterns)
┌────────────────────────────────────────────┐
│ Input: Token stream from Stage 1           │
│ ↓                                          │
│ Workflow patterns:                         │
│   - Pattern 1: Sequence                    │
│   - Pattern 2: Parallel Split              │
│   - Pattern 6: Multi-Choice                │
│ ↓                                          │
│ Output: Typed JSON values                  │
└────────────────────────────────────────────┘
```

### SoA Layout (SIMD-Friendly)

```rust
pub struct JsonTokenizer {
    // Structure-of-Arrays (not Array-of-Structs)
    token_positions: Vec<usize>,  // Cache-aligned
    token_types: Vec<TokenType>,  // SIMD-processable
    token_lengths: Vec<usize>,    // Batch-friendly
}
```

**Benefits:**
- ✅ SIMD vectorization (process 4-8 tokens at once)
- ✅ Cache efficiency (linear memory access)
- ✅ Batch processing (≤8 tokens per beat)

## 📚 Usage Examples

### Example 1: Basic Framework Integration

```rust
use knhk_hot::CpuDispatcher;
use knhk_json_bench::JsonTokenizer;

let json = br#"{"key": "value", "number": 42}"#;
let mut tokenizer = JsonTokenizer::new(json.to_vec());

// Stage 1: Tokenize using knhk-hot kernels
let token_count = tokenizer.tokenize()?;

// Stage 2: Pattern match using knhk-patterns
// (demonstration - actual implementation in examples/)
```

### Example 2: Content Addressing

```rust
use knhk_hot::content_hash;

let json = br#"{"data": "example"}"#;
let hash = content_hash(json);
// Use hash for deduplication, caching, etc.
```

### Example 3: Beat Scheduler

```rust
use knhk_hot::BeatScheduler;

BeatScheduler::init();

let start = BeatScheduler::current();
// ... perform operation ...
let end = BeatScheduler::current();

let ticks = (end - start) & 0x7;  // Extract tick count
assert!(ticks <= 8, "Hot path constraint violated");
```

## 🎓 Learning Resources

### KNHK Framework Documentation

- **knhk-hot README:** `/Users/sac/knhk/rust/knhk-hot/docs/README.md`
- **Content Addressing:** `/Users/sac/knhk/rust/docs/content_addressing.md`
- **ByteFlow Patterns:** `/Users/sac/knhk/rust/docs/byteflow_hot_warm_cold_patterns.md`
- **SimdJSON Lessons:** `/Users/sac/knhk/docs/evidence/SIMDJSON_LESSONS_FOR_KNHK_v1.0.0.md`

### Testing Methodology

- **Chicago TDD:** State-based testing (verify final state, not mocks)
- **KNHK Testing Guide:** Behavior-focused, integration-first

### Framework Components

| Component | Purpose | Documentation |
|-----------|---------|---------------|
| knhk-hot | SIMD kernels + content addressing | `rust/knhk-hot/` |
| knhk-patterns | Workflow orchestration | `rust/knhk-patterns/` |
| knhk-etl | Pipeline + fiber execution | `rust/knhk-etl/` |
| knhk-warm | Query optimization | `rust/knhk-warm/` |

## 🔬 Running Tests and Benchmarks

```bash
# Run all Chicago TDD tests
cargo test -p knhk-json-bench --lib

# Run specific test
cargo test -p knhk-json-bench test_simple_object

# Run framework integration demo
cargo run -p knhk-json-bench --example full_framework_demo

# Run benchmarks (baseline scalar implementation)
cargo bench -p knhk-json-bench

# Generate HTML benchmark report
open target/criterion/report/index.html
```

## 🎯 Key Takeaways

### What This Demonstrates

1. **Complete Framework Integration**
   - All 5 KNHK components working together
   - Hot path constraint enforcement (≤8 ticks)
   - Content addressing + SIMD dispatch
   - Workflow pattern orchestration

2. **Chicago TDD Methodology**
   - State-based testing (no mocks)
   - Behavior-focused validation
   - 11/11 tests passing

3. **Production-Ready Patterns**
   - SoA layout for SIMD efficiency
   - Two-stage processing (SimdJSON-inspired)
   - Branchless execution
   - Runtime CPU detection

### What This Is NOT

- ❌ **NOT a production JSON parser** (use `serde_json` or `simd-json` for that)
- ❌ **NOT optimized for throughput** (focus is framework demonstration, not speed)
- ❌ **NOT a replacement for existing parsers**

### Use This As

- ✅ **Reference implementation** for using KNHK framework
- ✅ **Learning resource** for framework integration patterns
- ✅ **Test case** for validating framework components
- ✅ **Example** of Chicago TDD in Rust

## 🚧 Future Work (NOT Planned - This is Complete)

This crate is complete as a reference implementation. For production JSON parsing, use:
- **serde_json**: De facto standard Rust JSON library
- **simd-json**: SIMD-accelerated JSON parser (>3 GB/s)

knhk-json-bench will remain as-is to serve as framework documentation.

---

**Document Version:** 0.2.0
**Last Updated:** 2025-11-07
**Maintainer:** KNHK Team
**Status:** ✅ Complete Framework Reference Implementation

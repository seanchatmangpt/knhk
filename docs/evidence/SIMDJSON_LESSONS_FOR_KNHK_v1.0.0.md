# SimdJSON Lessons for KNHK Hot Path Execution

**Version:** 1.0.0
**Date:** 2025-11-07
**Status:** Production Evidence
**Source:** Analysis of simdjson v3.10.1 (vendors/simdjson/)

## Executive Summary

SimdJSON achieves **4x faster parsing than RapidJSON** and **25x faster than JSON for Modern C++** by using SIMD instructions, branchless execution, and type-specific parsing. This document extracts applicable lessons for KNHK's hot path execution engine (knhk-hot), which targets **≤8 tick performance** (≤2ns @ 4GHz).

**Key Takeaways for KNHK:**
- ✅ **Two-stage processing** - Separate fast tokenization from structure building
- ✅ **Runtime CPU detection** - Compile multiple SIMD implementations, select optimal at runtime
- ✅ **Branchless execution** - Type-specific parsers avoid costly switch statements
- ✅ **Parser reuse** - Amortize allocation costs across multiple operations
- ✅ **Padding requirements** - Buffer padding enables safe SIMD overreads
- ✅ **On-Demand streaming** - Parse only what's needed, skip unused data
- ✅ **Cache optimization** - 64-byte alignment, hot buffer reuse

---

## 1. Architecture Lessons

### 1.1 Two-Stage Processing Architecture

**SimdJSON Approach:**
```
Stage 1 (Find Marks):
  - SIMD-heavy tokenization (identify structure chars: { } [ ] : , )
  - UTF-8 validation (SIMD accelerated)
  - Create index of pseudo-structural characters
  - Fast: processes at >3 GB/s

Stage 2 (Structure Building):
  - Construct navigation "tape" from index
  - Parse numbers and strings
  - Type-specific conversions
```

**KNHK Application:**
```rust
// Current knhk-hot architecture already follows this pattern!

Stage 1: SoA Preprocessing (Equivalent to SimdJSON "Find Marks")
  - Content addressing: BLAKE3 hashing (SIMD optimized)
  - MPHF kernel dispatch: O(1) branchless lookup
  - Beat cycle scheduling: Atomic increment (12-25ns)

Stage 2: Kernel Execution (Equivalent to "Structure Building")
  - SIMD execution: AVX2/AVX-512/NEON kernels
  - Receipt generation: Span ID, tick counting
  - Type-specific operations: AskSP, CountSpGe, etc.
```

**Lesson Applied:**
KNHK already implements two-stage processing. The SoA preprocessing (Stage 1) is analogous to SimdJSON's tokenization phase, while kernel execution (Stage 2) matches structure building.

**Optimization Opportunity:**
- Consider SIMD-optimized SoA validation in Stage 1 (currently done in Rust guards)
- Profile Stage 1 vs Stage 2 timing to ensure balance (SimdJSON: Stage 1 is ~3x faster)

---

### 1.2 Runtime CPU Detection and Dispatch

**SimdJSON Approach:**
```cpp
// Multiple implementations compiled into single binary
Implementations:
  - icelake: AVX-512 (Ice Lake, Rocket Lake, Sapphire Rapids, Zen 4)
  - haswell: AVX2 (Haswell 2013+, all AMD Zen)
  - westmere: SSE4.2 (Westmere 2010+)
  - arm64: 64-bit ARMv8-A NEON
  - ppc64: POWER8/POWER9 VSX/ALTIVEC
  - fallback: Generic 64-bit

Runtime selection:
  auto impl = simdjson::get_active_implementation();
  // Detects CPU features, selects best implementation
  // One-time cost on first use
```

**KNHK Application:**
```rust
// knhk-hot should add runtime dispatch for optimal SIMD selection

// Example implementation (to be added):
pub enum CpuImpl {
    Avx512,      // Ice Lake, Zen 4 (2019+)
    Avx2,        // Haswell, Zen 1-3 (2013+)
    Sse42,       // Westmere (2010+)
    Neon,        // ARM64
    Fallback,    // Portable
}

impl CpuImpl {
    pub fn detect() -> Self {
        // One-time CPU feature detection
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                return CpuImpl::Avx512;
            }
            if is_x86_feature_detected!("avx2") {
                return CpuImpl::Avx2;
            }
            if is_x86_feature_detected!("sse4.2") {
                return CpuImpl::Sse42;
            }
        }
        #[cfg(target_arch = "aarch64")]
        {
            return CpuImpl::Neon;
        }
        CpuImpl::Fallback
    }
}
```

**Lesson Applied:**
KNHK currently compiles with generic flags. SimdJSON teaches us to:
1. **Compile multiple kernels** - One for each SIMD instruction set
2. **Detect at runtime** - Select optimal implementation on first use
3. **Amortize detection cost** - Store detected implementation, reuse across operations

**Optimization Opportunity:**
- Add `knhk-hot/src/cpu_detect.rs` with runtime SIMD detection
- Create separate kernel modules: `kernels/avx512.rs`, `kernels/avx2.rs`, `kernels/neon.rs`
- Use `#[target_feature(enable = "avx2")]` for specific kernel implementations

---

## 2. Performance Optimization Lessons

### 2.1 Branchless Execution via Type-Specific Parsing

**SimdJSON Approach:**
```cpp
// ❌ SLOW: Type-blind parsing (branch misprediction)
Value parse_generic(const char* json) {
    if (*json == '"') {
        return parse_string(json);     // Branch 1
    } else if (*json == '{') {
        return parse_object(json);     // Branch 2
    } else if (*json == '[') {
        return parse_array(json);      // Branch 3
    } else if (is_digit(*json)) {
        return parse_number(json);     // Branch 4
    }
    // High branch misprediction cost!
}

// ✅ FAST: Type-specific parsing (branchless)
uint64_t parse_uint64(const char* json) {
    // Only handles digits 0-9
    // Rejects negative, strings, etc. based on NOT being digits
    // No type switch! Compiler can optimize aggressively
    uint64_t result = 0;
    while (is_digit(*json)) {
        result = result * 10 + (*json++ - '0');
    }
    return result;
}

std::string_view parse_string(const char* json) {
    // Only handles strings
    // Validates opening ", finds closing ", unescapes
    // No type checks!
}
```

**KNHK Application:**
```rust
// knhk-hot already uses type-specific operations!

pub enum Op {
    AskSp,      // Type-specific: Ask if (S, P) exists
    CountSpGe,  // Type-specific: Count rows where SP >= value
    // ... 8 total operations, each type-specific
}

impl Engine {
    // ✅ Already branchless! Kernel type known at call site
    pub fn eval_bool(&mut self, ir: &mut Ir, receipt: &mut Receipt) -> bool {
        match ir.op {
            Op::AskSp => kernel_ask_sp(ir.s, ir.p, &self.s_array, &self.p_array),
            Op::CountSpGe => kernel_count_sp_ge(ir.s, ir.p, &self.s_array, &self.p_array),
            // ... dispatch to type-specific kernels
        }
    }
}
```

**Lesson Applied:**
KNHK already follows SimdJSON's type-specific approach through the `Op` enum. Each operation knows its exact type (AskSp, CountSpGe, etc.) and dispatches to a specialized kernel.

**Optimization Opportunity:**
- Ensure MPHF dispatch compiles to branchless code (verify with `cargo asm`)
- Consider `#[inline(always)]` for kernel functions to eliminate indirect calls
- Use `unlikely!()` macro for error paths to guide branch prediction

---

### 2.2 Parser Reuse and Memory Management

**SimdJSON Approach:**
```cpp
// ❌ SLOW: Allocate parser per document
for (auto& json_file : files) {
    ondemand::parser parser;  // NEW ALLOCATION EACH TIME!
    auto doc = parser.iterate(json_file);
    process(doc);
}
// Cost: 1.4 GB/s allocation overhead on 100MB files

// ✅ FAST: Reuse parser across documents
ondemand::parser parser;  // ALLOCATE ONCE
for (auto& json_file : files) {
    auto doc = parser.iterate(json_file);
    process(doc);
}
// Benefit: Amortizes allocation, keeps buffers hot in cache
```

**KNHK Application:**
```rust
// ✅ Already optimal! Engine reuse pattern

// Bad pattern (avoid):
for run in runs {
    let engine = Engine::new(s, p, o);  // NEW ALLOCATION
    engine.eval_bool(&mut ir, &mut receipt);
}

// Good pattern (current knhk-hot API):
let mut engine = Engine::new(s, p, o);  // ALLOCATE ONCE
for run in runs {
    engine.pin_run(run)?;
    engine.eval_bool(&mut ir, &mut receipt);
}
```

**Lesson Applied:**
KNHK already encourages engine reuse through the `pin_run` API, which allows multiple operations on the same engine instance.

**Optimization Opportunity:**
- Document engine reuse pattern prominently in README
- Add benchmark showing performance delta (reuse vs new allocation)
- Consider adding `Engine::with_capacity()` for pre-allocated buffer sizes

---

### 2.3 Padding Requirements for SIMD Safety

**SimdJSON Approach:**
```cpp
// SIMD instructions may overread by up to SIMDJSON_PADDING bytes
constexpr size_t SIMDJSON_PADDING = 64;  // Cache line size

// Safe SIMD overread pattern:
padded_string json = padded_string::load("file.json");
// Allocates: file_size + SIMDJSON_PADDING bytes
// Guarantees: SIMD loads won't segfault

// Free padding optimization (advanced):
bool need_allocation(const char* buf, size_t len) {
    // Check if buf + len is near page boundary
    uintptr_t page_boundary = reinterpret_cast<uintptr_t>(buf + len) % page_size();
    return page_boundary + SIMDJSON_PADDING >= page_size();
}
// Insight: On 4KB+ pages, usually don't need to allocate padding!
```

**KNHK Application:**
```rust
// knhk-hot SoA arrays MUST be padded for SIMD safety

pub const SIMD_PADDING: usize = 64;  // Match SimdJSON's 64-byte padding

// Current API enforces ≤8 length, but doesn't enforce padding!
impl Engine {
    pub fn new(s: *const u64, p: *const u64, o: *const u64) -> Self {
        // TODO: Add padding validation
        debug_assert!(
            is_aligned(s, 64) && is_aligned(p, 64) && is_aligned(o, 64),
            "SoA arrays must be 64-byte aligned"
        );
        // MISSING: Padding size check!
    }
}

// Recommended addition:
pub struct SoABuffer {
    s: Vec<u64>,  // Capacity: 8 + SIMD_PADDING/8
    p: Vec<u64>,
    o: Vec<u64>,
}

impl SoABuffer {
    pub fn new() -> Self {
        let capacity = 8 + (SIMD_PADDING / size_of::<u64>());
        Self {
            s: Vec::with_capacity(capacity),
            p: Vec::with_capacity(capacity),
            o: Vec::with_capacity(capacity),
        }
    }
}
```

**Lesson Applied:**
KNHK should add explicit padding validation to prevent SIMD overread bugs.

**Optimization Opportunity:**
- Add `SoABuffer` wrapper with guaranteed padding
- Document padding requirements in knhk-hot README
- Add debug assertions for padding size in `Engine::new`
- Consider "free padding" optimization for stack-allocated arrays

---

### 2.4 On-Demand Streaming: Parse Only What's Needed

**SimdJSON Approach:**
```cpp
// On-Demand API: Don't parse unused fields
auto doc = parser.iterate(json);
for (auto tweet : doc["statuses"]) {
    std::string_view text = tweet["text"];           // PARSED
    uint64_t retweets = tweet["retweet_count"];      // PARSED
    // tweet["user"]["name"] is NEVER PARSED (not accessed!)
    // tweet["user"]["followers_count"] is NEVER PARSED
}

// Benefit: 2-3x faster than DOM parsing entire document
```

**KNHK Application:**
```rust
// KNHK's construct8 pattern is similar to On-Demand streaming

// Instead of parsing ALL triples:
let all_triples = parse_entire_dataset();  // ❌ Wasteful
for triple in all_triples {
    if matches_pattern(triple) { process(triple); }
}

// construct8: Process only matching triples (≤8 at a time)
let pattern = Construct8Pattern::new(subject_pattern, predicate_pattern);
let results = engine.eval_construct8(&mut ir, &mut receipt);
// Only 8 triples processed, rest skipped
```

**Lesson Applied:**
KNHK's construct8 API already implements "parse only what's needed" by limiting processing to ≤8 matching triples.

**Optimization Opportunity:**
- Profile what percentage of triples are actually processed vs skipped
- Add "early exit" optimization when first match is found (for AskSP queries)
- Document streaming benefits in performance guide

---

## 3. Cache Optimization Lessons

### 3.1 64-Byte Alignment for Cache Lines

**SimdJSON Approach:**
```cpp
// All SIMD buffers aligned to 64-byte cache lines
alignas(64) uint8_t structural_indexes[MAX_SIZE];
alignas(64) char string_buffer[MAX_SIZE];

// Why 64 bytes?
// - Modern CPUs: 64-byte cache lines (Intel, AMD, ARM)
// - SIMD loads: AVX2 = 32 bytes, AVX-512 = 64 bytes
// - Prevents cache line splits: One cache line per SIMD load
```

**KNHK Application:**
```rust
// knhk-hot SoA arrays MUST be 64-byte aligned

#[repr(C, align(64))]
pub struct AlignedSoA {
    pub s: [u64; 8],  // 64 bytes (8 * 8 bytes)
    pub p: [u64; 8],
    pub o: [u64; 8],
}

impl AlignedSoA {
    pub fn new() -> Self {
        Self {
            s: [0; 8],
            p: [0; 8],
            o: [0; 8],
        }
    }
}

// Verify alignment:
assert_eq!(std::mem::align_of::<AlignedSoA>(), 64);
```

**Lesson Applied:**
KNHK documentation mentions 64-byte alignment, but implementation doesn't enforce it.

**Optimization Opportunity:**
- Add `#[repr(C, align(64))]` to SoA structures
- Use `aligned-vec` crate for Vec<u64> with guaranteed alignment
- Add compile-time assertion: `static_assert!(align_of::<SoA>() == 64)`

---

### 3.2 String Buffer Reuse Pattern

**SimdJSON Approach:**
```cpp
class Parser {
    std::vector<char> string_buffer;  // Reused across parses

    std::string_view parse_string(const char* json) {
        size_t offset = string_buffer.size();
        // Append unescaped string to buffer
        unescape_into(json, string_buffer);
        size_t len = string_buffer.size() - offset;
        return std::string_view(string_buffer.data() + offset, len);
    }
};

// Benefit: All strings in single contiguous buffer
// - Better cache locality (strings close together)
// - Single allocation for all strings in document
// - string_view: Zero-copy, no per-string malloc
```

**KNHK Application:**
```rust
// knhk-etl could adopt this pattern for RawTriple strings

// Current (inefficient):
pub struct RawTriple {
    pub subject: String,    // Heap allocation 1
    pub predicate: String,  // Heap allocation 2
    pub object: String,     // Heap allocation 3
    pub graph: Option<String>,
}

// Optimized (SimdJSON style):
pub struct TripleArena {
    buffer: Vec<u8>,  // Single buffer for ALL strings
}

pub struct RawTriple<'a> {
    pub subject: &'a str,    // Points into arena
    pub predicate: &'a str,  // Points into arena
    pub object: &'a str,     // Points into arena
}

impl TripleArena {
    pub fn push_triple(&mut self, s: &str, p: &str, o: &str) -> RawTriple<'_> {
        let s_offset = self.buffer.len();
        self.buffer.extend_from_slice(s.as_bytes());
        let p_offset = self.buffer.len();
        self.buffer.extend_from_slice(p.as_bytes());
        let o_offset = self.buffer.len();
        self.buffer.extend_from_slice(o.as_bytes());

        unsafe {
            RawTriple {
                subject: std::str::from_utf8_unchecked(&self.buffer[s_offset..p_offset]),
                predicate: std::str::from_utf8_unchecked(&self.buffer[p_offset..o_offset]),
                object: std::str::from_utf8_unchecked(&self.buffer[o_offset..]),
            }
        }
    }
}
```

**Lesson Applied:**
KNHK's `RawTriple` could benefit from arena allocation pattern.

**Optimization Opportunity:**
- Add `TripleArena` to knhk-etl for batch ingestion
- Benchmark arena vs individual `String` allocations
- Document lifecycle: Arena lives for pipeline duration

---

## 4. SIMD-Specific Lessons

### 4.1 SIMD Instruction Selection Strategy

**SimdJSON Approach:**
```cpp
// Compile WITHOUT architecture-specific flags
// ❌ DON'T: g++ -mavx2 simdjson.cpp  (breaks runtime dispatch)
// ✅ DO:    g++ simdjson.cpp          (generic compilation)

// Use target attributes for specific functions
#if defined(__GNUC__) && !defined(_MSC_VER)
  #define TARGET_HASWELL __attribute__((target("avx2,bmi,pclmul,lzcnt")))
#else
  #define TARGET_HASWELL
#endif

TARGET_HASWELL
uint64_t parse_number_avx2(const char* json) {
    // Can use AVX2 intrinsics here
    __m256i data = _mm256_loadu_si256((__m256i*)json);
    // ...
}
```

**KNHK Application:**
```rust
// Rust equivalent using target_feature

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn kernel_ask_sp_avx2(
    s: u64, p: u64,
    s_array: &[u64],
    p_array: &[u64]
) -> bool {
    use std::arch::x86_64::*;

    // AVX2: Process 4 u64s at once
    let s_vec = _mm256_set1_epi64x(s as i64);
    let p_vec = _mm256_set1_epi64x(p as i64);

    for chunk in s_array.chunks(4) {
        let s_data = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
        let p_data = _mm256_loadu_si256(p_array.as_ptr() as *const __m256i);

        let s_cmp = _mm256_cmpeq_epi64(s_vec, s_data);
        let p_cmp = _mm256_cmpeq_epi64(p_vec, p_data);
        let match_mask = _mm256_and_si256(s_cmp, p_cmp);

        if _mm256_testz_si256(match_mask, match_mask) == 0 {
            return true;  // Found match
        }
    }
    false
}
```

**Lesson Applied:**
KNHK should add architecture-specific kernel implementations.

**Optimization Opportunity:**
- Create `knhk-hot/src/kernels/` directory
  - `kernels/avx512.rs` - AVX-512 implementations
  - `kernels/avx2.rs` - AVX2 implementations
  - `kernels/neon.rs` - ARM NEON implementations
  - `kernels/portable.rs` - Generic fallback
- Use `#[target_feature]` for each kernel variant
- Benchmark speedup: Expect 2-4x for SIMD vs scalar

---

### 4.2 Avoiding AVX-512 Downclocking

**SimdJSON Approach:**
```cpp
// On older Intel CPUs (pre-Ice Lake), AVX-512 causes downclocking
// SimdJSON strategy:
// 1. Detect CPU model at runtime
// 2. Use AVX-512 only on Ice Lake/Zen 4 or newer
// 3. Fall back to AVX2 on older CPUs

// Ice Lake detection:
bool is_icelake() {
    // Ice Lake: AVX-512 VBMI2 support (no downclocking)
    return __builtin_cpu_supports("avx512vbmi2");
}

if (is_icelake()) {
    use_avx512_kernel();  // Fast, no downclocking
} else {
    use_avx2_kernel();    // Avoid downclocking penalty
}
```

**KNHK Application:**
```rust
// Detect Ice Lake/Zen 4 for safe AVX-512 usage

#[cfg(target_arch = "x86_64")]
fn detect_avx512_safe() -> bool {
    // Only use AVX-512 on CPUs without downclocking penalty
    if is_x86_feature_detected!("avx512vbmi2") {
        true  // Ice Lake, Tiger Lake, Sapphire Rapids, Zen 4
    } else {
        false  // Skylake-X, Cascade Lake (downclocking risk)
    }
}

pub fn select_kernel() -> KernelImpl {
    #[cfg(target_arch = "x86_64")]
    {
        if detect_avx512_safe() {
            KernelImpl::Avx512  // ✅ Safe, no downclocking
        } else if is_x86_feature_detected!("avx2") {
            KernelImpl::Avx2    // ✅ Best for older Intel
        } else {
            KernelImpl::Sse42
        }
    }
    #[cfg(target_arch = "aarch64")]
    KernelImpl::Neon
}
```

**Lesson Applied:**
KNHK should implement CPU model detection to avoid AVX-512 downclocking penalty.

**Optimization Opportunity:**
- Add CPU model detection via `cpuid` crate
- Document AVX-512 usage policy in README
- Provide `KNHK_FORCE_AVX512` env var for advanced users

---

## 5. Error Handling and Safety

### 5.1 Error Chaining Pattern

**SimdJSON Approach:**
```cpp
// Error chaining: Delay error checks until final use
simdjson_result<uint64_t> value =
    doc.get_object()              // Could fail (not an object)
       ["statuses"]               // Could fail (key not found)
       .get_array()               // Could fail (not an array)
       .at(0)                     // Could fail (empty array)
       .get_object()              // Could fail (not an object)
       ["id"]                     // Could fail (key not found)
       .get_uint64();             // Could fail (not uint64)

// Error checked only at final cast:
uint64_t id = value;  // Throws exception if any step failed

// Or explicit check:
if (value.error()) { handle_error(); }
uint64_t id = value.value();
```

**KNHK Application:**
```rust
// Rust's Result<T, E> naturally supports error chaining via `?`

pub fn reconcile_pipeline(
    delta: &[RawTriple],
    soa: &SoAArrays,
) -> Result<Vec<Action>, ReconcileError> {
    let predicate = soa.p[0];

    // Error chaining with `?` operator
    let kernel_type = self.hook_registry
        .get_kernel(predicate)?;           // Propagate error

    let (cycles, mask) = KernelExecutor::execute_dispatch(
        kernel_type,
        &soa.s, &soa.p, &soa.o,
        delta.len()
    )?;                                    // Propagate error

    let ticks = cycles / self.cycles_per_tick;
    if ticks > self.tick_budget {
        return Err(ReconcileError::BudgetExceeded {
            actual: ticks,
            limit: self.tick_budget
        });
    }

    Ok(generate_actions(delta, mask))
}
```

**Lesson Applied:**
KNHK already uses Rust's `Result<T, E>` which provides excellent error chaining via `?`.

**Optimization Opportunity:**
- Document error chaining pattern in knhk-etl README
- Add integration tests for error propagation paths
- Consider `thiserror` crate for better error messages

---

## 6. Testing and Validation

### 6.1 Fuzzing for Edge Cases

**SimdJSON Approach:**
```cpp
// fuzz/ directory: Extensive fuzzing infrastructure
// Tests millions of edge cases automatically

// Example: UTF-8 validation fuzzer
extern "C" int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size) {
    simdjson::ondemand::parser parser;
    auto doc = parser.iterate(data, size, size + simdjson::SIMDJSON_PADDING);

    // Should either parse successfully or return error
    // Should NEVER crash or corrupt memory
    if (doc.error() == simdjson::SUCCESS) {
        // Validate parsed document
        auto root = doc.get_object();
        // ...
    }
    return 0;
}
```

**KNHK Application:**
```rust
// Add fuzzing to knhk-hot for edge case discovery

// fuzz/fuzz_targets/fuzz_kernel_execution.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use knhk_hot::{Engine, Ir, Op, Receipt};

fuzz_target!(|data: &[u8]| {
    if data.len() < 24 { return; }  // Need at least 3 u64s

    // Parse fuzz input into SoA arrays
    let s_array: [u64; 8] = [0; 8];
    let p_array: [u64; 8] = [0; 8];
    let o_array: [u64; 8] = [0; 8];

    // Copy fuzz data (safely)
    for i in 0..8.min(data.len() / 8) {
        let val = u64::from_le_bytes(data[i*8..(i+1)*8].try_into().unwrap());
        s_array[i] = val;
        p_array[i] = val;
        o_array[i] = val;
    }

    let mut engine = Engine::new(
        s_array.as_ptr(),
        p_array.as_ptr(),
        o_array.as_ptr(),
    );

    let mut ir = Ir {
        op: Op::AskSp,
        s: s_array[0],
        p: p_array[0],
        // ...
    };

    let mut receipt = Receipt::default();

    // Should never crash, even with random inputs
    let _ = engine.eval_bool(&mut ir, &mut receipt);

    // Validate tick budget ALWAYS honored
    assert!(receipt.actual_ticks <= 8, "Tick budget violated!");
});
```

**Lesson Applied:**
KNHK should add fuzzing to discover edge cases in kernel execution.

**Optimization Opportunity:**
- Add `cargo fuzz` targets for all kernel operations
- Run continuous fuzzing in CI (24-48 hours per release)
- Document discovered edge cases in test suite

---

## 7. Documentation and API Design

### 7.1 Progressive Disclosure in Documentation

**SimdJSON Approach:**
```markdown
# Quick Start (5 minutes)
```cpp
#include "simdjson.h"
using namespace simdjson;

ondemand::parser parser;
auto json = "{ \"key\": \"value\" }"_padded;
auto doc = parser.iterate(json);
std::string_view value = doc["key"];
```

# Advanced: Parser Reuse (After basic usage)
[Details on memory management...]

# Expert: Runtime Dispatch (For performance tuning)
[Details on CPU selection...]
```

**KNHK Application:**
```markdown
# knhk-hot Quick Start (Current README structure is good!)

## Quick Start
```rust
let mut engine = Engine::new(s, p, o);
let result = engine.eval_bool(&mut ir, &mut receipt);
assert!(receipt.actual_ticks <= 8);
```

## Advanced: Engine Reuse (ADD THIS SECTION)
[Document reuse pattern, show benchmarks]

## Expert: SIMD Selection (ADD THIS SECTION)
[Document CPU detection, architecture selection]
```

**Lesson Applied:**
KNHK documentation should follow SimdJSON's progressive disclosure pattern.

**Optimization Opportunity:**
- Restructure knhk-hot/docs/README.md with three levels:
  - Quick Start (5 min to first working code)
  - Advanced (Performance optimization, best practices)
  - Expert (SIMD internals, tick budget tuning)
- Add runnable examples for each level
- Link to relevant code locations (e.g., `src/engine.rs:142`)

---

## 8. Concrete Implementation Roadmap

### Phase 1: Foundation (v1.1) - 2-3 weeks

**Priority 1: Runtime CPU Detection**
```rust
// File: knhk-hot/src/cpu_detect.rs
pub fn detect_optimal_impl() -> CpuImpl { ... }
```
- **Benefit**: 2-4x speedup on modern CPUs
- **Risk**: Low (fallback to portable impl)
- **Testing**: Unit tests + CI matrix (SSE4.2, AVX2, AVX-512, NEON)

**Priority 2: SoA Padding Validation**
```rust
// File: knhk-hot/src/engine.rs
const SIMD_PADDING: usize = 64;

impl Engine {
    pub fn new(s: *const u64, p: *const u64, o: *const u64, capacity: usize) -> Self {
        debug_assert!(capacity >= 8 + SIMD_PADDING / 8);
        // ...
    }
}
```
- **Benefit**: Prevent SIMD overread bugs
- **Risk**: Low (debug-only check)
- **Testing**: Negative tests (too small buffer should panic)

**Priority 3: Documentation Restructure**
- Quick Start section (5 min)
- Advanced section (reuse patterns)
- Expert section (SIMD selection)

---

### Phase 2: SIMD Kernels (v1.2) - 4-6 weeks

**Priority 1: AVX2 Kernels**
```rust
// File: knhk-hot/src/kernels/avx2.rs
#[target_feature(enable = "avx2")]
unsafe fn kernel_ask_sp_avx2(...) { ... }
```
- **Benefit**: 2-3x speedup on Haswell+ CPUs
- **Risk**: Medium (SIMD complexity)
- **Testing**: Property-based tests (results match portable impl)

**Priority 2: AVX-512 Kernels (Ice Lake+ only)**
```rust
// File: knhk-hot/src/kernels/avx512.rs
#[target_feature(enable = "avx512f,avx512vbmi2")]
unsafe fn kernel_ask_sp_avx512(...) { ... }
```
- **Benefit**: 3-4x speedup on Ice Lake/Zen 4
- **Risk**: High (downclocking on older CPUs)
- **Testing**: Benchmark on multiple CPU models

**Priority 3: ARM NEON Kernels**
```rust
// File: knhk-hot/src/kernels/neon.rs
#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn kernel_ask_sp_neon(...) { ... }
```
- **Benefit**: Optimal performance on ARM (M1/M2 Macs, AWS Graviton)
- **Risk**: Low (NEON is standard on ARM64)
- **Testing**: GitHub Actions ARM runners

---

### Phase 3: Advanced Optimizations (v1.3) - 6-8 weeks

**Priority 1: Arena Allocator for RawTriple**
```rust
// File: knhk-etl/src/triple_arena.rs
pub struct TripleArena { buffer: Vec<u8> }
```
- **Benefit**: Reduce allocation overhead by 50-70%
- **Risk**: Medium (lifecycle management)
- **Testing**: Benchmark vs current String-based approach

**Priority 2: Free Padding Optimization**
```rust
// File: knhk-hot/src/padding.rs
fn need_padding_allocation(buf: *const u64, len: usize) -> bool { ... }
```
- **Benefit**: Eliminate padding allocation in common case
- **Risk**: High (platform-specific, sanitizer flags)
- **Testing**: Valgrind clean, MSAN clean

**Priority 3: Fuzzing Infrastructure**
```bash
# Add to CI
cargo +nightly fuzz run fuzz_kernel_execution -- -max_total_time=3600
```
- **Benefit**: Discover edge cases automatically
- **Risk**: Low (testing only)
- **Testing**: Run 24h before each release

---

## 9. Benchmarking Strategy

### 9.1 SimdJSON Benchmark Methodology

**SimdJSON Approach:**
```cpp
// benchmark/parse.cpp
// 1. Warm-up: Parse file 10 times to warm cache
// 2. Measurement: Parse file 100 times, record min/median/max
// 3. Report: GB/s throughput (file_size * iterations / time)

// Flags:
// -H: Omit allocation time (amortized measurement)
// -d: Use specific implementation (for comparison)

// Example:
$ ./parse twitter.json           # 3.2 GB/s (with allocation)
$ ./parse -H twitter.json        # 4.1 GB/s (amortized)
$ ./parse -d haswell twitter.json   # 3.8 GB/s (AVX2)
$ ./parse -d icelake twitter.json   # 4.3 GB/s (AVX-512)
```

**KNHK Application:**
```rust
// benchmark/hot_path_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_kernel_ask_sp(c: &mut Criterion) {
    let mut group = c.benchmark_group("kernel_ask_sp");

    // Test with different SIMD implementations
    for impl_name in &["portable", "avx2", "avx512", "neon"] {
        group.bench_with_input(
            BenchmarkId::new("impl", impl_name),
            impl_name,
            |b, &impl_name| {
                let engine = Engine::new_with_impl(impl_name, s, p, o);
                b.iter(|| engine.eval_bool(&mut ir, &mut receipt));
            }
        );
    }

    group.finish();
}

criterion_group!(benches, bench_kernel_ask_sp);
criterion_main!(benches);
```

**Lesson Applied:**
KNHK should add comprehensive benchmarks comparing SIMD implementations.

**Optimization Opportunity:**
- Add `make benchmark-simd` target
- Report ticks AND GB/s throughput
- Include CPU model in benchmark output
- Track performance across releases (regression detection)

---

## 10. Summary: Top 10 Actionable Lessons

| # | Lesson | KNHK Status | Priority | Estimated Effort |
|---|--------|-------------|----------|------------------|
| 1 | **Runtime CPU detection** | ❌ Missing | 🔴 P0 | 1 week |
| 2 | **SIMD kernel variants** (AVX2, AVX-512, NEON) | ❌ Missing | 🔴 P0 | 4 weeks |
| 3 | **64-byte alignment enforcement** | ⚠️ Partial | 🟡 P1 | 3 days |
| 4 | **Padding validation** | ❌ Missing | 🟡 P1 | 1 week |
| 5 | **Engine reuse documentation** | ⚠️ Partial | 🟡 P1 | 2 days |
| 6 | **Arena allocator for RawTriple** | ❌ Missing | 🟢 P2 | 2 weeks |
| 7 | **Free padding optimization** | ❌ Missing | 🟢 P2 | 1 week |
| 8 | **Fuzzing infrastructure** | ❌ Missing | 🟢 P2 | 1 week |
| 9 | **Progressive disclosure docs** | ⚠️ Partial | 🟢 P2 | 1 week |
| 10 | **SIMD benchmark suite** | ❌ Missing | 🟢 P2 | 1 week |

**Total Estimated Effort: 12-14 weeks (3-4 months)**

---

## 11. References

### SimdJSON Resources
- **Paper**: [Parsing Gigabytes of JSON per Second](https://arxiv.org/abs/1902.08318) (VLDB 2019)
- **On-Demand Paper**: [On-Demand JSON: A Better Way to Parse Documents?](http://arxiv.org/abs/2312.17149) (SPE 2024)
- **UTF-8 Paper**: [Validating UTF-8 In Less Than One Instruction Per Byte](https://arxiv.org/abs/2010.03090) (SPE 2021)
- **Repository**: https://github.com/simdjson/simdjson
- **Documentation**: https://simdjson.github.io/simdjson/

### KNHK Resources
- **knhk-hot README**: `/Users/sac/knhk/rust/knhk-hot/docs/README.md`
- **Content Addressing**: `/Users/sac/knhk/rust/docs/content_addressing.md`
- **ByteFlow Patterns**: `/Users/sac/knhk/rust/docs/byteflow_hot_warm_cold_patterns.md`

### Performance Analysis Tools
- **Godbolt Compiler Explorer**: https://godbolt.org (verify SIMD code generation)
- **Intel VTune**: Profile SIMD performance and downclocking
- **Linux `perf`**: Measure branch mispredictions, cache misses
- **`cargo asm`**: Inspect Rust codegen for SIMD instructions

---

**Document Version:** 1.0.0
**Last Updated:** 2025-11-07
**Maintainer:** KNHK Team
**Evidence Type:** Production Lessons from SimdJSON v3.10.1

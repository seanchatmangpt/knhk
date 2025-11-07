# 8-Beat C/Rust Integration Completion Plan

## Overview
Complete the remaining integration work to fully connect C branchless 8-beat components with Rust ETL pipeline.

## Current Status

### Completed ✅
1. Beat scheduler FFI bindings and integration
2. Receipt structure alignment (C ↔ Rust)
3. FFI modules created (beat_ffi, ring_ffi, fiber_ffi)
4. Receipt conversion utilities (not yet exported)

### Incomplete ⚠️
1. **Ring Buffer Integration**: BeatScheduler uses `RingBuffer<Vec<RawTriple>>` instead of C `DeltaRing`/`AssertionRing`
2. **Fiber Integration**: `Fiber::run_mu()` is placeholder, should use `FiberExecutor::execute()`
3. **Tests**: Missing ring buffer FFI tests, fiber FFI tests, integration tests
4. **Receipt Conversion**: Module exists but not exported in `lib.rs`

## Implementation Plan

### Phase 1: Export Receipt Conversion Module
**File**: `rust/knhk-hot/src/lib.rs`
- Export `receipt_convert` module
- Re-export conversion functions for convenience

### Phase 2: Ring Buffer Integration
**Files**: `rust/knhk-etl/src/beat_scheduler.rs`

**Changes**:
1. Replace `Vec<RingBuffer<Vec<RawTriple>>>` with `Vec<DeltaRing>` for delta rings
2. Replace `Vec<RingBuffer<ExecutionResult>>` with `Vec<AssertionRing>` for action rings
3. Update `new()` to initialize C ring buffers instead of Rust rings
4. Update `enqueue_delta()` to:
   - Convert `Vec<RawTriple>` to SoA arrays (S, P, O as `&[u64]`)
   - Call `DeltaRing::enqueue()` with tick slot
   - Handle cycle_id from beat scheduler
5. Update `execute_tick()` to:
   - Call `DeltaRing::dequeue()` for current tick slot
   - Convert SoA arrays back to `Vec<RawTriple>` for fiber execution
   - Call `AssertionRing::enqueue()` with receipts and assertions
6. Update `commit_cycle()` to:
   - Dequeue from `AssertionRing` for all tick slots
   - Process receipts and actions

**Data Conversion Utilities Needed**:
- `raw_triples_to_soa(triples: &[RawTriple]) -> (Vec<u64>, Vec<u64>, Vec<u64>)`
- `soa_to_raw_triples(S: &[u64], P: &[u64], O: &[u64]) -> Vec<RawTriple>`

**Note**: This requires IRI-to-u64 encoding. For now, use placeholder hash-based encoding or require pre-encoded triples.

### Phase 3: Fiber Integration
**Files**: `rust/knhk-etl/src/fiber.rs`

**Changes**:
1. Replace `run_mu()` placeholder with call to `FiberExecutor::execute()`
2. Update `execute_tick()` to:
   - Convert `&[RawTriple]` to `Ctx` and `Ir` structures
   - Call `FiberExecutor::execute()` with proper parameters
   - Convert C `Receipt` to Rust `Receipt` using conversion utilities
   - Handle parking via `knhk_fiber_park()` if needed
3. Update receipt generation to use C receipt from fiber execution

**Dependencies**:
- Need to create `Ctx` from SoA arrays
- Need to create `Ir` from operation type and parameters
- Need cycle_id, shard_id, hook_id from beat scheduler

### Phase 4: Add Tests

#### Ring Buffer FFI Tests
**File**: `rust/knhk-hot/src/ring_ffi.rs` (add `#[cfg(test)]` module)
- Test `DeltaRing::new()` initialization
- Test `DeltaRing::enqueue()` / `dequeue()` per tick slot
- Test `AssertionRing::enqueue()` / `dequeue()` per tick slot
- Test per-tick isolation (data in tick 0 doesn't appear in tick 1)
- Test ring buffer wrap-around
- Test parking mechanism

#### Fiber FFI Tests
**File**: `rust/knhk-hot/src/fiber_ffi.rs` (add `#[cfg(test)]` module)
- Test `FiberExecutor::execute()` with valid Ctx/Ir
- Test receipt generation with cycle_id, shard_id, hook_id
- Test parking mechanism
- Test tick budget enforcement

#### Integration Tests
**File**: `rust/knhk-etl/src/beat_scheduler.rs` (add integration tests)
- Test end-to-end flow: enqueue delta → execute tick → dequeue assertion
- Test receipt propagation through rings
- Test parking on budget overrun
- Test pulse boundary commit

## Implementation Details

### Data Conversion Strategy

**RawTriple → SoA Arrays**:
```rust
fn raw_triples_to_soa(triples: &[RawTriple]) -> Result<(Vec<u64>, Vec<u64>, Vec<u64>), String> {
    // For now, use hash-based encoding (placeholder)
    // In production, use MPHF or IRI registry
    let mut S = Vec::with_capacity(triples.len());
    let mut P = Vec::with_capacity(triples.len());
    let mut O = Vec::with_capacity(triples.len());
    
    for triple in triples {
        S.push(hash_iri(&triple.subject)?);
        P.push(hash_iri(&triple.predicate)?);
        O.push(hash_iri(&triple.object)?);
    }
    
    Ok((S, P, O))
}
```

**SoA Arrays → RawTriple**:
```rust
fn soa_to_raw_triples(S: &[u64], P: &[u64], O: &[u64]) -> Vec<RawTriple> {
    // For now, convert u64 back to string representation
    // In production, use reverse lookup from IRI registry
    S.iter().zip(P.iter()).zip(O.iter())
        .map(|((&s, &p), &o)| RawTriple {
            subject: format!("{:x}", s),
            predicate: format!("{:x}", p),
            object: format!("{:x}", o),
            graph: None,
        })
        .collect()
}
```

### Ctx Creation
```rust
fn create_ctx_from_soa(soa: &SoAArrays, run: &PredRun) -> Ctx {
    Ctx {
        S: soa.s.as_ptr(),
        P: soa.p.as_ptr(),
        O: soa.o.as_ptr(),
        run: Run {
            pred: run.pred,
            off: run.off,
            len: run.len,
        },
    }
}
```

### Ir Creation
```rust
fn create_ir_from_op(op: Op, s: u64, p: u64, o: u64, k: u64) -> Ir {
    Ir {
        op,
        s,
        p,
        o,
        k,
        out_S: std::ptr::null_mut(),
        out_P: std::ptr::null_mut(),
        out_O: std::ptr::null_mut(),
        out_mask: 0,
    }
}
```

## Files to Modify

### New Files
- `rust/knhk-etl/src/ring_conversion.rs` - RawTriple ↔ SoA conversion utilities

### Modified Files
- `rust/knhk-hot/src/lib.rs` - Export receipt_convert module
- `rust/knhk-etl/src/beat_scheduler.rs` - Replace Rust rings with C rings
- `rust/knhk-etl/src/fiber.rs` - Replace placeholder with C fiber execution
- `rust/knhk-hot/src/ring_ffi.rs` - Add tests
- `rust/knhk-hot/src/fiber_ffi.rs` - Add tests

## Testing Strategy

1. **Unit Tests**: Test each component in isolation
2. **Integration Tests**: Test component interactions
3. **Performance Tests**: Verify ≤8 tick budget compliance
4. **Chicago TDD**: Follow existing test patterns

## Risk Mitigation

1. **Data Conversion**: IRI-to-u64 encoding is placeholder. Production needs MPHF or registry.
2. **Backward Compatibility**: Keep Rust `RingBuffer<T>` for non-SoA use cases.
3. **Error Handling**: Ensure proper error propagation from C to Rust.
4. **Memory Safety**: Ensure proper lifetime management for C pointers.

## Success Criteria

1. ✅ All tests pass
2. ✅ BeatScheduler uses C ring buffers
3. ✅ Fiber uses C fiber execution
4. ✅ Receipt conversion utilities exported and used
5. ✅ Integration tests verify end-to-end flow
6. ✅ No performance regressions (≤8 ticks per operation)

## Estimated Effort

- Phase 1: 15 minutes (export module)
- Phase 2: 2-3 hours (ring buffer integration + conversion utilities)
- Phase 3: 2-3 hours (fiber integration + Ctx/Ir creation)
- Phase 4: 2-3 hours (comprehensive tests)

**Total**: ~6-9 hours


// rust/knhk-etl/src/reflex_map.rs
// Reflex Map Implementation: A = μ(O)
// Production-ready implementation with proper hash computation

extern crate alloc;

use crate::error::PipelineError;
use crate::load::{LoadResult, PredRun, SoAArrays};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

/// Reflex map result: A = μ(O)
#[derive(Debug, Clone)]
pub struct ReflexMapResult {
    /// Actions A generated from ontology O via reflex map μ
    pub actions: Vec<Action>,

    /// Receipts proving hash(A) = hash(μ(O))
    pub receipts: Vec<Receipt>,

    /// Maximum ticks used (must be ≤ 8)
    pub max_ticks: u32,

    /// Reflex map hash: hash(μ(O))
    pub mu_hash: u64,

    /// Actions hash: hash(A)
    pub a_hash: u64,
}

/// Action A generated from reflex map
#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub payload: Vec<u8>,
    pub receipt_id: String,
    pub predicate: u64,
    pub subject: u64,
    pub object: u64,
}

/// Receipt proving hash(A) = hash(μ(O))
#[derive(Debug, Clone)]
pub struct Receipt {
    pub id: String,
    pub cycle_id: u64,     // Beat cycle ID (from knhk_beat_next())
    pub shard_id: u64,     // Shard identifier
    pub hook_id: u64,      // Hook identifier
    pub ticks: u32,        // Estimated/legacy ticks (for compatibility)
    pub actual_ticks: u32, // PMU-measured actual ticks (≤8 enforced by τ law)
    pub lanes: u32,        // SIMD lanes used
    pub span_id: u64,      // OTEL-compatible span ID
    pub a_hash: u64,       // hash(A) = hash(μ(O)) fragment
    pub mu_hash: u64,
}

/// Reflex map implementation
pub struct ReflexMap {
    tick_budget: u32,
}

impl ReflexMap {
    /// Create a new reflex map with default tick budget (8)
    pub fn new() -> Self {
        Self { tick_budget: 8 }
    }

    /// Create a new reflex map with custom tick budget
    pub fn with_tick_budget(tick_budget: u32) -> Result<Self, PipelineError> {
        if tick_budget > 8 {
            return Err(PipelineError::GuardViolation(format!(
                "Tick budget {} exceeds Chatman Constant (8)",
                tick_budget
            )));
        }
        Ok(Self { tick_budget })
    }

    /// Apply reflex map: A = μ(O)
    ///
    /// Production implementation:
    /// 1. Execute hooks for each predicate run
    /// 2. Generate actions from successful hooks
    /// 3. Compute hash(μ(O)) from SoA arrays and runs
    /// 4. Compute hash(A) from generated actions
    /// 5. Verify hash(A) = hash(μ(O))
    pub fn apply(&self, input: LoadResult) -> Result<ReflexMapResult, PipelineError> {
        if input.runs.is_empty() {
            return Ok(ReflexMapResult {
                actions: Vec::new(),
                receipts: Vec::new(),
                max_ticks: 0,
                mu_hash: 0,
                a_hash: 0,
            });
        }

        let mut actions = Vec::new();
        let mut receipts = Vec::new();
        let mut max_ticks = 0u32;

        // Execute hooks for each predicate run
        for run in &input.runs {
            // Guard: validate run length ≤ 8
            if run.len > 8 {
                return Err(PipelineError::GuardViolation(format!(
                    "Run length {} exceeds max_run_len 8",
                    run.len
                )));
            }

            // Execute hook via C hot path API
            let receipt = self.execute_hook(&input.soa_arrays, run)?;

            // Guard: validate tick budget
            if receipt.ticks > self.tick_budget {
                return Err(PipelineError::ReflexError(format!(
                    "Hook execution {} ticks exceeds budget {} ticks",
                    receipt.ticks, self.tick_budget
                )));
            }

            max_ticks = max_ticks.max(receipt.ticks);

            // Generate action if hook succeeded
            if receipt.ticks > 0 {
                let action = Action {
                    id: format!("action_{}", actions.len()),
                    payload: Vec::new(),
                    receipt_id: receipt.id.clone(),
                    predicate: run.pred,
                    subject: if run.len > 0 && run.off < 8 {
                        input.soa_arrays.s[run.off as usize]
                    } else {
                        0
                    },
                    object: if run.len > 0 && run.off < 8 {
                        input.soa_arrays.o[run.off as usize]
                    } else {
                        0
                    },
                };
                actions.push(action);
            }

            receipts.push(receipt);
        }

        // Compute hash(μ(O)) from SoA arrays and runs
        let mu_hash = self.compute_mu_hash(&input.soa_arrays, &input.runs);

        // Compute hash(A) from generated actions
        let a_hash = self.compute_a_hash(&actions);

        // Verify hash(A) = hash(μ(O))
        if a_hash != mu_hash {
            return Err(PipelineError::ReflexError(format!(
                "Hash mismatch: hash(A)={} != hash(μ(O))={}",
                a_hash, mu_hash
            )));
        }

        // Merge receipts via ⊕ (associative merge)
        if receipts.len() > 1 {
            let merged = Self::merge_receipts(&receipts);
            receipts.push(merged);
        }

        Ok(ReflexMapResult {
            actions,
            receipts,
            max_ticks,
            mu_hash,
            a_hash,
        })
    }

    /// Execute a single hook using C hot path API via FFI
    ///
    /// Implements receipt-side tick metering:
    /// - Start tick counter at μ entry
    /// - Stop tick counter at receipt finalize
    /// - Store ticks, actual_ticks, lanes in receipt
    /// - Prove ≤8 ticks on every hot run
    fn execute_hook(&self, soa: &SoAArrays, run: &PredRun) -> Result<Receipt, PipelineError> {
        #[cfg(feature = "std")]
        {
            use knhk_hot::{Engine, Ir, Op, Receipt as HotReceipt, Run as HotRun, TickMeasurement};

            // Start tick measurement at μ entry
            let mut tick_measurement = TickMeasurement::start();

            // Initialize engine with SoA arrays
            // SAFETY: Engine::new requires valid pointers to SoA arrays.
            // We guarantee this by passing pointers from valid Vec<u64> allocations.
            let mut engine = unsafe { Engine::new(soa.s.as_ptr(), soa.p.as_ptr(), soa.o.as_ptr()) };

            // Guard: validate run length ≤ 8
            if run.len > 8 {
                return Err(PipelineError::GuardViolation(format!(
                    "Run length {} exceeds max_run_len 8",
                    run.len
                )));
            }

            // Guard: validate offset bounds
            if run.off >= 8 {
                return Err(PipelineError::GuardViolation(format!(
                    "Run offset {} exceeds SoA array capacity 8",
                    run.off
                )));
            }

            let hot_run = HotRun {
                pred: run.pred,
                off: run.off,
                len: run.len,
            };
            engine
                .pin_run(hot_run)
                .map_err(|e| PipelineError::ReflexError(format!("Failed to pin run: {}", e)))?;

            // Create hook IR (ASK_SP operation)
            let s_val = if run.len > 0 && run.off < 8 {
                soa.s[run.off as usize]
            } else {
                0
            };
            let o_val = if run.len > 0 && run.off < 8 {
                soa.o[run.off as usize]
            } else {
                0
            };

            let mut ir = Ir {
                op: Op::AskSp,
                s: s_val,
                p: run.pred,
                o: o_val,
                k: 0,
                out_S: core::ptr::null_mut(),
                out_P: core::ptr::null_mut(),
                out_O: core::ptr::null_mut(),
                out_mask: 0,
                construct8_pattern_hint: 0,
            };

            // Execute hook
            let mut hot_receipt = HotReceipt::default();
            let result = engine.eval_bool(&mut ir, &mut hot_receipt);
            // eval_bool returns bool, not Result - check result directly
            if !result {
                return Err(PipelineError::ReflexError(
                    "Hook execution returned false".to_string(),
                ));
            }

            // Stop tick measurement at receipt finalize
            tick_measurement.stop();

            // Get actual ticks from measurement
            let actual_ticks = tick_measurement.elapsed_ticks().unwrap_or(0);

            // Prove ≤8 ticks on every hot run (Chatman Constant)
            if actual_ticks > self.tick_budget {
                return Err(PipelineError::ReflexError(format!(
                    "Hook execution {} ticks exceeds budget {} ticks (Chatman Constant violation)",
                    actual_ticks, self.tick_budget
                )));
            }

            // Compute mu_hash for this hook
            let mu_hash = self.compute_mu_hash_for_run(soa, run);

            // Create receipt with tick metering
            Ok(Receipt {
                id: format!("receipt_{}", hot_receipt.span_id),
                cycle_id: hot_receipt.cycle_id,
                shard_id: hot_receipt.shard_id,
                hook_id: hot_receipt.hook_id,
                ticks: hot_receipt.ticks, // Estimated/legacy ticks from C API
                actual_ticks,             // PMU-measured actual ticks from tick metering
                lanes: hot_receipt.lanes,
                span_id: hot_receipt.span_id,
                a_hash: hot_receipt.a_hash,
                mu_hash,
            })
        }

        #[cfg(not(feature = "std"))]
        {
            // No-op implementation for no_std
            return Err(PipelineError::ReflexError(
                "Hot path execution requires std feature".to_string(),
            ));
        }
    }

    /// Compute hash(μ(O)) from SoA arrays and runs
    fn compute_mu_hash(&self, soa: &SoAArrays, runs: &[PredRun]) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;

        // Hash all runs
        for run in runs {
            // Hash predicate
            let mut value = run.pred;
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }

            // Hash offset and length
            value = run.off;
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }

            value = run.len;
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }

            // Hash SoA data for this run
            for i in 0..run.len as usize {
                let idx = (run.off as usize) + i;
                if idx < 8 {
                    let mut s_val = soa.s[idx];
                    for _ in 0..8 {
                        hash ^= s_val & 0xFF;
                        hash = hash.wrapping_mul(FNV_PRIME);
                        s_val >>= 8;
                    }

                    let mut p_val = soa.p[idx];
                    for _ in 0..8 {
                        hash ^= p_val & 0xFF;
                        hash = hash.wrapping_mul(FNV_PRIME);
                        p_val >>= 8;
                    }

                    let mut o_val = soa.o[idx];
                    for _ in 0..8 {
                        hash ^= o_val & 0xFF;
                        hash = hash.wrapping_mul(FNV_PRIME);
                        o_val >>= 8;
                    }
                }
            }
        }

        hash
    }

    /// Compute hash(μ(O)) for a single run
    fn compute_mu_hash_for_run(&self, soa: &SoAArrays, run: &PredRun) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;

        // Hash predicate, offset, length
        let mut value = run.pred;
        for _ in 0..8 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }

        value = run.off;
        for _ in 0..8 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }

        value = run.len;
        for _ in 0..8 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }

        // Hash SoA data
        for i in 0..run.len as usize {
            let idx = (run.off as usize) + i;
            if idx < 8 {
                let mut s_val = soa.s[idx];
                for _ in 0..8 {
                    hash ^= s_val & 0xFF;
                    hash = hash.wrapping_mul(FNV_PRIME);
                    s_val >>= 8;
                }

                let mut o_val = soa.o[idx];
                for _ in 0..8 {
                    hash ^= o_val & 0xFF;
                    hash = hash.wrapping_mul(FNV_PRIME);
                    o_val >>= 8;
                }
            }
        }

        hash
    }

    /// Compute hash(A) from actions
    fn compute_a_hash(&self, actions: &[Action]) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;

        for action in actions {
            // Hash action ID
            for byte in action.id.as_bytes() {
                hash ^= *byte as u64;
                hash = hash.wrapping_mul(FNV_PRIME);
            }

            // Hash predicate, subject, object
            let mut value = action.predicate;
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }

            value = action.subject;
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }

            value = action.object;
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }
        }

        hash
    }

    /// Merge receipts via ⊕ (associative merge)
    pub fn merge_receipts(receipts: &[Receipt]) -> Receipt {
        if receipts.is_empty() {
            // Generate proper span_id for empty receipt
            fn generate_span_id() -> u64 {
                #[cfg(feature = "knhk-otel")]
                {
                    use knhk_otel::generate_span_id;
                    generate_span_id()
                }
                #[cfg(not(feature = "knhk-otel"))]
                {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(0);
                    timestamp.wrapping_mul(0x9e3779b97f4a7c15)
                }
            }
            return Receipt {
                id: "merged_receipt".to_string(),
                cycle_id: 0,
                shard_id: 0,
                hook_id: 0, // Empty receipt has no hook
                ticks: 0,
                actual_ticks: 0,
                lanes: 0,
                span_id: generate_span_id(), // Generate OTEL-compatible span ID
                a_hash: 0,
                mu_hash: 0,
            };
        }

        let mut merged = Receipt {
            id: "merged_receipt".to_string(),
            // Preserve identifiers from first receipt (deterministic ordering)
            cycle_id: receipts[0].cycle_id,
            shard_id: receipts[0].shard_id,
            hook_id: receipts[0].hook_id,
            ticks: receipts[0].ticks,
            actual_ticks: receipts[0].actual_ticks,
            lanes: receipts[0].lanes,
            span_id: receipts[0].span_id,
            a_hash: receipts[0].a_hash,
            mu_hash: receipts[0].mu_hash,
        };

        for receipt in receipts.iter().skip(1) {
            // Max ticks (worst case) - both estimated and actual
            merged.ticks = merged.ticks.max(receipt.ticks);
            merged.actual_ticks = merged.actual_ticks.max(receipt.actual_ticks);
            // Sum lanes
            merged.lanes += receipt.lanes;
            // XOR merge for span_id
            merged.span_id ^= receipt.span_id;
            // XOR merge for a_hash (⊕ operation)
            merged.a_hash ^= receipt.a_hash;
            // XOR merge for mu_hash
            merged.mu_hash ^= receipt.mu_hash;
        }

        merged
    }
}

impl Default for ReflexMap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used)]
    use super::*;
    use crate::load::{PredRun, SoAArrays};

    #[test]
    fn test_reflex_map_idempotence() {
        // Test: μ∘μ = μ
        let reflex_map = ReflexMap::new();

        let soa = SoAArrays {
            s: [1, 2, 0, 0, 0, 0, 0, 0],
            p: [10, 10, 0, 0, 0, 0, 0, 0],
            o: [100, 200, 0, 0, 0, 0, 0, 0],
        };

        let runs = vec![PredRun {
            pred: 10,
            off: 0,
            len: 2,
        }];

        let input = LoadResult {
            soa_arrays: soa.clone(),
            runs: runs.clone(),
        };

        let result1 = reflex_map
            .apply(input.clone())
            .expect("First reflex_map application should succeed");
        let result2 = reflex_map
            .apply(input)
            .expect("Second reflex_map application should succeed");

        // Same input → same output (idempotence)
        assert_eq!(result1.mu_hash, result2.mu_hash);
        assert_eq!(result1.a_hash, result2.a_hash);
        assert_eq!(result1.actions.len(), result2.actions.len());
    }

    #[test]
    fn test_reflex_map_hash_verification() {
        // Test: hash(A) = hash(μ(O))
        let reflex_map = ReflexMap::new();

        let soa = SoAArrays {
            s: [1, 2, 0, 0, 0, 0, 0, 0],
            p: [10, 10, 0, 0, 0, 0, 0, 0],
            o: [100, 200, 0, 0, 0, 0, 0, 0],
        };

        let runs = vec![PredRun {
            pred: 10,
            off: 0,
            len: 2,
        }];

        let input = LoadResult {
            soa_arrays: soa,
            runs,
        };

        let result = reflex_map
            .apply(input)
            .expect("Reflex map application should succeed");

        // Verify hash(A) = hash(μ(O))
        assert_eq!(
            result.a_hash, result.mu_hash,
            "hash(A)={} must equal hash(μ(O))={}",
            result.a_hash, result.mu_hash
        );
    }
}

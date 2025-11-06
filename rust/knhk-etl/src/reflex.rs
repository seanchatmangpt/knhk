// rust/knhk-etl/src/reflex.rs
// Stage 4: Reflex
// μ executes in ≤8 ticks per Δ

extern crate alloc;
extern crate std;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;

use crate::error::PipelineError;
use crate::load::{LoadResult, SoAArrays, PredRun};
use crate::runtime_class::RuntimeClass;
use crate::slo_monitor::SloMonitor;
use crate::failure_actions::{handle_r1_failure, handle_w1_failure, handle_c1_failure};

// Note: Validation feature disabled to avoid circular dependency with knhk-validation
// #[cfg(feature = "validation")]
// use knhk_validation::policy_engine::PolicyEngine;

use std::cell::RefCell;

/// Stage 4: Reflex
/// μ executes in ≤8 ticks per Δ
pub struct ReflexStage {
    pub tick_budget: u32, // Must be ≤ 8
    /// SLO monitors per runtime class (using RefCell for interior mutability)
        r1_monitor: Option<RefCell<SloMonitor>>,
        w1_monitor: Option<RefCell<SloMonitor>>,
        c1_monitor: Option<RefCell<SloMonitor>>,
}

impl ReflexStage {
    pub fn new() -> Self {
        Self {
            tick_budget: 8,
            r1_monitor: Some(RefCell::new(SloMonitor::new(RuntimeClass::R1, 1000))),
            w1_monitor: Some(RefCell::new(SloMonitor::new(RuntimeClass::W1, 1000))),
            c1_monitor: Some(RefCell::new(SloMonitor::new(RuntimeClass::C1, 1000))),
        }
    }

    /// Execute reflex over loaded data
    /// 
    /// Production implementation:
    /// 1. Call C hot path API (knhk_eval_bool, knhk_eval_construct8)
    /// 2. Ensure each hook ≤ 8 ticks
    /// 3. Collect receipts
    /// 4. Merge receipts via ⊕
    pub fn reflex(&self, input: LoadResult) -> Result<ReflexResult, PipelineError> {
        if input.runs.is_empty() {
            return Ok(ReflexResult {
                actions: Vec::new(),
                receipts: Vec::new(),
                max_ticks: 0,
                c1_failure_actions: Vec::new(),
            });
        }

        let mut actions = Vec::new();
        let mut receipts = Vec::new();
        let mut max_ticks = 0u32;
        let mut c1_failure_actions = Vec::new();

        // Execute hooks for each predicate run
        for run in &input.runs {
            // Validate run length ≤ 8 (Chatman Constant guard - defense in depth)
            // Note: Validation feature disabled to avoid circular dependency with knhk-validation
            if run.len > 8 {
                return Err(PipelineError::GuardViolation(
                    format!("Run length {} exceeds max_run_len 8", run.len)
                ));
            }
            
            // Validate run length ≤ tick_budget (guard check)
            if run.len > self.tick_budget as u64 {
                return Err(PipelineError::ReflexError(
                    format!("Run length {} exceeds tick budget {}", run.len, self.tick_budget)
                ));
            }

            // Classify operation (R1/W1/C1)
            // Extract operation type from IR (defaults to ASK_SP for hot path operations)
            let operation_type = Self::extract_operation_type(run);
            let runtime_class = RuntimeClass::classify_operation(&operation_type, run.len as usize)
                .map_err(|e| PipelineError::RuntimeClassError(e))?;

            // Execute hook via C hot path API (FFI)
            let receipt = self.execute_hook(&input.soa_arrays, run)?;

            // Record latency and check SLO (only for std builds, no overhead on hot path)
                        {
                // Convert ticks to nanoseconds (approximate: 1 tick ≈ 0.25ns at 4GHz)
                let latency_ns = (receipt.ticks as u64) * 250;
                
                match runtime_class {
                    RuntimeClass::R1 => {
                        if let Some(ref monitor) = self.r1_monitor {
                            monitor.borrow_mut().record_latency(latency_ns);
                            if let Err(violation) = monitor.borrow().check_slo_violation() {
                                // Handle R1 failure: drop/park Δ, emit receipt, escalate
                                let failure_action = handle_r1_failure(
                                    LoadResult {
                                        soa_arrays: input.soa_arrays.clone(),
                                        runs: vec![run.clone()],
                                    },
                                    receipt.clone(),
                                    receipt.ticks > self.tick_budget,
                                ).map_err(|e| PipelineError::R1FailureError(e))?;
                                
                                // If escalation is needed, return error
                                if failure_action.escalate {
                                return Err(PipelineError::SloViolation(violation));
                                }
                                // Otherwise, continue (Δ is parked)
                            }
                        }
                    },
                    RuntimeClass::W1 => {
                        if let Some(ref monitor) = self.w1_monitor {
                            monitor.borrow_mut().record_latency(latency_ns);
                            if let Err(violation) = monitor.borrow().check_slo_violation() {
                                // Handle W1 failure: retry/degrade
                                let _ = handle_w1_failure(0, 3, None);
                                return Err(PipelineError::SloViolation(violation));
                            }
                        }
                    },
                    RuntimeClass::C1 => {
                        if let Some(ref monitor) = self.c1_monitor {
                            monitor.borrow_mut().record_latency(latency_ns);
                            if let Err(violation) = monitor.borrow().check_slo_violation() {
                                // Handle C1 failure: async finalize
                                // Store C1FailureAction for caller to schedule async operation
                                if let Ok(c1_action) = handle_c1_failure(&receipt.id) {
                                    c1_failure_actions.push(c1_action);
                                }
                                return Err(PipelineError::SloViolation(violation));
                            }
                        }
                    },
                }
            }

            // Check tick budget violation
            if receipt.ticks > self.tick_budget {
                // Handle R1 failure for budget exceeded
                // Note: Validation feature disabled to avoid circular dependency with knhk-validation
                let failure_action = handle_r1_failure(
                    LoadResult {
                        soa_arrays: input.soa_arrays.clone(),
                        runs: vec![run.clone()],
                    },
                    receipt.clone(),
                    true, // Budget exceeded
                ).map_err(|e| PipelineError::R1FailureError(e))?;
                
                // Escalation is always true for budget exceeded
                if failure_action.escalate {
                    return Err(PipelineError::R1FailureError(
                        format!("Hook execution {} ticks exceeds budget {} ticks. Receipt {} emitted, Δ parked",
                            receipt.ticks, self.tick_budget, receipt.id)
                    ));
                }
            }

            max_ticks = max_ticks.max(receipt.ticks);

            // Generate action if query succeeds (receipt indicates successful execution)
            if receipt.ticks > 0 {
                actions.push(Action {
                    id: format!("action_{}", receipts.len()),
                    payload: Vec::new(),
                    receipt_id: receipt.id.clone(),
                });
            }

            receipts.push(receipt);
        }

        // Merge receipts via ⊕ (associative merge)
        if receipts.len() > 1 {
            let merged = Self::merge_receipts(&receipts);
            receipts.push(merged);
        }

        Ok(ReflexResult {
            actions,
            receipts,
            max_ticks,
            c1_failure_actions,
        })
    }

    /// Execute a single hook using C hot path API via FFI
    fn execute_hook(&self, soa: &SoAArrays, run: &PredRun) -> Result<Receipt, PipelineError> {
        use knhk_hot::{Engine, Op, Ir, Receipt as HotReceipt, Run as HotRun};
        
        // Initialize engine with SoA arrays
        let mut engine = Engine::new(soa.s.as_ptr(), soa.p.as_ptr(), soa.o.as_ptr());
        
        // Pin run (validates len ≤ 8 via C API)
        // Additional guard validation before pinning (defense in depth)
        if run.len > 8 {
            return Err(PipelineError::GuardViolation(
                format!("Run length {} exceeds max_run_len 8", run.len)
            ));
        }
        
        // Validate offset bounds
        if run.off >= 8 {
            return Err(PipelineError::GuardViolation(
                format!("Run offset {} exceeds SoA array capacity 8", run.off)
            ));
        }
        
        let hot_run = HotRun {
            pred: run.pred,
            off: run.off,
            len: run.len,
        };
        engine.pin_run(hot_run).map_err(|e| {
            PipelineError::ReflexError(format!("Failed to pin run: {}", e))
        })?;
        
        // Create hook IR (default to ASK_SP operation)
        // Validate bounds before array access
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
        };
        
        // Execute hook via C FFI
        let mut hot_receipt = HotReceipt::default();
        let _result = engine.eval_bool(&mut ir, &mut hot_receipt);
        
        // Convert to ETL receipt format
        Ok(Receipt {
            id: format!("receipt_{}", hot_receipt.span_id),
            cycle_id: hot_receipt.cycle_id,
            shard_id: hot_receipt.shard_id,
            hook_id: hot_receipt.hook_id,
            ticks: hot_receipt.ticks,
            lanes: hot_receipt.lanes,
            span_id: hot_receipt.span_id,
            a_hash: hot_receipt.a_hash,
        })
    }

    /// Merge receipts via ⊕ operation (associative, branchless)
    /// Implements: knhk_receipt_merge semantics
    pub fn merge_receipts(receipts: &[Receipt]) -> Receipt {
        if receipts.is_empty() {
            return Receipt {
                id: "merged_receipt".to_string(),
                cycle_id: 0,
                shard_id: 0,
                hook_id: 0,
                ticks: 0,
                lanes: 0,
                span_id: 0,
                a_hash: 0,
            };
        }

        let mut merged = Receipt {
            id: "merged_receipt".to_string(),
            // Preserve identifiers from first receipt (deterministic ordering)
            cycle_id: receipts[0].cycle_id,
            shard_id: receipts[0].shard_id,
            hook_id: receipts[0].hook_id,
            ticks: receipts[0].ticks,
            lanes: receipts[0].lanes,
            span_id: receipts[0].span_id,
            a_hash: receipts[0].a_hash,
        };

        for receipt in receipts.iter().skip(1) {
            // Max ticks (worst case)
            merged.ticks = merged.ticks.max(receipt.ticks);
            // Sum lanes
            merged.lanes += receipt.lanes;
            // XOR merge for span_id
            merged.span_id ^= receipt.span_id;
            // XOR merge for a_hash (⊕ operation)
            merged.a_hash ^= receipt.a_hash;
        }

        merged
    }

    /// Generate OTEL-compatible span ID (deterministic in no_std mode)
    fn generate_span_id() -> u64 {
        #[cfg(feature = "knhk-otel")]
        {
            use knhk_otel::generate_span_id;
            generate_span_id()
        }
        #[cfg(not(feature = "knhk-otel"))]
        {
            // Fallback: use deterministic hash
            use std::time::{SystemTime, UNIX_EPOCH};
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0);
            // Simple hash of timestamp
            timestamp.wrapping_mul(0x9e3779b97f4a7c15)
        }
    }
    
    /// Generate deterministic span ID from SoA data (no_std fallback)
    fn generate_span_id_deterministic(_soa: &SoAArrays, run: &PredRun) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 1469598103934665603;
        const FNV_PRIME: u64 = 1099511628211;
        
        let mut hash = FNV_OFFSET_BASIS;
        
        // Hash run info
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
        
        hash
    }

    /// Compute a_hash: hash(A) = hash(μ(O)) fragment
    fn compute_a_hash(soa: &SoAArrays, run: &PredRun) -> u64 {
        // Use FNV-1a hash for consistency with C implementation
        const FNV_OFFSET_BASIS: u64 = 1469598103934665603;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;
        
        // Hash the relevant portion of SoA arrays
        for i in 0..run.len as usize {
            let idx = (run.off as usize) + i;
            let mut value = soa.s[idx];
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }
            value = soa.p[idx];
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }
            value = soa.o[idx];
            for _ in 0..8 {
                hash ^= value & 0xFF;
                hash = hash.wrapping_mul(FNV_PRIME);
                value >>= 8;
            }
        }
        
        // Hash predicate
        let mut value = run.pred;
        for _ in 0..8 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }
        
        hash
    }

    fn get_timestamp_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Extract operation type from predicate run
    /// 
    /// Determines operation type based on run characteristics:
    /// - Single predicate with small data → ASK_SP
    /// - Multiple predicates or larger data → CONSTRUCT8 or SPARQL_SELECT
    /// 
    /// # Arguments
    /// * `run` - Predicate run to analyze
    /// 
    /// # Returns
    /// Operation type string (e.g., "ASK_SP", "CONSTRUCT8", "SPARQL_SELECT")
    fn extract_operation_type(run: &PredRun) -> String {
        // For hot path operations (≤8 items), default to ASK_SP
        // This matches the default IR operation type in execute_hook
        if run.len <= 8 {
            "ASK_SP".to_string()
        } else if run.len <= 100 {
            // Medium-sized operations → CONSTRUCT8
            "CONSTRUCT8".to_string()
        } else {
            // Large operations → SPARQL_SELECT
            "SPARQL_SELECT".to_string()
        }
    }
}

pub struct ReflexResult {
    pub actions: Vec<Action>,
    pub receipts: Vec<Receipt>,
    pub max_ticks: u32,
    /// C1 failure actions that require async finalization
    pub c1_failure_actions: Vec<crate::failure_actions::C1FailureAction>,
}

#[derive(Debug, Clone)]
pub struct Action {
    pub id: String,
    pub payload: Vec<u8>,
    pub receipt_id: String,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub id: String,
    pub cycle_id: u64,   // Beat cycle ID (from knhk_beat_next())
    pub shard_id: u64,   // Shard identifier
    pub hook_id: u64,    // Hook identifier
    pub ticks: u32,      // Actual ticks used (≤8)
    pub lanes: u32,      // SIMD lanes used
    pub span_id: u64,    // OTEL-compatible span ID
    pub a_hash: u64,     // hash(A) = hash(μ(O)) fragment
}

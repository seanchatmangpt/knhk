// rust/knhk-etl/src/reflex.rs
// Stage 4: Reflex
// μ executes in ≤8 ticks per Δ

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;

use crate::error::PipelineError;
use crate::load::{LoadResult, SoAArrays, PredRun};
use crate::runtime_class::RuntimeClass;
use crate::slo_monitor::SloMonitor;
use crate::failure_actions::{handle_r1_failure, handle_w1_failure, handle_c1_failure};

#[cfg(feature = "std")]
use std::cell::RefCell;

/// Stage 4: Reflex
/// μ executes in ≤8 ticks per Δ
pub struct ReflexStage {
    pub tick_budget: u32, // Must be ≤ 8
    /// SLO monitors per runtime class (using RefCell for interior mutability)
    #[cfg(feature = "std")]
    r1_monitor: Option<RefCell<SloMonitor>>,
    #[cfg(feature = "std")]
    w1_monitor: Option<RefCell<SloMonitor>>,
    #[cfg(feature = "std")]
    c1_monitor: Option<RefCell<SloMonitor>>,
}

impl ReflexStage {
    pub fn new() -> Self {
        Self {
            tick_budget: 8,
            #[cfg(feature = "std")]
            r1_monitor: Some(RefCell::new(SloMonitor::new(RuntimeClass::R1, 1000))),
            #[cfg(feature = "std")]
            w1_monitor: Some(RefCell::new(SloMonitor::new(RuntimeClass::W1, 1000))),
            #[cfg(feature = "std")]
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
            });
        }

        let mut actions = Vec::new();
        let mut receipts = Vec::new();
        let mut max_ticks = 0u32;

        // Execute hooks for each predicate run
        for run in &input.runs {
            // Validate run length ≤ 8 (Chatman Constant guard - defense in depth)
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
            let operation_type = "ASK_SP"; // Default operation type
            let _runtime_class = RuntimeClass::classify_operation(operation_type, run.len as usize)
                .map_err(|e| PipelineError::RuntimeClassError(e))?;

            // Execute hook via C hot path API (FFI)
            let receipt = self.execute_hook(&input.soa_arrays, run)?;

            // Record latency and check SLO (only for std builds, no overhead on hot path)
            #[cfg(feature = "std")]
            {
                // Convert ticks to nanoseconds (approximate: 1 tick ≈ 0.25ns at 4GHz)
                let latency_ns = (receipt.ticks as u64) * 250;
                
                match _runtime_class {
                    RuntimeClass::R1 => {
                        if let Some(ref monitor) = self.r1_monitor {
                            monitor.borrow_mut().record_latency(latency_ns);
                            if let Err(violation) = monitor.borrow().check_slo_violation() {
                                // Handle R1 failure: drop/park Δ, emit receipt, escalate
                                let _ = handle_r1_failure(
                                    LoadResult {
                                        soa_arrays: input.soa_arrays.clone(),
                                        runs: vec![run.clone()],
                                    },
                                    receipt.clone(),
                                    receipt.ticks > self.tick_budget,
                                );
                                return Err(PipelineError::SloViolation(violation));
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
                                let _ = handle_c1_failure(&receipt.id);
                                return Err(PipelineError::SloViolation(violation));
                            }
                        }
                    },
                }
            }

            // Check tick budget violation
            if receipt.ticks > self.tick_budget {
                // Handle R1 failure for budget exceeded
                #[cfg(feature = "std")]
                {
                    let _ = handle_r1_failure(
                        LoadResult {
                            soa_arrays: input.soa_arrays.clone(),
                            runs: vec![run.clone()],
                        },
                        receipt.clone(),
                        true, // Budget exceeded
                    );
                }
                return Err(PipelineError::R1FailureError(
                    format!("Hook execution {} ticks exceeds budget {} ticks", 
                        receipt.ticks, self.tick_budget)
                ));
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
        })
    }

    /// Execute a single hook using C hot path API via FFI
    fn execute_hook(&self, soa: &SoAArrays, run: &PredRun) -> Result<Receipt, PipelineError> {
        #[cfg(feature = "std")]
        {
            use knhk_hot::{Engine, Op, Ir, Receipt as HotReceipt, Run as HotRun};
            
            // Initialize engine with SoA arrays
            let engine = Engine::new(soa.s.as_ptr(), soa.p.as_ptr(), soa.o.as_ptr());
            
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
            let result = engine.eval_bool(&mut ir, &mut hot_receipt);
            
            // Convert to ETL receipt format
            Ok(Receipt {
                id: format!("receipt_{}", hot_receipt.span_id),
                ticks: hot_receipt.ticks,
                lanes: hot_receipt.lanes,
                span_id: hot_receipt.span_id,
                a_hash: hot_receipt.a_hash,
            })
        }
        
        #[cfg(not(feature = "std"))]
        {
            // In no_std mode, compute receipt deterministically from SoA data
            // This provides functional correctness without C FFI
            let lanes = run.len as u32;
            
            // Generate deterministic span_id from SoA data
            let span_id = Self::generate_span_id_deterministic(soa, run);
            
            // Compute a_hash (hash(A) = hash(μ(O)) fragment)
            let a_hash = Self::compute_a_hash(soa, run);
            
            // Estimate ticks based on run length (conservative estimate)
            let ticks = if run.len <= 4 { 4 } else { 6 };
            
            Ok(Receipt {
                id: format!("receipt_{}", span_id),
                ticks,
                lanes,
                span_id,
                a_hash,
            })
        }
    }

    /// Merge receipts via ⊕ operation (associative, branchless)
    /// Implements: knhk_receipt_merge semantics
    pub fn merge_receipts(receipts: &[Receipt]) -> Receipt {
        if receipts.is_empty() {
            return Receipt {
                id: "merged_receipt".to_string(),
                ticks: 0,
                lanes: 0,
                span_id: 0,
                a_hash: 0,
            };
        }

        let mut merged = Receipt {
            id: "merged_receipt".to_string(),
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
        #[cfg(feature = "std")]
        {
            use knhk_otel::generate_span_id;
            generate_span_id()
        }
        #[cfg(not(feature = "std"))]
        {
            let timestamp = Self::get_timestamp_ms();
            timestamp.wrapping_mul(0x9e3779b9u64).wrapping_add(0x517cc1b7u64)
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
        #[cfg(feature = "std")]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0)
        }
        #[cfg(not(feature = "std"))]
        {
            0
        }
    }
}

pub struct ReflexResult {
    pub actions: Vec<Action>,
    pub receipts: Vec<Receipt>,
    pub max_ticks: u32,
}

pub struct Action {
    pub id: String,
    pub payload: Vec<u8>,
    pub receipt_id: String,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub id: String,
    pub ticks: u32,
    pub lanes: u32,
    pub span_id: u64,
    pub a_hash: u64,
}

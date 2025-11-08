// rust/knhk-etl/src/reconcile.rs
// μ(Δ) reconciliation function - LAW: A = μ(O)
// Transforms observations into actions via kernel dispatch

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hash::{hash_actions, hash_delta, hash_soa};
use crate::hook_registry::HookRegistry;
use crate::ingest::RawTriple;
use crate::load::SoAArrays;
use crate::reflex::{Action, Receipt};
use knhk_hot::{KernelExecutor, KernelType};

/// Reconciliation error types
#[derive(Debug, Clone)]
pub enum ReconcileError {
    /// No hook registered for predicate
    NoHook,
    /// Tick budget exceeded
    BudgetExceeded { actual: u64, limit: u64 },
    /// Provenance law violated
    ProvenanceViolation { expected: u64, actual: u64 },
    /// Invalid SoA data
    InvalidSoa(String),
    /// Kernel execution error
    KernelError(String),
}

impl core::fmt::Display for ReconcileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ReconcileError::NoHook => write!(f, "No hook registered for predicate"),
            ReconcileError::BudgetExceeded { actual, limit } => {
                write!(f, "Budget exceeded: {} > {}", actual, limit)
            }
            ReconcileError::ProvenanceViolation { expected, actual } => {
                write!(
                    f,
                    "Provenance violation: expected {:#x}, got {:#x}",
                    expected, actual
                )
            }
            ReconcileError::InvalidSoa(msg) => write!(f, "Invalid SoA: {}", msg),
            ReconcileError::KernelError(msg) => write!(f, "Kernel error: {}", msg),
        }
    }
}

// Hook registry moved to separate module: crate::hook_registry

/// Reconciliation context
pub struct ReconcileContext {
    /// Hook registry
    hook_registry: HookRegistry,
    /// Tick budget (≤8)
    tick_budget: u64,
    /// Cycles per tick (for conversion)
    cycles_per_tick: u64,
}

impl ReconcileContext {
    /// Create new reconciliation context
    pub fn new(tick_budget: u64) -> Self {
        if tick_budget > 8 {
            panic!("tick_budget {} exceeds Chatman Constant (8)", tick_budget);
        }

        Self {
            hook_registry: HookRegistry::new(),
            tick_budget,
            cycles_per_tick: 1, // Conservative: 1 cycle = 1 tick
        }
    }

    /// Register hook for predicate (without guard)
    pub fn register_hook(&mut self, predicate: u64, kernel_type: KernelType) {
        use crate::hook_registry::guards;
        let _ = self.hook_registry.register_hook(
            predicate,
            kernel_type,
            guards::always_valid,
            Vec::new(),
        );
    }

    /// Register hook with guard function
    pub fn register_hook_with_guard(
        &mut self,
        predicate: u64,
        kernel_type: KernelType,
        guard: crate::hook_registry::GuardFn,
        invariants: Vec<String>,
    ) -> Result<u64, crate::hook_registry::HookRegistryError> {
        self.hook_registry
            .register_hook(predicate, kernel_type, guard, invariants)
    }

    /// Reconcile delta: A = μ(O)
    ///
    /// # Arguments
    /// * `delta` - Observations to reconcile
    /// * `soa` - SoA arrays (pre-converted)
    /// * `tick` - Current tick
    ///
    /// # Returns
    /// - Ok(actions): Actions with verified provenance
    /// - Err(error): Reconciliation error
    pub fn reconcile_delta(
        &self,
        delta: &[RawTriple],
        soa: &SoAArrays,
        tick: u64,
    ) -> Result<Vec<Action>, ReconcileError> {
        if delta.is_empty() {
            return Ok(Vec::new());
        }

        // Validate SoA bounds
        if delta.len() > 8 {
            return Err(ReconcileError::InvalidSoa(format!(
                "Delta length {} exceeds 8",
                delta.len()
            )));
        }

        let mut actions = Vec::new();

        // Extract predicate (assume homogeneous for ≤8 items)
        let predicate = soa.p[0];

        // Lookup hook for this predicate
        let kernel_type = if self.hook_registry.has_hook(predicate) {
            self.hook_registry.get_kernel(predicate)
        } else {
            return Err(ReconcileError::NoHook);
        };

        // Check guard (O ⊨ Σ) - validate all triples in delta
        for triple in delta {
            if !self.hook_registry.check_guard(predicate, triple) {
                return Err(ReconcileError::KernelError(
                    "Guard validation failed".to_string(),
                ));
            }
        }

        // Execute kernel via dispatch (branchless)
        let (cycles, out_mask) =
            KernelExecutor::execute_dispatch(kernel_type, &soa.s, &soa.p, &soa.o, delta.len())
                .map_err(ReconcileError::KernelError)?;

        // Check τ ≤ 8 constraint (convert cycles to ticks)
        let ticks = cycles / self.cycles_per_tick;
        if ticks > self.tick_budget {
            return Err(ReconcileError::BudgetExceeded {
                actual: ticks,
                limit: self.tick_budget,
            });
        }

        // Generate actions from mask (only validated rows)
        for (i, triple) in delta.iter().enumerate() {
            if (out_mask & (1 << i)) != 0 {
                // Serialize triple into payload for hash provenance: hash(A) = hash(μ(O))
                // Format: "subject|predicate|object" (deterministic, matches hash_delta order)
                let payload = format!("{}|{}|{}", triple.subject, triple.predicate, triple.object)
                    .into_bytes();

                actions.push(Action {
                    id: format!("action_{}", i),
                    payload,
                    receipt_id: format!("receipt_{}_{}", tick, i),
                });
            }
        }

        // LAW: hash(A) = hash(μ(O))
        let hash_a = hash_actions(&actions);
        let hash_mu_o = hash_delta(delta);

        if hash_a != hash_mu_o {
            return Err(ReconcileError::ProvenanceViolation {
                expected: hash_mu_o,
                actual: hash_a,
            });
        }

        Ok(actions)
    }

    /// Reconcile with full receipt generation
    ///
    /// Returns actions and receipt with provenance
    pub fn reconcile_with_receipt(
        &self,
        delta: &[RawTriple],
        soa: &SoAArrays,
        tick: u64,
        cycle_id: u64,
        shard_id: u64,
        hook_id: u64,
    ) -> Result<(Vec<Action>, Receipt), ReconcileError> {
        if delta.is_empty() {
            return Ok((
                Vec::new(),
                Receipt {
                    id: format!("receipt_{}_{}", cycle_id, tick),
                    cycle_id,
                    shard_id,
                    hook_id,
                    ticks: 0,
                    actual_ticks: 0,
                    lanes: 0,
                    span_id: {
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
                    }, // Generate OTEL-compatible span ID
                    a_hash: 0,
                },
            ));
        }

        // Validate SoA bounds
        if delta.len() > 8 {
            return Err(ReconcileError::InvalidSoa(format!(
                "Delta length {} exceeds 8",
                delta.len()
            )));
        }

        // Extract predicate
        let predicate = soa.p[0];

        // Lookup hook
        let kernel_type = if self.hook_registry.has_hook(predicate) {
            self.hook_registry.get_kernel(predicate)
        } else {
            return Err(ReconcileError::NoHook);
        };

        // Check guard for all triples
        for triple in delta {
            if !self.hook_registry.check_guard(predicate, triple) {
                return Err(ReconcileError::KernelError(
                    "Guard validation failed".to_string(),
                ));
            }
        }

        // Execute kernel
        let (cycles, out_mask) =
            KernelExecutor::execute_dispatch(kernel_type, &soa.s, &soa.p, &soa.o, delta.len())
                .map_err(ReconcileError::KernelError)?;

        // Convert cycles to ticks
        let ticks = (cycles / self.cycles_per_tick) as u32;

        // Check budget
        if ticks as u64 > self.tick_budget {
            return Err(ReconcileError::BudgetExceeded {
                actual: ticks as u64,
                limit: self.tick_budget,
            });
        }

        // Generate actions with payload for hash provenance
        let mut actions = Vec::new();
        for (i, quad) in delta.iter().enumerate() {
            if (out_mask & (1 << i)) != 0 {
                // Serialize triple into payload for hash provenance: hash(A) = hash(μ(O))
                // Format: "subject|predicate|object|graph" (deterministic, matches hash_delta order)
                let payload = if let Some(ref graph) = quad.graph {
                    format!(
                        "{}|{}|{}|{}",
                        quad.subject, quad.predicate, quad.object, graph
                    )
                    .into_bytes()
                } else {
                    format!("{}|{}|{}", quad.subject, quad.predicate, quad.object).into_bytes()
                };

                actions.push(Action {
                    id: format!("action_{}_{}", tick, i),
                    payload,
                    receipt_id: format!("receipt_{}_{}", cycle_id, tick),
                });
            }
        }

        // Compute provenance hash
        let a_hash = hash_soa(&soa.s, &soa.p, &soa.o, delta.len());

        // Generate span ID (deterministic based on cycle + tick)
        let span_id = (cycle_id << 32) | tick;

        // Create receipt
        let receipt = Receipt {
            id: format!("receipt_{}_{}", cycle_id, tick),
            cycle_id,
            shard_id,
            hook_id,
            ticks,
            actual_ticks: ticks,
            lanes: delta.len() as u32,
            span_id,
            a_hash,
        };

        // Verify provenance LAW
        let hash_a = hash_actions(&actions);
        let hash_mu_o = hash_delta(delta);

        if hash_a != hash_mu_o {
            return Err(ReconcileError::ProvenanceViolation {
                expected: hash_mu_o,
                actual: hash_a,
            });
        }

        Ok((actions, receipt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_registry_integration() {
        let mut ctx = ReconcileContext::new(8);
        ctx.register_hook(100, KernelType::AskSp);
        ctx.register_hook(200, KernelType::CountSpGe);

        // Verify hooks registered
        assert!(ctx.hook_registry.has_hook(100));
        assert!(ctx.hook_registry.has_hook(200));
        assert_eq!(ctx.hook_registry.get_kernel(100), KernelType::AskSp);
        assert_eq!(ctx.hook_registry.get_kernel(200), KernelType::CountSpGe);
    }

    #[test]
    fn test_reconcile_context_creation() {
        let ctx = ReconcileContext::new(8);
        assert_eq!(ctx.tick_budget, 8);
    }

    #[test]
    #[should_panic(expected = "exceeds Chatman Constant")]
    fn test_reconcile_context_invalid_budget() {
        ReconcileContext::new(9);
    }

    #[test]
    fn test_reconcile_delta_empty() {
        let ctx = ReconcileContext::new(8);
        let delta: Vec<RawTriple> = Vec::new();
        let soa = SoAArrays::new();

        let result = ctx.reconcile_delta(&delta, &soa, 0);
        assert!(result.is_ok());
        assert_eq!(
            result
                .expect("Empty delta reconciliation should succeed")
                .len(),
            0
        );
    }

    #[test]
    fn test_reconcile_delta_exceeds_bounds() {
        let ctx = ReconcileContext::new(8);
        let delta: Vec<RawTriple> = (0..10)
            .map(|i| RawTriple {
                subject: format!("s{}", i),
                predicate: format!("p{}", i),
                object: format!("o{}", i),
                graph: None,
            })
            .collect();
        let soa = SoAArrays::new();

        let result = ctx.reconcile_delta(&delta, &soa, 0);
        assert!(matches!(result, Err(ReconcileError::InvalidSoa(_))));
    }
}

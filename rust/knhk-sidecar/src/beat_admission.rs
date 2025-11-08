// rust/knhk-sidecar/src/beat_admission.rs
// Beat-driven admission for 8-beat epoch system
// W1 routing for CONSTRUCT8 operations and L1 locality protection

use crate::error::{SidecarError, SidecarResult};
use knhk_etl::beat_scheduler::{BeatScheduler, BeatSchedulerError};
use knhk_etl::ingest::RawTriple;
use std::sync::{Arc, Mutex};

/// Variable marker for unbound variables in CONSTRUCT templates
/// 0xFFFF_FFFF_FFFF_FFFF indicates an unbound variable slot
const VARIABLE_MARKER: u64 = 0xFFFF_FFFF_FFFF_FFFF;

/// Path tier for delta routing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PathTier {
    /// R1: Hot path - τ ≤ 8 ticks (~2ns), branchless kernels
    R1,
    /// W1: Warm path - τ ≤ 500ms, CONSTRUCT8 operations
    W1,
    /// C1: Cold path - async finalization, long-running queries
    C1,
}

/// Operation type classification for routing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperationType {
    /// ASK query - hot path kernel
    Ask,
    /// COUNT query - hot path kernel
    Count,
    /// VALIDATE query - hot path kernel
    Validate,
    /// CONSTRUCT query - W1 epistemology generation
    Construct,
    /// Template-based generation - W1
    Template,
    /// SELECT query - may route to W1/C1 based on complexity
    Select,
    /// Update operation - W1 for writes
    Update,
}

/// Reason for parking delta to W1/C1
#[derive(Debug, Clone, PartialEq)]
pub enum ParkReason {
    /// Requires CONSTRUCT8 epistemology generation
    WarmPathRequired,
    /// Estimated ticks exceed 8-tick budget
    BudgetExceeded,
    /// L1 cache miss predicted - protect hot path
    ColdCache,
    /// Complex query requiring async processing
    ComplexQuery,
}

/// Admission decision for delta routing
#[derive(Debug, Clone, PartialEq)]
pub enum AdmissionDecision {
    /// Admit to R1 hot path (≤8 ticks)
    Admit {
        /// Beat tick when admitted (0-7)
        tick: u64,
        /// Estimated execution ticks
        estimated_ticks: u64,
    },
    /// Park to W1/C1 (>8 ticks or CONSTRUCT)
    Park {
        /// Reason for parking
        reason: ParkReason,
        /// Destination tier
        destination: PathTier,
        /// Estimated execution ticks
        estimated_ticks: u64,
    },
}

/// Delta representation for admission analysis
pub struct Delta {
    /// Raw triples in the delta
    pub triples: Vec<RawTriple>,
    /// Operation type (if classifiable)
    pub operation_type: Option<OperationType>,
    /// Encoded S/P/O values (if available)
    pub encoded_triples: Vec<(u64, u64, u64)>,
}

impl Delta {
    /// Create new delta from raw triples
    pub fn new(triples: Vec<RawTriple>) -> Self {
        Self {
            triples,
            operation_type: None,
            encoded_triples: Vec::new(),
        }
    }

    /// Create delta with operation type
    pub fn with_operation(mut self, op_type: OperationType) -> Self {
        self.operation_type = Some(op_type);
        self
    }

    /// Add encoded triples (after dictionary encoding)
    pub fn with_encoded(mut self, encoded: Vec<(u64, u64, u64)>) -> Self {
        self.encoded_triples = encoded;
        self
    }

    /// Check if delta requires CONSTRUCT8 (epistemology generation)
    ///
    /// CONSTRUCT8 is needed when:
    /// 1. Triple has unbound variables (S/P/O == VARIABLE_MARKER)
    /// 2. Operation type is Construct or Template
    /// 3. Blank node creation required (detected by "_:" prefix)
    pub fn requires_construct(&self) -> bool {
        // Check encoded triples for variable markers
        for (s, p, o) in &self.encoded_triples {
            if *s == VARIABLE_MARKER || *p == VARIABLE_MARKER || *o == VARIABLE_MARKER {
                return true;
            }
        }

        // Check operation type
        if let Some(op_type) = &self.operation_type {
            if matches!(op_type, OperationType::Construct | OperationType::Template) {
                return true;
            }
        }

        // Check for blank nodes in raw triples (indicates generation)
        for triple in &self.triples {
            if triple.subject.starts_with("_:") || triple.object.starts_with("_:") {
                return true;
            }
        }

        false
    }

    /// Estimate complexity for routing decision
    pub fn estimate_complexity(&self) -> usize {
        let triple_count = self.triples.len().max(self.encoded_triples.len());

        // Base complexity is triple count
        let mut complexity = triple_count;

        // Add complexity for CONSTRUCT operations
        if self.requires_construct() {
            complexity += 20; // CONSTRUCT adds ~200 ticks
        }

        // Add complexity for blank nodes
        let blank_node_count = self
            .triples
            .iter()
            .filter(|t| t.subject.starts_with("_:") || t.object.starts_with("_:"))
            .count();
        complexity += blank_node_count * 5;

        complexity
    }
}

/// L1 locality predictor (stub for now)
pub struct LocalityPredictor {
    /// Current beat tick
    current_tick: u64,
}

impl LocalityPredictor {
    pub fn new() -> Self {
        Self { current_tick: 0 }
    }

    /// Estimate ticks for delta execution
    pub fn estimate_ticks(&self, delta: &Delta) -> u64 {
        let complexity = delta.estimate_complexity();

        // Base estimation:
        // - Simple ASK/COUNT: 2-8 ticks
        // - CONSTRUCT: 200+ ticks
        // - Complex query: 1000+ ticks
        if delta.requires_construct() {
            200 // CONSTRUCT8 baseline
        } else if complexity <= 8 {
            2 + (complexity as u64) // Hot path kernels
        } else {
            50 + (complexity as u64 * 10) // Complex operations
        }
    }

    /// Check if delta data is likely in L1 cache
    pub fn check_l1_locality(&self, delta: &Delta) -> bool {
        // Heuristic: small deltas (≤8 triples) likely fit in L1
        // Production would use actual cache tracking
        delta.triples.len() <= 8 && delta.encoded_triples.len() <= 8
    }

    /// Update tick counter
    pub fn update_tick(&mut self, tick: u64) {
        self.current_tick = tick;
    }
}

/// Beat admission manager with W1 routing
pub struct BeatAdmission {
    /// Beat scheduler (shared across all admission requests)
    beat_scheduler: Arc<Mutex<BeatScheduler>>,
    /// Default domain ID for admission (0 = default domain)
    default_domain_id: usize,
    /// L1 locality predictor (wrapped in Mutex for Sync)
    predictor: Mutex<LocalityPredictor>,
}

// SAFETY: BeatAdmission contains raw pointers in BeatScheduler, but we ensure
// thread safety by only accessing it through the Mutex. The raw pointers are
// only used within the C FFI layer and are not accessed concurrently.
unsafe impl Send for BeatAdmission {}
unsafe impl Sync for BeatAdmission {}

impl BeatAdmission {
    /// Create new beat admission manager
    pub fn new(beat_scheduler: Arc<Mutex<BeatScheduler>>, default_domain_id: usize) -> Self {
        Self {
            beat_scheduler,
            default_domain_id,
            predictor: Mutex::new(LocalityPredictor::new()),
        }
    }

    /// Admit delta with W1 routing decision
    ///
    /// Decision flow:
    /// 1. Check if CONSTRUCT8 required → W1
    /// 2. Estimate ticks → if >8 → W1
    /// 3. Check L1 locality → if cold → W1
    /// 4. Otherwise → R1 (hot path)
    pub fn admit_delta_with_routing(&self, delta: &Delta) -> AdmissionDecision {
        // Get current tick from beat scheduler
        let current_tick = match self.get_current_tick() {
            Ok(tick) => tick,
            Err(_) => 0, // Fallback if lock fails
        };

        // Lock predictor for this routing decision
        // If lock is poisoned (panic in another thread), recover by parking to W1 (safe default)
        let mut predictor = match self.predictor.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                // Mutex was poisoned by a panic in another thread
                // Log and return safe default: park to W1
                #[cfg(feature = "otel")]
                tracing::warn!("Predictor mutex poisoned, parking delta to W1 as safe default");
                return AdmissionDecision::Park {
                    reason: ParkReason::WarmPathRequired,
                    destination: PathTier::W1,
                    estimated_ticks: 200, // Conservative estimate
                };
            }
        };
        predictor.update_tick(current_tick);

        // CHECK 1: Does delta require CONSTRUCT8? → W1
        if delta.requires_construct() {
            return AdmissionDecision::Park {
                reason: ParkReason::WarmPathRequired,
                destination: PathTier::W1,
                estimated_ticks: 200, // CONSTRUCT8 ~200 ticks (50ns)
            };
        }

        // CHECK 2: Existing heatmap + L1 prediction
        let predicted_ticks = predictor.estimate_ticks(delta);
        if predicted_ticks > 8 {
            return AdmissionDecision::Park {
                reason: ParkReason::BudgetExceeded,
                destination: PathTier::W1,
                estimated_ticks: predicted_ticks,
            };
        }

        // CHECK 3: L1 locality check
        let l1_ready = predictor.check_l1_locality(delta);
        if !l1_ready {
            return AdmissionDecision::Park {
                reason: ParkReason::ColdCache,
                destination: PathTier::W1,
                estimated_ticks: predicted_ticks + 50, // + L1 miss penalty
            };
        }

        // ADMIT to R1 (hot path)
        AdmissionDecision::Admit {
            tick: current_tick,
            estimated_ticks: predicted_ticks,
        }
    }

    /// Admit delta to beat scheduler (original API for backwards compatibility)
    /// Returns cycle_id for response correlation
    pub fn admit_delta(
        &self,
        delta: Vec<RawTriple>,
        domain_id: Option<usize>,
    ) -> SidecarResult<u64> {
        let domain = domain_id.unwrap_or(self.default_domain_id);

        // Get current cycle from beat scheduler
        let current_cycle = self
            .beat_scheduler
            .lock()
            .map_err(|e| {
                SidecarError::internal_error(format!(
                    "Failed to acquire beat scheduler lock: {}",
                    e
                ))
            })?
            .current_cycle();

        // Enqueue delta to delta ring with cycle_id stamping
        self.beat_scheduler
            .lock()
            .map_err(|e| {
                SidecarError::internal_error(format!(
                    "Failed to acquire beat scheduler lock: {}",
                    e
                ))
            })?
            .enqueue_delta(domain, delta, current_cycle)
            .map_err(|e| match e {
                BeatSchedulerError::RingBufferFull => SidecarError::BatchError {
                    context: crate::error::ErrorContext::new(
                        "SIDECAR_BEAT_RING_FULL",
                        "Delta ring buffer is full - backpressure applied",
                    )
                    .with_attribute("domain_id", domain.to_string())
                    .with_attribute("cycle_id", current_cycle.to_string()),
                },
                BeatSchedulerError::InvalidDomainCount => {
                    SidecarError::config_error(format!("Invalid domain ID: {}", domain))
                }
                _ => SidecarError::internal_error(format!("Beat scheduler error: {:?}", e)),
            })?;

        Ok(current_cycle)
    }

    /// Get current cycle
    pub fn get_current_cycle(&self) -> SidecarResult<u64> {
        Ok(self
            .beat_scheduler
            .lock()
            .map_err(|e| {
                SidecarError::internal_error(format!(
                    "Failed to acquire beat scheduler lock: {}",
                    e
                ))
            })?
            .current_cycle())
    }

    /// Get current tick (0-7)
    pub fn get_current_tick(&self) -> SidecarResult<u64> {
        Ok(self
            .beat_scheduler
            .lock()
            .map_err(|e| {
                SidecarError::internal_error(format!(
                    "Failed to acquire beat scheduler lock: {}",
                    e
                ))
            })?
            .current_tick())
    }

    /// Check if admission should throttle (backpressure)
    pub fn should_throttle(&self, _domain_id: Option<usize>) -> SidecarResult<bool> {
        // Check if delta ring is full
        // Note: RingBuffer doesn't expose is_full() directly, so we check by attempting enqueue
        // In production, would add is_full() method to RingBuffer
        Ok(false) // Placeholder - would check ring buffer capacity
    }

    /// Get park count (number of parked deltas)
    pub fn get_park_count(&self) -> SidecarResult<usize> {
        Ok(self
            .beat_scheduler
            .lock()
            .map_err(|e| {
                SidecarError::internal_error(format!(
                    "Failed to acquire beat scheduler lock: {}",
                    e
                ))
            })?
            .park_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_marker_constant() {
        assert_eq!(VARIABLE_MARKER, 0xFFFF_FFFF_FFFF_FFFF);
    }

    #[test]
    fn test_path_tier_values() {
        assert_eq!(PathTier::R1, PathTier::R1);
        assert_eq!(PathTier::W1, PathTier::W1);
        assert_eq!(PathTier::C1, PathTier::C1);
    }

    #[test]
    fn test_delta_requires_construct_with_variables() {
        let delta = Delta {
            triples: vec![],
            operation_type: None,
            encoded_triples: vec![
                (VARIABLE_MARKER, 123, 456), // Subject is variable
            ],
        };

        assert!(delta.requires_construct());
    }

    #[test]
    fn test_delta_requires_construct_with_operation_type() {
        let delta = Delta {
            triples: vec![],
            operation_type: Some(OperationType::Construct),
            encoded_triples: vec![],
        };

        assert!(delta.requires_construct());
    }

    #[test]
    fn test_delta_requires_construct_with_blank_nodes() {
        let delta = Delta {
            triples: vec![RawTriple {
                subject: "_:b1".to_string(),
                predicate: "http://example.org/prop".to_string(),
                object: "value".to_string(),
                graph: None,
            }],
            operation_type: None,
            encoded_triples: vec![],
        };

        assert!(delta.requires_construct());
    }

    #[test]
    fn test_delta_no_construct_required() {
        let delta = Delta {
            triples: vec![RawTriple {
                subject: "http://example.org/subject".to_string(),
                predicate: "http://example.org/predicate".to_string(),
                object: "value".to_string(),
                graph: None,
            }],
            operation_type: Some(OperationType::Ask),
            encoded_triples: vec![(1, 2, 3)],
        };

        assert!(!delta.requires_construct());
    }

    #[test]
    fn test_construct8_routes_to_w1() {
        let beat_scheduler = Arc::new(Mutex::new(
            BeatScheduler::new(1).expect("Failed to create beat scheduler"),
        ));
        let admission = BeatAdmission::new(beat_scheduler, 0);

        let delta_with_variables = Delta {
            triples: vec![],
            operation_type: Some(OperationType::Construct),
            encoded_triples: vec![(VARIABLE_MARKER, 123, 456)],
        };

        let decision = admission.admit_delta_with_routing(&delta_with_variables);

        match decision {
            AdmissionDecision::Park {
                destination,
                reason,
                ..
            } => {
                assert_eq!(destination, PathTier::W1);
                assert_eq!(reason, ParkReason::WarmPathRequired);
            }
            _ => panic!("Expected Park decision for CONSTRUCT8"),
        }
    }

    #[test]
    fn test_hot_path_admission() {
        let beat_scheduler = Arc::new(Mutex::new(
            BeatScheduler::new(1).expect("Failed to create beat scheduler"),
        ));
        let admission = BeatAdmission::new(beat_scheduler, 0);

        let simple_delta = Delta {
            triples: vec![RawTriple {
                subject: "http://example.org/s".to_string(),
                predicate: "http://example.org/p".to_string(),
                object: "value".to_string(),
                graph: None,
            }],
            operation_type: Some(OperationType::Ask),
            encoded_triples: vec![(1, 2, 3)],
        };

        let decision = admission.admit_delta_with_routing(&simple_delta);

        match decision {
            AdmissionDecision::Admit {
                estimated_ticks, ..
            } => {
                assert!(estimated_ticks <= 8, "Hot path should be ≤8 ticks");
            }
            _ => panic!("Expected Admit decision for simple ASK"),
        }
    }

    #[test]
    fn test_budget_exceeded_routes_to_w1() {
        let beat_scheduler = Arc::new(Mutex::new(
            BeatScheduler::new(1).expect("Failed to create beat scheduler"),
        ));
        let admission = BeatAdmission::new(beat_scheduler, 0);

        // Create delta with high complexity
        let mut complex_triples = Vec::new();
        for i in 0..20 {
            complex_triples.push(RawTriple {
                subject: format!("http://example.org/s{}", i),
                predicate: "http://example.org/p".to_string(),
                object: format!("value{}", i),
                graph: None,
            });
        }

        let complex_delta = Delta {
            triples: complex_triples,
            operation_type: Some(OperationType::Select),
            encoded_triples: vec![],
        };

        let decision = admission.admit_delta_with_routing(&complex_delta);

        match decision {
            AdmissionDecision::Park {
                destination,
                reason,
                estimated_ticks,
            } => {
                assert_eq!(destination, PathTier::W1);
                assert_eq!(reason, ParkReason::BudgetExceeded);
                assert!(estimated_ticks > 8, "Complex query should exceed 8 ticks");
            }
            _ => panic!("Expected Park decision for complex query"),
        }
    }

    #[test]
    fn test_locality_predictor_estimates() {
        let predictor = LocalityPredictor::new();

        // Simple delta
        let simple = Delta {
            triples: vec![RawTriple {
                subject: "s".to_string(),
                predicate: "p".to_string(),
                object: "o".to_string(),
                graph: None,
            }],
            operation_type: Some(OperationType::Ask),
            encoded_triples: vec![(1, 2, 3)],
        };
        assert!(predictor.estimate_ticks(&simple) <= 8);

        // CONSTRUCT delta
        let construct = Delta {
            triples: vec![],
            operation_type: Some(OperationType::Construct),
            encoded_triples: vec![(VARIABLE_MARKER, 2, 3)],
        };
        assert_eq!(predictor.estimate_ticks(&construct), 200);
    }

    #[test]
    fn test_delta_estimate_complexity() {
        let simple = Delta {
            triples: vec![RawTriple {
                subject: "s".to_string(),
                predicate: "p".to_string(),
                object: "o".to_string(),
                graph: None,
            }],
            operation_type: None,
            encoded_triples: vec![],
        };
        assert_eq!(simple.estimate_complexity(), 1);

        let construct = Delta {
            triples: vec![],
            operation_type: Some(OperationType::Construct),
            encoded_triples: vec![(VARIABLE_MARKER, 2, 3)],
        };
        assert_eq!(construct.estimate_complexity(), 21); // 1 + 20 for CONSTRUCT

        let with_blank = Delta {
            triples: vec![RawTriple {
                subject: "_:b1".to_string(),
                predicate: "p".to_string(),
                object: "_:b2".to_string(),
                graph: None,
            }],
            operation_type: None,
            encoded_triples: vec![],
        };
        assert_eq!(with_blank.estimate_complexity(), 11); // 1 + 2*5 for blank nodes
    }
}

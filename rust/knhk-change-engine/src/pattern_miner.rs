//! Pattern Miner - Detects schema drift and patterns in receipts
//!
//! The Pattern Miner maintains a rolling window of recent operations and continuously
//! analyzes them to detect:
//!
//! - Schema mismatches (triples that don't conform to Σ)
//! - Repeated structures (candidates for new classes/properties)
//! - Guard violations (operations that nearly violate constraints)
//! - Performance regressions (operations slower than expected)

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn, span, Level};

/// Pattern Miner - detects schema drift and patterns in receipts
#[derive(Debug)]
pub struct PatternMiner {
    /// Rolling window of recent receipts
    observation_window: VecDeque<Receipt>,

    /// Maximum window size
    window_size: usize,

    /// Detected patterns (thread-safe, updated continuously)
    patterns: Arc<RwLock<DetectedPatterns>>,
}

/// A receipt from a Σ operation (simplified for now)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Unique receipt ID
    pub id: String,

    /// Timestamp (ticks)
    pub timestamp: u64,

    /// Operations performed
    pub operations: Vec<Operation>,

    /// Performance metrics
    pub metrics: PerformanceMetrics,
}

/// A single operation in a receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation type
    pub op_type: OperationType,

    /// Triple affected
    pub triple: Triple,

    /// Result
    pub result: OperationResult,
}

/// Operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// Insert triple
    Insert,
    /// Query triple
    Query,
    /// Delete triple
    Delete,
}

/// Triple (simplified RDF triple)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Triple {
    /// Subject
    pub subject: String,
    /// Predicate
    pub predicate: String,
    /// Object
    pub object: String,
}

/// Operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    /// Success
    Success,
    /// Schema mismatch
    SchemaMismatch(String),
    /// Guard violation
    GuardViolation(String),
    /// Performance issue
    PerformanceIssue(String),
}

/// Performance metrics for a receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total ticks consumed
    pub total_ticks: u32,
    /// Operations per second
    pub ops_per_sec: f64,
}

/// Detected patterns (mutable, updated continuously)
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DetectedPatterns {
    /// Schema mismatches: triples that don't conform to current Σ
    pub schema_mismatches: Vec<SchemaMismatch>,

    /// Repeated structures: triples with same pattern in different contexts
    pub repeated_structures: Vec<RepeatedStructure>,

    /// Guard violations: operations close to violating a guard
    pub guard_violations: Vec<GuardViolation>,

    /// Performance regressions: operations slower than expected
    pub performance_regressions: Vec<PerfRegression>,
}

/// Schema mismatch detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaMismatch {
    /// The triple that doesn't match
    pub triple: Triple,

    /// Reason for mismatch
    pub reason: String,

    /// How many times seen
    pub frequency: u32,
}

/// Repeated structure detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedStructure {
    /// Pattern (None = wildcard)
    pub pattern: (Option<String>, String, Option<String>),

    /// Example triples matching this pattern
    pub examples: Vec<(String, String)>,

    /// Count of occurrences
    pub count: u32,

    /// Suggested class name
    pub candidate_class: Option<String>,
}

/// Guard violation (near-miss)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardViolation {
    /// Guard name
    pub guard_name: String,

    /// Number of near-misses
    pub near_miss_count: u32,

    /// Affected subjects
    pub affected_subjects: Vec<String>,
}

/// Performance regression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfRegression {
    /// Operator name
    pub operator: String,

    /// Observed latency (ticks)
    pub observed_latency_ticks: u32,

    /// Expected latency (ticks)
    pub expected_latency_ticks: u32,

    /// Regression factor (observed / expected)
    pub regression_factor: f64,
}

impl PatternMiner {
    /// Create a new pattern miner with default window size
    pub fn new(window_size: usize) -> Self {
        let span = span!(Level::INFO, "pattern_miner_init", window_size);
        let _enter = span.enter();

        info!(window_size, "Initializing pattern miner");

        Self {
            observation_window: VecDeque::with_capacity(window_size),
            window_size,
            patterns: Arc::new(RwLock::new(DetectedPatterns::default())),
        }
    }

    /// Scan a receipt and update detected patterns
    pub fn scan_receipt(&mut self, receipt: Receipt) {
        let span = span!(Level::DEBUG, "scan_receipt", receipt_id = %receipt.id);
        let _enter = span.enter();

        debug!(receipt_id = %receipt.id, ops = receipt.operations.len(), "Scanning receipt");

        // Add to window
        self.observation_window.push_back(receipt.clone());

        // Maintain window size
        if self.observation_window.len() > self.window_size {
            self.observation_window.pop_front();
        }

        // Analyze patterns
        self.analyze_patterns(&receipt);
    }

    /// Analyze patterns in a receipt
    fn analyze_patterns(&mut self, receipt: &Receipt) {
        let mut patterns = self.patterns.write();

        // 1. Detect schema mismatches
        for op in &receipt.operations {
            if let OperationResult::SchemaMismatch(reason) = &op.result {
                self.record_schema_mismatch(&mut patterns, &op.triple, reason);
            }
        }

        // 2. Detect repeated structures
        self.detect_repeated_structures(&mut patterns);

        // 3. Detect guard violations
        for op in &receipt.operations {
            if let OperationResult::GuardViolation(guard) = &op.result {
                self.record_guard_violation(&mut patterns, guard, &op.triple.subject);
            }
        }

        // 4. Detect performance regressions (Chatman Constant: ≤8 ticks)
        if receipt.metrics.total_ticks > 8 {
            self.record_perf_regression(&mut patterns, receipt);
        }
    }

    /// Record a schema mismatch
    fn record_schema_mismatch(&self, patterns: &mut DetectedPatterns, triple: &Triple, reason: &str) {
        // Find existing mismatch or create new
        // Group by predicate and reason (not exact triple match, since subjects vary)
        if let Some(mismatch) = patterns.schema_mismatches.iter_mut()
            .find(|m| m.triple.predicate == triple.predicate && m.reason == reason) {
            mismatch.frequency += 1;
        } else {
            patterns.schema_mismatches.push(SchemaMismatch {
                triple: triple.clone(),
                reason: reason.to_string(),
                frequency: 1,
            });
        }
    }

    /// Detect repeated structures in observation window
    fn detect_repeated_structures(&self, patterns: &mut DetectedPatterns) {
        // Clear existing repeated structures before rebuilding
        patterns.repeated_structures.clear();

        let mut predicate_groups: HashMap<String, Vec<Triple>> = HashMap::new();

        // Group triples by predicate
        for receipt in &self.observation_window {
            for op in &receipt.operations {
                predicate_groups
                    .entry(op.triple.predicate.clone())
                    .or_default()
                    .push(op.triple.clone());
            }
        }

        // Find predicates with high frequency
        for (predicate, triples) in predicate_groups {
            if triples.len() >= 5 {
                // Candidate for new property or class
                let examples: Vec<_> = triples.iter()
                    .take(3)
                    .map(|t| (t.subject.clone(), t.object.clone()))
                    .collect();

                patterns.repeated_structures.push(RepeatedStructure {
                    pattern: (None, predicate.clone(), None),
                    examples,
                    count: triples.len() as u32,
                    candidate_class: Some(format!("{}Class", predicate)),
                });
            }
        }
    }

    /// Record a guard violation
    fn record_guard_violation(&self, patterns: &mut DetectedPatterns, guard_name: &str, subject: &str) {
        if let Some(violation) = patterns.guard_violations.iter_mut()
            .find(|v| v.guard_name == guard_name) {
            violation.near_miss_count += 1;
            if !violation.affected_subjects.contains(&subject.to_string()) {
                violation.affected_subjects.push(subject.to_string());
            }
        } else {
            patterns.guard_violations.push(GuardViolation {
                guard_name: guard_name.to_string(),
                near_miss_count: 1,
                affected_subjects: vec![subject.to_string()],
            });
        }
    }

    /// Record performance regression
    fn record_perf_regression(&self, patterns: &mut DetectedPatterns, receipt: &Receipt) {
        const EXPECTED_TICKS: u32 = 8; // Chatman Constant

        let regression_factor = receipt.metrics.total_ticks as f64 / EXPECTED_TICKS as f64;

        warn!(
            receipt_id = %receipt.id,
            observed = receipt.metrics.total_ticks,
            expected = EXPECTED_TICKS,
            factor = regression_factor,
            "Performance regression detected"
        );

        patterns.performance_regressions.push(PerfRegression {
            operator: receipt.id.clone(),
            observed_latency_ticks: receipt.metrics.total_ticks,
            expected_latency_ticks: EXPECTED_TICKS,
            regression_factor,
        });
    }

    /// Get current detected patterns (read-only snapshot)
    pub fn detected_patterns(&self) -> DetectedPatterns {
        self.patterns.read().clone()
    }

    /// Reset patterns (after committing ΔΣ that addresses them)
    pub fn acknowledge_patterns(&mut self) {
        let span = span!(Level::INFO, "acknowledge_patterns");
        let _enter = span.enter();

        info!("Acknowledging patterns, resetting detection state");

        let mut patterns = self.patterns.write();
        *patterns = DetectedPatterns::default();
    }

    /// Get observation window size
    pub fn window_size(&self) -> usize {
        self.window_size
    }

    /// Get current window occupancy
    pub fn window_occupancy(&self) -> usize {
        self.observation_window.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_miner_creation() {
        let miner = PatternMiner::new(100);
        assert_eq!(miner.window_size(), 100);
        assert_eq!(miner.window_occupancy(), 0);
    }

    #[test]
    fn test_scan_receipt_updates_window() {
        let mut miner = PatternMiner::new(10);

        let receipt = Receipt {
            id: "r1".to_string(),
            timestamp: 1000,
            operations: vec![],
            metrics: PerformanceMetrics {
                total_ticks: 5,
                ops_per_sec: 100.0,
            },
        };

        miner.scan_receipt(receipt);
        assert_eq!(miner.window_occupancy(), 1);
    }

    #[test]
    fn test_window_size_limit() {
        let mut miner = PatternMiner::new(3);

        for i in 0..5 {
            let receipt = Receipt {
                id: format!("r{}", i),
                timestamp: i as u64 * 1000,
                operations: vec![],
                metrics: PerformanceMetrics {
                    total_ticks: 5,
                    ops_per_sec: 100.0,
                },
            };
            miner.scan_receipt(receipt);
        }

        assert_eq!(miner.window_occupancy(), 3);
    }
}

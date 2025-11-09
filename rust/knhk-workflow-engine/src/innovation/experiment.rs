//! Innovation experiment runner
//!
//! Provides unified experiment execution framework for:
//! - Deterministic execution experiments
//! - Formal verification experiments
//! - Hardware acceleration experiments
//! - Zero-copy optimization experiments

use crate::case::{Case, CaseId};
use crate::error::{WorkflowError, WorkflowResult};
use crate::innovation::{
    DeterministicContext, DeterministicExecutor, FormalVerifier, HardwareAccelerator,
    VerificationResult, ZeroCopyTriple, ZeroCopyTripleBatch,
};
use crate::parser::WorkflowSpec;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Experiment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    /// Experiment type
    pub experiment_type: ExperimentType,
    /// Experiment name
    pub name: String,
    /// Success status
    pub success: bool,
    /// Execution time (nanoseconds)
    pub execution_time_ns: u64,
    /// Results data (JSON)
    pub results: serde_json::Value,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Metadata
    pub metadata: serde_json::Value,
}

/// Experiment type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperimentType {
    /// Deterministic execution experiment
    Deterministic,
    /// Formal verification experiment
    FormalVerification,
    /// Hardware acceleration experiment
    HardwareAcceleration,
    /// Zero-copy optimization experiment
    ZeroCopy,
    /// Integrated experiment (all types)
    Integrated,
}

/// Experiment runner
pub struct ExperimentRunner {
    /// Deterministic executor
    deterministic_executor: DeterministicExecutor,
    /// Hardware accelerator
    hardware_accelerator: HardwareAccelerator,
    /// Experiment results
    results: Vec<ExperimentResult>,
}

impl ExperimentRunner {
    /// Create new experiment runner
    pub fn new(seed: u64) -> Self {
        Self {
            deterministic_executor: DeterministicExecutor::new(seed),
            hardware_accelerator: HardwareAccelerator::new(),
            results: Vec::new(),
        }
    }

    /// Run deterministic execution experiment
    pub async fn run_deterministic_experiment(
        &mut self,
        name: String,
        case: &Case,
    ) -> WorkflowResult<ExperimentResult> {
        let start = Instant::now();

        let result = self
            .deterministic_executor
            .execute_deterministic(case)
            .await;

        let execution_time_ns = start.elapsed().as_nanos() as u64;

        match result {
            Ok(context) => {
                let results = serde_json::json!({
                    "case_id": context.case_id.to_string(),
                    "input_hash": context.input_hash,
                    "seed": context.seed,
                    "steps": context.trace.len(),
                    "total_ticks": context.trace.iter().map(|s| s.ticks as u64).sum::<u64>(),
                });

                let experiment_result = ExperimentResult {
                    experiment_type: ExperimentType::Deterministic,
                    name,
                    success: true,
                    execution_time_ns,
                    results,
                    error: None,
                    metadata: serde_json::json!({
                        "trace_length": context.trace.len(),
                        "delta_log_length": self.deterministic_executor.get_delta_log(&case.id).await.len(),
                    }),
                };

                self.results.push(experiment_result.clone());
                Ok(experiment_result)
            }
            Err(e) => {
                let experiment_result = ExperimentResult {
                    experiment_type: ExperimentType::Deterministic,
                    name,
                    success: false,
                    execution_time_ns,
                    results: serde_json::json!({}),
                    error: Some(e.to_string()),
                    metadata: serde_json::json!({}),
                };

                self.results.push(experiment_result.clone());
                Ok(experiment_result)
            }
        }
    }

    /// Run formal verification experiment
    pub fn run_formal_verification_experiment(
        &mut self,
        name: String,
        spec: &WorkflowSpec,
    ) -> WorkflowResult<ExperimentResult> {
        let start = Instant::now();

        let result = FormalVerifier::verify_workflow(spec);

        let execution_time_ns = start.elapsed().as_nanos() as u64;

        match result {
            Ok(verification_result) => {
                let results = serde_json::json!({
                    "passed": verification_result.passed,
                    "properties_verified": verification_result.properties.len(),
                    "violations": verification_result.violations.len(),
                    "properties": verification_result.properties.iter().map(|p| serde_json::json!({
                        "name": p.name,
                        "description": p.description,
                        "verified": p.verified,
                    })).collect::<Vec<_>>(),
                });

                let experiment_result = ExperimentResult {
                    experiment_type: ExperimentType::FormalVerification,
                    name,
                    success: verification_result.passed,
                    execution_time_ns,
                    results,
                    error: if verification_result.passed {
                        None
                    } else {
                        Some(format!(
                            "{} violations found",
                            verification_result.violations.len()
                        ))
                    },
                    metadata: serde_json::json!({
                        "violations": verification_result.violations.iter().map(|v| serde_json::json!({
                            "property": v.property,
                            "description": v.description,
                            "location": v.location,
                        })).collect::<Vec<_>>(),
                    }),
                };

                self.results.push(experiment_result.clone());
                Ok(experiment_result)
            }
            Err(e) => {
                let experiment_result = ExperimentResult {
                    experiment_type: ExperimentType::FormalVerification,
                    name,
                    success: false,
                    execution_time_ns,
                    results: serde_json::json!({}),
                    error: Some(e.to_string()),
                    metadata: serde_json::json!({}),
                };

                self.results.push(experiment_result.clone());
                Ok(experiment_result)
            }
        }
    }

    /// Run hardware acceleration experiment
    pub fn run_hardware_acceleration_experiment(
        &mut self,
        name: String,
        data: &[u8],
    ) -> WorkflowResult<ExperimentResult> {
        let start = Instant::now();

        let hash = self.hardware_accelerator.accelerated_hash(data);
        let pattern_match = self
            .hardware_accelerator
            .accelerated_pattern_match(b"test", data);

        let execution_time_ns = start.elapsed().as_nanos() as u64;

        let results = serde_json::json!({
            "hash": hash,
            "pattern_match": pattern_match,
            "acceleration_type": format!("{:?}", self.hardware_accelerator.acceleration()),
            "simd_available": self.hardware_accelerator.is_simd_available(),
            "gpu_available": self.hardware_accelerator.is_gpu_available(),
        });

        let experiment_result = ExperimentResult {
            experiment_type: ExperimentType::HardwareAcceleration,
            name,
            success: true,
            execution_time_ns,
            results,
            error: None,
            metadata: serde_json::json!({
                "data_size": data.len(),
            }),
        };

        self.results.push(experiment_result.clone());
        Ok(experiment_result)
    }

    /// Run zero-copy optimization experiment
    pub fn run_zero_copy_experiment(
        &mut self,
        name: String,
        triples: Vec<(&str, &str, &str)>,
    ) -> WorkflowResult<ExperimentResult> {
        let start = Instant::now();

        let mut batch = ZeroCopyTripleBatch::new(1000);
        let mut borrowed_count = 0;
        let mut owned_count = 0;

        let total_triples = triples.len();
        for (subject, predicate, object) in triples {
            let triple = ZeroCopyTriple::borrowed(subject, predicate, object, None);
            if triple.is_fully_borrowed() {
                borrowed_count += 1;
            } else {
                owned_count += 1;
            }
            batch.add(triple).map_err(|e| {
                WorkflowError::Internal(format!("Failed to add triple to batch: {}", e))
            })?;
        }

        let execution_time_ns = start.elapsed().as_nanos() as u64;

        let results = serde_json::json!({
            "batch_size": batch.len(),
            "borrowed_count": borrowed_count,
            "owned_count": owned_count,
            "fully_zero_copy": borrowed_count == batch.len(),
        });

        let experiment_result = ExperimentResult {
            experiment_type: ExperimentType::ZeroCopy,
            name,
            success: true,
            execution_time_ns,
            results,
            error: None,
            metadata: serde_json::json!({
                "total_triples": total_triples,
            }),
        };

        self.results.push(experiment_result.clone());
        Ok(experiment_result)
    }

    /// Run integrated experiment (all types)
    pub async fn run_integrated_experiment(
        &mut self,
        name: String,
        spec: &WorkflowSpec,
        case: &Case,
        data: &[u8],
    ) -> WorkflowResult<ExperimentResult> {
        let start = Instant::now();

        // Run all experiments
        let deterministic_result = self
            .run_deterministic_experiment(format!("{}_deterministic", name), case)
            .await?;

        let formal_result =
            self.run_formal_verification_experiment(format!("{}_formal", name), spec)?;

        let hardware_result =
            self.run_hardware_acceleration_experiment(format!("{}_hardware", name), data)?;

        let zero_copy_result = self.run_zero_copy_experiment(
            format!("{}_zero_copy", name),
            vec![
                (
                    "http://example.org/subject1",
                    "http://example.org/predicate1",
                    "http://example.org/object1",
                ),
                (
                    "http://example.org/subject2",
                    "http://example.org/predicate2",
                    "http://example.org/object2",
                ),
            ],
        )?;

        let execution_time_ns = start.elapsed().as_nanos() as u64;

        let results = serde_json::json!({
            "deterministic": deterministic_result.results,
            "formal_verification": formal_result.results,
            "hardware_acceleration": hardware_result.results,
            "zero_copy": zero_copy_result.results,
        });

        let experiment_result = ExperimentResult {
            experiment_type: ExperimentType::Integrated,
            name,
            success: deterministic_result.success
                && formal_result.success
                && hardware_result.success
                && zero_copy_result.success,
            execution_time_ns,
            results,
            error: if deterministic_result.success
                && formal_result.success
                && hardware_result.success
                && zero_copy_result.success
            {
                None
            } else {
                Some(format!(
                    "Some experiments failed: deterministic={}, formal={}, hardware={}, zero_copy={}",
                    deterministic_result.success,
                    formal_result.success,
                    hardware_result.success,
                    zero_copy_result.success
                ))
            },
            metadata: serde_json::json!({
                "sub_experiments": vec![
                    deterministic_result.name,
                    formal_result.name,
                    hardware_result.name,
                    zero_copy_result.name,
                ],
            }),
        };

        self.results.push(experiment_result.clone());
        Ok(experiment_result)
    }

    /// Get all experiment results
    pub fn get_results(&self) -> &[ExperimentResult] {
        &self.results
    }

    /// Get experiment summary
    pub fn get_summary(&self) -> serde_json::Value {
        let total = self.results.len();
        let successful = self.results.iter().filter(|r| r.success).count();
        let failed = total - successful;

        let by_type: std::collections::HashMap<String, usize> = self
            .results
            .iter()
            .map(|r| (format!("{:?}", r.experiment_type), 1))
            .fold(std::collections::HashMap::new(), |mut acc, (k, v)| {
                *acc.entry(k).or_insert(0) += v;
                acc
            });

        serde_json::json!({
            "total_experiments": total,
            "successful": successful,
            "failed": failed,
            "success_rate": if total > 0 { successful as f64 / total as f64 } else { 0.0 },
            "by_type": by_type,
            "total_execution_time_ns": self.results.iter().map(|r| r.execution_time_ns).sum::<u64>(),
        })
    }
}

//! Load Testing Validator
//!
//! Performs stress testing with 100+ workflow cases to validate:
//! - Performance under load
//! - Concurrency correctness
//! - Resource management
//! - Throughput and latency metrics

use std::marker::PhantomData;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::task::JoinSet;

use crate::api::models::requests::CreateCaseRequest;
use crate::api::service::CaseService;
use crate::error::{WorkflowError, WorkflowResult};
use crate::validation::phases::core::{Phase, PhaseContext, PhaseMetadata, PhaseResult, PhaseStatus};

/// Load testing data
#[derive(Debug, Clone)]
pub struct LoadTestingData {
    /// Number of cases created
    pub cases_created: usize,
    /// Number of successful case creations
    pub successful_cases: usize,
    /// Number of failed case creations
    pub failed_cases: usize,
    /// Average latency per case (ms)
    pub avg_latency_ms: f64,
    /// Min latency (ms)
    pub min_latency_ms: f64,
    /// Max latency (ms)
    pub max_latency_ms: f64,
    /// P50 latency (median)
    pub p50_latency_ms: f64,
    /// P95 latency
    pub p95_latency_ms: f64,
    /// P99 latency
    pub p99_latency_ms: f64,
    /// Total duration (ms)
    pub total_duration_ms: f64,
    /// Throughput (cases/second)
    pub throughput: f64,
}

/// Load testing phase
pub struct LoadTestingPhase<M = ()> {
    /// Number of test cases to create
    num_cases: usize,
    /// Maximum acceptable average latency (ms)
    max_avg_latency_ms: f64,
    /// Maximum acceptable failure rate
    max_failure_rate: f64,
    _phantom: PhantomData<M>,
}

impl<M> LoadTestingPhase<M> {
    /// Create a new load testing phase
    pub fn new() -> Self {
        Self {
            num_cases: 100,
            max_avg_latency_ms: 100.0,
            max_failure_rate: 0.05, // 5%
            _phantom: PhantomData,
        }
    }

    /// Set number of test cases
    pub fn with_num_cases(mut self, num_cases: usize) -> Self {
        self.num_cases = num_cases.max(1);
        self
    }

    /// Set maximum acceptable average latency
    pub fn with_max_avg_latency_ms(mut self, max_ms: f64) -> Self {
        self.max_avg_latency_ms = max_ms;
        self
    }

    /// Set maximum acceptable failure rate
    pub fn with_max_failure_rate(mut self, rate: f64) -> Self {
        self.max_failure_rate = rate.clamp(0.0, 1.0);
        self
    }
}

impl<M> Default for LoadTestingPhase<M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<M: Send + Sync> Phase<LoadTestingData, M> for LoadTestingPhase<M> {
    fn metadata() -> PhaseMetadata {
        PhaseMetadata {
            name: "load_testing",
            description: "Stress testing with 100+ workflow cases",
            version: "1.0.0",
            dependencies: &[],
            parallel: true,
        }
    }

    async fn execute(
        &self,
        ctx: PhaseContext,
    ) -> WorkflowResult<PhaseResult<LoadTestingData>> {
        let start = Instant::now();

        // Run load test
        let load_data = run_load_test(&ctx, self.num_cases).await?;

        // Calculate failure rate
        let failure_rate = load_data.failed_cases as f64 / load_data.cases_created as f64;

        // Determine status based on thresholds
        let status = if load_data.avg_latency_ms <= self.max_avg_latency_ms
            && failure_rate <= self.max_failure_rate
        {
            PhaseStatus::Pass
        } else if load_data.avg_latency_ms <= self.max_avg_latency_ms * 1.5
            && failure_rate <= self.max_failure_rate * 2.0
        {
            PhaseStatus::Warning
        } else {
            PhaseStatus::Fail
        };

        let passed = if load_data.avg_latency_ms <= self.max_avg_latency_ms {
            1
        } else {
            0
        } + if failure_rate <= self.max_failure_rate {
            1
        } else {
            0
        };
        let failed = 2 - passed;

        let mut result = PhaseResult::new("load_testing", status, load_data.clone())
            .with_duration(start.elapsed())
            .with_counts(load_data.successful_cases, load_data.failed_cases, 0);

        // Add detailed metrics
        result.add_metric("cases_created", load_data.cases_created as f64);
        result.add_metric("successful_cases", load_data.successful_cases as f64);
        result.add_metric("failed_cases", load_data.failed_cases as f64);
        result.add_metric("avg_latency_ms", load_data.avg_latency_ms);
        result.add_metric("min_latency_ms", load_data.min_latency_ms);
        result.add_metric("max_latency_ms", load_data.max_latency_ms);
        result.add_metric("p50_latency_ms", load_data.p50_latency_ms);
        result.add_metric("p95_latency_ms", load_data.p95_latency_ms);
        result.add_metric("p99_latency_ms", load_data.p99_latency_ms);
        result.add_metric("throughput", load_data.throughput);
        result.add_metric("failure_rate", failure_rate);

        // Add messages
        result.add_message(format!(
            "Load test: {} cases, {:.2}ms avg latency, {:.2} cases/sec",
            load_data.cases_created, load_data.avg_latency_ms, load_data.throughput
        ));
        result.add_message(format!(
            "Success rate: {:.2}% ({}/{})",
            (1.0 - failure_rate) * 100.0,
            load_data.successful_cases,
            load_data.cases_created
        ));
        result.add_message(format!(
            "Latency: min={:.2}ms, p50={:.2}ms, p95={:.2}ms, p99={:.2}ms, max={:.2}ms",
            load_data.min_latency_ms,
            load_data.p50_latency_ms,
            load_data.p95_latency_ms,
            load_data.p99_latency_ms,
            load_data.max_latency_ms
        ));

        if load_data.avg_latency_ms > self.max_avg_latency_ms {
            result.add_message(format!(
                "WARNING: Avg latency ({:.2}ms) exceeds threshold ({:.2}ms)",
                load_data.avg_latency_ms, self.max_avg_latency_ms
            ));
        }
        if failure_rate > self.max_failure_rate {
            result.add_message(format!(
                "WARNING: Failure rate ({:.2}%) exceeds threshold ({:.2}%)",
                failure_rate * 100.0,
                self.max_failure_rate * 100.0
            ));
        }

        Ok(result)
    }
}

/// Run load test with specified number of cases
async fn run_load_test(ctx: &PhaseContext, num_cases: usize) -> WorkflowResult<LoadTestingData> {
    let start = Instant::now();

    let service = CaseService::new(ctx.engine.clone());
    let mut latencies: Vec<f64> = Vec::new();
    let mut successful = 0;
    let mut failed = 0;

    // Create cases concurrently (batched for reasonable concurrency)
    let batch_size = 10;
    let num_batches = (num_cases + batch_size - 1) / batch_size;

    for batch in 0..num_batches {
        let batch_start = batch * batch_size;
        let batch_end = ((batch + 1) * batch_size).min(num_cases);

        let mut join_set = JoinSet::new();

        for i in batch_start..batch_end {
            let service = service.clone();
            let spec_id = ctx.spec_id;

            join_set.spawn(async move {
                let case_start = Instant::now();

                let request = CreateCaseRequest {
                    spec_id,
                    data: serde_json::json!({
                        "test_case_id": i,
                        "load_test": true,
                    }),
                };

                let result = service.create_case(request).await;
                let latency = case_start.elapsed().as_secs_f64() * 1000.0; // Convert to ms

                (result, latency)
            });
        }

        // Collect batch results
        while let Some(result) = join_set.join_next().await {
            match result {
                Ok((Ok(_case_response), latency)) => {
                    successful += 1;
                    latencies.push(latency);
                }
                Ok((Err(_), latency)) => {
                    failed += 1;
                    latencies.push(latency); // Still track latency for failed cases
                }
                Err(_) => {
                    failed += 1;
                    // Task panic - don't add latency
                }
            }
        }
    }

    let total_duration = start.elapsed().as_secs_f64() * 1000.0; // ms

    // Calculate statistics
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let avg_latency_ms = if !latencies.is_empty() {
        latencies.iter().sum::<f64>() / latencies.len() as f64
    } else {
        0.0
    };

    let min_latency_ms = latencies.first().copied().unwrap_or(0.0);
    let max_latency_ms = latencies.last().copied().unwrap_or(0.0);

    let p50_latency_ms = percentile(&latencies, 0.50);
    let p95_latency_ms = percentile(&latencies, 0.95);
    let p99_latency_ms = percentile(&latencies, 0.99);

    let throughput = if total_duration > 0.0 {
        (num_cases as f64) / (total_duration / 1000.0) // cases per second
    } else {
        0.0
    };

    Ok(LoadTestingData {
        cases_created: num_cases,
        successful_cases: successful,
        failed_cases: failed,
        avg_latency_ms,
        min_latency_ms,
        max_latency_ms,
        p50_latency_ms,
        p95_latency_ms,
        p99_latency_ms,
        total_duration_ms: total_duration,
        throughput,
    })
}

/// Calculate percentile from sorted latencies
fn percentile(sorted_latencies: &[f64], p: f64) -> f64 {
    if sorted_latencies.is_empty() {
        return 0.0;
    }

    let index = (p * (sorted_latencies.len() - 1) as f64).round() as usize;
    sorted_latencies[index.min(sorted_latencies.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile_calculation() {
        let latencies = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

        assert_eq!(percentile(&latencies, 0.50), 50.0);
        assert_eq!(percentile(&latencies, 0.95), 100.0);
        assert_eq!(percentile(&latencies, 0.00), 10.0);
        assert_eq!(percentile(&latencies, 1.00), 100.0);
    }

    #[test]
    fn test_percentile_empty() {
        let latencies: Vec<f64> = vec![];
        assert_eq!(percentile(&latencies, 0.50), 0.0);
    }
}

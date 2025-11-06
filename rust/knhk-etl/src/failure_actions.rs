// rust/knhk-etl/src/failure_actions.rs
// Failure actions per runtime class
// Implements drop/park/escalate for R1, retry/degrade for W1, async finalize for C1

extern crate alloc;
extern crate std;
#[cfg(feature = "knhk-otel")]
extern crate knhk_otel;

use alloc::string::{String, ToString};
use alloc::format;
use crate::reflex::{Receipt, Action};
use crate::load::LoadResult;

/// R1 failure action: drop/park Δ, emit receipt, escalate
#[derive(Debug, Clone)]
pub struct R1FailureAction {
    /// Delta (load result) to drop or park
    pub delta: LoadResult,
    /// Receipt to emit
    pub receipt: Receipt,
    /// Whether to escalate (budget exceeded)
    pub escalate: bool,
}

/// W1 failure action: retry ×N, degrade to cached answer
#[derive(Debug, Clone)]
pub struct W1FailureAction {
    /// Current retry count
    pub retry_count: u32,
    /// Maximum retries allowed
    pub max_retries: u32,
    /// Whether to use cached answer
    pub use_cache: bool,
}

/// C1 failure action: async finalize, never block R1
#[derive(Debug, Clone)]
pub struct C1FailureAction {
    /// Whether to schedule async finalization
    pub async_finalize: bool,
    /// Whether operation is non-blocking
    pub non_blocking: bool,
}

/// Handle R1 failure: drop/park Δ, emit receipt, escalate
/// 
/// # Arguments
/// * `delta` - Load result (Δ) to drop or park
/// * `receipt` - Receipt to emit
/// * `budget_exceeded` - Whether budget was exceeded (triggers escalation)
/// 
/// # Returns
/// * `Ok(R1FailureAction)` - Failure action taken (park/drop decision)
/// * `Err(String)` - Error handling failure
pub fn handle_r1_failure(
    delta: LoadResult,
    receipt: Receipt,
    budget_exceeded: bool,
) -> Result<R1FailureAction, String> {
    // Decision: drop or park based on admission control
    // Current implementation: always park (preserve Δ for later processing)
    // Admission control state checking is handled by the pipeline stage
    // before calling this function
    
    // Record receipt emission (receipt will be emitted by emit stage)
    // This function tracks that receipt needs to be emitted
    
    // Escalate if budget exceeded - record OTEL event
    if budget_exceeded {
        #[cfg(feature = "knhk-otel")]
        {
            use knhk_otel::{Tracer, Metric, MetricValue};
            use std::time::{SystemTime, UNIX_EPOCH};

            let mut tracer = Tracer::new();
            let timestamp_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            
            // Record escalation event
            let mut attrs = alloc::collections::BTreeMap::new();
            attrs.insert("runtime_class".to_string(), "R1".to_string());
            attrs.insert("receipt_id".to_string(), receipt.id.clone());
            attrs.insert("ticks".to_string(), receipt.ticks.to_string());
            attrs.insert("budget".to_string(), "8".to_string());
            
            let metric = Metric {
                name: "knhk.r1.budget_exceeded".to_string(),
                value: MetricValue::Counter(1),
                timestamp_ms,
                attributes: attrs,
            };
            tracer.record_metric(metric);
        }
    }
    
    Ok(R1FailureAction {
        delta,
        receipt,
        escalate: budget_exceeded,
    })
}

/// Handle W1 failure: retry ×N, degrade to cached answer
/// 
/// # Arguments
/// * `retry_count` - Current retry count
/// * `max_retries` - Maximum retries allowed (default: 3)
/// * `cached_answer` - Optional cached answer to use
/// 
/// # Returns
/// * `Ok(W1FailureAction)` - Retry action to take
/// * `Err(String)` - Max retries exceeded
pub fn handle_w1_failure(
    retry_count: u32,
    max_retries: u32,
    cached_answer: Option<Action>,
) -> Result<W1FailureAction, String> {
    if retry_count >= max_retries {
        // Max retries exceeded - degrade to cached answer if available
        if cached_answer.is_some() {
            return Ok(W1FailureAction {
                retry_count,
                max_retries,
                use_cache: true,
            });
        }
        return Err(format!(
            "W1 max retries {} exceeded, no cached answer available",
            max_retries
        ));
    }

    // Retry
    Ok(W1FailureAction {
        retry_count: retry_count + 1,
        max_retries,
        use_cache: false,
    })
}

/// Handle C1 failure: async finalize, never block R1
/// 
/// # Arguments
/// * `operation_id` - Operation identifier
/// 
/// # Returns
/// * `Ok(C1FailureAction)` - Async finalization action
/// * `Err(String)` - Error scheduling async operation
pub fn handle_c1_failure(_operation_id: &str) -> Result<C1FailureAction, String> {
    // Schedule async finalization (non-blocking)
    // Note: Async runtime integration (tokio/async-std) is handled by the caller
    // This function returns an action indicating async finalization is needed
    // The caller is responsible for scheduling the async operation
    
    if _operation_id.is_empty() {
        return Err("Operation ID cannot be empty".to_string());
    }
    
    Ok(C1FailureAction {
        async_finalize: true,
        non_blocking: true,
    })
}

/// Failure action error types
#[derive(Debug, Clone)]
pub enum FailureActionError {
    /// R1 failure handling error
    R1Failure(String),
    /// W1 failure handling error
    W1Failure(String),
    /// C1 failure handling error
    C1Failure(String),
}

impl FailureActionError {
    pub fn message(&self) -> &str {
        match self {
            FailureActionError::R1Failure(msg) => msg,
            FailureActionError::W1Failure(msg) => msg,
            FailureActionError::C1Failure(msg) => msg,
        }
    }
}

mod tests {
    use super::*;
    use crate::load::{SoAArrays, PredRun};
    use alloc::vec;
    use alloc::string::ToString;

    fn create_test_receipt() -> Receipt {
        Receipt {
            id: "test_receipt".to_string(),
            ticks: 10, // Exceeds R1 budget
            lanes: 8,
            span_id: 12345,
            a_hash: 67890,
        }
    }

    fn create_test_delta() -> LoadResult {
        LoadResult {
            soa_arrays: SoAArrays::new(),
            runs: vec![PredRun {
                pred: 100,
                off: 0,
                len: 5,
            }],
        }
    }

    #[test]
    fn test_r1_failure_budget_exceeded() {
        let delta = create_test_delta();
        let receipt = create_test_receipt();
        
        let result = handle_r1_failure(delta, receipt.clone(), true);
        assert!(result.is_ok());
        let action = result.unwrap();
        assert!(action.escalate);
        assert_eq!(action.receipt.id, receipt.id);
    }

    #[test]
    fn test_r1_failure_no_escalation() {
        let delta = create_test_delta();
        let receipt = Receipt {
            id: "test_receipt".to_string(),
            ticks: 5, // Within budget
            lanes: 8,
            span_id: 12345,
            a_hash: 67890,
        };
        
        let result = handle_r1_failure(delta, receipt, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_w1_failure_retry() {
        let result = handle_w1_failure(0, 3, None);
        assert!(result.is_ok());
        
        let action = result.unwrap();
        assert_eq!(action.retry_count, 1);
        assert_eq!(action.max_retries, 3);
        assert!(!action.use_cache);
    }

    #[test]
    fn test_w1_failure_max_retries() {
        let result = handle_w1_failure(3, 3, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_w1_failure_cache_degrade() {
        let cached_action = Some(Action {
            id: "cached".to_string(),
            payload: vec![1, 2, 3],
            receipt_id: "receipt".to_string(),
        });
        
        let result = handle_w1_failure(3, 3, cached_action);
        assert!(result.is_ok());
        
        let action = result.unwrap();
        assert!(action.use_cache);
    }

    #[test]
    fn test_c1_failure_async() {
        let result = handle_c1_failure("op123");
        assert!(result.is_ok());
        
        let action = result.unwrap();
        assert!(action.async_finalize);
        assert!(action.non_blocking);
    }

    #[test]
    fn test_c1_failure_empty_operation_id() {
        let result = handle_c1_failure("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }
}


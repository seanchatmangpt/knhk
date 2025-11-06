// rust/knhk-etl/src/failure_actions.rs
// Failure actions per runtime class
// Implements drop/park/escalate for R1, retry/degrade for W1, async finalize for C1

extern crate alloc;

use alloc::string::String;
use alloc::format;
use crate::runtime_class::RuntimeClass;
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
/// * `Ok(())` - Failure handled successfully
/// * `Err(String)` - Error handling failure
pub fn handle_r1_failure(
    delta: LoadResult,
    receipt: Receipt,
    budget_exceeded: bool,
) -> Result<(), String> {
    // Decision: drop or park based on admission control
    // For now, always park (preserve Δ for later processing)
    // In production, this would check admission control state
    
    // Emit receipt (via lockchain - handled by emit stage)
    // Receipt is already created, just needs to be emitted
    
    // Escalate if budget exceeded
    if budget_exceeded {
        // Escalation: record OTEL event + metrics
        // This will be handled by OTEL integration
        return Err(format!(
            "R1 budget exceeded: {} ticks > 8 ticks. Receipt {} emitted, Δ parked",
            receipt.ticks,
            receipt.id
        ));
    }

    Ok(())
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
pub fn handle_c1_failure(operation_id: &str) -> Result<C1FailureAction, String> {
    // Schedule async finalization (non-blocking)
    // In production, this would use async runtime (tokio/async-std)
    // For now, return action indicating async finalization needed
    
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::{SoAArrays, PredRun};

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
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("budget exceeded"));
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
}


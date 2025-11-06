// rust/knhk-etl/tests/failure_actions_test.rs
// Tests for failure actions per runtime class

use knhk_etl::failure_actions::{handle_r1_failure, handle_w1_failure, handle_c1_failure};
use knhk_etl::load::{LoadResult, SoAArrays, PredRun};
use knhk_etl::reflex::Receipt;

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
    assert!(result.unwrap_err().contains("max retries"));
}

#[test]
fn test_w1_failure_cache_degrade() {
    use knhk_etl::reflex::Action;
    
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

#[test]
fn test_w1_failure_multiple_retries() {
    let mut retry_count = 0;
    let max_retries = 3;
    
    // Simulate retry loop
    loop {
        let result = handle_w1_failure(retry_count, max_retries, None);
        match result {
            Ok(action) => {
                retry_count = action.retry_count;
                if retry_count >= max_retries {
                    break;
                }
            },
            Err(_) => {
                // Max retries exceeded
                break;
            },
        }
    }
    
    assert!(retry_count >= max_retries);
}


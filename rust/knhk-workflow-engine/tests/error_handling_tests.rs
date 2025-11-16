//! Comprehensive error handling tests
//!
//! Tests all error paths, recovery strategies, and error propagation.

use knhk_workflow_engine::error::*;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

mod error_types {
    use super::*;

    #[test]
    fn test_spec_not_found_error() {
        let error = WorkflowError::SpecNotFound {
            spec_id: "workflow-123".to_string(),
        };
        assert!(error.to_string().contains("workflow-123"));
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_case_not_found_error() {
        let error = WorkflowError::CaseNotFound {
            case_id: "case-456".to_string(),
        };
        assert!(error.user_message().contains("case-456"));
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_pattern_execution_failed() {
        let io_error = std::io::Error::new(std::io::ErrorKind::Other, "test error");
        let error = WorkflowError::PatternExecutionFailed {
            pattern_id: 42,
            source: Box::new(io_error),
        };
        assert!(error.to_string().contains("42"));
        assert!(!error.is_recoverable());
    }

    #[test]
    fn test_resource_allocation_failed() {
        let error = WorkflowError::ResourceAllocationFailed {
            resource_id: "resource-1".to_string(),
            reason: "max capacity reached".to_string(),
        };
        assert!(error.to_string().contains("resource-1"));
        assert!(error.to_string().contains("max capacity"));
    }

    #[test]
    fn test_deadlock_detected() {
        let cycles = vec![
            vec!["r1".to_string(), "r2".to_string(), "r1".to_string()],
            vec!["r3".to_string(), "r4".to_string(), "r3".to_string()],
        ];
        let error = WorkflowError::DeadlockDetected {
            cycles_count: 2,
            cycles: cycles.clone(),
        };
        assert_eq!(error.severity(), "critical");
        assert!(error.user_message().contains("2"));
    }

    #[test]
    fn test_timeout_error() {
        let error = WorkflowError::Timeout {
            resource_type: "database".to_string(),
            duration_ms: 5000,
        };
        assert!(error.is_recoverable());
        assert_eq!(error.severity(), "warning");
        assert!(error.user_message().contains("5000"));
    }

    #[test]
    fn test_recoverable_error() {
        let error = WorkflowError::Recoverable {
            message: "temporary failure".to_string(),
        };
        assert!(error.is_recoverable());
    }

    #[test]
    fn test_error_severity_levels() {
        let critical = WorkflowError::DeadlockDetected {
            cycles_count: 1,
            cycles: vec![],
        };
        assert_eq!(critical.severity(), "critical");

        let error = WorkflowError::PatternExecutionFailed {
            pattern_id: 1,
            source: Box::new(std::io::Error::new(std::io::ErrorKind::Other, "test")),
        };
        assert_eq!(error.severity(), "error");

        let warning = WorkflowError::Timeout {
            resource_type: "lock".to_string(),
            duration_ms: 1000,
        };
        assert_eq!(warning.severity(), "warning");
    }
}

mod error_context {
    use super::*;
    use anyhow::Context as AnyhowContext;

    #[test]
    fn test_error_context_static() {
        let result: WorkflowResult<()> = Err(WorkflowError::CaseNotFound {
            case_id: "test-123".to_string(),
        });

        let with_context = result.context("Failed to load case");
        assert!(with_context.is_err());
        let err = with_context.unwrap_err();
        assert!(err.to_string().contains("Failed to load case"));
        assert!(err.to_string().contains("test-123"));
    }

    #[test]
    fn test_error_context_dynamic() {
        let case_id = "case-789";
        let result: WorkflowResult<()> = Err(WorkflowError::Internal {
            message: "database error".to_string(),
        });

        let with_context = result.with_context(|| format!("Failed for case {}", case_id));
        assert!(with_context.is_err());
        let err = with_context.unwrap_err();
        assert!(err.to_string().contains("case-789"));
    }

    #[test]
    fn test_into_workflow_result() {
        let anyhow_result: anyhow::Result<i32> = Err(anyhow::anyhow!("anyhow error"));
        let workflow_result = anyhow_result.into_workflow_result();

        assert!(workflow_result.is_err());
        match workflow_result {
            Err(WorkflowError::Internal { message }) => {
                assert!(message.contains("anyhow error"));
            }
            _ => panic!("Expected Internal error"),
        }
    }
}

mod error_recovery {
    use super::*;

    #[tokio::test]
    async fn test_retry_strategy_timeout() {
        let error = WorkflowError::Timeout {
            resource_type: "lock".to_string(),
            duration_ms: 100,
        };

        match error.recovery_strategy() {
            RecoveryStrategy::Retry {
                max_attempts,
                backoff_ms,
                max_backoff_ms,
            } => {
                assert_eq!(max_attempts, 3);
                assert!(backoff_ms > 0);
                assert!(max_backoff_ms >= backoff_ms);
            }
            _ => panic!("Expected Retry strategy"),
        }
    }

    #[tokio::test]
    async fn test_retry_strategy_external_system() {
        let error = WorkflowError::ExternalSystem {
            system_name: "payment-gateway".to_string(),
            message: "connection failed".to_string(),
        };

        match error.recovery_strategy() {
            RecoveryStrategy::Retry { max_attempts, .. } => {
                assert!(max_attempts > 0);
            }
            _ => panic!("Expected Retry strategy"),
        }
    }

    #[tokio::test]
    async fn test_fail_fast_strategy() {
        let error = WorkflowError::Parse {
            message: "invalid JSON".to_string(),
        };

        match error.recovery_strategy() {
            RecoveryStrategy::FailFast => {}
            _ => panic!("Expected FailFast strategy"),
        }
    }

    #[tokio::test]
    async fn test_recover_from_error_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let error = WorkflowError::Timeout {
            resource_type: "test".to_string(),
            duration_ms: 100,
        };

        let result = recover_from_error(&error, move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                if count < 1 {
                    Err(WorkflowError::Timeout {
                        resource_type: "test".to_string(),
                        duration_ms: 100,
                    })
                } else {
                    Ok(42)
                }
            })
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_recover_from_error_max_attempts() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let error = WorkflowError::Timeout {
            resource_type: "test".to_string(),
            duration_ms: 100,
        };

        let result = recover_from_error(&error, move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                counter.fetch_add(1, Ordering::SeqCst);
                Err(WorkflowError::Timeout {
                    resource_type: "test".to_string(),
                    duration_ms: 100,
                })
            })
        })
        .await;

        assert!(result.is_err());
        // Initial + 3 retries = 4
        assert_eq!(counter.load(Ordering::SeqCst), 4);
    }

    #[test]
    fn test_retry_sync_success() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_sync(
            move || {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(WorkflowError::Recoverable {
                        message: "temp".to_string(),
                    })
                } else {
                    Ok(100)
                }
            },
            5,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 100);
    }
}

mod error_backtrace {
    use super::*;

    #[test]
    fn test_error_chain_creation() {
        let error = WorkflowError::CaseNotFound {
            case_id: "test-123".to_string(),
        };
        let chain = ErrorChain::new(error);

        assert_eq!(chain.depth(), 1);
        assert!(chain.root_error().is_some());
    }

    #[test]
    fn test_error_chain_context() {
        let error = WorkflowError::Internal {
            message: "db error".to_string(),
        };
        let chain = ErrorChain::new(error)
            .add_context("Failed to save state".to_string())
            .add_context("Workflow execution failed".to_string());

        assert_eq!(chain.depth(), 3);
        let contexts = chain.contexts();
        assert_eq!(contexts.len(), 3);
        assert!(contexts[1].contains("save state"));
        assert!(contexts[2].contains("Workflow execution"));
    }

    #[test]
    fn test_error_chain_display() {
        let error = WorkflowError::Timeout {
            resource_type: "mutex".to_string(),
            duration_ms: 5000,
        };
        let chain = ErrorChain::new(error).add_context("Resource allocation failed".to_string());

        let display = chain.display_chain();
        assert!(display.contains("0:"));
        assert!(display.contains("1:"));
        assert!(display.contains("Timeout"));
        assert!(display.contains("Resource allocation"));
    }

    #[test]
    fn test_capture_error() {
        let error = WorkflowError::DeadlockDetected {
            cycles_count: 1,
            cycles: vec![vec!["r1".to_string(), "r2".to_string()]],
        };
        let chain = capture_error(error);

        assert!(chain.root_error().is_some());
        assert!(chain
            .root_error()
            .unwrap()
            .contains("Deadlock detected"));
    }
}

mod error_sources {
    use super::*;

    #[test]
    fn test_state_store_error() {
        let error = StateStoreError::KeyNotFound("my-key".to_string());
        assert!(error.to_string().contains("my-key"));
    }

    #[test]
    fn test_state_store_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let state_error: StateStoreError = io_error.into();
        assert!(matches!(state_error, StateStoreError::IoError(_)));
    }

    #[test]
    fn test_rdf_validation_error() {
        let error = RdfValidationError::InvalidTriple("bad triple".to_string());
        assert!(error.to_string().contains("bad triple"));
    }

    #[test]
    fn test_rdf_type_mismatch() {
        let error = RdfValidationError::TypeMismatch {
            expected: "String".to_string(),
            actual: "Integer".to_string(),
        };
        assert!(error.to_string().contains("String"));
        assert!(error.to_string().contains("Integer"));
    }

    #[test]
    fn test_connector_error() {
        let error = ConnectorError::RateLimitExceeded {
            retry_after_ms: 5000,
        };
        assert!(error.to_string().contains("5000"));
    }

    #[test]
    fn test_circuit_breaker_error() {
        let error = CircuitBreakerError::Open;
        assert_eq!(error.to_string(), "Circuit breaker is open");
    }
}

mod error_conversions {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let workflow_error: WorkflowError = io_error.into();
        match workflow_error {
            WorkflowError::StatePersistence { message } => {
                assert!(message.contains("file not found"));
            }
            _ => panic!("Expected StatePersistence error"),
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_error = serde_json::from_str::<i32>("invalid").unwrap_err();
        let workflow_error: WorkflowError = json_error.into();
        match workflow_error {
            WorkflowError::Parse { message } => {
                assert!(message.contains("JSON"));
            }
            _ => panic!("Expected Parse error"),
        }
    }

    #[test]
    fn test_state_store_error_conversion() {
        let state_error = StateStoreError::KeyNotFound("test".to_string());
        let workflow_error: WorkflowError = state_error.into();
        match workflow_error {
            WorkflowError::StateStoreError(_) => {}
            _ => panic!("Expected StateStoreError"),
        }
    }

    #[test]
    fn test_rdf_validation_error_conversion() {
        let rdf_error = RdfValidationError::InvalidTriple("bad".to_string());
        let workflow_error: WorkflowError = rdf_error.into();
        match workflow_error {
            WorkflowError::RdfValidationError(_) => {}
            _ => panic!("Expected RdfValidationError"),
        }
    }
}

mod integration_tests {
    use super::*;

    async fn simulate_case_execution(
        should_fail: bool,
        recoverable: bool,
    ) -> WorkflowResult<String> {
        if should_fail {
            if recoverable {
                Err(WorkflowError::Timeout {
                    resource_type: "database".to_string(),
                    duration_ms: 1000,
                })
            } else {
                Err(WorkflowError::CaseNotFound {
                    case_id: "test-case".to_string(),
                })
            }
        } else {
            Ok("success".to_string())
        }
    }

    #[tokio::test]
    async fn test_end_to_end_success() {
        let result = simulate_case_execution(false, false).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_end_to_end_non_recoverable_error() {
        let result = simulate_case_execution(true, false).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(!error.is_recoverable());
    }

    #[tokio::test]
    async fn test_end_to_end_recoverable_error() {
        let result = simulate_case_execution(true, true).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.is_recoverable());
    }

    #[tokio::test]
    async fn test_error_chain_propagation() {
        async fn level3() -> WorkflowResult<()> {
            Err(WorkflowError::Internal {
                message: "level 3 error".to_string(),
            })
        }

        async fn level2() -> WorkflowResult<()> {
            level3().await.map_err(|e| {
                let chain = ErrorChain::new(e).add_context("level 2 context".to_string());
                WorkflowError::Internal {
                    message: chain.display_chain(),
                }
            })
        }

        async fn level1() -> WorkflowResult<()> {
            level2().await.map_err(|e| {
                let chain = ErrorChain::new(e).add_context("level 1 context".to_string());
                WorkflowError::Internal {
                    message: chain.display_chain(),
                }
            })
        }

        let result = level1().await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            WorkflowError::Internal { message } => {
                assert!(message.contains("level 1"));
            }
            _ => panic!("Expected Internal error"),
        }
    }
}

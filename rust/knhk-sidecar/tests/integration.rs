// knhk-sidecar: Integration tests

#[cfg(test)]
mod tests {
    use knhk_sidecar::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_executor() {
        use knhk_sidecar::retry::{RetryExecutor, RetryConfig};
        
        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 10,
            max_delay_ms: 100,
            multiplier: 2.0,
        };
        
        let executor = RetryExecutor::new(config);
        
        // Test successful execution
        let result = executor.execute(|| async {
            Ok::<String, SidecarError>("success".to_string())
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_circuit_breaker() {
        use knhk_sidecar::circuit_breaker::SidecarCircuitBreaker;
        
        let cb = SidecarCircuitBreaker::new("test_endpoint".to_string(), 3, 1000);
        
        // Test circuit breaker state
        let state = cb.state().unwrap();
        assert_eq!(format!("{:?}", state), "Closed");
    }

    #[test]
    fn test_batch_collector() {
        use knhk_sidecar::batch::{BatchCollector, BatchConfig};
        
        let config = BatchConfig {
            batch_window_ms: 10,
            max_batch_size: 100,
        };
        
        let collector = BatchCollector::new(config);
        
        // Add request
        let _rx = collector.add_request("test_request".to_string());
        
        // Check pending count
        assert_eq!(collector.pending_count(), 1);
    }

    #[test]
    fn test_health_checker() {
        use knhk_sidecar::health::HealthChecker;
        
        let health = HealthChecker::new();
        
        // Test liveness
        let (alive, _) = health.check_liveness();
        assert!(alive);
        
        // Test readiness
        let (ready, _) = health.check_readiness();
        assert!(ready);
    }

    #[test]
    fn test_metrics_collector() {
        use knhk_sidecar::metrics::MetricsCollector;
        
        let collector = MetricsCollector::new(1000);
        
        // Record some metrics
        collector.record_request(true);
        collector.record_request(false);
        collector.record_latency(10);
        
        // Get snapshot
        let snapshot = collector.snapshot();
        assert_eq!(snapshot.requests.total, 2);
        assert_eq!(snapshot.requests.success, 1);
        assert_eq!(snapshot.requests.failure, 1);
    }

    #[test]
    fn test_tls_config() {
        use knhk_sidecar::tls::TlsConfig;
        
        let config = TlsConfig::new();
        
        // Test default (disabled)
        assert!(!config.enabled);
        
        // Test validation (should pass when disabled)
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_error_types() {
        use knhk_sidecar::error::{SidecarError, is_retryable_error, is_guard_violation};
        
        // Test retryable errors
        let network_err = SidecarError::NetworkError("test".to_string());
        assert!(is_retryable_error(&network_err));
        
        // Test non-retryable errors
        let validation_err = SidecarError::ValidationError("test".to_string());
        assert!(!is_retryable_error(&validation_err));
        assert!(is_guard_violation(&validation_err));
    }

    #[test]
    fn test_config_loading() {
        use knhk_sidecar::config::SidecarConfig;
        
        // Test default config
        let config = SidecarConfig::default();
        assert_eq!(config.server.bind_address, "127.0.0.1:50051");
        assert_eq!(config.client.warm_orchestrator_url, "http://localhost:50052");
    }
}


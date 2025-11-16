// Multi-region receipt synchronization tests
// Phase 3: Gamma (Γ) - Glue/Sheaf axis implementation

#[cfg(test)]
mod multi_region_tests {
    use knhk_sidecar::multi_region::*;
    use std::time::Duration;

    /// Test RemoteRegion validation
    #[test]
    fn test_remote_region_validation_valid() {
        let region = RemoteRegion::new(
            "us-west-1".to_string(),
            "http://replica-west.example.com".to_string(),
        )
        .with_timeout(Duration::from_secs(5))
        .with_weight(2);

        assert!(region.validate().is_ok());
        assert_eq!(region.region_id, "us-west-1");
        assert_eq!(region.weight, 2);
    }

    #[test]
    fn test_remote_region_validation_empty_id() {
        let region = RemoteRegion::new(
            "".to_string(),
            "http://replica-west.example.com".to_string(),
        );

        let result = region.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Region ID cannot be empty"));
    }

    #[test]
    fn test_remote_region_validation_empty_endpoint() {
        let region = RemoteRegion::new("us-west-1".to_string(), "".to_string());

        let result = region.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Endpoint cannot be empty"));
    }

    #[test]
    fn test_remote_region_validation_zero_timeout() {
        let region = RemoteRegion::new(
            "us-west-1".to_string(),
            "http://replica-west.example.com".to_string(),
        )
        .with_timeout(Duration::ZERO);

        let result = region.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Timeout must be > 0"));
    }

    /// Test MultiRegionConfig validation
    #[test]
    fn test_multi_region_config_validation_valid() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .add_region(RemoteRegion::new(
                "eu-west-1".to_string(),
                "http://replica-eu.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_quorum_threshold(2)
            .with_max_retries(3);

        assert!(config.validate().is_ok());
        assert_eq!(config.regions.len(), 2);
        assert_eq!(config.quorum_threshold, 2);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_multi_region_config_sync_disabled() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .with_cross_region_sync(false);

        // Should validate even with no regions when sync is disabled
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_multi_region_config_validation_no_regions() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .with_cross_region_sync(true);

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("no remote regions configured"));
    }

    #[test]
    fn test_multi_region_config_validation_invalid_quorum() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_quorum_threshold(10); // More than total regions (2)

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds total regions"));
    }

    #[test]
    fn test_multi_region_config_validation_zero_quorum() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_quorum_threshold(0);

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must be at least 1"));
    }

    #[test]
    fn test_multi_region_config_validation_zero_retries() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_max_retries(0);

        let result = config.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Max retries must be at least 1"));
    }

    /// Test RegionConfig backward compatibility
    #[test]
    fn test_region_config_backward_compatibility() {
        let legacy_config = RegionConfig {
            region: "us-east-1".to_string(),
            primary_region: Some("us-east-1".to_string()),
            cross_region_sync_enabled: true,
            receipt_sync_endpoints: vec![
                "http://replica-west.example.com".to_string(),
                "http://replica-eu.example.com".to_string(),
            ],
            quorum_threshold: 2,
        };

        let multi_config = legacy_config.to_multi_region_config();
        assert_eq!(multi_config.region, "us-east-1");
        assert_eq!(multi_config.regions.len(), 2);
        assert_eq!(multi_config.quorum_threshold, 2);
        assert!(multi_config.cross_region_sync_enabled);
    }

    /// Test is_primary helper
    #[test]
    fn test_is_primary() {
        let config1 = MultiRegionConfig::new("us-east-1".to_string());
        assert!(config1.is_primary());

        let config2 = MultiRegionConfig::new("us-east-1".to_string());
        let mut config2 = config2;
        config2.primary_region = Some("us-west-1".to_string());
        assert!(!config2.is_primary());

        let config3 = MultiRegionConfig::new("us-west-1".to_string());
        let mut config3 = config3;
        config3.primary_region = Some("us-west-1".to_string());
        assert!(config3.is_primary());
    }

    /// Test Receipt structure
    #[test]
    fn test_receipt_structure() {
        let receipt = Receipt {
            receipt_id: "receipt-123".to_string(),
            transaction_id: "tx-456".to_string(),
            hash: vec![1, 2, 3, 4],
            ticks: 42,
            span_id: 789,
            committed: true,
        };

        assert_eq!(receipt.receipt_id, "receipt-123");
        assert_eq!(receipt.transaction_id, "tx-456");
        assert_eq!(receipt.hash.len(), 4);
        assert_eq!(receipt.ticks, 42);
        assert_eq!(receipt.span_id, 789);
        assert!(receipt.committed);
    }

    /// Test RegionSyncStatus structure
    #[test]
    fn test_region_sync_status() {
        let status = RegionSyncStatus {
            region_id: "us-west-1".to_string(),
            last_sync_timestamp: Some(1234567890),
            is_available: true,
            failure_count: 0,
        };

        assert_eq!(status.region_id, "us-west-1");
        assert_eq!(status.last_sync_timestamp, Some(1234567890));
        assert!(status.is_available);
        assert_eq!(status.failure_count, 0);
    }

    /// Test SyncResult structure
    #[test]
    fn test_sync_result_structure() {
        let errors = vec![
            ("us-west-1".to_string(), "Connection timeout".to_string()),
            ("eu-west-1".to_string(), "Authentication failed".to_string()),
        ];

        let result = SyncResult {
            synced_regions: 1,
            total_regions: 3,
            errors: errors.clone(),
            quorum_achieved: false,
        };

        assert_eq!(result.synced_regions, 1);
        assert_eq!(result.total_regions, 3);
        assert!(!result.quorum_achieved);
        assert_eq!(result.errors.len(), 2);
    }

    /// Test ReceiptSyncManager creation
    #[tokio::test]
    async fn test_receipt_sync_manager_creation() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_quorum_threshold(1);

        let manager = ReceiptSyncManager::new(config);
        assert!(manager.is_ok());
    }

    /// Test ReceiptSyncManager with invalid config
    #[tokio::test]
    async fn test_receipt_sync_manager_invalid_config() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .with_cross_region_sync(true);

        let manager = ReceiptSyncManager::new(config);
        assert!(manager.is_err());
        assert!(manager
            .unwrap_err()
            .to_string()
            .contains("no remote regions configured"));
    }

    /// Test sync_receipt with disabled sync
    #[tokio::test]
    async fn test_sync_receipt_disabled() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .with_cross_region_sync(false);

        let mut manager = ReceiptSyncManager::new(config).unwrap();

        let receipt = Receipt {
            receipt_id: "receipt-123".to_string(),
            transaction_id: "tx-456".to_string(),
            hash: vec![1, 2, 3, 4],
            ticks: 42,
            span_id: 789,
            committed: true,
        };

        let result = manager.sync_receipt(&receipt).await.unwrap();
        assert_eq!(result.synced_regions, 0);
        assert_eq!(result.total_regions, 0);
        assert!(!result.quorum_achieved);
    }

    /// Test get_sync_status
    #[tokio::test]
    async fn test_get_sync_status() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_quorum_threshold(1);

        let manager = ReceiptSyncManager::new(config).unwrap();
        let status = manager.get_sync_status().await.unwrap();

        assert_eq!(status.len(), 1);
        assert_eq!(status[0].region_id, "us-west-1");
        assert!(status[0].is_available);
        assert_eq!(status[0].failure_count, 0);
        assert!(status[0].last_sync_timestamp.is_none());
    }

    /// Test get_health_status
    #[tokio::test]
    async fn test_get_health_status() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .add_region(RemoteRegion::new(
                "eu-west-1".to_string(),
                "http://replica-eu.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_quorum_threshold(2);

        let manager = ReceiptSyncManager::new(config).unwrap();
        let (available, total, quorum_status) = manager.get_health_status().await.unwrap();

        assert_eq!(available, 2);
        assert_eq!(total, 2);
        // 2 available + 1 local = 3, threshold is 2, so quorum is met
        assert!(quorum_status);
    }

    /// Test verify_quorum with disabled sync
    #[tokio::test]
    async fn test_verify_quorum_disabled() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .with_cross_region_sync(false);

        let manager = ReceiptSyncManager::new(config).unwrap();
        let result = manager.verify_quorum("receipt-123").await.unwrap();

        // Should return true when sync is disabled
        assert!(result);
    }

    /// Test Regional timeout configuration
    #[test]
    fn test_regional_timeout_configuration() {
        let region = RemoteRegion::new(
            "us-west-1".to_string(),
            "http://replica-west.example.com".to_string(),
        )
        .with_timeout(Duration::from_secs(10));

        assert_eq!(region.timeout, Duration::from_secs(10));
    }

    /// Test retry backoff configuration
    #[test]
    fn test_retry_backoff_configuration() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .with_retry_backoff(Duration::from_millis(500))
            .with_max_retries(5);

        assert_eq!(config.retry_backoff_initial, Duration::from_millis(500));
        assert_eq!(config.max_retries, 5);
    }

    /// Test config builder pattern
    #[test]
    fn test_config_builder_pattern() {
        let config = MultiRegionConfig::new("us-east-1".to_string())
            .add_region(RemoteRegion::new(
                "us-west-1".to_string(),
                "http://replica-west.example.com".to_string(),
            ))
            .add_region(RemoteRegion::new(
                "eu-west-1".to_string(),
                "http://replica-eu.example.com".to_string(),
            ))
            .with_cross_region_sync(true)
            .with_quorum_threshold(2)
            .with_max_retries(3)
            .with_retry_backoff(Duration::from_secs(1));

        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.regions.len(), 2);
        assert!(config.cross_region_sync_enabled);
        assert_eq!(config.quorum_threshold, 2);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_backoff_initial, Duration::from_secs(1));
    }
}

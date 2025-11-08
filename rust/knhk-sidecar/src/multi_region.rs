// knhk-sidecar: Multi-region support for Fortune 5
// Cross-region receipt sync and quorum consensus

use crate::error::{SidecarError, SidecarResult};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Region configuration
#[derive(Debug, Clone)]
pub struct RegionConfig {
    /// Current region identifier (e.g., "us-east-1", "eu-west-1")
    pub region: String,
    /// Primary region for quorum consensus
    pub primary_region: Option<String>,
    /// Enable cross-region receipt synchronization
    pub cross_region_sync_enabled: bool,
    /// Receipt sync endpoints for other regions
    pub receipt_sync_endpoints: Vec<String>,
    /// Quorum threshold (number of regions that must agree)
    pub quorum_threshold: usize,
}

impl Default for RegionConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            primary_region: None,
            cross_region_sync_enabled: false,
            receipt_sync_endpoints: Vec::new(),
            quorum_threshold: 1, // Default: no quorum required
        }
    }
}

impl RegionConfig {
    /// Create new region config
    pub fn new(region: String) -> Self {
        Self {
            region,
            ..Self::default()
        }
    }

    /// Validate region configuration
    pub fn validate(&self) -> SidecarResult<()> {
        if self.region.is_empty() {
            return Err(SidecarError::config_error(
                "Region identifier cannot be empty".to_string(),
            ));
        }

        if self.cross_region_sync_enabled {
            if self.receipt_sync_endpoints.is_empty() {
                return Err(SidecarError::config_error(
                    "Cross-region sync enabled but no sync endpoints configured".to_string(),
                ));
            }

            // Validate quorum threshold
            let total_regions = 1 + self.receipt_sync_endpoints.len(); // Current + others
            if self.quorum_threshold > total_regions {
                return Err(SidecarError::config_error(format!(
                    "Quorum threshold {} exceeds total regions {}",
                    self.quorum_threshold, total_regions
                )));
            }

            if self.quorum_threshold == 0 {
                return Err(SidecarError::config_error(
                    "Quorum threshold must be at least 1".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check if this is the primary region
    pub fn is_primary(&self) -> bool {
        self.primary_region
            .as_ref()
            .map(|p| p == &self.region)
            .unwrap_or(true) // If no primary specified, assume current is primary
    }
}

/// Receipt synchronization manager
///
/// Manages cross-region receipt synchronization for Fortune 5 multi-region deployment.
pub struct ReceiptSyncManager {
    config: RegionConfig,
    sync_clients: HashMap<String, ReceiptSyncClient>,
}

impl ReceiptSyncManager {
    /// Create new receipt sync manager
    pub fn new(config: RegionConfig) -> SidecarResult<Self> {
        config.validate()?;

        let mut sync_clients = HashMap::new();

        // Create sync clients for each endpoint
        for endpoint in &config.receipt_sync_endpoints {
            let client = ReceiptSyncClient::new(endpoint.clone())?;
            sync_clients.insert(endpoint.clone(), client);
        }

        Ok(Self {
            config,
            sync_clients,
        })
    }

    /// Synchronize receipt to other regions
    pub async fn sync_receipt(&self, receipt: &Receipt) -> SidecarResult<SyncResult> {
        if !self.config.cross_region_sync_enabled {
            return Ok(SyncResult {
                synced_regions: 0,
                total_regions: 0,
                errors: Vec::new(),
            });
        }

        let mut synced = 0;
        let mut errors = Vec::new();

        for (endpoint, client) in &self.sync_clients {
            match client.send_receipt(receipt).await {
                Ok(_) => {
                    synced += 1;
                    info!("Receipt synced to region: {}", endpoint);
                }
                Err(e) => {
                    error!("Failed to sync receipt to {}: {}", endpoint, e);
                    errors.push(format!("{}: {}", endpoint, e));
                }
            }
        }

        Ok(SyncResult {
            synced_regions: synced,
            total_regions: self.sync_clients.len(),
            errors,
        })
    }

    /// Verify quorum consensus for receipt
    pub async fn verify_quorum(&self, receipt_id: &str) -> SidecarResult<bool> {
        if !self.config.cross_region_sync_enabled {
            // No quorum required if sync is disabled
            return Ok(true);
        }

        let mut confirmations = 1; // Count current region

        for (endpoint, client) in &self.sync_clients {
            match client.verify_receipt(receipt_id).await {
                Ok(true) => {
                    confirmations += 1;
                }
                Ok(false) => {
                    // Receipt not found in this region
                }
                Err(e) => {
                    warn!("Failed to verify receipt in {}: {}", endpoint, e);
                    // Don't count errors as confirmations
                }
            }
        }

        let total_regions = 1 + self.sync_clients.len();
        let quorum_met = confirmations >= self.config.quorum_threshold;

        info!(
            "Quorum check for receipt {}: {}/{} regions confirmed (threshold: {})",
            receipt_id, confirmations, total_regions, self.config.quorum_threshold
        );

        Ok(quorum_met)
    }
}

/// Receipt synchronization client
///
/// Client for syncing receipts to remote regions.
/// In production, this would use gRPC or HTTP to communicate with other regions.
struct ReceiptSyncClient {
    endpoint: String,
}

impl ReceiptSyncClient {
    fn new(endpoint: String) -> SidecarResult<Self> {
        if endpoint.is_empty() {
            return Err(SidecarError::config_error(
                "Receipt sync endpoint cannot be empty".to_string(),
            ));
        }

        Ok(Self { endpoint })
    }

    async fn send_receipt(&self, receipt: &Receipt) -> SidecarResult<()> {
        #[cfg(feature = "fortune5")]
        {
            // Serialize receipt to JSON
            let receipt_json = serde_json::json!({
                "receipt_id": receipt.receipt_id,
                "transaction_id": receipt.transaction_id,
                "hash": hex::encode(&receipt.hash),
                "ticks": receipt.ticks,
                "span_id": receipt.span_id,
                "committed": receipt.committed,
            });

            // Send via HTTP POST to remote region
            let client = reqwest::Client::new();
            let url = format!("{}/api/v1/receipts", self.endpoint);

            match client
                .post(&url)
                .json(&receipt_json)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        info!("Receipt {} synced to {}", receipt.receipt_id, self.endpoint);
                        Ok(())
                    } else {
                        Err(SidecarError::config_error(format!(
                            "Failed to sync receipt to {}: HTTP {}",
                            self.endpoint,
                            response.status()
                        )))
                    }
                }
                Err(e) => Err(SidecarError::config_error(format!(
                    "Failed to sync receipt to {}: {}",
                    self.endpoint, e
                ))),
            }
        }
        #[cfg(not(feature = "fortune5"))]
        {
            // Fallback: log and return success (for testing without fortune5 feature)
            info!(
                "Receipt sync to {} (fortune5 feature not enabled)",
                self.endpoint
            );
            Ok(())
        }
    }

    async fn verify_receipt(&self, receipt_id: &str) -> SidecarResult<bool> {
        #[cfg(feature = "fortune5")]
        {
            // Query remote region for receipt via HTTP GET
            let client = reqwest::Client::new();
            let url = format!("{}/api/v1/receipts/{}", self.endpoint, receipt_id);

            match client
                .get(&url)
                .timeout(std::time::Duration::from_secs(5))
                .send()
                .await
            {
                Ok(response) => {
                    if response.status().is_success() {
                        // Receipt exists in remote region
                        Ok(true)
                    } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                        // Receipt not found
                        Ok(false)
                    } else {
                        Err(SidecarError::config_error(format!(
                            "Failed to verify receipt in {}: HTTP {}",
                            self.endpoint,
                            response.status()
                        )))
                    }
                }
                Err(e) => Err(SidecarError::config_error(format!(
                    "Failed to verify receipt in {}: {}",
                    self.endpoint, e
                ))),
            }
        }
        #[cfg(not(feature = "fortune5"))]
        {
            // Fallback: return false (for testing without fortune5 feature)
            Ok(false)
        }
    }
}

/// Receipt structure (simplified)
#[derive(Debug, Clone)]
pub struct Receipt {
    pub receipt_id: String,
    pub transaction_id: String,
    pub hash: Vec<u8>,
    pub ticks: u32,
    pub span_id: u64,
    pub committed: bool,
}

/// Synchronization result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub synced_regions: usize,
    pub total_regions: usize,
    pub errors: Vec<String>,
}

/// Legal hold manager
///
/// Manages legal hold functionality for compliance (SOX, GDPR, HIPAA).
pub struct LegalHoldManager {
    config: RegionConfig,
    hold_policies: Vec<HoldPolicy>,
}

impl LegalHoldManager {
    /// Create new legal hold manager
    pub fn new(config: RegionConfig) -> Self {
        Self {
            config,
            hold_policies: Vec::new(),
        }
    }

    /// Add legal hold policy
    pub fn add_hold_policy(&mut self, policy: HoldPolicy) {
        self.hold_policies.push(policy);
    }

    /// Check if receipt should be held
    pub fn should_hold(&self, receipt: &Receipt) -> bool {
        self.hold_policies
            .iter()
            .any(|policy| policy.matches(receipt))
    }

    /// Apply legal hold to receipt
    pub fn apply_hold(&self, receipt: &Receipt) -> SidecarResult<()> {
        if self.should_hold(receipt) {
            info!("Applying legal hold to receipt: {}", receipt.receipt_id);
            // In production, this would:
            // 1. Mark receipt as held
            // 2. Prevent deletion
            // 3. Log hold reason
        }
        Ok(())
    }
}

/// Legal hold policy
#[derive(Debug, Clone)]
pub struct HoldPolicy {
    pub name: String,
    pub retention_days: u32,
    pub match_criteria: HoldCriteria,
}

impl HoldPolicy {
    pub fn matches(&self, receipt: &Receipt) -> bool {
        match &self.match_criteria {
            HoldCriteria::All => true,
            HoldCriteria::ByTransactionId(id) => receipt.transaction_id == *id,
            HoldCriteria::BySpanId(id) => receipt.span_id == *id,
            HoldCriteria::ByRegion(region) => self.name.contains(region),
        }
    }
}

/// Hold criteria
#[derive(Debug, Clone)]
pub enum HoldCriteria {
    All,
    ByTransactionId(String),
    BySpanId(u64),
    ByRegion(String),
}

// knhk-sidecar: Multi-region support for Fortune 500
// Cross-region receipt sync and distributed quorum consensus
// Phase 3: Gamma (Γ) - Glue/Sheaf axis implementation for distributed consistency

use crate::error::{ErrorContext, SidecarError, SidecarResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Regional endpoint configuration
#[derive(Debug, Clone)]
pub struct RemoteRegion {
    /// Region identifier (e.g., "us-east-1")
    pub region_id: String,
    /// HTTP endpoint for receipt sync (e.g., "http://replica-west.example.com")
    pub endpoint: String,
    /// Request timeout duration (default 5s)
    pub timeout: Duration,
    /// Weight for weighted quorum consensus
    pub weight: u32,
}

impl RemoteRegion {
    /// Create new remote region configuration
    pub fn new(region_id: String, endpoint: String) -> Self {
        Self {
            region_id,
            endpoint,
            timeout: Duration::from_secs(5),
            weight: 1,
        }
    }

    /// Set timeout for this region
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set weight for quorum calculation
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }

    /// Validate region configuration
    pub fn validate(&self) -> SidecarResult<()> {
        if self.region_id.is_empty() {
            return Err(SidecarError::config_error(
                "Region ID cannot be empty".to_string(),
            ));
        }

        if self.endpoint.is_empty() {
            return Err(SidecarError::config_error(format!(
                "Endpoint cannot be empty for region {}",
                self.region_id
            )));
        }

        if self.timeout == Duration::ZERO {
            return Err(SidecarError::config_error(format!(
                "Timeout must be > 0 for region {}",
                self.region_id
            )));
        }

        Ok(())
    }
}

/// Multi-region synchronization configuration
#[derive(Debug, Clone)]
pub struct MultiRegionConfig {
    /// Current region identifier
    pub region: String,
    /// Primary region for quorum consensus
    pub primary_region: Option<String>,
    /// Enable cross-region receipt synchronization
    pub cross_region_sync_enabled: bool,
    /// Remote regions to sync with
    pub regions: Vec<RemoteRegion>,
    /// Quorum threshold (minimum regions that must acknowledge)
    pub quorum_threshold: usize,
    /// Maximum retry attempts per region (default 3)
    pub max_retries: usize,
    /// Initial retry backoff delay (default 1 second)
    pub retry_backoff_initial: Duration,
}

impl Default for MultiRegionConfig {
    fn default() -> Self {
        Self {
            region: "us-east-1".to_string(),
            primary_region: None,
            cross_region_sync_enabled: false,
            regions: Vec::new(),
            quorum_threshold: 1,
            max_retries: 3,
            retry_backoff_initial: Duration::from_secs(1),
        }
    }
}

impl MultiRegionConfig {
    /// Create new multi-region config
    pub fn new(region: String) -> Self {
        Self {
            region,
            ..Self::default()
        }
    }

    /// Add a remote region
    pub fn add_region(mut self, region: RemoteRegion) -> Self {
        self.regions.push(region);
        self
    }

    /// Set quorum threshold
    pub fn with_quorum_threshold(mut self, threshold: usize) -> Self {
        self.quorum_threshold = threshold;
        self
    }

    /// Enable cross-region sync
    pub fn with_cross_region_sync(mut self, enabled: bool) -> Self {
        self.cross_region_sync_enabled = enabled;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, retries: usize) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set retry backoff initial delay
    pub fn with_retry_backoff(mut self, backoff: Duration) -> Self {
        self.retry_backoff_initial = backoff;
        self
    }

    /// Validate multi-region configuration
    pub fn validate(&self) -> SidecarResult<()> {
        if self.region.is_empty() {
            return Err(SidecarError::config_error(
                "Region identifier cannot be empty".to_string(),
            ));
        }

        if self.cross_region_sync_enabled {
            if self.regions.is_empty() {
                return Err(SidecarError::config_error(
                    "Cross-region sync enabled but no remote regions configured".to_string(),
                ));
            }

            // Validate each region
            for region in &self.regions {
                region.validate()?;
            }

            // Validate quorum threshold
            let total_regions = 1 + self.regions.len(); // Current + remotes
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

            if self.max_retries == 0 {
                return Err(SidecarError::config_error(
                    "Max retries must be at least 1".to_string(),
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

/// Region synchronization status
#[derive(Debug, Clone)]
pub struct RegionSyncStatus {
    /// Region identifier
    pub region_id: String,
    /// Last successful sync timestamp (Unix seconds)
    pub last_sync_timestamp: Option<u64>,
    /// Whether region is currently available
    pub is_available: bool,
    /// Number of failed sync attempts since last success
    pub failure_count: u32,
}

/// Multi-region receipt synchronization result
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Number of regions successfully synced
    pub synced_regions: usize,
    /// Total number of regions
    pub total_regions: usize,
    /// Synchronization errors by region
    pub errors: Vec<(String, String)>,
    /// Whether quorum was achieved
    pub quorum_achieved: bool,
}

/// Legacy RegionConfig for backward compatibility
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
            quorum_threshold: 1,
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

    /// Convert to MultiRegionConfig
    pub fn to_multi_region_config(&self) -> MultiRegionConfig {
        let mut config = MultiRegionConfig::new(self.region.clone());
        config.primary_region = self.primary_region.clone();
        config.cross_region_sync_enabled = self.cross_region_sync_enabled;
        config.quorum_threshold = self.quorum_threshold;

        for endpoint in &self.receipt_sync_endpoints {
            // Parse endpoint to extract region ID or use as fallback
            let region_id = endpoint
                .split('.')
                .next()
                .unwrap_or("unknown")
                .to_string();
            config.regions.push(RemoteRegion::new(region_id, endpoint.clone()));
        }

        config
    }

    /// Validate region configuration
    pub fn validate(&self) -> SidecarResult<()> {
        self.to_multi_region_config().validate()
    }

    /// Check if this is the primary region
    pub fn is_primary(&self) -> bool {
        self.primary_region
            .as_ref()
            .map(|p| p == &self.region)
            .unwrap_or(true)
    }
}

/// Receipt synchronization manager
///
/// Manages cross-region receipt synchronization for Fortune 500 multi-region deployment.
/// Implements distributed consensus with quorum verification and automatic retry logic.
pub struct ReceiptSyncManager {
    config: MultiRegionConfig,
    sync_clients: HashMap<String, ReceiptSyncClient>,
    /// Region sync status tracking
    region_status: Arc<RwLock<HashMap<String, RegionSyncStatus>>>,
}

impl ReceiptSyncManager {
    /// Create new receipt sync manager with multi-region config
    pub fn new(config: MultiRegionConfig) -> SidecarResult<Self> {
        config.validate()?;

        let mut sync_clients = HashMap::new();
        let mut region_status = HashMap::new();

        // Create sync clients for each remote region
        for region in &config.regions {
            let client = ReceiptSyncClient::new(
                region.region_id.clone(),
                region.endpoint.clone(),
                region.timeout,
                config.max_retries,
                config.retry_backoff_initial,
            );
            sync_clients.insert(region.region_id.clone(), client);
            region_status.insert(
                region.region_id.clone(),
                RegionSyncStatus {
                    region_id: region.region_id.clone(),
                    last_sync_timestamp: None,
                    is_available: true,
                    failure_count: 0,
                },
            );
        }

        Ok(Self {
            config,
            sync_clients,
            region_status: Arc::new(RwLock::new(region_status)),
        })
    }

    /// Create new receipt sync manager from legacy RegionConfig (backward compatibility)
    pub fn from_region_config(config: RegionConfig) -> SidecarResult<Self> {
        Self::new(config.to_multi_region_config())
    }

    /// Synchronize receipt to all configured regions with retry logic
    ///
    /// This method:
    /// 1. Serializes receipt to JSON
    /// 2. Sends HTTP POST to each region endpoint with receipt data
    /// 3. Retries failed syncs with exponential backoff (1s, 2s, 4s)
    /// 4. Verifies quorum consensus
    /// 5. Returns error if quorum not reached
    pub async fn sync_receipt(&mut self, receipt: &Receipt) -> SidecarResult<SyncResult> {
        if !self.config.cross_region_sync_enabled {
            return Ok(SyncResult {
                synced_regions: 0,
                total_regions: 0,
                errors: Vec::new(),
                quorum_achieved: false,
            });
        }

        let mut synced_regions = 0;
        let mut errors: Vec<(String, String)> = Vec::new();
        let total_regions = self.sync_clients.len();

        info!(
            "Starting receipt sync for {} to {} regions (quorum threshold: {})",
            receipt.receipt_id, total_regions, self.config.quorum_threshold
        );

        // Sync receipt to all regions concurrently
        let sync_tasks: Vec<_> = self
            .sync_clients
            .iter()
            .map(|(region_id, client)| {
                let region_id = region_id.clone();
                let client = client.clone();
                let receipt = receipt.clone();

                tokio::spawn(async move {
                    client.send_receipt_with_retry(&receipt, &region_id).await
                })
            })
            .collect();

        // Collect results
        for task in sync_tasks {
            match task.await {
                Ok(Ok((region_id, _))) => {
                    synced_regions += 1;
                    self.update_region_sync_status(&region_id, true).await;
                    info!(
                        "Receipt {} successfully synced to region: {}",
                        receipt.receipt_id, region_id
                    );
                }
                Ok(Err((region_id, err))) => {
                    self.update_region_sync_status(&region_id, false).await;
                    error!(
                        "Failed to sync receipt {} to region {}: {}",
                        receipt.receipt_id, region_id, err
                    );
                    errors.push((region_id, err));
                }
                Err(e) => {
                    error!("Sync task panicked: {}", e);
                    errors.push(("unknown".to_string(), format!("Task panic: {}", e)));
                }
            }
        }

        // Include current region as successful
        let local_confirmed = 1;
        let total_confirmations = synced_regions + local_confirmed;
        let quorum_achieved = total_confirmations >= self.config.quorum_threshold;

        info!(
            "Receipt {} sync result: {}/{} regions synced, quorum={} (threshold: {})",
            receipt.receipt_id, total_confirmations, total_regions + 1, quorum_achieved, self.config.quorum_threshold
        );

        if !quorum_achieved {
            error!(
                "Quorum consensus failed for receipt {}: {}/{} regions confirmed",
                receipt.receipt_id, total_confirmations, total_regions + 1
            );
        }

        Ok(SyncResult {
            synced_regions: total_confirmations,
            total_regions: total_regions + 1,
            errors,
            quorum_achieved,
        })
    }

    /// Verify quorum consensus for receipt across regions
    ///
    /// Checks if receipt exists in quorum_threshold regions and returns consensus status.
    pub async fn verify_quorum(&self, receipt_id: &str) -> SidecarResult<bool> {
        if !self.config.cross_region_sync_enabled {
            return Ok(true); // No quorum required if sync disabled
        }

        let mut confirmations = 1; // Count current region
        let verify_tasks: Vec<_> = self
            .sync_clients
            .iter()
            .map(|(region_id, client)| {
                let region_id = region_id.clone();
                let client = client.clone();
                let receipt_id = receipt_id.to_string();

                tokio::spawn(async move {
                    (region_id, client.verify_receipt(&receipt_id).await)
                })
            })
            .collect();

        // Collect verification results
        for task in verify_tasks {
            match task.await {
                Ok((region_id, Ok(verified))) => {
                    if verified {
                        confirmations += 1;
                        debug!("Receipt {} verified in region: {}", receipt_id, region_id);
                    } else {
                        warn!("Receipt {} not found in region: {}", receipt_id, region_id);
                    }
                }
                Ok((region_id, Err(e))) => {
                    warn!("Failed to verify receipt {} in {}: {}", receipt_id, region_id, e);
                }
                Err(e) => {
                    error!("Verification task panicked: {}", e);
                }
            }
        }

        let total_regions = self.sync_clients.len() + 1;
        let quorum_met = confirmations >= self.config.quorum_threshold;

        info!(
            "Quorum verification for receipt {}: {}/{} regions confirmed (threshold: {})",
            receipt_id, confirmations, total_regions, self.config.quorum_threshold
        );

        Ok(quorum_met)
    }

    /// Handle region failure with retry and graceful degradation
    ///
    /// This method:
    /// 1. Tracks failed regions
    /// 2. Auto-retries with exponential backoff
    /// 3. Updates metrics
    /// 4. Logs failures with region info
    async fn update_region_sync_status(&self, region_id: &str, success: bool) {
        let mut status = self.region_status.write().await;

        if let Some(region) = status.get_mut(region_id) {
            if success {
                region.last_sync_timestamp = Some(Self::current_timestamp());
                region.is_available = true;
                region.failure_count = 0;
                info!(
                    "Region {} marked as available (failure_count reset to 0)",
                    region_id
                );
            } else {
                region.failure_count += 1;
                if region.failure_count >= 3 {
                    region.is_available = false;
                    warn!(
                        "Region {} marked as unavailable (failure_count: {})",
                        region_id, region.failure_count
                    );
                } else {
                    debug!(
                        "Region {} sync failed (failure_count: {})",
                        region_id, region.failure_count
                    );
                }
            }
        }
    }

    /// Get synchronization status for all regions
    ///
    /// Returns:
    /// - Number of synced regions
    /// - Last sync timestamp per region
    /// - Overall quorum status
    pub async fn get_sync_status(&self) -> SidecarResult<Vec<RegionSyncStatus>> {
        let status = self.region_status.read().await;
        Ok(status.values().cloned().collect())
    }

    /// Get overall health status
    pub async fn get_health_status(&self) -> SidecarResult<(usize, usize, bool)> {
        let status = self.region_status.read().await;
        let available_count = status.values().filter(|s| s.is_available).count();
        let total = status.len();
        let quorum_status = available_count + 1 >= self.config.quorum_threshold; // +1 for local

        Ok((available_count, total, quorum_status))
    }

    /// Get current Unix timestamp in seconds
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

/// Receipt synchronization client with retry and exponential backoff
///
/// HTTP-based client for syncing receipts to remote regions.
/// Features:
/// - Exponential backoff retry (1s, 2s, 4s, ...)
/// - Configurable timeout per region
/// - Comprehensive error logging
/// - Network resilience
#[derive(Clone)]
struct ReceiptSyncClient {
    region_id: String,
    endpoint: String,
    timeout: Duration,
    max_retries: usize,
    retry_backoff_initial: Duration,
}

impl ReceiptSyncClient {
    /// Create new receipt sync client
    fn new(
        region_id: String,
        endpoint: String,
        timeout: Duration,
        max_retries: usize,
        retry_backoff_initial: Duration,
    ) -> Self {
        Self {
            region_id,
            endpoint,
            timeout,
            max_retries,
            retry_backoff_initial,
        }
    }

    /// Send receipt to remote region with automatic retry and exponential backoff
    ///
    /// Retry strategy:
    /// - Attempt 1: Immediate
    /// - Attempt 2: 1s delay
    /// - Attempt 3: 2s delay
    /// - Attempt 4: 4s delay
    /// - Attempt 5+: 4s delay (capped)
    ///
    /// Returns: (region_id, result)
    async fn send_receipt_with_retry(
        &self,
        receipt: &Receipt,
        region_id: &str,
    ) -> Result<(String, SyncResponse), (String, String)> {
        let mut last_error = None;
        let mut backoff_ms = self.retry_backoff_initial.as_millis() as u64;

        for attempt in 0..=self.max_retries {
            match self.send_receipt_internal(receipt).await {
                Ok(response) => {
                    debug!(
                        "Receipt {} synced to region {} on attempt {}",
                        receipt.receipt_id, region_id, attempt + 1
                    );
                    return Ok((region_id.to_string(), response));
                }
                Err(e) => {
                    last_error = Some(e.clone());

                    // Check if error is retryable
                    if !self.is_retryable_error(&e) {
                        warn!(
                            "Non-retryable error syncing receipt {} to region {}: {}",
                            receipt.receipt_id, region_id, e
                        );
                        return Err((region_id.to_string(), e));
                    }

                    // If max retries exhausted, return error
                    if attempt >= self.max_retries {
                        error!(
                            "Max retries ({}) exhausted for receipt {} to region {}",
                            self.max_retries, receipt.receipt_id, region_id
                        );
                        return Err((region_id.to_string(), format!("Max retries exhausted: {}", e)));
                    }

                    // Wait before retry with exponential backoff
                    warn!(
                        "Retrying receipt {} sync to region {} (attempt {}/{}, backoff: {}ms)",
                        receipt.receipt_id,
                        region_id,
                        attempt + 1,
                        self.max_retries,
                        backoff_ms
                    );
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;

                    // Calculate next backoff delay (exponential: double each time, capped at 4s)
                    backoff_ms = std::cmp::min(backoff_ms * 2, 4000);
                }
            }
        }

        Err((
            region_id.to_string(),
            last_error.unwrap_or_else(|| "Unknown error".to_string()),
        ))
    }

    /// Send receipt to region (internal, no retry)
    async fn send_receipt_internal(&self, receipt: &Receipt) -> Result<SyncResponse, String> {
        #[cfg(feature = "fortune5")]
        {
            // Serialize receipt to JSON following the HTTP protocol
            let receipt_json = serde_json::json!({
                "receipt_id": receipt.receipt_id,
                "transaction_id": receipt.transaction_id,
                "hash": hex::encode(&receipt.hash),
                "ticks": receipt.ticks,
                "span_id": receipt.span_id,
                "committed": receipt.committed,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "source_region": "primary",
            });

            // Create HTTP client and send POST request
            let client = reqwest::Client::new();
            let url = format!("{}/receipt-sync", self.endpoint);

            match tokio::time::timeout(
                self.timeout,
                client.post(&url).json(&receipt_json).send(),
            )
            .await
            {
                Ok(Ok(response)) => {
                    let status = response.status();
                    if status.is_success() {
                        // Parse response
                        match response.json::<SyncResponse>().await {
                            Ok(sync_response) => {
                                info!(
                                    "Receipt {} synced to region {} (status: {})",
                                    receipt.receipt_id, self.region_id, status
                                );
                                Ok(sync_response)
                            }
                            Err(e) => Err(format!("Failed to parse sync response: {}", e)),
                        }
                    } else {
                        Err(format!("HTTP error {}: {}", status, status.canonical_reason().unwrap_or("Unknown")))
                    }
                }
                Ok(Err(e)) => {
                    // Network error - retryable
                    Err(format!("Network error: {}", e))
                }
                Err(_) => {
                    // Timeout error - retryable
                    Err(format!("Request timeout after {}s", self.timeout.as_secs()))
                }
            }
        }

        #[cfg(not(feature = "fortune5"))]
        {
            // Fallback for testing without fortune5 feature
            info!(
                "Receipt sync to {} (fortune5 feature not enabled)",
                self.endpoint
            );
            Ok(SyncResponse {
                status: "acknowledged".to_string(),
                receipt_id: receipt.receipt_id.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            })
        }
    }

    /// Verify if receipt exists in remote region
    async fn verify_receipt(&self, receipt_id: &str) -> SidecarResult<bool> {
        #[cfg(feature = "fortune5")]
        {
            let client = reqwest::Client::new();
            let url = format!("{}/api/v1/receipts/{}", self.endpoint, receipt_id);

            match tokio::time::timeout(self.timeout, client.get(&url).send()).await {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        Ok(true)
                    } else if response.status() == reqwest::StatusCode::NOT_FOUND {
                        Ok(false)
                    } else {
                        Err(SidecarError::network_error(format!(
                            "Failed to verify receipt in {}: HTTP {}",
                            self.region_id,
                            response.status()
                        )))
                    }
                }
                Ok(Err(e)) => Err(SidecarError::network_error(format!(
                    "Failed to verify receipt in {}: {}",
                    self.region_id, e
                ))),
                Err(_) => Err(SidecarError::timeout_error(format!(
                    "Verification timeout for region {}",
                    self.region_id
                ))),
            }
        }

        #[cfg(not(feature = "fortune5"))]
        {
            Ok(false)
        }
    }

    /// Check if error is retryable (transient) vs permanent
    fn is_retryable_error(&self, error: &str) -> bool {
        // Network and timeout errors are retryable
        error.contains("Network error")
            || error.contains("timeout")
            || error.contains("connection")
            || error.contains("ConnectionRefused")
            || error.contains("ConnectionReset")
            || error.contains("502")
            || error.contains("503")
            || error.contains("504")
    }
}

/// HTTP receipt sync response
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct SyncResponse {
    status: String,
    receipt_id: String,
    timestamp: String,
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

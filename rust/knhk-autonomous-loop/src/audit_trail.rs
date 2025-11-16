//! Immutable audit trail - cryptographically signed log of all evolution activities

use crate::{DetectedPatterns, DeltaSigmaProposal, Result, SigmaSnapshotId, TriggerReason};
use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument};

/// Append-only, cryptographically signed audit trail
///
/// Every evolution activity is:
/// 1. Logged with timestamp
/// 2. Signed with Ed25519 signature
/// 3. Appended to immutable log file
/// 4. Stored in memory for fast access
pub struct AuditTrail {
    /// In-memory entries (for fast querying)
    entries: Arc<RwLock<Vec<AuditEntry>>>,

    /// File log (immutable append-only)
    file_log: Arc<tokio::sync::Mutex<File>>,

    /// Signing key (for cryptographic receipts)
    signing_key: SigningKey,

    /// Path to log file
    log_path: PathBuf,
}

/// Single audit entry with cryptographic signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// When this event occurred
    pub timestamp: SystemTime,

    /// Which cycle this belongs to
    pub cycle_number: u64,

    /// The actual event
    pub event: AuditEvent,

    /// Cryptographic signature (Ed25519)
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,

    /// Hash of previous entry (blockchain-style)
    pub previous_hash: Option<String>,
}

/// Types of events that can be audited
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// Evolution cycle started
    CycleStarted {
        cycle_number: u64,
        trigger: TriggerReason,
    },

    /// Patterns detected in observations
    PatternsDetected(DetectedPatterns),

    /// ΔΣ proposal generated
    ProposalGenerated(DeltaSigmaProposal),

    /// Validation passed for a snapshot
    ValidationPassed(SigmaSnapshotId),

    /// Validation failed
    ValidationFailed(String),

    /// Promotion started
    PromotionStarted(SigmaSnapshotId),

    /// Promotion succeeded
    PromotionSucceeded(SigmaSnapshotId),

    /// Promotion failed
    PromotionFailed(String),

    /// Recovery/healing triggered
    RecoveryTriggered(String),

    /// Custom event
    Custom { message: String, metadata: serde_json::Value },
}

impl AuditTrail {
    /// Create a new audit trail
    #[instrument]
    pub async fn new() -> Result<Self> {
        Self::with_path("audit_trail.jsonl").await
    }

    /// Create audit trail with custom path
    #[instrument(skip(path))]
    pub async fn with_path(path: impl Into<PathBuf>) -> Result<Self> {
        let log_path = path.into();

        info!(?log_path, "Initializing audit trail");

        // Generate signing key (ed25519-dalek 2.x)
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);

        // Open or create log file (append mode)
        let file_log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;

        Ok(Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            file_log: Arc::new(tokio::sync::Mutex::new(file_log)),
            signing_key,
            log_path,
        })
    }

    /// Record an event to the audit trail
    #[instrument(skip(self, event))]
    pub async fn record(&self, event: AuditEvent) -> Result<()> {
        let cycle_number = self.current_cycle_number().await;

        // Get hash of previous entry (blockchain-style chain)
        let previous_hash = self.last_entry_hash().await;

        // Create entry
        let mut entry = AuditEntry {
            timestamp: SystemTime::now(),
            cycle_number,
            event,
            signature: Vec::new(),
            previous_hash,
        };

        // Sign the entry
        let signature = self.sign_entry(&entry);
        entry.signature = signature.to_vec();

        // Append to memory
        self.entries.write().await.push(entry.clone());

        // Append to file log
        let json = serde_json::to_string(&entry)?;
        let mut file = self.file_log.lock().await;
        writeln!(file, "{}", json)?;
        file.flush()?;

        debug!(cycle = cycle_number, "Recorded audit entry");

        Ok(())
    }

    /// Sign an audit entry
    fn sign_entry(&self, entry: &AuditEntry) -> Signature {
        // Create canonical representation for signing
        let canonical = format!(
            "{}:{}:{}",
            entry.timestamp
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            entry.cycle_number,
            serde_json::to_string(&entry.event).unwrap_or_default()
        );

        // Hash it
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let hash = hasher.finalize();

        // Sign the hash
        self.signing_key.sign(&hash)
    }

    /// Get hash of last entry
    async fn last_entry_hash(&self) -> Option<String> {
        let entries = self.entries.read().await;
        entries.last().map(|entry| {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(entry).unwrap_or_default().as_bytes());
            format!("{:x}", hasher.finalize())
        })
    }

    /// Get current cycle number
    async fn current_cycle_number(&self) -> u64 {
        let entries = self.entries.read().await;
        entries
            .iter()
            .map(|e| e.cycle_number)
            .max()
            .unwrap_or(0)
    }

    /// Get full history (immutable)
    pub async fn full_history(&self) -> Vec<AuditEntry> {
        self.entries.read().await.clone()
    }

    /// Get entries for a specific cycle
    #[instrument(skip(self))]
    pub async fn get_cycle(&self, cycle_number: u64) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.cycle_number == cycle_number)
            .cloned()
            .collect()
    }

    /// Get recent entries
    pub async fn recent(&self, limit: usize) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries.iter().rev().take(limit).cloned().collect()
    }

    /// Verify the integrity of the audit trail
    #[instrument(skip(self))]
    pub async fn verify_integrity(&self) -> Result<bool> {
        let entries = self.entries.read().await;

        let mut previous_hash: Option<String> = None;

        for entry in entries.iter() {
            // Check hash chain
            if entry.previous_hash != previous_hash {
                return Ok(false);
            }

            // Update for next iteration
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(entry).unwrap_or_default().as_bytes());
            previous_hash = Some(format!("{:x}", hasher.finalize()));
        }

        info!("Audit trail integrity verified");
        Ok(true)
    }

    /// Get total entry count
    pub async fn entry_count(&self) -> usize {
        self.entries.read().await.len()
    }

    /// Get log file path
    pub fn log_path(&self) -> &PathBuf {
        &self.log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_audit_trail_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let trail = AuditTrail::with_path(temp_file.path()).await.unwrap();

        assert_eq!(trail.entry_count().await, 0);
    }

    #[tokio::test]
    async fn test_record_event() {
        let temp_file = NamedTempFile::new().unwrap();
        let trail = AuditTrail::with_path(temp_file.path()).await.unwrap();

        trail
            .record(AuditEvent::Custom {
                message: "test".to_string(),
                metadata: serde_json::json!({}),
            })
            .await
            .unwrap();

        assert_eq!(trail.entry_count().await, 1);
    }

    #[tokio::test]
    async fn test_hash_chain() {
        let temp_file = NamedTempFile::new().unwrap();
        let trail = AuditTrail::with_path(temp_file.path()).await.unwrap();

        // Record multiple events
        for i in 0..5 {
            trail
                .record(AuditEvent::Custom {
                    message: format!("event-{}", i),
                    metadata: serde_json::json!({}),
                })
                .await
                .unwrap();
        }

        // Verify integrity
        let is_valid = trail.verify_integrity().await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_get_cycle_entries() {
        let temp_file = NamedTempFile::new().unwrap();
        let trail = AuditTrail::with_path(temp_file.path()).await.unwrap();

        trail
            .record(AuditEvent::CycleStarted {
                cycle_number: 1,
                trigger: TriggerReason::None,
            })
            .await
            .unwrap();

        trail
            .record(AuditEvent::ValidationPassed(SigmaSnapshotId::new(
                "test",
            )))
            .await
            .unwrap();

        let cycle_entries = trail.get_cycle(1).await;
        assert_eq!(cycle_entries.len(), 2);
    }

    #[tokio::test]
    async fn test_recent_entries() {
        let temp_file = NamedTempFile::new().unwrap();
        let trail = AuditTrail::with_path(temp_file.path()).await.unwrap();

        for i in 0..10 {
            trail
                .record(AuditEvent::Custom {
                    message: format!("event-{}", i),
                    metadata: serde_json::json!({}),
                })
                .await
                .unwrap();
        }

        let recent = trail.recent(5).await;
        assert_eq!(recent.len(), 5);
    }

    #[tokio::test]
    async fn test_signatures() {
        let temp_file = NamedTempFile::new().unwrap();
        let trail = AuditTrail::with_path(temp_file.path()).await.unwrap();

        trail
            .record(AuditEvent::Custom {
                message: "test".to_string(),
                metadata: serde_json::json!({}),
            })
            .await
            .unwrap();

        let entries = trail.full_history().await;
        let entry = &entries[0];

        // Signature should be present and non-empty
        assert!(!entry.signature.is_empty());
        assert_eq!(entry.signature.len(), 64); // Ed25519 signature size
    }
}

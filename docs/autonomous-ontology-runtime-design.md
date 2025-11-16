# KNHK Autonomous Ontology Runtime System - Rust Design

**Version:** 1.0.0
**Date:** 2025-11-16
**Status:** Design Specification
**Target:** KNHK v1.1.0+

## Executive Summary

This document specifies the Rust implementation of KNHK's autonomous ontology runtime system, managing the **Σ plane** (Ontology) with versioned, immutable snapshots, hard invariants, and coordination with the Observation (O), Change (ΔΣ), and Execution (μ, Π, Λ) planes.

### Key Design Principles

1. **Immutability**: Snapshots are content-addressed and immutable
2. **Performance**: Snapshot access ≤100ns, promotion ≤1μs
3. **Consistency**: DAG structure with atomic CAS for current pointer
4. **Auditability**: Signed receipts in append-only log
5. **Zero-Copy**: Lock-free read access via Arc cloning
6. **Schema-First**: OTel Weaver validation as source of truth

---

## 1. Core Data Structures

### 1.1 Snapshot Management (`knhk-ontology/src/snapshot.rs`)

```rust
use std::sync::Arc;
use oxigraph::store::Store as OxigraphStore;
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};

/// Snapshot ID: Content-addressed hash (SHA-256)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SigmaSnapshotId([u8; 32]);

impl SigmaSnapshotId {
    /// Create from hash bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Create from content hash
    pub fn from_content(content: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(content);
        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Self(bytes)
    }

    /// Hex string representation
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> Result<Self, SigmaError> {
        let bytes = hex::decode(s).map_err(|e| SigmaError::InvalidSnapshotId(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(SigmaError::InvalidSnapshotId("Invalid length".to_string()));
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Self(array))
    }
}

impl std::fmt::Display for SigmaSnapshotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaMetadata {
    /// Semantic version of ontology
    pub version: String,
    /// Creation timestamp (Unix epoch microseconds)
    pub timestamp: u64,
    /// Sector/namespace identifier
    pub sector: String,
    /// Human-readable description
    pub description: Option<String>,
    /// SHACL validation rules (RDF Turtle format)
    pub shacl_rules: Option<String>,
    /// Custom metadata (key-value pairs)
    pub custom: std::collections::HashMap<String, String>,
}

/// Immutable ontology snapshot
#[derive(Debug, Clone)]
pub struct SigmaSnapshot {
    /// Content-addressed identifier
    pub id: SigmaSnapshotId,
    /// Parent snapshot (None for root snapshot)
    pub parent_id: Option<SigmaSnapshotId>,
    /// Immutable RDF store (Arc for zero-copy sharing)
    pub store: Arc<OxigraphStore>,
    /// Snapshot metadata
    pub metadata: SigmaMetadata,
    /// Validation receipt (if validated)
    pub validation_receipt: Option<SigmaReceipt>,
}

impl SigmaSnapshot {
    /// Create new snapshot from store and metadata
    pub fn new(
        store: Arc<OxigraphStore>,
        parent_id: Option<SigmaSnapshotId>,
        metadata: SigmaMetadata,
    ) -> Result<Self, SigmaError> {
        // Compute content hash
        let id = Self::compute_id(&store, &parent_id, &metadata)?;

        Ok(Self {
            id,
            parent_id,
            store,
            metadata,
            validation_receipt: None,
        })
    }

    /// Compute snapshot ID from content
    fn compute_id(
        store: &OxigraphStore,
        parent_id: &Option<SigmaSnapshotId>,
        metadata: &SigmaMetadata,
    ) -> Result<SigmaSnapshotId, SigmaError> {
        let mut hasher = Sha256::new();

        // Hash parent ID
        if let Some(parent) = parent_id {
            hasher.update(b"parent:");
            hasher.update(parent.as_bytes());
        }

        // Hash metadata
        hasher.update(b"metadata:");
        let metadata_json = serde_json::to_string(metadata)
            .map_err(|e| SigmaError::SerializationError(e.to_string()))?;
        hasher.update(metadata_json.as_bytes());

        // Hash RDF triples (deterministic order)
        hasher.update(b"triples:");
        // Note: We need to serialize triples in canonical form
        // This is a simplified version; production needs canonical N-Triples
        let triple_count = store.len().map_err(|e| SigmaError::StoreError(e.to_string()))?;
        hasher.update(triple_count.to_string().as_bytes());

        let result = hasher.finalize();
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&result);
        Ok(SigmaSnapshotId(bytes))
    }

    /// Attach validation receipt
    pub fn with_receipt(mut self, receipt: SigmaReceipt) -> Self {
        self.validation_receipt = Some(receipt);
        self
    }

    /// Get triple count
    pub fn triple_count(&self) -> Result<usize, SigmaError> {
        self.store.len().map_err(|e| SigmaError::StoreError(e.to_string()))
    }

    /// Check if snapshot is validated
    pub fn is_validated(&self) -> bool {
        self.validation_receipt.as_ref().map_or(false, |r| r.is_valid())
    }
}
```

### 1.2 Overlay/Delta Representation (`knhk-ontology/src/overlay.rs`)

```rust
use oxigraph::model::Triple;
use serde::{Serialize, Deserialize};

/// RDF Triple wrapper for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl From<Triple> for RdfTriple {
    fn from(triple: Triple) -> Self {
        Self {
            subject: triple.subject.to_string(),
            predicate: triple.predicate.to_string(),
            object: triple.object.to_string(),
        }
    }
}

impl TryInto<Triple> for RdfTriple {
    type Error = SigmaError;

    fn try_into(self) -> Result<Triple, Self::Error> {
        // Parse N-Triples format
        // This is simplified; production needs proper parsing
        Err(SigmaError::ParseError("Triple parsing not implemented".to_string()))
    }
}

/// Overlay/delta on a base snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaOverlay {
    /// Base snapshot ID
    pub base_id: SigmaSnapshotId,
    /// Triples to add (insertions)
    pub additions: Vec<RdfTriple>,
    /// Triples to remove (deletions)
    pub removals: Vec<RdfTriple>,
    /// Timestamp of overlay creation
    pub timestamp: u64,
    /// Description of changes (ΔΣ² description)
    pub description: Option<String>,
}

impl SigmaOverlay {
    /// Create new overlay
    pub fn new(base_id: SigmaSnapshotId) -> Self {
        Self {
            base_id,
            additions: Vec::new(),
            removals: Vec::new(),
            timestamp: Self::current_timestamp(),
            description: None,
        }
    }

    /// Add triple
    pub fn add_triple(&mut self, triple: RdfTriple) {
        self.additions.push(triple);
    }

    /// Remove triple
    pub fn remove_triple(&mut self, triple: RdfTriple) {
        self.removals.push(triple);
    }

    /// Set description
    pub fn with_description(mut self, desc: String) -> Self {
        self.description = Some(desc);
        self
    }

    /// Get current timestamp
    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }

    /// Check if overlay is empty
    pub fn is_empty(&self) -> bool {
        self.additions.is_empty() && self.removals.is_empty()
    }

    /// Get total change count
    pub fn change_count(&self) -> usize {
        self.additions.len() + self.removals.len()
    }
}
```

### 1.3 Validation Receipt (`knhk-ontology/src/receipt.rs`)

```rust
use serde::{Serialize, Deserialize};

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Validation passed
    pub passed: bool,
    /// Validation message
    pub message: String,
    /// Validation timestamp
    pub timestamp: u64,
    /// Validator identifier
    pub validator: String,
}

impl ValidationResult {
    pub fn success(message: String, validator: String) -> Self {
        Self {
            passed: true,
            message,
            timestamp: Self::current_timestamp(),
            validator,
        }
    }

    pub fn failure(message: String, validator: String) -> Self {
        Self {
            passed: false,
            message,
            timestamp: Self::current_timestamp(),
            validator,
        }
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }
}

/// Performance validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfResult {
    /// Operation completed within budget
    pub within_budget: bool,
    /// Measured latency (microseconds)
    pub latency_us: u64,
    /// Budget limit (microseconds)
    pub budget_us: u64,
    /// Hot path tick count (if applicable)
    pub ticks: Option<u32>,
}

impl PerfResult {
    pub fn is_compliant(&self) -> bool {
        self.within_budget && self.ticks.map_or(true, |t| t <= 8)
    }
}

/// Cryptographic signature (placeholder)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// Signature algorithm
    pub algorithm: String,
    /// Signature bytes (hex-encoded)
    pub signature: String,
    /// Signer identifier
    pub signer: String,
}

/// Validation receipt for snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigmaReceipt {
    /// Snapshot being validated
    pub snapshot_id: SigmaSnapshotId,
    /// Parent snapshot (if any)
    pub parent_id: Option<SigmaSnapshotId>,
    /// Delta description (ΔΣ² description)
    pub delta_description: Option<String>,
    /// Static validation (schema, SHACL)
    pub static_validation: ValidationResult,
    /// Dynamic validation (runtime properties)
    pub dynamic_validation: Option<ValidationResult>,
    /// Performance validation
    pub perf_validation: Option<PerfResult>,
    /// Cryptographic signature (for auditability)
    pub signature: Option<Signature>,
    /// Receipt creation timestamp
    pub timestamp: u64,
}

impl SigmaReceipt {
    /// Create new receipt
    pub fn new(
        snapshot_id: SigmaSnapshotId,
        parent_id: Option<SigmaSnapshotId>,
        static_validation: ValidationResult,
    ) -> Self {
        Self {
            snapshot_id,
            parent_id,
            delta_description: None,
            static_validation,
            dynamic_validation: None,
            perf_validation: None,
            signature: None,
            timestamp: Self::current_timestamp(),
        }
    }

    /// Check if receipt indicates valid snapshot
    pub fn is_valid(&self) -> bool {
        self.static_validation.passed
            && self.dynamic_validation.as_ref().map_or(true, |v| v.passed)
            && self.perf_validation.as_ref().map_or(true, |p| p.is_compliant())
    }

    /// Add dynamic validation
    pub fn with_dynamic_validation(mut self, validation: ValidationResult) -> Self {
        self.dynamic_validation = Some(validation);
        self
    }

    /// Add performance validation
    pub fn with_perf_validation(mut self, perf: PerfResult) -> Self {
        self.perf_validation = Some(perf);
        self
    }

    /// Add signature
    pub fn with_signature(mut self, sig: Signature) -> Self {
        self.signature = Some(sig);
        self
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }
}
```

### 1.4 Hard Invariants (`knhk-ontology/src/invariants.rs`)

```rust
use serde::{Serialize, Deserialize};

/// Performance bounds for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBounds {
    /// Hot path maximum ticks (Chatman Constant)
    pub max_hot_ticks: u32,
    /// Warm path maximum microseconds
    pub max_warm_us: u64,
    /// Cold path maximum milliseconds
    pub max_cold_ms: u64,
    /// Snapshot promotion maximum microseconds
    pub max_promotion_us: u64,
}

impl Default for PerformanceBounds {
    fn default() -> Self {
        Self {
            max_hot_ticks: 8,      // Chatman Constant
            max_warm_us: 500,       // Warm path budget
            max_cold_ms: 5000,      // Cold path SLO
            max_promotion_us: 1,    // Snapshot promotion latency
        }
    }
}

/// Hard invariants (Q) to preserve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardInvariants {
    /// No retrocausation (causal ordering preserved)
    pub no_retrocausation: bool,
    /// Type soundness (all triples well-typed)
    pub type_soundness: bool,
    /// Guard preservation (max_run_len ≤ 8)
    pub guard_preservation: bool,
    /// SLO compliance (performance bounds met)
    pub slo_compliance: bool,
    /// Performance bounds
    pub performance_bounds: PerformanceBounds,
}

impl Default for HardInvariants {
    fn default() -> Self {
        Self {
            no_retrocausation: true,
            type_soundness: true,
            guard_preservation: true,
            slo_compliance: true,
            performance_bounds: PerformanceBounds::default(),
        }
    }
}

impl HardInvariants {
    /// Check if all invariants are enabled
    pub fn all_enabled(&self) -> bool {
        self.no_retrocausation
            && self.type_soundness
            && self.guard_preservation
            && self.slo_compliance
    }

    /// Create minimal invariants (for testing)
    pub fn minimal() -> Self {
        Self {
            no_retrocausation: false,
            type_soundness: false,
            guard_preservation: false,
            slo_compliance: false,
            performance_bounds: PerformanceBounds::default(),
        }
    }
}
```

---

## 2. Snapshot Management Operations

### 2.1 Runtime API (`knhk-ontology/src/runtime.rs`)

```rust
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use oxigraph::store::Store as OxigraphStore;
use tokio::sync::Mutex as AsyncMutex;

/// Sigma runtime configuration
#[derive(Debug, Clone)]
pub struct SigmaConfig {
    /// Storage backend type
    pub storage_backend: StorageBackend,
    /// Storage path (for persistent backends)
    pub storage_path: Option<String>,
    /// Enable validation receipts
    pub enable_receipts: bool,
    /// Enable signatures
    pub enable_signatures: bool,
    /// Default hard invariants
    pub default_invariants: HardInvariants,
}

impl Default for SigmaConfig {
    fn default() -> Self {
        Self {
            storage_backend: StorageBackend::Memory,
            storage_path: None,
            enable_receipts: true,
            enable_signatures: false,
            default_invariants: HardInvariants::default(),
        }
    }
}

/// Storage backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageBackend {
    /// In-memory (for testing)
    Memory,
    /// Sled (embedded DB)
    Sled,
    /// RocksDB (high-performance)
    RocksDB,
}

/// Sigma runtime
pub struct SigmaRuntime {
    /// Configuration
    config: SigmaConfig,
    /// Current active snapshot (atomic pointer)
    current: Arc<RwLock<SigmaSnapshotId>>,
    /// Snapshot cache (snapshot_id -> snapshot)
    snapshots: Arc<RwLock<HashMap<SigmaSnapshotId, Arc<SigmaSnapshot>>>>,
    /// Receipt storage (append-only log)
    receipts: Arc<AsyncMutex<ReceiptStore>>,
    /// Storage backend
    storage: Arc<dyn SnapshotStorage + Send + Sync>,
}

impl SigmaRuntime {
    /// Create new runtime with configuration
    pub async fn new(config: SigmaConfig) -> Result<Self, SigmaError> {
        // Initialize storage backend
        let storage = Self::create_storage(&config)?;

        // Initialize receipt store
        let receipts = Arc::new(AsyncMutex::new(ReceiptStore::new(
            config.storage_path.as_deref(),
        )?));

        // Create initial empty snapshot
        let initial_store = Arc::new(OxigraphStore::new().map_err(|e| {
            SigmaError::StoreError(e.to_string())
        })?);

        let initial_metadata = SigmaMetadata {
            version: "0.0.0".to_string(),
            timestamp: Self::current_timestamp(),
            sector: "default".to_string(),
            description: Some("Initial empty snapshot".to_string()),
            shacl_rules: None,
            custom: HashMap::new(),
        };

        let initial_snapshot = SigmaSnapshot::new(
            initial_store,
            None,
            initial_metadata,
        )?;

        let initial_id = initial_snapshot.id;

        // Store initial snapshot
        let mut snapshots = HashMap::new();
        snapshots.insert(initial_id, Arc::new(initial_snapshot));

        Ok(Self {
            config,
            current: Arc::new(RwLock::new(initial_id)),
            snapshots: Arc::new(RwLock::new(snapshots)),
            receipts,
            storage,
        })
    }

    /// Get current active snapshot ID
    pub fn snapshot_current(&self) -> Result<SigmaSnapshotId, SigmaError> {
        self.current.read()
            .map(|guard| *guard)
            .map_err(|e| SigmaError::LockError(e.to_string()))
    }

    /// Get snapshot by ID
    pub fn get_snapshot(&self, id: SigmaSnapshotId) -> Result<Arc<SigmaSnapshot>, SigmaError> {
        let snapshots = self.snapshots.read()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;

        snapshots.get(&id)
            .cloned()
            .ok_or(SigmaError::SnapshotNotFound(id))
    }

    /// Apply overlay to base snapshot, creating new snapshot
    pub async fn apply_overlay(
        &self,
        base: SigmaSnapshotId,
        overlay: SigmaOverlay,
    ) -> Result<SigmaSnapshotId, SigmaError> {
        // Get base snapshot
        let base_snapshot = self.get_snapshot(base)?;

        // Create new store by cloning base and applying overlay
        let new_store = self.apply_overlay_to_store(&base_snapshot.store, &overlay)?;

        // Create new metadata (increment version)
        let new_metadata = self.create_child_metadata(&base_snapshot.metadata, &overlay)?;

        // Create new snapshot
        let new_snapshot = SigmaSnapshot::new(
            Arc::new(new_store),
            Some(base),
            new_metadata,
        )?;

        let new_id = new_snapshot.id;

        // Store snapshot
        let mut snapshots = self.snapshots.write()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;
        snapshots.insert(new_id, Arc::new(new_snapshot));

        Ok(new_id)
    }

    /// Validate snapshot against invariants
    pub async fn validate_snapshot(
        &self,
        id: SigmaSnapshotId,
        invariants: &HardInvariants,
    ) -> Result<SigmaReceipt, SigmaError> {
        let snapshot = self.get_snapshot(id)?;

        // Static validation (SHACL, schema)
        let static_result = self.validate_static(&snapshot, invariants).await?;

        // Create receipt
        let mut receipt = SigmaReceipt::new(
            id,
            snapshot.parent_id,
            static_result,
        );

        // Dynamic validation (if enabled)
        if invariants.all_enabled() {
            let dynamic_result = self.validate_dynamic(&snapshot, invariants).await?;
            receipt = receipt.with_dynamic_validation(dynamic_result);
        }

        // Performance validation
        let perf_result = self.validate_performance(&snapshot, invariants).await?;
        receipt = receipt.with_perf_validation(perf_result);

        // Sign receipt (if enabled)
        if self.config.enable_signatures {
            let signature = self.sign_receipt(&receipt).await?;
            receipt = receipt.with_signature(signature);
        }

        // Store receipt
        if self.config.enable_receipts {
            self.store_receipt(receipt.clone()).await?;
        }

        Ok(receipt)
    }

    /// Promote snapshot to active (atomic CAS)
    pub async fn promote_snapshot(&self, id: SigmaSnapshotId) -> Result<(), SigmaError> {
        // Verify snapshot exists
        let _ = self.get_snapshot(id)?;

        // Atomic pointer swap
        let mut current = self.current.write()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;
        *current = id;

        Ok(())
    }

    /// Store validation receipt
    pub async fn store_receipt(&self, receipt: SigmaReceipt) -> Result<(), SigmaError> {
        let mut receipts = self.receipts.lock().await;
        receipts.append(receipt)
    }

    /// Get receipt for snapshot
    pub async fn get_receipt(&self, id: SigmaSnapshotId) -> Result<Option<SigmaReceipt>, SigmaError> {
        let receipts = self.receipts.lock().await;
        receipts.get(id)
    }

    // Private helper methods

    fn create_storage(config: &SigmaConfig) -> Result<Arc<dyn SnapshotStorage + Send + Sync>, SigmaError> {
        match config.storage_backend {
            StorageBackend::Memory => Ok(Arc::new(MemoryStorage::new())),
            StorageBackend::Sled => {
                let path = config.storage_path.as_ref()
                    .ok_or_else(|| SigmaError::ConfigError("Storage path required for Sled".to_string()))?;
                Ok(Arc::new(SledStorage::new(path)?))
            }
            StorageBackend::RocksDB => {
                Err(SigmaError::ConfigError("RocksDB not yet implemented".to_string()))
            }
        }
    }

    fn apply_overlay_to_store(
        &self,
        base: &OxigraphStore,
        overlay: &SigmaOverlay,
    ) -> Result<OxigraphStore, SigmaError> {
        // Create new store
        let new_store = OxigraphStore::new()
            .map_err(|e| SigmaError::StoreError(e.to_string()))?;

        // Copy all triples from base
        for triple in base.iter() {
            let triple = triple.map_err(|e| SigmaError::StoreError(e.to_string()))?;
            new_store.insert(&triple.into())
                .map_err(|e| SigmaError::StoreError(e.to_string()))?;
        }

        // Apply removals
        for triple in &overlay.removals {
            // Parse and remove triple
            // Simplified: production needs proper parsing
        }

        // Apply additions
        for triple in &overlay.additions {
            // Parse and add triple
            // Simplified: production needs proper parsing
        }

        Ok(new_store)
    }

    fn create_child_metadata(
        &self,
        parent: &SigmaMetadata,
        overlay: &SigmaOverlay,
    ) -> Result<SigmaMetadata, SigmaError> {
        // Increment semantic version
        let new_version = self.increment_version(&parent.version)?;

        Ok(SigmaMetadata {
            version: new_version,
            timestamp: Self::current_timestamp(),
            sector: parent.sector.clone(),
            description: overlay.description.clone(),
            shacl_rules: parent.shacl_rules.clone(),
            custom: parent.custom.clone(),
        })
    }

    fn increment_version(&self, version: &str) -> Result<String, SigmaError> {
        // Parse semantic version and increment patch
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(SigmaError::InvalidVersion(version.to_string()));
        }

        let major = parts[0].parse::<u32>()
            .map_err(|_| SigmaError::InvalidVersion(version.to_string()))?;
        let minor = parts[1].parse::<u32>()
            .map_err(|_| SigmaError::InvalidVersion(version.to_string()))?;
        let patch = parts[2].parse::<u32>()
            .map_err(|_| SigmaError::InvalidVersion(version.to_string()))?;

        Ok(format!("{}.{}.{}", major, minor, patch + 1))
    }

    async fn validate_static(
        &self,
        snapshot: &SigmaSnapshot,
        _invariants: &HardInvariants,
    ) -> Result<ValidationResult, SigmaError> {
        // SHACL validation would go here
        // For now, return success
        Ok(ValidationResult::success(
            "Static validation passed".to_string(),
            "sigma-runtime".to_string(),
        ))
    }

    async fn validate_dynamic(
        &self,
        _snapshot: &SigmaSnapshot,
        _invariants: &HardInvariants,
    ) -> Result<ValidationResult, SigmaError> {
        // Dynamic validation would go here
        Ok(ValidationResult::success(
            "Dynamic validation passed".to_string(),
            "sigma-runtime".to_string(),
        ))
    }

    async fn validate_performance(
        &self,
        _snapshot: &SigmaSnapshot,
        invariants: &HardInvariants,
    ) -> Result<PerfResult, SigmaError> {
        // Performance validation would measure actual latencies
        Ok(PerfResult {
            within_budget: true,
            latency_us: 0,
            budget_us: invariants.performance_bounds.max_warm_us,
            ticks: None,
        })
    }

    async fn sign_receipt(&self, _receipt: &SigmaReceipt) -> Result<Signature, SigmaError> {
        // Cryptographic signing would go here
        Ok(Signature {
            algorithm: "none".to_string(),
            signature: "unsigned".to_string(),
            signer: "sigma-runtime".to_string(),
        })
    }

    fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0)
    }
}
```

---

## 3. Storage Backend Abstraction

### 3.1 Storage Trait (`knhk-ontology/src/storage.rs`)

```rust
use async_trait::async_trait;

/// Snapshot storage backend
#[async_trait]
pub trait SnapshotStorage {
    /// Store snapshot
    async fn store_snapshot(&self, snapshot: &SigmaSnapshot) -> Result<(), SigmaError>;

    /// Load snapshot by ID
    async fn load_snapshot(&self, id: SigmaSnapshotId) -> Result<Option<SigmaSnapshot>, SigmaError>;

    /// Delete snapshot (for cleanup)
    async fn delete_snapshot(&self, id: SigmaSnapshotId) -> Result<(), SigmaError>;

    /// List all snapshot IDs
    async fn list_snapshots(&self) -> Result<Vec<SigmaSnapshotId>, SigmaError>;

    /// Get storage statistics
    async fn stats(&self) -> Result<StorageStats, SigmaError>;
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total snapshots stored
    pub total_snapshots: usize,
    /// Total size in bytes
    pub total_size_bytes: u64,
    /// Oldest snapshot timestamp
    pub oldest_timestamp: Option<u64>,
    /// Newest snapshot timestamp
    pub newest_timestamp: Option<u64>,
}

/// In-memory storage (for testing)
pub struct MemoryStorage {
    snapshots: Arc<RwLock<HashMap<SigmaSnapshotId, SigmaSnapshot>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SnapshotStorage for MemoryStorage {
    async fn store_snapshot(&self, snapshot: &SigmaSnapshot) -> Result<(), SigmaError> {
        let mut snapshots = self.snapshots.write()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;
        snapshots.insert(snapshot.id, snapshot.clone());
        Ok(())
    }

    async fn load_snapshot(&self, id: SigmaSnapshotId) -> Result<Option<SigmaSnapshot>, SigmaError> {
        let snapshots = self.snapshots.read()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;
        Ok(snapshots.get(&id).cloned())
    }

    async fn delete_snapshot(&self, id: SigmaSnapshotId) -> Result<(), SigmaError> {
        let mut snapshots = self.snapshots.write()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;
        snapshots.remove(&id);
        Ok(())
    }

    async fn list_snapshots(&self) -> Result<Vec<SigmaSnapshotId>, SigmaError> {
        let snapshots = self.snapshots.read()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;
        Ok(snapshots.keys().copied().collect())
    }

    async fn stats(&self) -> Result<StorageStats, SigmaError> {
        let snapshots = self.snapshots.read()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;

        let timestamps: Vec<u64> = snapshots.values()
            .map(|s| s.metadata.timestamp)
            .collect();

        Ok(StorageStats {
            total_snapshots: snapshots.len(),
            total_size_bytes: 0, // TODO: calculate actual size
            oldest_timestamp: timestamps.iter().min().copied(),
            newest_timestamp: timestamps.iter().max().copied(),
        })
    }
}

/// Sled-based persistent storage
pub struct SledStorage {
    db: sled::Db,
}

impl SledStorage {
    pub fn new(path: &str) -> Result<Self, SigmaError> {
        let db = sled::open(path)
            .map_err(|e| SigmaError::StorageError(e.to_string()))?;
        Ok(Self { db })
    }
}

#[async_trait]
impl SnapshotStorage for SledStorage {
    async fn store_snapshot(&self, snapshot: &SigmaSnapshot) -> Result<(), SigmaError> {
        // Serialize snapshot
        let serialized = bincode::serialize(snapshot)
            .map_err(|e| SigmaError::SerializationError(e.to_string()))?;

        // Store in sled
        self.db.insert(snapshot.id.as_bytes(), serialized)
            .map_err(|e| SigmaError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn load_snapshot(&self, id: SigmaSnapshotId) -> Result<Option<SigmaSnapshot>, SigmaError> {
        // Load from sled
        let data = self.db.get(id.as_bytes())
            .map_err(|e| SigmaError::StorageError(e.to_string()))?;

        match data {
            Some(bytes) => {
                let snapshot = bincode::deserialize(&bytes)
                    .map_err(|e| SigmaError::SerializationError(e.to_string()))?;
                Ok(Some(snapshot))
            }
            None => Ok(None),
        }
    }

    async fn delete_snapshot(&self, id: SigmaSnapshotId) -> Result<(), SigmaError> {
        self.db.remove(id.as_bytes())
            .map_err(|e| SigmaError::StorageError(e.to_string()))?;
        Ok(())
    }

    async fn list_snapshots(&self) -> Result<Vec<SigmaSnapshotId>, SigmaError> {
        let mut ids = Vec::new();
        for item in self.db.iter() {
            let (key, _) = item.map_err(|e| SigmaError::StorageError(e.to_string()))?;
            if key.len() == 32 {
                let mut bytes = [0u8; 32];
                bytes.copy_from_slice(&key);
                ids.push(SigmaSnapshotId::from_bytes(bytes));
            }
        }
        Ok(ids)
    }

    async fn stats(&self) -> Result<StorageStats, SigmaError> {
        let snapshots = self.list_snapshots().await?;

        let mut timestamps = Vec::new();
        let mut total_size = 0u64;

        for id in &snapshots {
            if let Some(snapshot) = self.load_snapshot(*id).await? {
                timestamps.push(snapshot.metadata.timestamp);
                if let Some(data) = self.db.get(id.as_bytes())
                    .map_err(|e| SigmaError::StorageError(e.to_string()))? {
                    total_size += data.len() as u64;
                }
            }
        }

        Ok(StorageStats {
            total_snapshots: snapshots.len(),
            total_size_bytes: total_size,
            oldest_timestamp: timestamps.iter().min().copied(),
            newest_timestamp: timestamps.iter().max().copied(),
        })
    }
}
```

### 3.2 Receipt Store (`knhk-ontology/src/receipt_store.rs`)

```rust
use sled::Db;

/// Append-only receipt store
pub struct ReceiptStore {
    db: Option<Db>,
    memory: HashMap<SigmaSnapshotId, SigmaReceipt>,
}

impl ReceiptStore {
    /// Create new receipt store
    pub fn new(path: Option<&str>) -> Result<Self, SigmaError> {
        let db = match path {
            Some(p) => {
                let db_path = format!("{}/receipts", p);
                Some(sled::open(db_path)
                    .map_err(|e| SigmaError::StorageError(e.to_string()))?)
            }
            None => None,
        };

        Ok(Self {
            db,
            memory: HashMap::new(),
        })
    }

    /// Append receipt to log
    pub fn append(&mut self, receipt: SigmaReceipt) -> Result<(), SigmaError> {
        let id = receipt.snapshot_id;

        // Store in memory
        self.memory.insert(id, receipt.clone());

        // Store in persistent log (if available)
        if let Some(db) = &self.db {
            let serialized = bincode::serialize(&receipt)
                .map_err(|e| SigmaError::SerializationError(e.to_string()))?;

            db.insert(id.as_bytes(), serialized)
                .map_err(|e| SigmaError::StorageError(e.to_string()))?;
        }

        Ok(())
    }

    /// Get receipt by snapshot ID
    pub fn get(&self, id: SigmaSnapshotId) -> Result<Option<SigmaReceipt>, SigmaError> {
        // Check memory first
        if let Some(receipt) = self.memory.get(&id) {
            return Ok(Some(receipt.clone()));
        }

        // Check persistent store
        if let Some(db) = &self.db {
            if let Some(data) = db.get(id.as_bytes())
                .map_err(|e| SigmaError::StorageError(e.to_string()))? {
                let receipt = bincode::deserialize(&data)
                    .map_err(|e| SigmaError::SerializationError(e.to_string()))?;
                return Ok(Some(receipt));
            }
        }

        Ok(None)
    }

    /// List all receipts
    pub fn list(&self) -> Result<Vec<SigmaReceipt>, SigmaError> {
        let mut receipts: Vec<SigmaReceipt> = self.memory.values().cloned().collect();

        if let Some(db) = &self.db {
            for item in db.iter() {
                let (_, value) = item.map_err(|e| SigmaError::StorageError(e.to_string()))?;
                let receipt = bincode::deserialize(&value)
                    .map_err(|e| SigmaError::SerializationError(e.to_string()))?;
                receipts.push(receipt);
            }
        }

        // Deduplicate (memory takes precedence)
        receipts.sort_by_key(|r| r.timestamp);
        receipts.dedup_by_key(|r| r.snapshot_id);

        Ok(receipts)
    }
}
```

---

## 4. Error Types

### 4.1 Sigma Errors (`knhk-ontology/src/error.rs`)

```rust
use thiserror::Error;

/// Sigma runtime errors
#[derive(Error, Debug, Clone)]
pub enum SigmaError {
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(SigmaSnapshotId),

    #[error("Invalid snapshot ID: {0}")]
    InvalidSnapshotId(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(String),

    #[error("Store error: {0}")]
    StoreError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Lock error: {0}")]
    LockError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Invariant violation: {0}")]
    InvariantViolation(String),

    #[error("Performance violation: {0}")]
    PerformanceViolation(String),

    #[error("Signature error: {0}")]
    SignatureError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<std::io::Error> for SigmaError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}
```

---

## 5. Integration APIs

### 5.1 C FFI for Hot Path (`knhk-ontology/src/ffi.rs`)

```rust
use std::os::raw::c_char;
use std::ffi::{CStr, CString};

/// C-compatible snapshot descriptor (read-only)
#[repr(C)]
pub struct CSigmaSnapshot {
    /// Snapshot ID (32 bytes)
    pub id: [u8; 32],
    /// Parent ID (32 bytes, all zeros if None)
    pub parent_id: [u8; 32],
    /// Triple count
    pub triple_count: usize,
    /// Timestamp
    pub timestamp: u64,
    /// Sector name (null-terminated C string)
    pub sector: *const c_char,
}

/// Get current snapshot descriptor (for C hot path)
#[no_mangle]
pub extern "C" fn sigma_get_current_snapshot(
    runtime: *const SigmaRuntime,
    out: *mut CSigmaSnapshot,
) -> i32 {
    if runtime.is_null() || out.is_null() {
        return -1;
    }

    let runtime = unsafe { &*runtime };

    match runtime.snapshot_current() {
        Ok(id) => {
            match runtime.get_snapshot(id) {
                Ok(snapshot) => {
                    let descriptor = CSigmaSnapshot {
                        id: *snapshot.id.as_bytes(),
                        parent_id: snapshot.parent_id.map_or([0u8; 32], |p| *p.as_bytes()),
                        triple_count: snapshot.triple_count().unwrap_or(0),
                        timestamp: snapshot.metadata.timestamp,
                        sector: CString::new(snapshot.metadata.sector.clone())
                            .unwrap_or_default()
                            .into_raw(),
                    };
                    unsafe { *out = descriptor };
                    0
                }
                Err(_) => -2,
            }
        }
        Err(_) => -3,
    }
}

/// Free snapshot descriptor resources
#[no_mangle]
pub extern "C" fn sigma_free_snapshot_descriptor(desc: *mut CSigmaSnapshot) {
    if desc.is_null() {
        return;
    }

    unsafe {
        let desc_ref = &*desc;
        if !desc_ref.sector.is_null() {
            let _ = CString::from_raw(desc_ref.sector as *mut c_char);
        }
    }
}
```

### 5.2 Rust API for Warm Path (`knhk-ontology/src/lib.rs`)

```rust
//! KNHK Ontology Runtime - Autonomous Ontology System (Σ Plane)
//!
//! Manages versioned, immutable ontology snapshots with hard invariants.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub mod error;
pub mod ffi;
pub mod invariants;
pub mod overlay;
pub mod receipt;
pub mod receipt_store;
pub mod runtime;
pub mod snapshot;
pub mod storage;

// Re-exports
pub use error::SigmaError;
pub use invariants::{HardInvariants, PerformanceBounds};
pub use overlay::{RdfTriple, SigmaOverlay};
pub use receipt::{PerfResult, SigmaReceipt, Signature, ValidationResult};
pub use receipt_store::ReceiptStore;
pub use runtime::{SigmaConfig, SigmaRuntime, StorageBackend};
pub use snapshot::{SigmaMetadata, SigmaSnapshot, SigmaSnapshotId};
pub use storage::{MemoryStorage, SledStorage, SnapshotStorage, StorageStats};

/// Result type for sigma operations
pub type SigmaResult<T> = Result<T, SigmaError>;
```

### 5.3 CLI Integration (`knhk-cli` integration example)

```rust
// In knhk-cli/src/commands/sigma.rs

use clap::Parser;
use knhk_ontology::{SigmaRuntime, SigmaConfig, SigmaOverlay};

#[derive(Parser)]
pub struct SigmaCommands {
    #[command(subcommand)]
    command: SigmaSubcommands,
}

#[derive(Parser)]
enum SigmaSubcommands {
    /// Get current snapshot ID
    Current,
    /// List all snapshots
    List,
    /// Show snapshot details
    Show {
        /// Snapshot ID (hex)
        #[arg(long)]
        id: String,
    },
    /// Create overlay and new snapshot
    Overlay {
        /// Base snapshot ID
        #[arg(long)]
        base: String,
        /// Description
        #[arg(long)]
        description: String,
    },
    /// Promote snapshot to active
    Promote {
        /// Snapshot ID to promote
        #[arg(long)]
        id: String,
    },
    /// Validate snapshot
    Validate {
        /// Snapshot ID to validate
        #[arg(long)]
        id: String,
    },
}

pub async fn handle_sigma_command(cmd: SigmaCommands) -> Result<(), Box<dyn std::error::Error>> {
    let config = SigmaConfig::default();
    let runtime = SigmaRuntime::new(config).await?;

    match cmd.command {
        SigmaSubcommands::Current => {
            let id = runtime.snapshot_current()?;
            println!("Current snapshot: {}", id);
        }
        SigmaSubcommands::List => {
            // Implementation
            println!("Listing snapshots...");
        }
        SigmaSubcommands::Show { id } => {
            // Implementation
            println!("Showing snapshot: {}", id);
        }
        SigmaSubcommands::Overlay { base, description } => {
            // Implementation
            println!("Creating overlay on: {}", base);
        }
        SigmaSubcommands::Promote { id } => {
            // Implementation
            println!("Promoting snapshot: {}", id);
        }
        SigmaSubcommands::Validate { id } => {
            // Implementation
            println!("Validating snapshot: {}", id);
        }
    }

    Ok(())
}
```

---

## 6. Crate Structure

### 6.1 Directory Layout

```
rust/knhk-ontology/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API and re-exports
│   ├── error.rs            # Error types
│   ├── snapshot.rs         # Snapshot types
│   ├── overlay.rs          # Overlay/delta types
│   ├── receipt.rs          # Validation receipt types
│   ├── invariants.rs       # Hard invariants
│   ├── runtime.rs          # Sigma runtime
│   ├── storage.rs          # Storage backend trait
│   ├── receipt_store.rs    # Receipt append-only log
│   ├── ffi.rs              # C FFI for hot path
│   └── validators/         # Validation modules
│       ├── mod.rs
│       ├── shacl.rs        # SHACL validation
│       ├── static_val.rs   # Static validation
│       ├── dynamic_val.rs  # Dynamic validation
│       └── perf_val.rs     # Performance validation
├── tests/
│   ├── integration.rs      # Integration tests
│   ├── snapshot_test.rs    # Snapshot tests
│   ├── overlay_test.rs     # Overlay tests
│   └── performance_test.rs # Performance tests
├── benches/
│   ├── snapshot_bench.rs   # Snapshot benchmarks
│   └── promotion_bench.rs  # Promotion latency benchmarks
└── examples/
    ├── basic_usage.rs      # Basic usage example
    └── change_engine.rs    # Change engine integration
```

### 6.2 Cargo.toml

```toml
[package]
name = "knhk-ontology"
version = "1.0.0"
edition = "2021"
license = "MIT"
authors = ["KNHK Team"]
description = "Autonomous ontology runtime system (Σ plane)"

[dependencies]
# RDF storage
oxigraph = { workspace = true }

# Persistent storage
sled = { workspace = true }

# Async runtime
tokio = { workspace = true, features = ["full"] }
async-trait = "0.1"

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
bincode = { workspace = true }

# Hashing
sha2 = { workspace = true }
blake3 = { workspace = true }
hex = { workspace = true }

# Error handling
thiserror = { workspace = true }
anyhow = { workspace = true }

# Collections
hashbrown = { workspace = true }

# Tracing
tracing = { workspace = true }

[dev-dependencies]
criterion = { workspace = true }
tempfile = { workspace = true }
proptest = { workspace = true }

[lib]
crate-type = ["lib", "cdylib", "staticlib"]

[[bench]]
name = "snapshot_bench"
harness = false

[[bench]]
name = "promotion_bench"
harness = false
```

---

## 7. Performance Design

### 7.1 Latency Targets

| Operation | Target | Budget |
|-----------|--------|--------|
| Snapshot access (read) | ≤100ns | Hot path |
| Snapshot promotion | ≤1μs | Warm path |
| Overlay application | ≤500μs | Warm path |
| SHACL validation | ≤100ms | Cold path |
| Receipt storage | ≤10ms | Cold path |

### 7.2 Concurrency Strategy

**Lock Strategy:**
- **Σ_current pointer**: RwLock with minimal lock scope (≤10 CPU cycles)
- **Snapshot cache**: RwLock for map updates, Arc cloning for reads
- **Receipt store**: Async Mutex (append-only, low contention)

**Lock-Free Reads:**
```rust
// Zero-copy snapshot access
let id = runtime.snapshot_current()?;  // Read lock (≤10 cycles)
let snapshot = runtime.get_snapshot(id)?;  // Arc clone (lock-free)
// snapshot can be used without holding any locks
```

**Atomic Promotion:**
```rust
// CAS operation for snapshot promotion
runtime.promote_snapshot(new_id).await?;  // Single write lock
```

### 7.3 Memory Layout

**Snapshot Storage:**
- Oxigraph store: Memory-mapped files for large graphs
- Metadata: Small structs (≤1KB each)
- Arc overhead: 16 bytes per snapshot reference

**Memory Cost Estimates:**
- Small ontology (1K triples): ~100KB per snapshot
- Medium ontology (100K triples): ~10MB per snapshot
- Large ontology (1M triples): ~100MB per snapshot

**Memory Optimization:**
- Snapshot deduplication via content addressing
- LRU cache for recently accessed snapshots
- Memory-mapped files for cold snapshots

---

## 8. Integration Points

### 8.1 With ggen (Code Generator)

```rust
// ggen queries current snapshot as cache key
let sigma_id = runtime.snapshot_current()?;
let cache_key = format!("ggen:{}", sigma_id);

// Generated code is tagged with snapshot ID
let generated_code = ggen::generate(snapshot, cache_key)?;
```

### 8.2 With Validators (ΔΣ Engine)

```rust
// Validator proposes change
let overlay = SigmaOverlay::new(base_id)
    .with_description("Add new workflow pattern".to_string());

// Apply and validate
let new_id = runtime.apply_overlay(base_id, overlay).await?;
let receipt = runtime.validate_snapshot(new_id, &invariants).await?;

if receipt.is_valid() {
    // Promote to active
    runtime.promote_snapshot(new_id).await?;
} else {
    // Reject change
    eprintln!("Validation failed: {}", receipt.static_validation.message);
}
```

### 8.3 With CLI

```bash
# Query current snapshot
knhk sigma current

# Validate snapshot
knhk sigma validate --id abc123

# Promote snapshot
knhk sigma promote --id def456

# Show snapshot details
knhk sigma show --id abc123
```

### 8.4 With Change Engine (Webhooks)

```rust
use axum::{Router, Json};

#[derive(Deserialize)]
struct ChangeProposal {
    base_id: String,
    description: String,
    additions: Vec<RdfTriple>,
    removals: Vec<RdfTriple>,
}

async fn handle_proposal(
    Json(proposal): Json<ChangeProposal>,
    runtime: Arc<SigmaRuntime>,
) -> Result<Json<SigmaReceipt>, StatusCode> {
    // Parse base ID
    let base_id = SigmaSnapshotId::from_hex(&proposal.base_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Create overlay
    let mut overlay = SigmaOverlay::new(base_id)
        .with_description(proposal.description);
    overlay.additions = proposal.additions;
    overlay.removals = proposal.removals;

    // Apply and validate
    let new_id = runtime.apply_overlay(base_id, overlay).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let invariants = HardInvariants::default();
    let receipt = runtime.validate_snapshot(new_id, &invariants).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Auto-promote if valid
    if receipt.is_valid() {
        runtime.promote_snapshot(new_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(receipt))
}

pub fn sigma_router(runtime: Arc<SigmaRuntime>) -> Router {
    Router::new()
        .route("/propose", axum::routing::post(handle_proposal))
}
```

---

## 9. Test Strategy

### 9.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_snapshot_creation() {
        let config = SigmaConfig::default();
        let runtime = SigmaRuntime::new(config).await.unwrap();

        let id = runtime.snapshot_current().unwrap();
        let snapshot = runtime.get_snapshot(id).unwrap();

        assert_eq!(snapshot.triple_count().unwrap(), 0);
        assert_eq!(snapshot.metadata.version, "0.0.0");
    }

    #[tokio::test]
    async fn test_overlay_application() {
        let config = SigmaConfig::default();
        let runtime = SigmaRuntime::new(config).await.unwrap();

        let base_id = runtime.snapshot_current().unwrap();

        let mut overlay = SigmaOverlay::new(base_id);
        overlay.add_triple(RdfTriple {
            subject: "http://example.org/s".to_string(),
            predicate: "http://example.org/p".to_string(),
            object: "http://example.org/o".to_string(),
        });

        let new_id = runtime.apply_overlay(base_id, overlay).await.unwrap();
        assert_ne!(base_id, new_id);
    }

    #[tokio::test]
    async fn test_snapshot_promotion() {
        let config = SigmaConfig::default();
        let runtime = SigmaRuntime::new(config).await.unwrap();

        let old_id = runtime.snapshot_current().unwrap();
        let overlay = SigmaOverlay::new(old_id);
        let new_id = runtime.apply_overlay(old_id, overlay).await.unwrap();

        runtime.promote_snapshot(new_id).await.unwrap();

        let current = runtime.snapshot_current().unwrap();
        assert_eq!(current, new_id);
    }
}
```

### 9.2 Integration Tests

```rust
// tests/integration.rs

use knhk_ontology::*;

#[tokio::test]
async fn test_full_workflow() {
    // Create runtime
    let config = SigmaConfig::default();
    let runtime = SigmaRuntime::new(config).await.unwrap();

    // Get initial snapshot
    let base_id = runtime.snapshot_current().unwrap();

    // Create overlay
    let overlay = SigmaOverlay::new(base_id)
        .with_description("Test change".to_string());

    // Apply overlay
    let new_id = runtime.apply_overlay(base_id, overlay).await.unwrap();

    // Validate
    let invariants = HardInvariants::default();
    let receipt = runtime.validate_snapshot(new_id, &invariants).await.unwrap();
    assert!(receipt.is_valid());

    // Store receipt
    runtime.store_receipt(receipt.clone()).await.unwrap();

    // Retrieve receipt
    let stored = runtime.get_receipt(new_id).await.unwrap();
    assert!(stored.is_some());

    // Promote
    runtime.promote_snapshot(new_id).await.unwrap();
    assert_eq!(runtime.snapshot_current().unwrap(), new_id);
}
```

### 9.3 Performance Tests

```rust
// benches/promotion_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knhk_ontology::*;

fn bench_snapshot_promotion(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let config = SigmaConfig::default();
    let runtime = rt.block_on(async {
        SigmaRuntime::new(config).await.unwrap()
    });

    let base_id = runtime.snapshot_current().unwrap();
    let overlay = SigmaOverlay::new(base_id);
    let new_id = rt.block_on(async {
        runtime.apply_overlay(base_id, overlay).await.unwrap()
    });

    c.bench_function("snapshot_promotion", |b| {
        b.iter(|| {
            rt.block_on(async {
                runtime.promote_snapshot(black_box(new_id)).await.unwrap()
            })
        })
    });
}

criterion_group!(benches, bench_snapshot_promotion);
criterion_main!(benches);
```

### 9.4 Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_snapshot_id_deterministic(seed in any::<u64>()) {
        // Same content should produce same snapshot ID
        let store1 = create_store_with_seed(seed);
        let store2 = create_store_with_seed(seed);

        let id1 = SigmaSnapshot::compute_id(&store1, &None, &metadata).unwrap();
        let id2 = SigmaSnapshot::compute_id(&store2, &None, &metadata).unwrap();

        prop_assert_eq!(id1, id2);
    }
}
```

---

## 10. OpenTelemetry Integration

### 10.1 Telemetry Schema

```yaml
# registry/sigma-runtime.yaml
groups:
  - id: sigma.runtime
    prefix: sigma.runtime
    spans:
      - id: snapshot.create
        span_name: "sigma.snapshot.create"
        attributes:
          - id: snapshot.id
            type: string
            brief: "Snapshot ID (hex)"
          - id: snapshot.parent_id
            type: string
            brief: "Parent snapshot ID"
          - id: snapshot.triple_count
            type: int
            brief: "Number of triples"
        events:
          - id: overlay.applied
            event_name: "overlay.applied"

      - id: snapshot.validate
        span_name: "sigma.snapshot.validate"
        attributes:
          - id: validation.passed
            type: boolean
          - id: validation.ticks
            type: int
            brief: "Hot path ticks (if applicable)"

      - id: snapshot.promote
        span_name: "sigma.snapshot.promote"
        attributes:
          - id: promotion.latency_us
            type: int
            brief: "Promotion latency in microseconds"
```

### 10.2 Instrumentation

```rust
use tracing::{instrument, info, warn};

impl SigmaRuntime {
    #[instrument(skip(self, overlay), fields(
        snapshot.id = %base,
        snapshot.change_count = overlay.change_count()
    ))]
    pub async fn apply_overlay(
        &self,
        base: SigmaSnapshotId,
        overlay: SigmaOverlay,
    ) -> Result<SigmaSnapshotId, SigmaError> {
        info!("Applying overlay to snapshot");

        let new_id = /* ... */;

        info!("Overlay applied successfully");
        Ok(new_id)
    }

    #[instrument(skip(self), fields(
        snapshot.id = %id,
        promotion.latency_us
    ))]
    pub async fn promote_snapshot(&self, id: SigmaSnapshotId) -> Result<(), SigmaError> {
        let start = std::time::Instant::now();

        // Atomic promotion
        let mut current = self.current.write()
            .map_err(|e| SigmaError::LockError(e.to_string()))?;
        *current = id;

        let latency_us = start.elapsed().as_micros() as u64;
        tracing::Span::current().record("promotion.latency_us", latency_us);

        if latency_us > 1 {
            warn!("Promotion latency exceeded 1μs target: {}μs", latency_us);
        }

        Ok(())
    }
}
```

---

## 11. Summary & Next Steps

### 11.1 Implementation Priorities

1. **Phase 1: Core Infrastructure** (Week 1-2)
   - Snapshot data structures
   - Memory storage backend
   - Basic runtime operations
   - Unit tests

2. **Phase 2: Persistence** (Week 3)
   - Sled storage backend
   - Receipt store
   - Integration tests

3. **Phase 3: Validation** (Week 4)
   - SHACL validation
   - Hard invariants checking
   - Performance validation

4. **Phase 4: Integration** (Week 5-6)
   - C FFI for hot path
   - CLI commands
   - Webhook API
   - ggen integration

5. **Phase 5: Optimization** (Week 7-8)
   - Performance tuning
   - Memory optimization
   - Benchmarking
   - Production hardening

### 11.2 Success Criteria

- [ ] All unit tests pass
- [ ] Integration tests pass
- [ ] Snapshot promotion latency ≤1μs (p99)
- [ ] Snapshot access latency ≤100ns (p99)
- [ ] Weaver validation passes
- [ ] Zero unwrap/expect in production code
- [ ] Clippy warnings = 0
- [ ] Test coverage ≥80%
- [ ] Documentation complete
- [ ] C FFI tested with hot path

### 11.3 Open Questions

1. **SHACL Validation**: Which library to use? (rudof, shacl-rs, or custom?)
2. **Cryptographic Signing**: ed25519? RSA? Custom PKI?
3. **RDF Canonicalization**: Which algorithm? (RDF Dataset Canonicalization?)
4. **Snapshot Garbage Collection**: When to prune old snapshots?
5. **Distributed Coordination**: Raft? Paxos? Custom?

---

## Appendix A: Type Summary

### Core Types

```rust
// Identifiers
pub struct SigmaSnapshotId([u8; 32]);

// Snapshots
pub struct SigmaSnapshot { /* ... */ }
pub struct SigmaMetadata { /* ... */ }
pub struct SigmaOverlay { /* ... */ }
pub struct RdfTriple { /* ... */ }

// Validation
pub struct SigmaReceipt { /* ... */ }
pub struct ValidationResult { /* ... */ }
pub struct PerfResult { /* ... */ }
pub struct Signature { /* ... */ }

// Invariants
pub struct HardInvariants { /* ... */ }
pub struct PerformanceBounds { /* ... */ }

// Runtime
pub struct SigmaRuntime { /* ... */ }
pub struct SigmaConfig { /* ... */ }
pub enum StorageBackend { /* ... */ }

// Storage
pub trait SnapshotStorage { /* ... */ }
pub struct MemoryStorage { /* ... */ }
pub struct SledStorage { /* ... */ }
pub struct ReceiptStore { /* ... */ }

// Errors
pub enum SigmaError { /* ... */ }
```

### Trait Summary

```rust
#[async_trait]
pub trait SnapshotStorage {
    async fn store_snapshot(&self, snapshot: &SigmaSnapshot) -> Result<(), SigmaError>;
    async fn load_snapshot(&self, id: SigmaSnapshotId) -> Result<Option<SigmaSnapshot>, SigmaError>;
    async fn delete_snapshot(&self, id: SigmaSnapshotId) -> Result<(), SigmaError>;
    async fn list_snapshots(&self) -> Result<Vec<SigmaSnapshotId>, SigmaError>;
    async fn stats(&self) -> Result<StorageStats, SigmaError>;
}
```

---

**End of Design Document**

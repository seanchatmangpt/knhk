# KNHK Autonomous Ontology Runtime - API Quick Reference

**Version:** 1.0.0
**Crate:** `knhk-ontology`

## Core Types

### SigmaSnapshotId

```rust
pub struct SigmaSnapshotId([u8; 32]);

impl SigmaSnapshotId {
    pub fn from_bytes(bytes: [u8; 32]) -> Self;
    pub fn from_hex(s: &str) -> Result<Self, SigmaError>;
    pub fn to_hex(&self) -> String;
    pub fn as_bytes(&self) -> &[u8; 32];
}
```

### SigmaSnapshot

```rust
pub struct SigmaSnapshot {
    pub id: SigmaSnapshotId,
    pub parent_id: Option<SigmaSnapshotId>,
    pub store: Arc<OxigraphStore>,
    pub metadata: SigmaMetadata,
    pub validation_receipt: Option<SigmaReceipt>,
}

impl SigmaSnapshot {
    pub fn new(
        store: Arc<OxigraphStore>,
        parent_id: Option<SigmaSnapshotId>,
        metadata: SigmaMetadata,
    ) -> Result<Self, SigmaError>;

    pub fn with_receipt(self, receipt: SigmaReceipt) -> Self;
    pub fn triple_count(&self) -> Result<usize, SigmaError>;
    pub fn is_validated(&self) -> bool;
}
```

### SigmaMetadata

```rust
pub struct SigmaMetadata {
    pub version: String,                              // Semantic version
    pub timestamp: u64,                               // Unix epoch μs
    pub sector: String,                               // Namespace
    pub description: Option<String>,                  // Human description
    pub shacl_rules: Option<String>,                  // SHACL validation rules
    pub custom: HashMap<String, String>,              // Custom metadata
}
```

### SigmaOverlay

```rust
pub struct SigmaOverlay {
    pub base_id: SigmaSnapshotId,
    pub additions: Vec<RdfTriple>,
    pub removals: Vec<RdfTriple>,
    pub timestamp: u64,
    pub description: Option<String>,
}

impl SigmaOverlay {
    pub fn new(base_id: SigmaSnapshotId) -> Self;
    pub fn add_triple(&mut self, triple: RdfTriple);
    pub fn remove_triple(&mut self, triple: RdfTriple);
    pub fn with_description(self, desc: String) -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn change_count(&self) -> usize;
}
```

### RdfTriple

```rust
pub struct RdfTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

// Conversion from oxigraph::model::Triple
impl From<Triple> for RdfTriple;
```

### SigmaReceipt

```rust
pub struct SigmaReceipt {
    pub snapshot_id: SigmaSnapshotId,
    pub parent_id: Option<SigmaSnapshotId>,
    pub delta_description: Option<String>,
    pub static_validation: ValidationResult,
    pub dynamic_validation: Option<ValidationResult>,
    pub perf_validation: Option<PerfResult>,
    pub signature: Option<Signature>,
    pub timestamp: u64,
}

impl SigmaReceipt {
    pub fn new(
        snapshot_id: SigmaSnapshotId,
        parent_id: Option<SigmaSnapshotId>,
        static_validation: ValidationResult,
    ) -> Self;

    pub fn is_valid(&self) -> bool;
    pub fn with_dynamic_validation(self, validation: ValidationResult) -> Self;
    pub fn with_perf_validation(self, perf: PerfResult) -> Self;
    pub fn with_signature(self, sig: Signature) -> Self;
}
```

### ValidationResult

```rust
pub struct ValidationResult {
    pub passed: bool,
    pub message: String,
    pub timestamp: u64,
    pub validator: String,
}

impl ValidationResult {
    pub fn success(message: String, validator: String) -> Self;
    pub fn failure(message: String, validator: String) -> Self;
}
```

### PerfResult

```rust
pub struct PerfResult {
    pub within_budget: bool,
    pub latency_us: u64,
    pub budget_us: u64,
    pub ticks: Option<u32>,
}

impl PerfResult {
    pub fn is_compliant(&self) -> bool;
}
```

### HardInvariants

```rust
pub struct HardInvariants {
    pub no_retrocausation: bool,
    pub type_soundness: bool,
    pub guard_preservation: bool,
    pub slo_compliance: bool,
    pub performance_bounds: PerformanceBounds,
}

impl HardInvariants {
    pub fn all_enabled(&self) -> bool;
    pub fn minimal() -> Self;
}

impl Default for HardInvariants;
```

### PerformanceBounds

```rust
pub struct PerformanceBounds {
    pub max_hot_ticks: u32,           // Default: 8 (Chatman Constant)
    pub max_warm_us: u64,             // Default: 500
    pub max_cold_ms: u64,             // Default: 5000
    pub max_promotion_us: u64,        // Default: 1
}

impl Default for PerformanceBounds;
```

---

## Runtime API

### SigmaRuntime

```rust
pub struct SigmaRuntime { /* ... */ }

impl SigmaRuntime {
    /// Create new runtime
    pub async fn new(config: SigmaConfig) -> Result<Self, SigmaError>;

    /// Get current active snapshot ID
    pub fn snapshot_current(&self) -> Result<SigmaSnapshotId, SigmaError>;

    /// Get snapshot by ID
    pub fn get_snapshot(&self, id: SigmaSnapshotId)
        -> Result<Arc<SigmaSnapshot>, SigmaError>;

    /// Apply overlay to create new snapshot
    pub async fn apply_overlay(
        &self,
        base: SigmaSnapshotId,
        overlay: SigmaOverlay,
    ) -> Result<SigmaSnapshotId, SigmaError>;

    /// Validate snapshot against invariants
    pub async fn validate_snapshot(
        &self,
        id: SigmaSnapshotId,
        invariants: &HardInvariants,
    ) -> Result<SigmaReceipt, SigmaError>;

    /// Promote snapshot to active (atomic CAS)
    pub async fn promote_snapshot(&self, id: SigmaSnapshotId)
        -> Result<(), SigmaError>;

    /// Store validation receipt
    pub async fn store_receipt(&self, receipt: SigmaReceipt)
        -> Result<(), SigmaError>;

    /// Get receipt for snapshot
    pub async fn get_receipt(&self, id: SigmaSnapshotId)
        -> Result<Option<SigmaReceipt>, SigmaError>;
}
```

### SigmaConfig

```rust
pub struct SigmaConfig {
    pub storage_backend: StorageBackend,
    pub storage_path: Option<String>,
    pub enable_receipts: bool,
    pub enable_signatures: bool,
    pub default_invariants: HardInvariants,
}

impl Default for SigmaConfig;
```

### StorageBackend

```rust
pub enum StorageBackend {
    Memory,      // In-memory (testing)
    Sled,        // Embedded DB (production)
    RocksDB,     // High-performance (future)
}
```

---

## Storage API

### SnapshotStorage Trait

```rust
#[async_trait]
pub trait SnapshotStorage {
    async fn store_snapshot(&self, snapshot: &SigmaSnapshot)
        -> Result<(), SigmaError>;

    async fn load_snapshot(&self, id: SigmaSnapshotId)
        -> Result<Option<SigmaSnapshot>, SigmaError>;

    async fn delete_snapshot(&self, id: SigmaSnapshotId)
        -> Result<(), SigmaError>;

    async fn list_snapshots(&self)
        -> Result<Vec<SigmaSnapshotId>, SigmaError>;

    async fn stats(&self)
        -> Result<StorageStats, SigmaError>;
}
```

### MemoryStorage

```rust
pub struct MemoryStorage { /* ... */ }

impl MemoryStorage {
    pub fn new() -> Self;
}

impl SnapshotStorage for MemoryStorage { /* ... */ }
```

### SledStorage

```rust
pub struct SledStorage { /* ... */ }

impl SledStorage {
    pub fn new(path: &str) -> Result<Self, SigmaError>;
}

impl SnapshotStorage for SledStorage { /* ... */ }
```

### ReceiptStore

```rust
pub struct ReceiptStore { /* ... */ }

impl ReceiptStore {
    pub fn new(path: Option<&str>) -> Result<Self, SigmaError>;
    pub fn append(&mut self, receipt: SigmaReceipt) -> Result<(), SigmaError>;
    pub fn get(&self, id: SigmaSnapshotId) -> Result<Option<SigmaReceipt>, SigmaError>;
    pub fn list(&self) -> Result<Vec<SigmaReceipt>, SigmaError>;
}
```

---

## Error Types

### SigmaError

```rust
pub enum SigmaError {
    SnapshotNotFound(SigmaSnapshotId),
    InvalidSnapshotId(String),
    InvalidVersion(String),
    StoreError(String),
    StorageError(String),
    SerializationError(String),
    ParseError(String),
    LockError(String),
    ConfigError(String),
    ValidationError(String),
    InvariantViolation(String),
    PerformanceViolation(String),
    SignatureError(String),
    IoError(String),
    Unknown(String),
}

impl std::fmt::Display for SigmaError;
impl std::error::Error for SigmaError;
impl From<std::io::Error> for SigmaError;
```

---

## C FFI API

### C Types

```c
// Snapshot descriptor (read-only)
typedef struct {
    uint8_t id[32];              // Snapshot ID
    uint8_t parent_id[32];       // Parent ID (all zeros if None)
    size_t triple_count;         // Number of triples
    uint64_t timestamp;          // Timestamp
    const char* sector;          // Sector name (null-terminated)
} CSigmaSnapshot;
```

### C Functions

```c
// Get current snapshot descriptor
int sigma_get_current_snapshot(
    const SigmaRuntime* runtime,
    CSigmaSnapshot* out
);

// Free snapshot descriptor resources
void sigma_free_snapshot_descriptor(CSigmaSnapshot* desc);
```

### Return Codes

```c
 0  - Success
-1  - Invalid arguments (null pointers)
-2  - Snapshot not found
-3  - Runtime error
```

---

## Usage Examples

### Basic Usage

```rust
use knhk_ontology::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create runtime
    let config = SigmaConfig::default();
    let runtime = SigmaRuntime::new(config).await?;

    // Get current snapshot
    let id = runtime.snapshot_current()?;
    println!("Current: {}", id);

    // Create overlay
    let mut overlay = SigmaOverlay::new(id);
    overlay.add_triple(RdfTriple {
        subject: "http://ex.org/s".to_string(),
        predicate: "http://ex.org/p".to_string(),
        object: "http://ex.org/o".to_string(),
    });

    // Apply overlay
    let new_id = runtime.apply_overlay(id, overlay).await?;

    // Validate
    let invariants = HardInvariants::default();
    let receipt = runtime.validate_snapshot(new_id, &invariants).await?;

    if receipt.is_valid() {
        // Promote
        runtime.promote_snapshot(new_id).await?;
        println!("Promoted: {}", new_id);
    }

    Ok(())
}
```

### With Persistence

```rust
use knhk_ontology::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SigmaConfig {
        storage_backend: StorageBackend::Sled,
        storage_path: Some("~/.knhk/sigma".to_string()),
        enable_receipts: true,
        ..Default::default()
    };

    let runtime = SigmaRuntime::new(config).await?;
    // Use runtime...

    Ok(())
}
```

### HTTP API Integration

```rust
use axum::{Router, Json, extract::State};
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct CreateRequest {
    description: String,
    additions: Vec<RdfTriple>,
}

async fn create_snapshot(
    State(runtime): State<Arc<SigmaRuntime>>,
    Json(req): Json<CreateRequest>,
) -> Result<Json<String>, StatusCode> {
    let base_id = runtime.snapshot_current()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut overlay = SigmaOverlay::new(base_id)
        .with_description(req.description);

    for triple in req.additions {
        overlay.add_triple(triple);
    }

    let new_id = runtime.apply_overlay(base_id, overlay).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(new_id.to_hex()))
}

fn app(runtime: Arc<SigmaRuntime>) -> Router {
    Router::new()
        .route("/api/snapshots", axum::routing::post(create_snapshot))
        .with_state(runtime)
}
```

---

## CLI Commands

### Basic Commands

```bash
# Get current snapshot
knhk sigma current

# List all snapshots
knhk sigma list

# Show snapshot details
knhk sigma show <id>
knhk sigma show current

# Create new snapshot
knhk sigma create \
    --description "Add workflow" \
    --add "http://ex.org/s,http://ex.org/p,http://ex.org/o"

# Validate snapshot
knhk sigma validate <id>
knhk sigma validate <id> --detailed

# Promote snapshot
knhk sigma promote <id>
knhk sigma promote <id> --skip-validation

# Export snapshot
knhk sigma export <id> --output snapshot.ttl --format turtle
```

### Advanced Commands

```bash
# Show snapshot lineage
knhk sigma lineage <id>

# Diff two snapshots
knhk sigma diff <id1> <id2>

# Rollback to previous snapshot
knhk sigma rollback

# Garbage collect old snapshots
knhk sigma gc --keep 10
```

---

## Performance Targets

| Operation | Target | Budget |
|-----------|--------|--------|
| Snapshot access | ≤100ns | Hot |
| Snapshot promotion | ≤1μs | Warm |
| Overlay application | ≤500μs | Warm |
| SHACL validation | ≤100ms | Cold |
| Receipt storage | ≤10ms | Cold |

---

## Telemetry Attributes

### Spans

- `sigma.snapshot.create` - Snapshot creation
- `sigma.snapshot.validate` - Snapshot validation
- `sigma.snapshot.promote` - Snapshot promotion

### Attributes

- `sigma.snapshot.id` - Snapshot ID (hex)
- `sigma.snapshot.parent_id` - Parent snapshot ID
- `sigma.snapshot.triple_count` - Triple count
- `sigma.validation.passed` - Validation result
- `sigma.validation.ticks` - Hot path ticks
- `sigma.promotion.latency_us` - Promotion latency

### Events

- `overlay.applied` - Overlay applied to base

---

## Common Patterns

### Pattern: Safe Concurrent Access

```rust
// Read lock for current ID
let id = runtime.snapshot_current()?;

// Lock-free snapshot access (Arc clone)
let snapshot = runtime.get_snapshot(id)?;

// Use snapshot without locks
for triple in snapshot.store.iter() {
    // Process triple
}
```

### Pattern: Validation Pipeline

```rust
async fn validate_and_promote(
    runtime: &SigmaRuntime,
    overlay: SigmaOverlay,
) -> Result<SigmaSnapshotId, SigmaError> {
    let base_id = overlay.base_id;

    // Apply
    let new_id = runtime.apply_overlay(base_id, overlay).await?;

    // Validate
    let invariants = HardInvariants::default();
    let receipt = runtime.validate_snapshot(new_id, &invariants).await?;

    if !receipt.is_valid() {
        return Err(SigmaError::ValidationError(
            receipt.static_validation.message
        ));
    }

    // Store receipt
    runtime.store_receipt(receipt).await?;

    // Promote
    runtime.promote_snapshot(new_id).await?;

    Ok(new_id)
}
```

### Pattern: Lineage Traversal

```rust
async fn get_lineage(
    runtime: &SigmaRuntime,
    id: SigmaSnapshotId,
) -> Result<Vec<SigmaSnapshot>, SigmaError> {
    let mut lineage = Vec::new();
    let mut current = Some(id);

    while let Some(id) = current {
        let snapshot = runtime.get_snapshot(id)?;
        current = snapshot.parent_id;
        lineage.push(snapshot.as_ref().clone());
    }

    Ok(lineage)
}
```

---

## Dependencies

### Required

```toml
oxigraph = "0.5"
sled = "0.34"
tokio = { version = "1.48", features = ["full"] }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
bincode = "1.3"
sha2 = "0.10"
blake3 = "1.8"
hex = "0.4"
thiserror = "2.0"
anyhow = "1.0"
hashbrown = "0.15"
tracing = "0.1"
```

### Optional

```toml
# For CLI
clap = { version = "4.5", features = ["derive"] }

# For HTTP API
axum = "0.8"
tower = "0.5"

# For testing
criterion = "0.5"
tempfile = "3.10"
proptest = "1.0"
```

---

## Feature Flags

```toml
[features]
default = ["std", "receipts"]
std = []
receipts = []
signatures = []
cli = ["dep:clap"]
http = ["dep:axum", "dep:tower"]
```

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | 2025-11-16 | Initial design |

---

**End of API Reference**

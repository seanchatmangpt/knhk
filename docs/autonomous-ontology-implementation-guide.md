# KNHK Autonomous Ontology Runtime - Implementation Guide

**Version:** 1.0.0
**Date:** 2025-11-16
**Companion to:** autonomous-ontology-runtime-design.md

## Quick Start

### 1. Add Crate to Workspace

```bash
cd /home/user/knhk/rust
mkdir -p knhk-ontology/src
```

Edit `/home/user/knhk/rust/Cargo.toml`:

```toml
[workspace]
members = [
    # ... existing members ...
    "knhk-ontology",
]

[workspace.dependencies]
# Add new dependencies
async-trait = "0.1"
```

### 2. Create Cargo.toml

Create `/home/user/knhk/rust/knhk-ontology/Cargo.toml`:

```toml
[package]
name = "knhk-ontology"
version = "1.0.0"
edition = "2021"
license = "MIT"
authors = ["KNHK Team"]
description = "Autonomous ontology runtime system (Σ plane)"
repository = "https://github.com/seanchatmangpt/knhk"

[dependencies]
# Core dependencies from workspace
oxigraph = { workspace = true }
sled = { workspace = true }
tokio = { workspace = true }
async-trait = "0.1"

# Serialization
serde = { workspace = true }
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
tokio-test = { workspace = true }

[lib]
crate-type = ["lib", "cdylib", "staticlib"]

[[bench]]
name = "snapshot_bench"
harness = false

[[bench]]
name = "promotion_bench"
harness = false
```

### 3. Implementation Order

**Phase 1: Minimal Working System (Day 1-2)**

1. Error types (`src/error.rs`)
2. Snapshot types (`src/snapshot.rs`)
3. Runtime skeleton (`src/runtime.rs`)
4. Memory storage (`src/storage.rs`)
5. Basic tests

**Phase 2: Persistence (Day 3-4)**

1. Overlay types (`src/overlay.rs`)
2. Receipt types (`src/receipt.rs`)
3. Receipt store (`src/receipt_store.rs`)
4. Sled storage backend
5. Integration tests

**Phase 3: Validation (Day 5-6)**

1. Invariants (`src/invariants.rs`)
2. Static validation
3. Dynamic validation
4. Performance validation
5. Validation tests

**Phase 4: Integration (Day 7-8)**

1. C FFI (`src/ffi.rs`)
2. CLI commands
3. HTTP API
4. Documentation
5. End-to-end tests

---

## Detailed Implementation Examples

### Example 1: Basic Usage

```rust
use knhk_ontology::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create runtime with default config
    let config = SigmaConfig::default();
    let runtime = SigmaRuntime::new(config).await?;

    // Get current snapshot
    let current_id = runtime.snapshot_current()?;
    println!("Current snapshot: {}", current_id);

    // Create an overlay (change proposal)
    let mut overlay = SigmaOverlay::new(current_id);
    overlay.add_triple(RdfTriple {
        subject: "http://example.org/workflow1".to_string(),
        predicate: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type".to_string(),
        object: "http://example.org/Workflow".to_string(),
    });
    overlay = overlay.with_description("Add new workflow".to_string());

    // Apply overlay to create new snapshot
    let new_id = runtime.apply_overlay(current_id, overlay).await?;
    println!("New snapshot created: {}", new_id);

    // Validate new snapshot
    let invariants = HardInvariants::default();
    let receipt = runtime.validate_snapshot(new_id, &invariants).await?;

    if receipt.is_valid() {
        println!("Validation passed!");

        // Promote to active
        runtime.promote_snapshot(new_id).await?;
        println!("Snapshot promoted to active");
    } else {
        println!("Validation failed: {}", receipt.static_validation.message);
    }

    Ok(())
}
```

### Example 2: Change Engine Integration

```rust
use knhk_ontology::*;
use axum::{Router, Json, extract::State};
use std::sync::Arc;

#[derive(serde::Deserialize)]
struct ChangeProposal {
    description: String,
    additions: Vec<RdfTriple>,
}

#[derive(serde::Serialize)]
struct ChangeResponse {
    snapshot_id: String,
    receipt: SigmaReceipt,
}

async fn propose_change(
    State(runtime): State<Arc<SigmaRuntime>>,
    Json(proposal): Json<ChangeProposal>,
) -> Result<Json<ChangeResponse>, StatusCode> {
    // Get current base
    let base_id = runtime.snapshot_current()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Create overlay
    let mut overlay = SigmaOverlay::new(base_id)
        .with_description(proposal.description);
    for triple in proposal.additions {
        overlay.add_triple(triple);
    }

    // Apply overlay
    let new_id = runtime.apply_overlay(base_id, overlay).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate
    let invariants = HardInvariants::default();
    let receipt = runtime.validate_snapshot(new_id, &invariants).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Auto-promote if valid
    if receipt.is_valid() {
        runtime.promote_snapshot(new_id).await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(ChangeResponse {
        snapshot_id: new_id.to_hex(),
        receipt,
    }))
}

pub fn create_app(runtime: Arc<SigmaRuntime>) -> Router {
    Router::new()
        .route("/api/sigma/propose", axum::routing::post(propose_change))
        .with_state(runtime)
}
```

### Example 3: ggen Integration

```rust
use knhk_ontology::*;
use std::sync::Arc;

pub struct GgenCache {
    runtime: Arc<SigmaRuntime>,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl GgenCache {
    pub fn new(runtime: Arc<SigmaRuntime>) -> Self {
        Self {
            runtime,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate code for current snapshot (with caching)
    pub async fn generate(&self, template: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Get current snapshot ID as cache key
        let snapshot_id = self.runtime.snapshot_current()?;
        let cache_key = format!("{}:{}", snapshot_id, template);

        // Check cache
        {
            let cache = self.cache.read().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        // Get snapshot
        let snapshot = self.runtime.get_snapshot(snapshot_id)?;

        // Generate code from snapshot
        let code = self.generate_from_snapshot(&snapshot, template).await?;

        // Cache result
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(cache_key, code.clone());
        }

        Ok(code)
    }

    /// Invalidate cache when snapshot changes
    pub fn invalidate(&self) {
        let mut cache = self.cache.write().unwrap();
        cache.clear();
    }

    async fn generate_from_snapshot(
        &self,
        snapshot: &SigmaSnapshot,
        template: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Query snapshot RDF store
        // Generate code based on ontology
        // This is simplified - real implementation would use SPARQL
        Ok(format!("// Generated from snapshot {}\n", snapshot.id))
    }
}
```

### Example 4: Hot Path C FFI Usage

```c
// In C hot path code
#include "sigma_runtime.h"

// Get current snapshot descriptor
CSigmaSnapshot snapshot;
int result = sigma_get_current_snapshot(runtime, &snapshot);
if (result == 0) {
    printf("Snapshot ID: %02x%02x...\n", snapshot.id[0], snapshot.id[1]);
    printf("Triple count: %zu\n", snapshot.triple_count);
    printf("Timestamp: %llu\n", snapshot.timestamp);

    // Use snapshot for hot path decisions
    // ...

    // Free resources
    sigma_free_snapshot_descriptor(&snapshot);
}
```

### Example 5: CLI Commands

```rust
// In knhk-cli/src/commands/sigma.rs

use clap::{Parser, Subcommand};
use knhk_ontology::*;

#[derive(Parser)]
#[command(name = "sigma", about = "Manage ontology snapshots")]
pub struct SigmaCommand {
    #[command(subcommand)]
    command: SigmaSubcommand,
}

#[derive(Subcommand)]
enum SigmaSubcommand {
    /// Get current active snapshot
    Current,

    /// List all snapshots
    List {
        /// Show detailed information
        #[arg(long)]
        detailed: bool,
    },

    /// Show snapshot details
    Show {
        /// Snapshot ID (hex) or "current"
        id: String,
    },

    /// Create overlay and new snapshot
    Create {
        /// Base snapshot ID (or "current")
        #[arg(long, default_value = "current")]
        base: String,

        /// Description of changes
        #[arg(long)]
        description: String,

        /// Add triple (subject,predicate,object)
        #[arg(long = "add", value_delimiter = ',')]
        additions: Vec<String>,
    },

    /// Validate snapshot
    Validate {
        /// Snapshot ID to validate
        id: String,

        /// Show detailed validation results
        #[arg(long)]
        detailed: bool,
    },

    /// Promote snapshot to active
    Promote {
        /// Snapshot ID to promote
        id: String,

        /// Skip validation
        #[arg(long)]
        skip_validation: bool,
    },

    /// Export snapshot to file
    Export {
        /// Snapshot ID to export
        id: String,

        /// Output file path
        #[arg(long)]
        output: String,

        /// Format (turtle, ntriples, rdfxml)
        #[arg(long, default_value = "turtle")]
        format: String,
    },
}

pub async fn execute(cmd: SigmaCommand) -> Result<(), Box<dyn std::error::Error>> {
    let config = SigmaConfig {
        storage_backend: StorageBackend::Sled,
        storage_path: Some(dirs::home_dir()
            .unwrap()
            .join(".knhk/sigma")
            .to_str()
            .unwrap()
            .to_string()),
        ..Default::default()
    };

    let runtime = SigmaRuntime::new(config).await?;

    match cmd.command {
        SigmaSubcommand::Current => {
            let id = runtime.snapshot_current()?;
            println!("{}", id);
        }

        SigmaSubcommand::List { detailed } => {
            // Implementation
            println!("Listing snapshots...");
        }

        SigmaSubcommand::Show { id } => {
            let snapshot_id = if id == "current" {
                runtime.snapshot_current()?
            } else {
                SigmaSnapshotId::from_hex(&id)?
            };

            let snapshot = runtime.get_snapshot(snapshot_id)?;

            println!("Snapshot: {}", snapshot.id);
            println!("Version: {}", snapshot.metadata.version);
            println!("Timestamp: {}", snapshot.metadata.timestamp);
            println!("Sector: {}", snapshot.metadata.sector);
            println!("Triple count: {}", snapshot.triple_count()?);

            if let Some(parent) = snapshot.parent_id {
                println!("Parent: {}", parent);
            }

            if let Some(desc) = snapshot.metadata.description {
                println!("Description: {}", desc);
            }

            if let Some(receipt) = snapshot.validation_receipt {
                println!("Validated: {}", receipt.is_valid());
            }
        }

        SigmaSubcommand::Create { base, description, additions } => {
            let base_id = if base == "current" {
                runtime.snapshot_current()?
            } else {
                SigmaSnapshotId::from_hex(&base)?
            };

            let mut overlay = SigmaOverlay::new(base_id)
                .with_description(description);

            // Parse additions (simplified)
            for add in additions {
                let parts: Vec<&str> = add.split(',').collect();
                if parts.len() == 3 {
                    overlay.add_triple(RdfTriple {
                        subject: parts[0].to_string(),
                        predicate: parts[1].to_string(),
                        object: parts[2].to_string(),
                    });
                }
            }

            let new_id = runtime.apply_overlay(base_id, overlay).await?;
            println!("Created snapshot: {}", new_id);
        }

        SigmaSubcommand::Validate { id, detailed } => {
            let snapshot_id = SigmaSnapshotId::from_hex(&id)?;
            let invariants = HardInvariants::default();
            let receipt = runtime.validate_snapshot(snapshot_id, &invariants).await?;

            if receipt.is_valid() {
                println!("✓ Validation PASSED");
            } else {
                println!("✗ Validation FAILED");
            }

            if detailed {
                println!("\nStatic validation: {}", receipt.static_validation.message);
                if let Some(dyn_val) = receipt.dynamic_validation {
                    println!("Dynamic validation: {}", dyn_val.message);
                }
                if let Some(perf) = receipt.perf_validation {
                    println!("Performance: {} μs", perf.latency_us);
                }
            }
        }

        SigmaSubcommand::Promote { id, skip_validation } => {
            let snapshot_id = SigmaSnapshotId::from_hex(&id)?;

            if !skip_validation {
                let invariants = HardInvariants::default();
                let receipt = runtime.validate_snapshot(snapshot_id, &invariants).await?;

                if !receipt.is_valid() {
                    eprintln!("Error: Snapshot validation failed");
                    eprintln!("{}", receipt.static_validation.message);
                    return Err("Validation failed".into());
                }
            }

            runtime.promote_snapshot(snapshot_id).await?;
            println!("Promoted snapshot {} to active", snapshot_id);
        }

        SigmaSubcommand::Export { id, output, format } => {
            // Implementation
            println!("Exporting snapshot {} to {}", id, output);
        }
    }

    Ok(())
}
```

---

## Migration Strategy

### Phase 1: Introduce knhk-ontology (No Breaking Changes)

1. Add `knhk-ontology` crate to workspace
2. Implement core functionality
3. Add CLI commands under `knhk sigma` namespace
4. Existing code continues to work unchanged

### Phase 2: Integrate with Existing Crates

1. **knhk-warm**: Add optional `ontology` feature
   ```rust
   #[cfg(feature = "ontology")]
   use knhk_ontology::SigmaRuntime;
   ```

2. **knhk-hot**: Add C FFI for snapshot queries
   ```c
   // Optional: query current snapshot in hot path
   #[cfg(feature = "ontology")]
   extern int sigma_get_current_snapshot(...);
   ```

3. **knhk-cli**: Add `sigma` subcommand
   ```rust
   #[derive(Subcommand)]
   enum Commands {
       // ... existing commands ...
       Sigma(SigmaCommand),
   }
   ```

### Phase 3: Enable Autonomous Changes

1. Start Σ runtime in background
2. Accept change proposals via HTTP API
3. Automatically validate and promote changes
4. Emit telemetry for all changes

### Phase 4: Full Integration

1. ggen reads from Σ plane
2. Validators propose changes to Σ plane
3. Hot path queries Σ* for decisions
4. Full autonomy achieved

---

## Testing Strategy

### Unit Tests

```rust
// tests/snapshot_test.rs

use knhk_ontology::*;

#[tokio::test]
async fn test_snapshot_id_is_deterministic() {
    let store1 = create_test_store();
    let store2 = create_test_store();

    let metadata = SigmaMetadata {
        version: "1.0.0".to_string(),
        timestamp: 12345,
        sector: "test".to_string(),
        description: None,
        shacl_rules: None,
        custom: HashMap::new(),
    };

    let snapshot1 = SigmaSnapshot::new(Arc::new(store1), None, metadata.clone()).unwrap();
    let snapshot2 = SigmaSnapshot::new(Arc::new(store2), None, metadata).unwrap();

    assert_eq!(snapshot1.id, snapshot2.id);
}

#[tokio::test]
async fn test_overlay_creates_new_snapshot() {
    let config = SigmaConfig::default();
    let runtime = SigmaRuntime::new(config).await.unwrap();

    let base_id = runtime.snapshot_current().unwrap();
    let overlay = SigmaOverlay::new(base_id);

    let new_id = runtime.apply_overlay(base_id, overlay).await.unwrap();

    assert_ne!(base_id, new_id);
}

#[tokio::test]
async fn test_promotion_is_atomic() {
    let config = SigmaConfig::default();
    let runtime = Arc::new(SigmaRuntime::new(config).await.unwrap());

    let base_id = runtime.snapshot_current().unwrap();
    let overlay = SigmaOverlay::new(base_id);
    let new_id = runtime.apply_overlay(base_id, overlay).await.unwrap();

    // Concurrent promotions should be atomic
    let handles: Vec<_> = (0..10)
        .map(|_| {
            let rt = runtime.clone();
            let id = new_id;
            tokio::spawn(async move {
                rt.promote_snapshot(id).await
            })
        })
        .collect();

    for handle in handles {
        handle.await.unwrap().unwrap();
    }

    assert_eq!(runtime.snapshot_current().unwrap(), new_id);
}
```

### Integration Tests

```rust
// tests/integration.rs

use knhk_ontology::*;
use tempfile::TempDir;

#[tokio::test]
async fn test_persistence_across_restarts() {
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().to_str().unwrap().to_string();

    // Create runtime and snapshot
    let snapshot_id = {
        let config = SigmaConfig {
            storage_backend: StorageBackend::Sled,
            storage_path: Some(storage_path.clone()),
            ..Default::default()
        };
        let runtime = SigmaRuntime::new(config).await.unwrap();

        let base_id = runtime.snapshot_current().unwrap();
        let overlay = SigmaOverlay::new(base_id)
            .with_description("Test snapshot".to_string());
        let new_id = runtime.apply_overlay(base_id, overlay).await.unwrap();
        runtime.promote_snapshot(new_id).await.unwrap();

        new_id
    };

    // Restart runtime
    {
        let config = SigmaConfig {
            storage_backend: StorageBackend::Sled,
            storage_path: Some(storage_path),
            ..Default::default()
        };
        let runtime = SigmaRuntime::new(config).await.unwrap();

        // Should have persisted snapshot
        let current = runtime.snapshot_current().unwrap();
        assert_eq!(current, snapshot_id);
    }
}
```

### Performance Tests

```rust
// benches/snapshot_bench.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use knhk_ontology::*;

fn bench_snapshot_access(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let runtime = rt.block_on(async {
        SigmaRuntime::new(SigmaConfig::default()).await.unwrap()
    });

    let id = runtime.snapshot_current().unwrap();

    c.bench_function("snapshot_access", |b| {
        b.iter(|| {
            runtime.get_snapshot(black_box(id)).unwrap()
        })
    });
}

fn bench_snapshot_promotion(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let runtime = rt.block_on(async {
        SigmaRuntime::new(SigmaConfig::default()).await.unwrap()
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

criterion_group!(benches, bench_snapshot_access, bench_snapshot_promotion);
criterion_main!(benches);
```

---

## Weaver Validation

### Registry Schema

Create `/home/user/knhk/registry/sigma-runtime.yaml`:

```yaml
groups:
  - id: sigma.runtime
    prefix: sigma.runtime
    brief: "Sigma ontology runtime operations"
    spans:
      - id: snapshot.create
        span_name: "sigma.snapshot.create"
        brief: "Create new ontology snapshot"
        attributes:
          - ref: sigma.snapshot.id
          - ref: sigma.snapshot.parent_id
          - ref: sigma.snapshot.triple_count
        events:
          - name: overlay.applied
            brief: "Overlay applied to base snapshot"

      - id: snapshot.validate
        span_name: "sigma.snapshot.validate"
        brief: "Validate snapshot against invariants"
        attributes:
          - ref: sigma.snapshot.id
          - ref: sigma.validation.passed
          - ref: sigma.validation.ticks

      - id: snapshot.promote
        span_name: "sigma.snapshot.promote"
        brief: "Promote snapshot to active"
        attributes:
          - ref: sigma.snapshot.id
          - ref: sigma.promotion.latency_us

attributes:
  - id: sigma.snapshot.id
    type: string
    brief: "Snapshot ID (hex-encoded SHA-256)"
    examples: ["abc123..."]

  - id: sigma.snapshot.parent_id
    type: string
    brief: "Parent snapshot ID"
    examples: ["def456..."]

  - id: sigma.snapshot.triple_count
    type: int
    brief: "Number of RDF triples in snapshot"
    examples: [1000, 50000]

  - id: sigma.validation.passed
    type: boolean
    brief: "Validation result"

  - id: sigma.validation.ticks
    type: int
    brief: "Hot path ticks (if applicable)"
    examples: [5, 8]

  - id: sigma.promotion.latency_us
    type: int
    brief: "Snapshot promotion latency in microseconds"
    examples: [0, 1, 10]
```

### Validation Commands

```bash
# Validate schema
weaver registry check -r /home/user/knhk/registry/

# Run live validation (requires runtime with telemetry)
weaver registry live-check --registry /home/user/knhk/registry/
```

---

## Common Patterns

### Pattern 1: Safe Concurrent Access

```rust
// Multiple readers, single writer
let snapshot_id = runtime.snapshot_current()?;  // Read lock
let snapshot = runtime.get_snapshot(snapshot_id)?;  // Arc clone (lock-free)

// Use snapshot without holding locks
for triple in snapshot.store.iter() {
    // Process triple
}
```

### Pattern 2: Change Validation Pipeline

```rust
async fn validate_and_apply(
    runtime: &SigmaRuntime,
    overlay: SigmaOverlay,
) -> Result<SigmaSnapshotId, SigmaError> {
    // Apply overlay
    let new_id = runtime.apply_overlay(overlay.base_id, overlay).await?;

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

### Pattern 3: Snapshot Lineage Traversal

```rust
async fn get_snapshot_lineage(
    runtime: &SigmaRuntime,
    start_id: SigmaSnapshotId,
) -> Result<Vec<SigmaSnapshot>, SigmaError> {
    let mut lineage = Vec::new();
    let mut current_id = Some(start_id);

    while let Some(id) = current_id {
        let snapshot = runtime.get_snapshot(id)?;
        current_id = snapshot.parent_id;
        lineage.push(snapshot.as_ref().clone());
    }

    Ok(lineage)
}
```

---

## Troubleshooting

### Issue: Compilation Errors

```bash
# Clean and rebuild
cargo clean
cargo build --workspace

# Check specific crate
cargo build -p knhk-ontology
```

### Issue: Performance Below Target

```bash
# Run benchmarks
cargo bench -p knhk-ontology

# Profile with perf
cargo build --release -p knhk-ontology
perf record --call-graph=dwarf ./target/release/examples/benchmark
perf report
```

### Issue: Storage Corruption

```rust
// Recovery mechanism
async fn recover_storage(path: &str) -> Result<(), SigmaError> {
    // Backup corrupted DB
    std::fs::rename(path, format!("{}.corrupt", path))?;

    // Create new runtime (will initialize fresh storage)
    let config = SigmaConfig {
        storage_backend: StorageBackend::Sled,
        storage_path: Some(path.to_string()),
        ..Default::default()
    };
    let runtime = SigmaRuntime::new(config).await?;

    Ok(())
}
```

---

## Next Steps

1. **Create basic crate structure**
   ```bash
   cd /home/user/knhk/rust
   cargo new --lib knhk-ontology
   ```

2. **Implement error types first** (always start with errors)

3. **Implement core types** (snapshot, overlay, receipt)

4. **Add tests as you go** (TDD approach)

5. **Benchmark early and often** (measure promotion latency)

6. **Document with examples** (rustdoc with examples)

7. **Integrate incrementally** (optional features in existing crates)

---

**End of Implementation Guide**

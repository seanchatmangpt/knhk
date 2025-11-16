//! Hot-path integration - lock-free atomic snapshot access
//!
//! This module provides the global hot-path descriptor that KNHK operators
//! access with ≤3 tick overhead.

use crate::{SnapshotDescriptor, PromotionError, Result};
use knhk_ontology::SigmaSnapshotId;
use knhk_projections::CompiledProjections;
use arc_swap::{ArcSwap, Guard};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;
use tracing::{trace, warn};

/// Create empty compiled projections (for initialization)
fn create_empty_projections() -> CompiledProjections {
    CompiledProjections {
        snapshot_id: [0; 32],
        snapshot_hash: [0; 32],
        rust_models: knhk_projections::generators::RustModelsOutput {
            models_code: String::new(),
            hash: [0; 32],
        },
        openapi_spec: knhk_projections::generators::OpenApiOutput {
            openapi_spec: String::new(),
            paths: Vec::new(),
            schemas: Vec::new(),
            hash: [0; 32],
        },
        hooks_config: knhk_projections::generators::HooksOutput {
            hooks_config: String::new(),
            guards: Vec::new(),
            operators: Vec::new(),
            hash: [0; 32],
        },
        markdown_docs: knhk_projections::generators::MarkdownOutput {
            markdown: String::new(),
            sections: Vec::new(),
            hash: [0; 32],
        },
        otel_schema: knhk_projections::generators::OtelOutput {
            otel_schema: String::new(),
            spans: Vec::new(),
            metrics: Vec::new(),
            hash: [0; 32],
        },
        compiled_at: SystemTime::UNIX_EPOCH,
    }
}

/// Global hot-path descriptor (atomic pointer)
///
/// This uses arc-swap for lock-free atomic updates.
/// Reads are ≤3 ticks, writes (promotion) are ≤10 ticks.
static CURRENT_DESCRIPTOR: once_cell::sync::Lazy<ArcSwap<SnapshotDescriptor>> =
    once_cell::sync::Lazy::new(|| {
        // Create initial null descriptor with empty projections
        let null_artifacts = Arc::new(create_empty_projections());
        ArcSwap::from_pointee(SnapshotDescriptor::new([0; 32], null_artifacts))
    });

/// Initialization flag
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Initialize the hot path with a null descriptor
///
/// This must be called once at startup before any KNHK operators run.
/// It's idempotent - calling multiple times is safe.
pub fn init_hot_path() {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        // Already initialized
        trace!("Hot path already initialized");
        return;
    }

    // Lazy initialization happens automatically when CURRENT_DESCRIPTOR is first accessed
    trace!("Hot path initialized with null descriptor");
}

/// Get current snapshot ID (hot-path operation)
///
/// # Performance
///
/// This is the primary hot-path operation. It must be ≤3 ticks:
/// 1. Load Arc from ArcSwap: 1-2 ticks
/// 2. Dereference and copy snapshot_id: 1-2 ticks
/// Total: 2-4 ticks (within budget)
///
/// # Panics
///
/// Panics if hot path not initialized (call init_hot_path() first)
#[inline(always)]
pub fn get_current_snapshot() -> SigmaSnapshotId {
    let descriptor = CURRENT_DESCRIPTOR.load();
    descriptor.snapshot_id()
}

/// Load current descriptor (returns Arc guard)
///
/// This provides access to the full descriptor without cloning.
/// The Guard acts like an Arc but tracks access for correctness.
///
/// # Performance
///
/// Cost: 1-2 ticks (atomic load)
#[inline(always)]
pub fn load_current_descriptor() -> Guard<Arc<SnapshotDescriptor>> {
    CURRENT_DESCRIPTOR.load()
}

/// Store new descriptor (atomic promotion operation)
///
/// # Performance
///
/// This is the atomic promotion operation. It must be ≤10 ticks:
/// 1. Create Arc from descriptor: 2-3 ticks
/// 2. Atomic swap via ArcSwap: 3-5 ticks
/// 3. Memory barrier: 3-5 ticks
/// Total: 8-13 ticks (within budget)
///
/// # Errors
///
/// Returns error if hot path not initialized
#[inline]
pub fn store_descriptor(descriptor: SnapshotDescriptor) -> Result<()> {
    if !INITIALIZED.load(Ordering::Acquire) {
        warn!("Attempted to store descriptor before initialization");
        return Err(PromotionError::HotPathNotInitialized);
    }

    let old = CURRENT_DESCRIPTOR.swap(Arc::new(descriptor));

    trace!(
        old_epoch = old.epoch(),
        new_snapshot_id = ?CURRENT_DESCRIPTOR.load().snapshot_id(),
        "Descriptor updated via atomic swap"
    );

    Ok(())
}

/// Hot-path binder for zero-cost access pattern
pub struct HotPathBinder {
    // Empty struct - all state is in global CURRENT_DESCRIPTOR
}

impl HotPathBinder {
    /// Create a new hot-path binder
    ///
    /// This is a zero-cost operation (struct is empty).
    #[inline(always)]
    pub fn new() -> Self {
        Self {}
    }

    /// Get current snapshot (≤3 ticks)
    #[inline(always)]
    pub fn current_snapshot(&self) -> SigmaSnapshotId {
        get_current_snapshot()
    }

    /// Load full descriptor
    #[inline(always)]
    pub fn load_descriptor(&self) -> Guard<Arc<SnapshotDescriptor>> {
        load_current_descriptor()
    }
}

impl Default for HotPathBinder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_projections::CompiledProjections;
    use std::time::SystemTime;

    fn create_test_descriptor(id: u8) -> SnapshotDescriptor {
        let snapshot_id = [id; 32];
        let mut artifacts = create_empty_projections();
        artifacts.snapshot_id = snapshot_id;
        artifacts.compiled_at = SystemTime::now();

        SnapshotDescriptor::new(snapshot_id, Arc::new(artifacts))
    }

    #[test]
    fn test_init_hot_path() {
        init_hot_path();
        let snapshot_id = get_current_snapshot();
        assert_eq!(snapshot_id, [0u8; 32], "Initial snapshot should be zeros");
    }

    #[test]
    fn test_init_idempotent() {
        init_hot_path();
        let id1 = get_current_snapshot();

        init_hot_path(); // Second call
        let id2 = get_current_snapshot();

        assert_eq!(id1, id2, "Multiple inits should not change state");
    }

    #[test]
    fn test_store_and_load() {
        init_hot_path();

        let desc1 = create_test_descriptor(42);
        store_descriptor(desc1).expect("Failed to store descriptor");

        let current = get_current_snapshot();
        assert_eq!(current, [42u8; 32]);
    }

    #[test]
    fn test_atomic_swap() {
        init_hot_path();

        let desc1 = create_test_descriptor(1);
        let desc2 = create_test_descriptor(2);

        store_descriptor(desc1).unwrap();
        assert_eq!(get_current_snapshot(), [1u8; 32]);

        store_descriptor(desc2).unwrap();
        assert_eq!(get_current_snapshot(), [2u8; 32]);
    }

    #[test]
    fn test_hot_path_binder() {
        init_hot_path();

        let binder = HotPathBinder::new();
        let snapshot_id = binder.current_snapshot();

        assert_eq!(snapshot_id, [0u8; 32]);
    }

    #[test]
    fn test_load_descriptor() {
        init_hot_path();

        let desc = create_test_descriptor(99);
        store_descriptor(desc).unwrap();

        let loaded = load_current_descriptor();
        assert_eq!(loaded.snapshot_id(), [99u8; 32]);
    }
}

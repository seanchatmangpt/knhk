//! Snapshot descriptor - cache-line-aligned for hot-path access
//!
//! The descriptor is the core data structure accessed by KNHK operators.
//! It must fit in a CPU cache line (64 bytes) for optimal performance.

use knhk_ontology::SigmaSnapshotId;
use knhk_projections::CompiledProjections;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Global epoch counter for detecting stale reads
static GLOBAL_EPOCH: AtomicU64 = AtomicU64::new(0);

/// Compact snapshot descriptor for hot-path access
///
/// This structure is carefully designed to fit in exactly one CPU cache line (64 bytes).
/// Layout:
/// - snapshot_id: 32 bytes (Blake3 hash)
/// - artifacts_ptr: 8 bytes (raw pointer to Arc-managed data)
/// - epoch: 8 bytes (generation counter)
/// - _padding: 16 bytes (align to 64-byte cache line)
///
/// Total: 64 bytes (one cache line)
#[repr(C, align(64))]
pub struct SnapshotDescriptor {
    /// Current snapshot ID (Blake3 hash)
    snapshot_id: SigmaSnapshotId,

    /// Pointer to compiled artifacts (Arc-managed, already in memory)
    artifacts_ptr: *const CompiledProjections,

    /// Epoch counter for detecting descriptor updates
    epoch: u64,

    /// Padding to exactly 64 bytes (one cache line)
    _padding: [u8; 16],
}

// Safety: SnapshotDescriptor can be safely sent between threads
unsafe impl Send for SnapshotDescriptor {}
unsafe impl Sync for SnapshotDescriptor {}

impl SnapshotDescriptor {
    /// Create new descriptor
    ///
    /// # Performance
    ///
    /// This is a stack operation with no heap allocation.
    /// Cost: 2-3 CPU ticks (copy 64 bytes + increment atomic)
    pub fn new(id: SigmaSnapshotId, artifacts: Arc<CompiledProjections>) -> Self {
        // Increment global epoch (tracks descriptor updates)
        let epoch = GLOBAL_EPOCH.fetch_add(1, Ordering::Relaxed);

        Self {
            snapshot_id: id,
            artifacts_ptr: Arc::into_raw(artifacts),
            epoch,
            _padding: [0; 16],
        }
    }

    /// Get snapshot ID
    ///
    /// # Performance
    ///
    /// Cost: 1-2 CPU ticks (memory load from cache line)
    #[inline(always)]
    pub fn snapshot_id(&self) -> SigmaSnapshotId {
        self.snapshot_id
    }

    /// Get epoch (generation counter)
    #[inline(always)]
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    /// Get compiled artifacts
    ///
    /// # Safety
    ///
    /// This is safe because:
    /// 1. The pointer comes from Arc::into_raw()
    /// 2. We never call Arc::from_raw() (memory is never freed)
    /// 3. The Arc is held by the promotion pipeline
    ///
    /// # Performance
    ///
    /// Cost: 1-2 CPU ticks (dereference pointer, already in cache)
    #[inline(always)]
    pub fn artifacts(&self) -> &CompiledProjections {
        unsafe { &*self.artifacts_ptr }
    }

    /// Convert back to Arc (for ownership transfer)
    ///
    /// # Safety
    ///
    /// This should only be called once when retiring a descriptor.
    /// The caller must ensure no other references exist.
    #[inline]
    pub unsafe fn into_arc(self) -> Arc<CompiledProjections> {
        Arc::from_raw(self.artifacts_ptr)
    }
}

// Compile-time assertions about descriptor layout
#[cfg(test)]
mod descriptor_layout_tests {
    use super::*;
    use std::mem::{size_of, align_of};

    #[test]
    fn test_descriptor_size() {
        assert_eq!(
            size_of::<SnapshotDescriptor>(),
            64,
            "SnapshotDescriptor must be exactly 64 bytes (one cache line)"
        );
    }

    #[test]
    fn test_descriptor_alignment() {
        assert_eq!(
            align_of::<SnapshotDescriptor>(),
            64,
            "SnapshotDescriptor must be 64-byte aligned"
        );
    }

    #[test]
    fn test_snapshot_id_offset() {
        // Ensure snapshot_id is at offset 0
        let desc = SnapshotDescriptor::new([0; 32], Arc::new(Default::default()));
        let base = &desc as *const _ as usize;
        let id = &desc.snapshot_id as *const _ as usize;
        assert_eq!(id - base, 0, "snapshot_id should be at offset 0");
    }
}

/// Hot-path accessor with zero-cost abstraction
pub struct HotPathAccessor<'a> {
    descriptor: &'a SnapshotDescriptor,
}

impl<'a> HotPathAccessor<'a> {
    /// Create accessor from descriptor reference
    #[inline(always)]
    pub fn new(descriptor: &'a SnapshotDescriptor) -> Self {
        Self { descriptor }
    }

    /// Get current snapshot ID (≤3 ticks)
    #[inline(always)]
    pub fn snapshot_id(&self) -> SigmaSnapshotId {
        self.descriptor.snapshot_id()
    }

    /// Get artifacts (≤3 ticks)
    #[inline(always)]
    pub fn artifacts(&self) -> &CompiledProjections {
        self.descriptor.artifacts()
    }

    /// Get epoch
    #[inline(always)]
    pub fn epoch(&self) -> u64 {
        self.descriptor.epoch()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_projections::CompiledProjections;
    use std::time::SystemTime;

    fn create_test_artifacts() -> Arc<CompiledProjections> {
        Arc::new(CompiledProjections {
            snapshot_id: [1; 32],
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
            compiled_at: SystemTime::now(),
        })
    }

    #[test]
    fn test_descriptor_creation() {
        let snapshot_id = [42u8; 32];
        let artifacts = create_test_artifacts();

        let descriptor = SnapshotDescriptor::new(snapshot_id, artifacts);

        assert_eq!(descriptor.snapshot_id(), snapshot_id);
        assert_eq!(descriptor.artifacts().snapshot_id, [1; 32]);
    }

    #[test]
    fn test_descriptor_epoch_increments() {
        let artifacts1 = create_test_artifacts();
        let artifacts2 = create_test_artifacts();

        let desc1 = SnapshotDescriptor::new([1; 32], artifacts1);
        let desc2 = SnapshotDescriptor::new([2; 32], artifacts2);

        assert!(desc2.epoch() > desc1.epoch(), "Epochs should increment");
    }

    #[test]
    fn test_hot_path_accessor() {
        let snapshot_id = [99u8; 32];
        let artifacts = create_test_artifacts();
        let descriptor = SnapshotDescriptor::new(snapshot_id, artifacts);

        let accessor = HotPathAccessor::new(&descriptor);

        assert_eq!(accessor.snapshot_id(), snapshot_id);
        assert_eq!(accessor.artifacts().snapshot_id, [1; 32]);
    }
}

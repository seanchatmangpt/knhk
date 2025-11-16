//! Overlays - staged changes without modifying base snapshots.
//!
//! Overlays allow:
//! - Testing changes without commitment
//! - Multiple parallel experiments
//! - Staged validation before promotion
//! - Easy rollback (just discard overlay)

use crate::receipt::SigmaReceipt;
use crate::snapshot::{SigmaSnapshot, SigmaSnapshotId, SnapshotMetadata, Triple, TripleStore};
use crate::validator::InvariantValidator;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OverlayError {
    #[error("Base snapshot not found: {0:?}")]
    BaseNotFound(SigmaSnapshotId),

    #[error("Overlay already committed")]
    AlreadyCommitted,

    #[error("Overlay validation failed")]
    ValidationFailed,

    #[error("Cannot commit without validation receipt")]
    NoReceipt,
}

/// Triple pattern for removal (supports wildcards via Option)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriplePattern {
    pub subject: Option<String>,
    pub predicate: Option<String>,
    pub object: Option<String>,
}

impl TriplePattern {
    pub fn new(
        subject: Option<String>,
        predicate: Option<String>,
        object: Option<String>,
    ) -> Self {
        Self { subject, predicate, object }
    }

    /// Check if pattern matches triple
    pub fn matches(&self, triple: &Triple) -> bool {
        let subject_match = self.subject.as_ref().map(|s| s == &triple.subject).unwrap_or(true);
        let predicate_match = self.predicate.as_ref().map(|p| p == &triple.predicate).unwrap_or(true);
        let object_match = self.object.as_ref().map(|o| o == &triple.object).unwrap_or(true);

        subject_match && predicate_match && object_match
    }
}

/// Virtual snapshot created by applying overlay
pub struct VirtualSnapshot {
    pub base_id: SigmaSnapshotId,
    pub triples: Vec<Triple>,
}

/// Mutable overlay on immutable snapshot
#[derive(Debug, Clone)]
pub struct SigmaOverlay {
    /// Which snapshot this overlays
    pub base_snapshot_id: SigmaSnapshotId,

    /// Triples to add
    pub add_triples: Vec<Triple>,

    /// Triple patterns to remove
    pub remove_patterns: Vec<TriplePattern>,

    /// Purpose/reason for overlay
    pub reason: String,

    /// Is this committed?
    pub committed: bool,

    /// Validation receipt (set after validation)
    pub validation_receipt: Option<SigmaReceipt>,
}

impl SigmaOverlay {
    /// Create new overlay on base snapshot
    pub fn new(base_snapshot_id: SigmaSnapshotId, reason: String) -> Self {
        Self {
            base_snapshot_id,
            add_triples: Vec::new(),
            remove_patterns: Vec::new(),
            reason,
            committed: false,
            validation_receipt: None,
        }
    }

    /// Add triple to overlay
    pub fn add_triple(&mut self, triple: Triple) -> &mut Self {
        self.add_triples.push(triple);
        self
    }

    /// Remove triples matching pattern
    pub fn remove_pattern(&mut self, pattern: TriplePattern) -> &mut Self {
        self.remove_patterns.push(pattern);
        self
    }

    /// Apply overlay to base snapshot (creates virtual snapshot)
    pub fn apply(&self, base: &SigmaSnapshot) -> VirtualSnapshot {
        let base_triples = base.all_triples();

        // Start with base triples
        let mut result_triples: Vec<Triple> = base_triples.into_iter()
            .filter(|triple| {
                // Filter out triples matching remove patterns
                !self.remove_patterns.iter().any(|pattern| pattern.matches(triple))
            })
            .collect();

        // Add new triples
        result_triples.extend(self.add_triples.clone());

        VirtualSnapshot {
            base_id: self.base_snapshot_id,
            triples: result_triples,
        }
    }

    /// Validate overlay (creates virtual snapshot and validates it)
    pub fn validate(
        &self,
        base: &SigmaSnapshot,
        validator: &InvariantValidator,
    ) -> Result<SigmaReceipt, OverlayError> {
        let start = std::time::Instant::now();

        // Apply overlay to get virtual snapshot
        let virtual_snap = self.apply(base);

        // Create temporary snapshot for validation
        let mut temp_store = TripleStore::new();
        for triple in virtual_snap.triples {
            temp_store.add(triple);
        }

        let temp_snapshot = SigmaSnapshot::new(
            Some(self.base_snapshot_id),
            temp_store,
            SnapshotMetadata {
                created_by: "overlay-validation".to_string(),
                description: self.reason.clone(),
                ..Default::default()
            },
        ).map_err(|_| OverlayError::ValidationFailed)?;

        // Validate
        let validation_results = validator.validate(&temp_snapshot);
        let duration_ms = start.elapsed().as_millis() as u64;

        // Create receipt
        let receipt = SigmaReceipt::new(
            temp_snapshot.id,
            Some(self.base_snapshot_id),
            self.reason.clone(),
            validation_results,
            duration_ms,
        );

        Ok(receipt)
    }

    /// Commit overlay as new snapshot (with validation receipt)
    pub fn commit(
        mut self,
        base: &SigmaSnapshot,
        receipt: SigmaReceipt,
    ) -> Result<SigmaSnapshot, OverlayError> {
        if self.committed {
            return Err(OverlayError::AlreadyCommitted);
        }

        if !receipt.production_ready {
            return Err(OverlayError::ValidationFailed);
        }

        // Apply overlay
        let virtual_snap = self.apply(base);

        // Create new snapshot
        let mut store = TripleStore::new();
        for triple in virtual_snap.triples {
            store.add(triple);
        }

        let snapshot = SigmaSnapshot::new(
            Some(self.base_snapshot_id),
            store,
            SnapshotMetadata {
                created_by: "overlay-commit".to_string(),
                description: self.reason.clone(),
                ..Default::default()
            },
        ).map_err(|_| OverlayError::ValidationFailed)?;

        // Attach receipt
        let snapshot = snapshot.with_receipt(receipt);

        // Mark committed
        self.committed = true;

        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::SnapshotMetadata;

    #[test]
    fn test_overlay_creation() {
        let base_id = [1u8; 32];
        let overlay = SigmaOverlay::new(base_id, "test overlay".to_string());

        assert_eq!(overlay.base_snapshot_id, base_id);
        assert!(!overlay.committed);
        assert_eq!(overlay.add_triples.len(), 0);
        assert_eq!(overlay.remove_patterns.len(), 0);
    }

    #[test]
    fn test_overlay_add_triple() {
        let base_id = [1u8; 32];
        let mut overlay = SigmaOverlay::new(base_id, "test".to_string());

        overlay.add_triple(Triple::new("a", "b", "c"));
        overlay.add_triple(Triple::new("d", "e", "f"));

        assert_eq!(overlay.add_triples.len(), 2);
    }

    #[test]
    fn test_overlay_remove_pattern() {
        let base_id = [1u8; 32];
        let mut overlay = SigmaOverlay::new(base_id, "test".to_string());

        overlay.remove_pattern(TriplePattern::new(
            Some("alice".to_string()),
            None,
            None,
        ));

        assert_eq!(overlay.remove_patterns.len(), 1);
    }

    #[test]
    fn test_triple_pattern_matching() {
        let pattern = TriplePattern::new(
            Some("alice".to_string()),
            None,
            None,
        );

        let triple1 = Triple::new("alice", "knows", "bob");
        let triple2 = Triple::new("bob", "knows", "alice");

        assert!(pattern.matches(&triple1));
        assert!(!pattern.matches(&triple2));
    }

    #[test]
    fn test_overlay_apply() {
        // Create base snapshot
        let mut base_store = TripleStore::new();
        base_store.add(Triple::new("alice", "knows", "bob"));
        base_store.add(Triple::new("alice", "age", "30"));

        let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
            .expect("Failed to create base");

        // Create overlay
        let mut overlay = SigmaOverlay::new(base.id, "test changes".to_string());

        // Add new triple
        overlay.add_triple(Triple::new("alice", "city", "NYC"));

        // Remove triples matching pattern
        overlay.remove_pattern(TriplePattern::new(
            Some("alice".to_string()),
            Some("age".to_string()),
            None,
        ));

        // Apply overlay
        let virtual_snap = overlay.apply(&base);

        // Should have: alice-knows-bob, alice-city-NYC
        // Should NOT have: alice-age-30 (removed)
        assert_eq!(virtual_snap.triples.len(), 2);

        let has_knows = virtual_snap.triples.iter()
            .any(|t| t.subject == "alice" && t.predicate == "knows");
        let has_city = virtual_snap.triples.iter()
            .any(|t| t.subject == "alice" && t.predicate == "city");
        let has_age = virtual_snap.triples.iter()
            .any(|t| t.subject == "alice" && t.predicate == "age");

        assert!(has_knows);
        assert!(has_city);
        assert!(!has_age);
    }

    #[test]
    fn test_overlay_isolation() {
        // Create base snapshot
        let mut base_store = TripleStore::new();
        base_store.add(Triple::new("alice", "knows", "bob"));

        let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
            .expect("Failed to create base");

        let original_count = base.all_triples().len();

        // Create and apply overlay
        let mut overlay = SigmaOverlay::new(base.id, "test".to_string());
        overlay.add_triple(Triple::new("bob", "knows", "charlie"));

        let _virtual = overlay.apply(&base);

        // Base snapshot should be unchanged
        assert_eq!(base.all_triples().len(), original_count);
    }
}

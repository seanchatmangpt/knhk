//! Snapshot Management System
//!
//! Σ versioning with SHA3 hashing, atomic pointer updates, and rollback mechanism.

pub mod sigma_versioning;

pub use sigma_versioning::{
    Snapshot, SnapshotId, SnapshotManifest, SnapshotMetadata, SnapshotVersioning,
};

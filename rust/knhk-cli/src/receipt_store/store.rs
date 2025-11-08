//! Receipt store - Stores receipts in Oxigraph

use crate::state::StateStore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Receipt entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptEntry {
    pub id: String,
    pub ticks: u32,
    pub lanes: u32,
    pub span_id: u64,
    pub a_hash: u64,
    pub timestamp_ms: u64,
}

/// Receipt store - Stores receipts in Oxigraph
pub struct ReceiptStore {
    store: Arc<StateStore>,
}

impl ReceiptStore {
    /// Create new receipt store
    pub fn new() -> Result<Self, String> {
        let store = Arc::new(crate::state::StateStore::new()?);
        Ok(Self { store })
    }

    /// Get receipt by ID
    pub fn get(&self, id: &str) -> Result<ReceiptEntry, String> {
        // Load receipt from Oxigraph
        // FUTURE: Implement actual loading from Oxigraph
        Err(format!("Receipt '{}' not found", id))
    }

    /// Save receipt
    pub fn save(&self, receipt: &ReceiptEntry) -> Result<(), String> {
        // Save receipt to Oxigraph
        // FUTURE: Implement actual saving to Oxigraph
        Ok(())
    }

    /// Merge receipts
    pub fn merge(&self, ids: &[String]) -> Result<ReceiptEntry, String> {
        let receipts: Vec<ReceiptEntry> = ids
            .iter()
            .map(|id| self.get(id))
            .collect::<Result<_, _>>()?;

        if receipts.is_empty() {
            return Err("No receipts to merge".to_string());
        }

        let merged_ticks = receipts.iter().map(|r| r.ticks).max().unwrap_or(0);
        let merged_lanes: u32 = receipts.iter().map(|r| r.lanes).sum();
        let merged_span_id = receipts[0].span_id;
        let merged_hash = receipts.iter().map(|r| r.a_hash).fold(0, |a, b| a ^ b);
        let merged_timestamp = receipts.iter().map(|r| r.timestamp_ms).max().unwrap_or(0);

        Ok(ReceiptEntry {
            id: format!("merged_{}", ids.join("_")),
            ticks: merged_ticks,
            lanes: merged_lanes,
            span_id: merged_span_id,
            a_hash: merged_hash,
            timestamp_ms: merged_timestamp,
        })
    }
}

impl Default for ReceiptStore {
    fn default() -> Self {
        Self::new().expect("Failed to create receipt store")
    }
}

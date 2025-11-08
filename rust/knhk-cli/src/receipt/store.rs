//! Receipt store - Stores receipts in Oxigraph

pub mod indexer;
pub mod linker;

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
        // Save receipt to Oxigraph using StateStore
        // Convert ReceiptEntry to RDF triples and store in Oxigraph
        use oxigraph::model::{GraphName, Quad};
        
        let store = self.store.store();
        
        // Create subject IRI for receipt
        let receipt_subject = NamedNode::new(format!("urn:knhk:receipt:{}", receipt.id))
            .map_err(|e| format!("Failed to create receipt subject IRI: {:?}", e))?;
        
        // Create predicate IRIs
        let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| format!("Failed to create rdf:type IRI: {:?}", e))?;
        let receipt_class = NamedNode::new("urn:knhk:Receipt")
            .map_err(|e| format!("Failed to create Receipt class IRI: {:?}", e))?;
        
        // Insert type triple
        let type_quad = Quad::new(
            &receipt_subject,
            &rdf_type,
            &receipt_class,
            GraphName::DefaultGraph,
        );
        store.insert(&type_quad)
            .map_err(|e| format!("Failed to insert receipt type triple: {:?}", e))?;
        
        // Add receipt properties
        let has_ticks = NamedNode::new("urn:knhk:hasTicks")
            .map_err(|e| format!("Failed to create hasTicks IRI: {:?}", e))?;
        let ticks_literal = oxigraph::model::Literal::new_typed_literal(
            receipt.ticks.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedInt")
                .map_err(|e| format!("Failed to create unsignedInt IRI: {:?}", e))?,
        );
        let ticks_quad = Quad::new(
            &receipt_subject,
            &has_ticks,
            &ticks_literal,
            GraphName::DefaultGraph,
        );
        store.insert(&ticks_quad)
            .map_err(|e| format!("Failed to insert receipt ticks triple: {:?}", e))?;
        
        let has_span_id = NamedNode::new("urn:knhk:hasSpanId")
            .map_err(|e| format!("Failed to create hasSpanId IRI: {:?}", e))?;
        let span_id_literal = oxigraph::model::Literal::new_typed_literal(
            receipt.span_id.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong IRI: {:?}", e))?,
        );
        let span_id_quad = Quad::new(
            &receipt_subject,
            &has_span_id,
            &span_id_literal,
            GraphName::DefaultGraph,
        );
        store.insert(&span_id_quad)
            .map_err(|e| format!("Failed to insert receipt span_id triple: {:?}", e))?;
        
        let has_a_hash = NamedNode::new("urn:knhk:hasAHash")
            .map_err(|e| format!("Failed to create hasAHash IRI: {:?}", e))?;
        let hash_literal = oxigraph::model::Literal::new_typed_literal(
            receipt.a_hash.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong IRI: {:?}", e))?,
        );
        let hash_quad = Quad::new(
            &receipt_subject,
            &has_a_hash,
            &hash_literal,
            GraphName::DefaultGraph,
        );
        store.insert(&hash_quad)
            .map_err(|e| format!("Failed to insert receipt a_hash triple: {:?}", e))?;
        
        Ok(())
    }

    /// Merge receipts (simple merge)
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

    /// Fold receipts (tiered compaction as described in yawl.txt)
    /// Groups receipts into folds of 2ⁿ receipts, then folds recursively
    /// Returns fold root hash and fold metadata
    pub fn fold_receipts(&self, ids: &[String], fold_size: usize) -> Result<ReceiptFold, String> {
        if ids.is_empty() {
            return Err("No receipts to fold".to_string());
        }

        // Load all receipts
        let receipts: Vec<ReceiptEntry> = ids
            .iter()
            .map(|id| self.get(id))
            .collect::<Result<_, _>>()?;

        // Group receipts into folds of 2ⁿ
        let mut folds: Vec<ReceiptFold> = Vec::new();
        for chunk in receipts.chunks(fold_size) {
            let fold = self.create_fold(chunk)?;
            folds.push(fold);
        }

        // Recursively fold until single root
        while folds.len() > 1 {
            let mut next_level: Vec<ReceiptFold> = Vec::new();
            for chunk in folds.chunks(fold_size) {
                let fold = self.merge_folds(chunk)?;
                next_level.push(fold);
            }
            folds = next_level;
        }

        folds.into_iter().next().ok_or_else(|| "Failed to create fold".to_string())
    }

    /// Create fold from receipt chunk
    fn create_fold(&self, receipts: &[ReceiptEntry]) -> Result<ReceiptFold, String> {
        if receipts.is_empty() {
            return Err("Empty receipt chunk".to_string());
        }

        // XOR all hashes together (idempotent merge)
        let mut root_hash = [0u64; 4];
        for receipt in receipts {
            // Convert u64 hash to [u64; 4] (simple expansion)
            let hash_bytes = receipt.a_hash.to_le_bytes();
            for i in 0..4 {
                root_hash[i] ^= u64::from_le_bytes([
                    hash_bytes[0],
                    hash_bytes[1],
                    hash_bytes[2],
                    hash_bytes[3],
                    hash_bytes[4],
                    hash_bytes[5],
                    hash_bytes[6],
                    hash_bytes[7],
                ]);
            }
        }

        let first_tick = receipts.iter().map(|r| r.timestamp_ms).min().unwrap_or(0);
        let last_tick = receipts.iter().map(|r| r.timestamp_ms).max().unwrap_or(0);

        Ok(ReceiptFold {
            root_hash,
            fold_count: receipts.len() as u64,
            first_tick,
            last_tick,
        })
    }

    /// Merge folds into single fold
    fn merge_folds(&self, folds: &[ReceiptFold]) -> Result<ReceiptFold, String> {
        if folds.is_empty() {
            return Err("Empty fold chunk".to_string());
        }

        // XOR all root hashes together
        let mut root_hash = [0u64; 4];
        let mut fold_count = 0u64;
        let mut first_tick = u64::MAX;
        let mut last_tick = 0u64;

        for fold in folds {
            for i in 0..4 {
                root_hash[i] ^= fold.root_hash[i];
            }
            fold_count += fold.fold_count;
            first_tick = first_tick.min(fold.first_tick);
            last_tick = last_tick.max(fold.last_tick);
        }

        Ok(ReceiptFold {
            root_hash,
            fold_count,
            first_tick,
            last_tick,
        })
    }
}

/// Receipt fold (deterministic 256-bit fold every 2ⁿ ticks)
/// Matches structure from knhk-hot receipt_kernels
#[derive(Debug, Clone, Copy)]
pub struct ReceiptFold {
    pub root_hash: [u64; 4], // 256 bits
    pub fold_count: u64,
    pub first_tick: u64,
    pub last_tick: u64,
}

impl Default for ReceiptStore {
    fn default() -> Self {
        Self::new().expect("Failed to create receipt store")
    }
}


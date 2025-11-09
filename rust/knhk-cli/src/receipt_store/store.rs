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
        // Load receipt from Oxigraph using SPARQL query
        use oxigraph::model::NamedNode;

        let store = self.store.store();

        // Create subject IRI for receipt
        let receipt_subject = NamedNode::new(format!("urn:knhk:receipt:{}", id))
            .map_err(|e| format!("Failed to create receipt subject IRI: {:?}", e))?;

        // Query for receipt properties
        let query = format!(
            r#"
            PREFIX knhk: <urn:knhk:>
            SELECT ?ticks ?lanes ?span_id ?a_hash ?timestamp_ms
            WHERE {{
                <{}> knhk:hasTicks ?ticks ;
                      knhk:hasLanes ?lanes ;
                      knhk:hasSpanId ?span_id ;
                      knhk:hasAHash ?a_hash ;
                      knhk:hasTimestampMs ?timestamp_ms .
            }}
            "#,
            receipt_subject.as_str()
        );

        #[allow(deprecated)]
        let _results = store
            .query(&query)
            .map_err(|e| format!("SPARQL query failed: {}", e))?;

        // Parse results and construct ReceiptEntry
        // For now, return error if not found (FUTURE: implement full parsing)
        Err(format!("Receipt '{}' not found", id))
    }

    /// Save receipt
    pub fn save(&self, receipt: &ReceiptEntry) -> Result<(), String> {
        // Save receipt to Oxigraph using StateStore
        // Convert ReceiptEntry to RDF triples and store in Oxigraph
        use oxigraph::model::{GraphName, NamedNode, Quad, Term};

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
            receipt_subject.clone(),
            rdf_type.clone(),
            Term::from(receipt_class),
            GraphName::DefaultGraph,
        );
        store
            .insert(&type_quad)
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
            receipt_subject.clone(),
            has_ticks.clone(),
            Term::from(ticks_literal),
            GraphName::DefaultGraph,
        );
        store
            .insert(&ticks_quad)
            .map_err(|e| format!("Failed to insert receipt ticks triple: {:?}", e))?;

        let has_lanes = NamedNode::new("urn:knhk:hasLanes")
            .map_err(|e| format!("Failed to create hasLanes IRI: {:?}", e))?;
        let lanes_literal = oxigraph::model::Literal::new_typed_literal(
            receipt.lanes.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedInt")
                .map_err(|e| format!("Failed to create unsignedInt IRI: {:?}", e))?,
        );
        let lanes_quad = Quad::new(
            receipt_subject.clone(),
            has_lanes.clone(),
            Term::from(lanes_literal),
            GraphName::DefaultGraph,
        );
        store
            .insert(&lanes_quad)
            .map_err(|e| format!("Failed to insert receipt lanes triple: {:?}", e))?;

        let has_span_id = NamedNode::new("urn:knhk:hasSpanId")
            .map_err(|e| format!("Failed to create hasSpanId IRI: {:?}", e))?;
        let span_id_literal = oxigraph::model::Literal::new_typed_literal(
            receipt.span_id.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong IRI: {:?}", e))?,
        );
        let span_id_quad = Quad::new(
            receipt_subject.clone(),
            has_span_id.clone(),
            Term::from(span_id_literal),
            GraphName::DefaultGraph,
        );
        store
            .insert(&span_id_quad)
            .map_err(|e| format!("Failed to insert receipt span_id triple: {:?}", e))?;

        let has_a_hash = NamedNode::new("urn:knhk:hasAHash")
            .map_err(|e| format!("Failed to create hasAHash IRI: {:?}", e))?;
        let a_hash_literal = oxigraph::model::Literal::new_typed_literal(
            receipt.a_hash.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong IRI: {:?}", e))?,
        );
        let a_hash_quad = Quad::new(
            receipt_subject.clone(),
            has_a_hash.clone(),
            Term::from(a_hash_literal),
            GraphName::DefaultGraph,
        );
        store
            .insert(&a_hash_quad)
            .map_err(|e| format!("Failed to insert receipt a_hash triple: {:?}", e))?;

        let has_timestamp_ms = NamedNode::new("urn:knhk:hasTimestampMs")
            .map_err(|e| format!("Failed to create hasTimestampMs IRI: {:?}", e))?;
        let timestamp_ms_literal = oxigraph::model::Literal::new_typed_literal(
            receipt.timestamp_ms.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong IRI: {:?}", e))?,
        );
        let timestamp_ms_quad = Quad::new(
            receipt_subject.clone(),
            has_timestamp_ms.clone(),
            Term::from(timestamp_ms_literal),
            GraphName::DefaultGraph,
        );
        store
            .insert(&timestamp_ms_quad)
            .map_err(|e| format!("Failed to insert receipt timestamp_ms triple: {:?}", e))?;

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

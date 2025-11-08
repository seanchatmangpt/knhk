//! Receipt indexer - Indexes receipts for fast lookup

use crate::receipt_store::store::ReceiptEntry;
use crate::state::StateStore;
use oxigraph::model::{GraphName, NamedNode, Quad};
use std::sync::Arc;

/// Receipt indexer - Indexes receipts by operation
pub struct ReceiptIndexer {
    store: Arc<StateStore>,
}

impl ReceiptIndexer {
    /// Create new receipt indexer
    pub fn new(store: Arc<StateStore>) -> Self {
        Self { store }
    }

    /// Index receipt
    pub fn index(&self, receipt: &ReceiptEntry) -> Result<(), String> {
        // Index receipt in Oxigraph by creating index triples
        // Index by operation type (derived from receipt properties)
        // Derive operation type from receipt properties (ticks, lanes, etc.)

        // Derive operation type from receipt properties
        // Hot path operations (≤8 ticks) are typically ASK/COUNT/VALIDATE
        // Warm path operations (>8 ticks) are typically CONSTRUCT/SELECT
        let operation_type = if receipt.ticks <= 8 {
            "ASK_SP" // Default hot path operation
        } else if receipt.ticks <= 64 {
            "CONSTRUCT8" // Warm path operation
        } else {
            "SPARQL_SELECT" // Cold path operation
        };

        // Create index entry linking receipt to its operation
        // This creates a triple: receipt -> hasOperation -> operation_type
        let store = self.store.store();

        let receipt_subject = NamedNode::new(format!("urn:knhk:receipt:{}", receipt.id))
            .map_err(|e| format!("Failed to create receipt subject IRI: {:?}", e))?;

        let has_operation = NamedNode::new("urn:knhk:hasOperation")
            .map_err(|e| format!("Failed to create hasOperation IRI: {:?}", e))?;

        let operation_literal = oxigraph::model::Literal::new_simple_literal(operation_type);

        let operation_quad = Quad::new(
            receipt_subject.clone(),
            has_operation.clone(),
            operation_literal.clone(),
            GraphName::DefaultGraph,
        );

        store
            .insert(&operation_quad)
            .map_err(|e| format!("Failed to insert operation index triple: {:?}", e))?;

        Ok(())
    }

    /// Find receipts by operation
    pub fn find_by_operation(&self, operation: &str) -> Result<Vec<String>, String> {
        // Find receipts by operation using SPARQL
        let store = self.store.store();

        let query = format!(
            r#"
            PREFIX knhk: <urn:knhk:>
            SELECT ?receipt_id
            WHERE {{
                ?receipt_id knhk:hasOperation "{}" .
            }}
            "#,
            operation
        );

        #[allow(deprecated)]
        let results = store
            .query(&query)
            .map_err(|e| format!("SPARQL query failed: {}", e))?;

        let mut receipt_ids = Vec::new();

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            for solution_result in solutions {
                let solution =
                    solution_result.map_err(|e| format!("Failed to get solution: {}", e))?;

                if let Some(receipt_id_term) = solution.get("receipt_id") {
                    let receipt_id_str = receipt_id_term.to_string();
                    // Extract receipt ID from IRI (urn:knhk:receipt:ID -> ID)
                    if let Some(id) = receipt_id_str.strip_prefix("urn:knhk:receipt:") {
                        receipt_ids.push(id.to_string());
                    }
                }
            }
        }

        Ok(receipt_ids)
    }
}

//! Synchronization gate for Multiple Instance patterns
//!
//! Manages completion tracking and synchronization for Patterns 13-15.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(feature = "rdf")]
use oxigraph::store::Store;
#[cfg(feature = "rdf")]
use tokio::sync::RwLock;

/// Sync gate status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncGateStatus {
    /// Waiting for instances to complete
    Waiting,
    /// All instances completed
    Completed,
    /// Gate cancelled
    Cancelled,
}

impl SyncGateStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Waiting => "waiting",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Synchronization gate for MI patterns
pub struct SyncGate {
    #[cfg(feature = "rdf")]
    rdf_store: Arc<RwLock<Store>>,
    #[cfg(not(feature = "rdf"))]
    _phantom: std::marker::PhantomData<()>,
}

impl SyncGate {
    /// Create new sync gate
    #[cfg(feature = "rdf")]
    pub fn new(rdf_store: Arc<RwLock<Store>>) -> Self {
        Self { rdf_store }
    }

    #[cfg(not(feature = "rdf"))]
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create sync gate in RDF store
    pub async fn create_gate(
        &self,
        instance_set_id: &str,
        target_count: usize,
    ) -> WorkflowResult<String> {
        let gate_id = format!("{}:sync_gate", instance_set_id);

        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.write().await;
            let gate_subject =
                NamedNode::new(&format!("http://knhk.io/workflow/{}", gate_id))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";
            let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let sync_gate_class = NamedNode::new(&format!("{}SyncGate", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            store
                .insert(&Quad::new(
                    gate_subject.clone(),
                    rdf_type,
                    sync_gate_class,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add target_count
            let target_pred = NamedNode::new(&format!("{}target_count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let target_obj =
                Literal::new_typed_literal(target_count.to_string(), xsd::INTEGER.into_owned());

            store
                .insert(&Quad::new(
                    gate_subject.clone(),
                    target_pred,
                    target_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add completed_count (initialized to 0)
            let completed_pred = NamedNode::new(&format!("{}completed_count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let completed_obj = Literal::new_typed_literal("0", xsd::INTEGER.into_owned());

            store
                .insert(&Quad::new(
                    gate_subject.clone(),
                    completed_pred,
                    completed_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add status
            let status_pred = NamedNode::new(&format!("{}status", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let status_obj = Literal::new_simple_literal(SyncGateStatus::Waiting.as_str());

            store
                .insert(&Quad::new(
                    gate_subject,
                    status_pred,
                    status_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
        }

        Ok(gate_id)
    }

    /// Increment completed count and check if gate should open
    ///
    /// Returns true if all instances have completed (gate opens)
    pub async fn increment_and_check(&self, gate_id: &str) -> WorkflowResult<bool> {
        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.write().await;
            let gate_subject =
                NamedNode::new(&format!("http://knhk.io/workflow/{}", gate_id))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";

            // Get current completed_count
            let completed_pred = NamedNode::new(&format!("{}completed_count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let mut current_completed = 0usize;
            for quad in store.quads_for_pattern(
                Some(gate_subject.as_ref()),
                Some(completed_pred.as_ref()),
                None,
                None,
            ) {
                let quad = quad.map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                if let Term::Literal(lit) = quad.object {
                    current_completed = lit
                        .value()
                        .parse::<usize>()
                        .map_err(|e| WorkflowError::InvalidInput(e.to_string()))?;
                }
            }

            // Increment completed_count
            let new_completed = current_completed + 1;

            // Remove old completed_count
            let quads_to_remove: Vec<_> = store
                .quads_for_pattern(
                    Some(gate_subject.as_ref()),
                    Some(completed_pred.as_ref()),
                    None,
                    None,
                )
                .collect::<Result<_, _>>()
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            for quad in quads_to_remove {
                store
                    .remove(&quad)
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            }

            // Add new completed_count
            let new_completed_obj =
                Literal::new_typed_literal(new_completed.to_string(), xsd::INTEGER.into_owned());

            store
                .insert(&Quad::new(
                    gate_subject.clone(),
                    completed_pred,
                    new_completed_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Get target_count
            let target_pred = NamedNode::new(&format!("{}target_count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let mut target_count = 0usize;
            for quad in store.quads_for_pattern(
                Some(gate_subject.as_ref()),
                Some(target_pred.as_ref()),
                None,
                None,
            ) {
                let quad = quad.map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                if let Term::Literal(lit) = quad.object {
                    target_count = lit
                        .value()
                        .parse::<usize>()
                        .map_err(|e| WorkflowError::InvalidInput(e.to_string()))?;
                }
            }

            // Check if gate should open
            if new_completed >= target_count {
                // Update status to Completed
                let status_pred = NamedNode::new(&format!("{}status", knhk_ns))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

                // Remove old status
                let status_quads: Vec<_> = store
                    .quads_for_pattern(
                        Some(gate_subject.as_ref()),
                        Some(status_pred.as_ref()),
                        None,
                        None,
                    )
                    .collect::<Result<_, _>>()
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

                for quad in status_quads {
                    store
                        .remove(&quad)
                        .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                }

                // Add new status
                let new_status = Literal::new_simple_literal(SyncGateStatus::Completed.as_str());
                store
                    .insert(&Quad::new(
                        gate_subject,
                        status_pred,
                        new_status,
                        GraphNameRef::DefaultGraph,
                    ))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

                return Ok(true); // Gate opens
            }

            Ok(false) // Still waiting
        }

        #[cfg(not(feature = "rdf"))]
        Ok(false)
    }

    /// Get gate status
    pub async fn get_status(&self, gate_id: &str) -> WorkflowResult<SyncGateStatus> {
        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.read().await;
            let gate_subject =
                NamedNode::new(&format!("http://knhk.io/workflow/{}", gate_id))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";
            let status_pred = NamedNode::new(&format!("{}status", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            for quad in store.quads_for_pattern(
                Some(gate_subject.as_ref()),
                Some(status_pred.as_ref()),
                None,
                None,
            ) {
                let quad = quad.map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                if let Term::Literal(lit) = quad.object {
                    return match lit.value() {
                        "waiting" => Ok(SyncGateStatus::Waiting),
                        "completed" => Ok(SyncGateStatus::Completed),
                        "cancelled" => Ok(SyncGateStatus::Cancelled),
                        _ => Err(WorkflowError::InvalidInput(format!(
                            "Invalid sync gate status: {}",
                            lit.value()
                        ))),
                    };
                }
            }

            Err(WorkflowError::InvalidInput(format!(
                "Sync gate not found: {}",
                gate_id
            )))
        }

        #[cfg(not(feature = "rdf"))]
        Ok(SyncGateStatus::Waiting)
    }

    /// Get progress (completed / target)
    pub async fn get_progress(&self, gate_id: &str) -> WorkflowResult<(usize, usize)> {
        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.read().await;
            let gate_subject =
                NamedNode::new(&format!("http://knhk.io/workflow/{}", gate_id))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";

            // Get completed_count
            let completed_pred = NamedNode::new(&format!("{}completed_count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let mut completed = 0usize;
            for quad in store.quads_for_pattern(
                Some(gate_subject.as_ref()),
                Some(completed_pred.as_ref()),
                None,
                None,
            ) {
                let quad = quad.map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                if let Term::Literal(lit) = quad.object {
                    completed = lit
                        .value()
                        .parse::<usize>()
                        .map_err(|e| WorkflowError::InvalidInput(e.to_string()))?;
                }
            }

            // Get target_count
            let target_pred = NamedNode::new(&format!("{}target_count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let mut target = 0usize;
            for quad in store.quads_for_pattern(
                Some(gate_subject.as_ref()),
                Some(target_pred.as_ref()),
                None,
                None,
            ) {
                let quad = quad.map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                if let Term::Literal(lit) = quad.object {
                    target = lit
                        .value()
                        .parse::<usize>()
                        .map_err(|e| WorkflowError::InvalidInput(e.to_string()))?;
                }
            }

            Ok((completed, target))
        }

        #[cfg(not(feature = "rdf"))]
        Ok((0, 0))
    }
}

#[cfg(not(feature = "rdf"))]
impl Default for SyncGate {
    fn default() -> Self {
        Self::new()
    }
}

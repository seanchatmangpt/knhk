//! Instance tracking for Multiple Instance patterns
//!
//! Manages instance lifecycle in RDF store with proper synchronization.

use crate::case::CaseId;
use crate::error::{WorkflowError, WorkflowResult};
use crate::patterns::PatternId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "rdf")]
use oxigraph::store::Store;
#[cfg(feature = "rdf")]
use tokio::sync::RwLock;

/// Instance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstanceStatus {
    /// Instance created but not started
    Pending,
    /// Instance currently executing
    Running,
    /// Instance completed successfully
    Completed,
    /// Instance failed
    Failed,
    /// Instance cancelled
    Cancelled,
}

impl InstanceStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Instance metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceMetadata {
    /// Instance ID
    pub instance_id: usize,
    /// Parent case ID
    pub case_id: CaseId,
    /// Pattern ID
    pub pattern_id: PatternId,
    /// Instance status
    pub status: InstanceStatus,
    /// Input data for this instance
    pub input_data: Option<serde_json::Value>,
    /// Output data from this instance
    pub output_data: Option<serde_json::Value>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Started timestamp
    pub started_at: Option<DateTime<Utc>>,
    /// Completed timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

/// Instance tracker for RDF-based instance management
pub struct InstanceTracker {
    #[cfg(feature = "rdf")]
    rdf_store: Arc<RwLock<Store>>,
    #[cfg(not(feature = "rdf"))]
    _phantom: std::marker::PhantomData<()>,
}

impl InstanceTracker {
    /// Create new instance tracker
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

    /// Create instance set in RDF store
    pub async fn create_instance_set(
        &self,
        case_id: CaseId,
        pattern_id: PatternId,
        instance_count: usize,
    ) -> WorkflowResult<String> {
        let instance_set_id = format!("{}:pattern_{}:instances", case_id, pattern_id.0);

        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.write().await;

            // Create instance set triple
            let instance_set_subject = NamedNode::new(&format!(
                "http://knhk.io/workflow/{}",
                instance_set_id
            ))
            .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";
            let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let instance_set_class = NamedNode::new(&format!("{}InstanceSet", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            store
                .insert(&Quad::new(
                    instance_set_subject.clone(),
                    rdf_type,
                    instance_set_class,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add pattern property
            let pattern_pred = NamedNode::new(&format!("{}pattern", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let pattern_obj = NamedNode::new(&format!(
                "http://knhk.io/workflow/pattern:{}",
                pattern_id.0
            ))
            .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            store
                .insert(&Quad::new(
                    instance_set_subject.clone(),
                    pattern_pred,
                    pattern_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add count property
            let count_pred = NamedNode::new(&format!("{}count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let count_obj = Literal::new_typed_literal(
                instance_count.to_string(),
                xsd::INTEGER.into_owned(),
            );

            store
                .insert(&Quad::new(
                    instance_set_subject.clone(),
                    count_pred,
                    count_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add status property
            let status_pred = NamedNode::new(&format!("{}status", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let status_obj = Literal::new_simple_literal("running");

            store
                .insert(&Quad::new(
                    instance_set_subject.clone(),
                    status_pred,
                    status_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add created_at property
            let created_pred = NamedNode::new(&format!("{}created_at", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let created_obj =
                Literal::new_typed_literal(Utc::now().to_rfc3339(), xsd::DATE_TIME.into_owned());

            store
                .insert(&Quad::new(
                    instance_set_subject,
                    created_pred,
                    created_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
        }

        Ok(instance_set_id)
    }

    /// Create individual instance
    pub async fn create_instance(
        &self,
        instance_set_id: &str,
        metadata: &InstanceMetadata,
    ) -> WorkflowResult<String> {
        let instance_id = format!("{}:instance_{}", instance_set_id, metadata.instance_id);

        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.write().await;
            let instance_subject =
                NamedNode::new(&format!("http://knhk.io/workflow/{}", instance_id))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";
            let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let task_instance_class = NamedNode::new(&format!("{}TaskInstance", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            store
                .insert(&Quad::new(
                    instance_subject.clone(),
                    rdf_type,
                    task_instance_class,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add parent_case
            let parent_pred = NamedNode::new(&format!("{}parent_case", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let parent_obj = NamedNode::new(&format!(
                "http://knhk.io/workflow/{}",
                metadata.case_id
            ))
            .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            store
                .insert(&Quad::new(
                    instance_subject.clone(),
                    parent_pred,
                    parent_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add instance_id
            let id_pred = NamedNode::new(&format!("{}instance_id", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let id_obj = Literal::new_typed_literal(
                metadata.instance_id.to_string(),
                xsd::INTEGER.into_owned(),
            );

            store
                .insert(&Quad::new(
                    instance_subject.clone(),
                    id_pred,
                    id_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add status
            let status_pred = NamedNode::new(&format!("{}status", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let status_obj = Literal::new_simple_literal(metadata.status.as_str());

            store
                .insert(&Quad::new(
                    instance_subject.clone(),
                    status_pred,
                    status_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add input_data if present
            if let Some(ref input_data) = metadata.input_data {
                let input_pred = NamedNode::new(&format!("{}input_data", knhk_ns))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                let input_obj = Literal::new_simple_literal(input_data.to_string());

                store
                    .insert(&Quad::new(
                        instance_subject.clone(),
                        input_pred,
                        input_obj,
                        GraphNameRef::DefaultGraph,
                    ))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            }

            // Add created_at
            let created_pred = NamedNode::new(&format!("{}created_at", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let created_obj = Literal::new_typed_literal(
                metadata.created_at.to_rfc3339(),
                xsd::DATE_TIME.into_owned(),
            );

            store
                .insert(&Quad::new(
                    instance_subject.clone(),
                    created_pred,
                    created_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Add executor
            let executor_pred = NamedNode::new(&format!("{}executor", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            let executor_obj = Literal::new_simple_literal("work-stealing");

            store
                .insert(&Quad::new(
                    instance_subject,
                    executor_pred,
                    executor_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
        }

        Ok(instance_id)
    }

    /// Update instance status
    pub async fn update_status(
        &self,
        instance_id: &str,
        new_status: InstanceStatus,
    ) -> WorkflowResult<()> {
        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.write().await;
            let instance_subject =
                NamedNode::new(&format!("http://knhk.io/workflow/{}", instance_id))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";
            let status_pred = NamedNode::new(&format!("{}status", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Remove old status
            let quads_to_remove: Vec<_> = store
                .quads_for_pattern(
                    Some(instance_subject.as_ref()),
                    Some(status_pred.as_ref()),
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

            // Add new status
            let status_obj = Literal::new_simple_literal(new_status.as_str());
            store
                .insert(&Quad::new(
                    instance_subject.clone(),
                    status_pred,
                    status_obj,
                    GraphNameRef::DefaultGraph,
                ))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            // Update completed_at if completed
            if new_status == InstanceStatus::Completed {
                let completed_pred = NamedNode::new(&format!("{}completed_at", knhk_ns))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                let completed_obj = Literal::new_typed_literal(
                    Utc::now().to_rfc3339(),
                    xsd::DATE_TIME.into_owned(),
                );

                store
                    .insert(&Quad::new(
                        instance_subject,
                        completed_pred,
                        completed_obj,
                        GraphNameRef::DefaultGraph,
                    ))
                    .map_err(|e| WorkflowError::RdfError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// Get instance count for instance set
    pub async fn get_instance_count(&self, instance_set_id: &str) -> WorkflowResult<usize> {
        #[cfg(feature = "rdf")]
        {
            use oxigraph::model::*;

            let store = self.rdf_store.read().await;
            let instance_set_subject = NamedNode::new(&format!(
                "http://knhk.io/workflow/{}",
                instance_set_id
            ))
            .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            let knhk_ns = "http://knhk.io/workflow#";
            let count_pred = NamedNode::new(&format!("{}count", knhk_ns))
                .map_err(|e| WorkflowError::RdfError(e.to_string()))?;

            for quad in store.quads_for_pattern(
                Some(instance_set_subject.as_ref()),
                Some(count_pred.as_ref()),
                None,
                None,
            ) {
                let quad = quad.map_err(|e| WorkflowError::RdfError(e.to_string()))?;
                if let Term::Literal(lit) = quad.object {
                    return lit
                        .value()
                        .parse::<usize>()
                        .map_err(|e| WorkflowError::InvalidInput(e.to_string()));
                }
            }

            Err(WorkflowError::InvalidInput(format!(
                "Instance set not found: {}",
                instance_set_id
            )))
        }

        #[cfg(not(feature = "rdf"))]
        Ok(0)
    }
}

#[cfg(not(feature = "rdf"))]
impl Default for InstanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

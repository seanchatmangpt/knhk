//! Connector integration for external systems

use crate::error::WorkflowResult;
use knhk_connectors::Connector;

/// Connector integration for external task execution
pub struct ConnectorIntegration {
    connectors: std::collections::HashMap<String, Box<dyn Connector>>,
}

impl ConnectorIntegration {
    /// Create new connector integration
    pub fn new() -> Self {
        Self {
            connectors: std::collections::HashMap::new(),
        }
    }

    /// Register a connector
    pub fn register_connector(&mut self, name: String, connector: Box<dyn Connector>) {
        self.connectors.insert(name, connector);
    }

    /// Execute a task via connector
    pub async fn execute_task(
        &mut self,
        connector_name: &str,
        _data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        let _connector = self.connectors.get_mut(connector_name).ok_or_else(|| {
            crate::error::WorkflowError::ResourceUnavailable(format!(
                "Connector {} not found",
                connector_name
            ))
        })?;

        // Execute task by fetching delta from connector and transforming to task result
        // Connectors are data sources - fetch delta and return as task execution result
        let delta = connector.fetch_delta().map_err(|e| {
            crate::error::WorkflowError::TaskExecutionFailed(format!(
                "Connector {} failed to fetch delta: {:?}",
                connector_name, e
            ))
        })?;

        // Transform delta to JSON result for task execution
        // Delta contains additions and removals of triples
        let additions_json: Vec<serde_json::Value> = delta
            .additions
            .iter()
            .map(|t| {
                serde_json::json!({
                    "subject": t.subject,
                    "predicate": t.predicate,
                    "object": t.object,
                    "graph": t.graph
                })
            })
            .collect();

        let removals_json: Vec<serde_json::Value> = delta
            .removals
            .iter()
            .map(|t| {
                serde_json::json!({
                    "subject": t.subject,
                    "predicate": t.predicate,
                    "object": t.object,
                    "graph": t.graph
                })
            })
            .collect();

        let result = serde_json::json!({
            "additions": additions_json,
            "removals": removals_json,
            "actor": delta.actor,
            "timestamp_ms": delta.timestamp_ms
        });

        Ok(result)
    }
}

impl Default for ConnectorIntegration {
    fn default() -> Self {
        Self::new()
    }
}

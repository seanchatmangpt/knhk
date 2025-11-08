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
        data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        let connector = self.connectors.get_mut(connector_name).ok_or_else(|| {
            crate::error::WorkflowError::ResourceUnavailable(format!(
                "Connector {} not found",
                connector_name
            ))
        })?;

        // Execute connector task
        // Note: Connector trait doesn't have execute_task method, so we use fetch_delta as a proxy
        // FUTURE: Extend Connector trait with execute_task method if needed
        match connector.as_mut().fetch_delta() {
            Ok(_delta) => {
                // Return the input data transformed (placeholder - actual transformation depends on connector)
                Ok(data)
            }
            Err(e) => Err(crate::error::WorkflowError::ExternalSystem(format!(
                "Connector execution failed: {:?}",
                e
            ))),
        }
    }
}

impl Default for ConnectorIntegration {
    fn default() -> Self {
        Self::new()
    }
}

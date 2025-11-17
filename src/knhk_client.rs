// KNHK API Client for Workflow Integration
// Async HTTP client for workflow case submission and task completion

use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum KnhkClientError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Workflow not found: {0}")]
    WorkflowNotFound(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCase {
    pub case_id: String,
    pub workflow_id: String,
    pub variables: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompletion {
    pub task_id: String,
    pub case_id: String,
    pub output: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub event_type: String,
    pub case_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: serde_json::Value,
}

/// KNHK API Client
pub struct KnhkClient {
    base_url: String,
    client: reqwest::Client,
}

impl KnhkClient {
    /// Create a new KNHK client
    pub fn new(base_url: String) -> Result<Self, KnhkClientError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            base_url,
            client,
        })
    }

    /// Submit a new workflow case
    pub async fn submit_case(&self, workflow_id: &str, variables: serde_json::Value) -> Result<WorkflowCase, KnhkClientError> {
        let url = format!("{}/api/v1/workflows/{}/cases", self.base_url, workflow_id);

        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "variables": variables
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KnhkClientError::ApiError(
                format!("Failed to submit case: {}", response.status())
            ));
        }

        let case: WorkflowCase = response.json().await?;
        Ok(case)
    }

    /// Complete a task
    pub async fn complete_task(&self, case_id: &str, task_id: &str, output: serde_json::Value) -> Result<(), KnhkClientError> {
        let url = format!("{}/api/v1/cases/{}/tasks/{}/complete", self.base_url, case_id, task_id);

        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "output": output
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KnhkClientError::ApiError(
                format!("Failed to complete task: {}", response.status())
            ));
        }

        Ok(())
    }

    /// Get case status
    pub async fn get_case_status(&self, case_id: &str) -> Result<serde_json::Value, KnhkClientError> {
        let url = format!("{}/api/v1/cases/{}", self.base_url, case_id);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KnhkClientError::ApiError(
                format!("Failed to get case status: {}", response.status())
            ));
        }

        let status = response.json().await?;
        Ok(status)
    }

    /// Get pending tasks for a case
    pub async fn get_pending_tasks(&self, case_id: &str) -> Result<Vec<serde_json::Value>, KnhkClientError> {
        let url = format!("{}/api/v1/cases/{}/tasks", self.base_url, case_id);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KnhkClientError::ApiError(
                format!("Failed to get tasks: {}", response.status())
            ));
        }

        let tasks = response.json().await?;
        Ok(tasks)
    }

    /// Listen for events (simulated with polling)
    pub async fn poll_events(&self, case_id: &str) -> Result<Vec<WorkflowEvent>, KnhkClientError> {
        let url = format!("{}/api/v1/cases/{}/events", self.base_url, case_id);

        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(KnhkClientError::ApiError(
                format!("Failed to poll events: {}", response.status())
            ));
        }

        let events = response.json().await?;
        Ok(events)
    }
}

/// Mock KNHK Client for testing (does not make real HTTP calls)
pub struct MockKnhkClient {
    cases: std::sync::Arc<tokio::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>>,
}

impl MockKnhkClient {
    pub fn new() -> Self {
        Self {
            cases: std::sync::Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub async fn submit_case(&self, workflow_id: &str, variables: serde_json::Value) -> Result<WorkflowCase, KnhkClientError> {
        let case_id = uuid::Uuid::new_v4().to_string();

        let case = WorkflowCase {
            case_id: case_id.clone(),
            workflow_id: workflow_id.to_string(),
            variables: variables.clone(),
        };

        let mut cases = self.cases.lock().await;
        cases.insert(case_id.clone(), serde_json::json!({
            "case_id": case_id,
            "workflow_id": workflow_id,
            "variables": variables,
            "status": "running",
        }));

        Ok(case)
    }

    pub async fn complete_task(&self, case_id: &str, task_id: &str, output: serde_json::Value) -> Result<(), KnhkClientError> {
        // In mock mode, just log the completion
        tracing::info!(
            case_id = %case_id,
            task_id = %task_id,
            "Task completed"
        );
        Ok(())
    }

    pub async fn get_case_status(&self, case_id: &str) -> Result<serde_json::Value, KnhkClientError> {
        let cases = self.cases.lock().await;
        cases.get(case_id)
            .cloned()
            .ok_or_else(|| KnhkClientError::WorkflowNotFound(case_id.to_string()))
    }

    pub async fn get_pending_tasks(&self, case_id: &str) -> Result<Vec<serde_json::Value>, KnhkClientError> {
        // Return mock tasks
        Ok(vec![
            serde_json::json!({
                "task_id": "task_1",
                "name": "qualify_lead",
                "case_id": case_id,
            }),
        ])
    }

    pub async fn poll_events(&self, case_id: &str) -> Result<Vec<WorkflowEvent>, KnhkClientError> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_client() {
        let client = MockKnhkClient::new();

        let case = client.submit_case("test_workflow", serde_json::json!({
            "test": "data"
        })).await.unwrap();

        assert!(!case.case_id.is_empty());
        assert_eq!(case.workflow_id, "test_workflow");
    }

    #[tokio::test]
    async fn test_task_completion() {
        let client = MockKnhkClient::new();

        let case = client.submit_case("test_workflow", serde_json::json!({})).await.unwrap();

        let result = client.complete_task(&case.case_id, "task_1", serde_json::json!({
            "result": "success"
        })).await;

        assert!(result.is_ok());
    }
}

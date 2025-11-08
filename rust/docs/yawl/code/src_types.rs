use serde::{Deserialize, Serialize};

pub type WorkflowSpecId = String;
pub type CaseId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSpec {
    pub id: WorkflowSpecId,
    pub name: String,
    pub tasks: serde_json::Value,
    pub conditions: serde_json::Value,
    pub start_condition: Option<String>,
    pub end_condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CaseState { Created, Running, Completed, Cancelled, Failed }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Case {
    pub id: CaseId,
    pub spec_id: WorkflowSpecId,
    pub state: CaseState,
    pub data: serde_json::Value,
}
impl Case {
    pub fn new(spec_id: WorkflowSpecId, data: serde_json::Value) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self { id, spec_id, state: CaseState::Created, data }
    }
    pub fn start(&mut self) { self.state = CaseState::Running; }
}

#[derive(thiserror::Error, Debug)]
pub enum StateError {
    #[error("persistence: {0}")] Persistence(String),
}
pub type StateResult<T> = Result<T, StateError>;
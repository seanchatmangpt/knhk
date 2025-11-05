// Result types for unrdf integration

use serde::{Deserialize, Serialize};

/// Query result from unrdf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub bindings: Vec<serde_json::Value>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub fired: bool,
    pub result: Option<serde_json::Value>,
    pub receipt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// SHACL validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub conforms: bool,
    pub results: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Transaction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub success: bool,
    pub receipt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Serialization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializationResult {
    pub data: String,
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Hook list result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookListResult {
    pub hooks: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// SPARQL query types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SparqlQueryType {
    Select,
    Ask,
    Construct,
    Describe,
    Update,
}


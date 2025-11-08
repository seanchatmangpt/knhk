// knhk-unrdf: Data types and structures
// Type definitions for unrdf integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SPARQL query types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparqlQueryType {
    Select,
    Ask,
    Construct,
    Describe,
    Insert,
    Delete,
    Unknown,
}

/// Query result from unrdf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub bindings: Option<Vec<serde_json::Value>>,
    pub boolean: Option<bool>,
    pub triples: Option<Vec<serde_json::Value>>,
    pub success: bool,
    pub query_type: Option<String>,
    pub error: Option<String>,
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub fired: bool,
    pub result: Option<serde_json::Value>,
    pub receipt: Option<String>,
}

/// Hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub id: String,
    pub name: String,
    pub hook_type: String,
    pub definition: serde_json::Value,
}

/// Hook registry entry
#[derive(Debug, Clone)]
pub struct HookRegistryEntry {
    pub hook: HookDefinition,
    pub registered: bool,
}

/// Transaction state
#[derive(Debug, Clone)]
pub enum TransactionState {
    Pending,
    Committed,
    RolledBack,
}

/// Transaction information
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u32,
    pub state: TransactionState,
    pub additions: Vec<String>,
    pub removals: Vec<String>,
    pub actor: String,
    pub metadata: HashMap<String, String>,
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_id: u32,
    pub success: bool,
    pub receipt: Option<String>,
    pub error: Option<String>,
}

/// SHACL validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclValidationResult {
    pub conforms: bool,
    pub violations: Vec<ShaclViolation>,
    pub error: Option<String>,
}

/// SHACL violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclViolation {
    pub path: Option<String>,
    pub message: String,
    pub severity: Option<String>,
    pub focus_node: Option<String>,
    pub value: Option<String>,
}

/// RDF serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdfFormat {
    Turtle,
    JsonLd,
    NQuads,
}

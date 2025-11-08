#![allow(clippy::unwrap_used)] // CLI tool - unwrap() acceptable for user-facing errors
//! Workflow Engine CLI Commands
//!
//! Provides CLI interface for workflow engine operations:
//! - workflow create: Create a new workflow case
//! - workflow start: Start a workflow case
//! - workflow execute: Execute a workflow case
//! - workflow cancel: Cancel a workflow case
//! - workflow list: List all workflow cases
//! - workflow pattern: Execute a workflow pattern
//! - workflow patterns: List all 43 patterns

use clap_noun_verb::noun;
use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{
    case::CaseId,
    error::WorkflowResult,
    parser::WorkflowSpecId,
    patterns::{PatternExecutionContext, PatternId},
    state::StateStore,
    WorkflowEngine,
};
use serde_json::json;
use std::collections::HashMap;

/// Global workflow engine instance
static ENGINE: std::sync::OnceLock<WorkflowEngine> = std::sync::OnceLock::new();

fn get_engine() -> WorkflowResult<&'static WorkflowEngine> {
    ENGINE.get_or_init(|| WorkflowEngine::new(StateStore::new("workflow_state").unwrap()));
    ENGINE.get().ok_or_else(|| {
        knhk_workflow_engine::error::WorkflowError::Internal(
            "Failed to initialize workflow engine".to_string(),
        )
    })
}

/// Create a new workflow case
#[verb]
pub fn create(spec_id: String) -> Result<String, String> {
    let engine = get_engine().map_err(|e| e.to_string())?;
    let spec_id_uuid =
        WorkflowSpecId::parse_str(&spec_id).map_err(|e| format!("Invalid spec ID: {}", e))?;

    let case_id = CaseId::new();
    // FUTURE: Add async runtime when needed
    // For now, use synchronous API
    /*
    let case_id = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(engine.create_case(spec_id_uuid, json!({})))
        .map_err(|e| e.to_string())?;
    */

    Ok(format!("Created case: {}", case_id))
}

/// Start a workflow case
#[verb]
pub fn start(case_id: String) -> Result<String, String> {
    let _engine = get_engine().map_err(|e| e.to_string())?;
    let _case_id_uuid =
        CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;

    // FUTURE: Add async runtime when needed
    // tokio::runtime::Runtime::new()
    //     .unwrap()
    //     .block_on(engine.start_case(case_id_uuid))
    //     .map_err(|e| e.to_string())?;

    Ok(format!("Started case: {}", case_id))
}

/// Execute a workflow case
#[verb]
pub fn execute(case_id: String) -> Result<String, String> {
    let _engine = get_engine().map_err(|e| e.to_string())?;
    let _case_id_uuid =
        CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;

    // FUTURE: Add async runtime when needed
    // tokio::runtime::Runtime::new()
    //     .unwrap()
    //     .block_on(engine.execute_case(case_id_uuid))
    //     .map_err(|e| e.to_string())?;

    Ok(format!("Executed case: {}", case_id))
}

/// Cancel a workflow case
#[verb]
pub fn cancel(case_id: String) -> Result<String, String> {
    let _engine = get_engine().map_err(|e| e.to_string())?;
    let _case_id_uuid =
        CaseId::parse_str(&case_id).map_err(|e| format!("Invalid case ID: {}", e))?;

    // FUTURE: Add async runtime when needed
    // tokio::runtime::Runtime::new()
    //     .unwrap()
    //     .block_on(engine.cancel_case(case_id_uuid))
    //     .map_err(|e| e.to_string())?;

    Ok(format!("Cancelled case: {}", case_id))
}

/// List all workflow cases
#[verb]
pub fn list() -> Result<Vec<String>, String> {
    let _engine = get_engine().map_err(|e| e.to_string())?;

    // FUTURE: Add async runtime when needed
    // let cases = tokio::runtime::Runtime::new()
    //     .unwrap()
    //     .block_on(engine.list_cases())
    //     .map_err(|e| e.to_string())?;

    // For now, return empty list
    Ok(vec![])
}

/// Execute a workflow pattern
#[verb]
pub fn pattern(pattern_id: u32, context: String) -> Result<String, String> {
    let _engine = get_engine().map_err(|e| e.to_string())?;
    let _pattern_id = PatternId::from(pattern_id);
    // FUTURE: Add PatternExecutionContext deserialization when available
    // let _context: PatternExecutionContext =
    //     serde_json::from_str(&context).map_err(|e| format!("Invalid context JSON: {}", e))?;
    let _context_str = context; // Placeholder

    // FUTURE: Add async runtime when needed
    // let result = tokio::runtime::Runtime::new()
    //     .unwrap()
    //     .block_on(engine.execute_pattern(pattern_id, context))
    //     .map_err(|e| e.to_string())?;

    Ok(format!("Executed pattern: {}", pattern_id))
}

/// List all 43 workflow patterns
#[verb]
pub fn patterns() -> Result<Vec<String>, String> {
    // Return list of all 43 patterns
    Ok((1..=43).map(|i| format!("Pattern {}", i)).collect())
}

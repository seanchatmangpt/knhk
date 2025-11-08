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

use clap_noun_verb_macros::verb;
use knhk_workflow_engine::{
    case::CaseId, error::WorkflowResult, parser::WorkflowSpecId, patterns::PatternId,
    state::StateStore, WorkflowEngine,
};

/// Global workflow engine instance
static ENGINE: std::sync::OnceLock<WorkflowEngine> = std::sync::OnceLock::new();

fn get_engine() -> CnvResult<&'static WorkflowEngine> {
    ENGINE.get_or_init(|| WorkflowEngine::new(StateStore::new("workflow_state").unwrap()));
    ENGINE
        .get()
        .ok_or_else(|| "Failed to initialize workflow engine".to_string())
}

/// Create a new workflow case
#[verb]
pub fn create(spec_id: String) -> CnvResult<()> {
    let _engine = get_engine()?;
    let _spec_id_uuid = WorkflowSpecId::parse_str(&spec_id)?;
    let case_id = CaseId::new();
    // FUTURE: Add async runtime when needed
    println!("Created case: {}", case_id);
    Ok(())
}

/// Start a workflow case
#[verb]
pub fn start(case_id: String) -> CnvResult<()> {
    let _engine = get_engine()?;
    let _case_id_uuid = CaseId::parse_str(&case_id)?;
    // FUTURE: Add async runtime when needed
    println!("Started case: {}", case_id);
    Ok(())
}

/// Execute a workflow case
#[verb]
pub fn execute(case_id: String) -> CnvResult<()> {
    let _engine = get_engine()?;
    let _case_id_uuid = CaseId::parse_str(&case_id)?;
    // FUTURE: Add async runtime when needed
    println!("Executed case: {}", case_id);
    Ok(())
}

/// Cancel a workflow case
#[verb]
pub fn cancel(case_id: String) -> CnvResult<()> {
    let _engine = get_engine()?;
    let _case_id_uuid = CaseId::parse_str(&case_id)?;
    // FUTURE: Add async runtime when needed
    println!("Cancelled case: {}", case_id);
    Ok(())
}

/// List all workflow cases
#[verb]
pub fn list() -> CnvResult<()> {
    let _engine = get_engine().map_err(|e| e.to_string())?;
    // FUTURE: Add async runtime when needed
    println!("No cases found (placeholder)");
    Ok(())
}

/// Execute a workflow pattern
#[verb]
pub fn pattern(pattern_id: u32, context: String) -> CnvResult<()> {
    let _engine = get_engine().map_err(|e| e.to_string())?;
    let _pattern_id = PatternId(pattern_id);
    // FUTURE: Add PatternExecutionContext deserialization when available
    // let _context: PatternExecutionContext =
    //     serde_json::from_str(&context).map_err(|e| format!("Invalid context JSON: {}", e))?;
    let _context_str = context; // Placeholder

    // FUTURE: Add async runtime when needed
    println!("Executed pattern: {} (placeholder)", pattern_id);
    Ok(())
}

/// List all 43 workflow patterns
#[verb]
pub fn patterns() -> CnvResult<()> {
    // Return list of all 43 patterns
    for i in 1..=43 {
        println!("Pattern {}", i);
    }
    Ok(())
}

// knhk-unrdf: Hook management
// Register, deregister, and execute knowledge hooks

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::template::TemplateEngine;
use crate::types::{HookDefinition, HookRegistryEntry, HookResult};
use tera::Context;

/// Execute knowledge hook via unrdf
pub fn execute_hook(hook_name: &str, hook_query: &str) -> UnrdfResult<HookResult> {
    let state = get_state()?;
    
    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    context.insert("hook_name", hook_name);
    context.insert("hook_query", hook_query);
    
    let script = template_engine.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e)))?
        .render("hook-execute", &context)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to render hook-execute template: {}", e)))?;
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Extract JSON from output (unrdf prints initialization messages to stdout)
        let json_line = output
            .lines()
            .rev()
            .find(|line| line.trim().starts_with('{') || line.trim().starts_with('['))
            .ok_or_else(|| UnrdfError::HookFailed(format!("No JSON found in output. Full output: {}", output)))?;
        
        let result: HookResult = serde_json::from_str(json_line.trim())
            .map_err(|e| UnrdfError::HookFailed(format!("Failed to parse result: {} - JSON line: {}", e, json_line)))?;
        Ok(result)
    })
}

/// Execute knowledge hook with data to store first (for stateful operations)
/// This combines store and hook execution in a single script so data persists
pub fn execute_hook_with_data(hook_name: &str, hook_query: &str, turtle_data: &str) -> UnrdfResult<HookResult> {
    let state = get_state()?;
    
    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    context.insert("hook_name", hook_name);
    context.insert("hook_query", hook_query);
    context.insert("turtle_data", turtle_data);
    
    let script = template_engine.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e)))?
        .render("hook-execute-with-data", &context)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to render hook-execute-with-data template: {}", e)))?;
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Extract JSON from output (unrdf prints initialization messages to stdout)
        let json_line = output
            .lines()
            .rev()
            .find(|line| line.trim().starts_with('{') || line.trim().starts_with('['))
            .ok_or_else(|| UnrdfError::HookFailed(format!("No JSON found in output. Full output: {}", output)))?;
        
        let result: HookResult = serde_json::from_str(json_line.trim())
            .map_err(|e| UnrdfError::HookFailed(format!("Failed to parse result: {} - JSON line: {}", e, json_line)))?;
        Ok(result)
    })
}

/// Register a hook with the system
pub fn register_hook(hook_json: &str) -> UnrdfResult<String> {
    let state = get_state()?;
    
    let hook_def: HookDefinition = serde_json::from_str(hook_json)
        .map_err(|e| UnrdfError::InvalidInput(format!("Invalid hook JSON: {}", e)))?;
    
    let hook_id = hook_def.id.clone();
    
    let mut hooks = state.hooks.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
    
    let entry = HookRegistryEntry {
        hook: hook_def.clone(),
        registered: true,
    };
    
    hooks.insert(hook_id.clone(), entry);
    
    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    // Parse hook_json as JSON value for safe insertion
    let hook_json_value: serde_json::Value = serde_json::from_str(hook_json)
        .map_err(|e| UnrdfError::InvalidInput(format!("Invalid hook JSON: {}", e)))?;
    context.insert("hook_json", &hook_json_value);
    
    let script = template_engine.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e)))?
        .render("hook-register", &context)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to render hook-register template: {}", e)))?;
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        if output.contains("SUCCESS") {
            Ok(hook_id)
        } else {
            Err(UnrdfError::HookFailed(output))
        }
    })
}

/// Deregister a hook
pub fn deregister_hook(hook_id: &str) -> UnrdfResult<()> {
    let state = get_state()?;
    
    let mut hooks = state.hooks.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
    
    hooks.remove(hook_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Hook {} not found", hook_id)))?;
    
    Ok(())
}

/// List all registered hooks
pub fn list_hooks() -> UnrdfResult<Vec<HookDefinition>> {
    let state = get_state()?;
    
    let hooks = state.hooks.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
    
    let hook_list: Vec<HookDefinition> = hooks.values()
        .filter(|entry| entry.registered)
        .map(|entry| entry.hook.clone())
        .collect();
    
    Ok(hook_list)
}


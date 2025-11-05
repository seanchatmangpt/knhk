// knhk-unrdf: Hook management
// Register, deregister, and execute knowledge hooks

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::types::{HookDefinition, HookRegistryEntry, HookResult};

/// Execute knowledge hook via unrdf
pub fn execute_hook(hook_name: &str, hook_query: &str) -> UnrdfResult<HookResult> {
    let state = get_state()?;
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore, defineHook, registerHook }} from './src/knowledge-engine/index.mjs';
        import {{ evaluateHook }} from './src/knowledge-engine/hook-management.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const hook = defineHook({{
                meta: {{
                    name: '{}',
                    description: 'KNHK hook'
                }},
                when: {{
                    kind: 'sparql-ask',
                    query: `{}`
                }},
                run: async (event) => {{
                    return {{ result: event.result ? 'Hook fired' : 'Hook not fired' }};
                }}
            }});
        
            await registerHook(hook);
            const receipt = await evaluateHook(hook, {{ persist: false }});
        
            console.log(JSON.stringify({{
                fired: receipt.fired || false,
                result: receipt.result || null,
                receipt: receipt.receipt || null
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ fired: false, result: null, error: err.message }}));
            process.exit(1);
        }});
        "#,
        hook_name,
        hook_query.replace('`', "\\`").replace('$', "\\$")
    );
    
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
    
    let escaped_query = hook_query.replace('`', "\\`").replace('$', "\\$");
    let escaped_data = turtle_data.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore, defineHook, registerHook }} from './src/knowledge-engine/index.mjs';
        import {{ evaluateHook }} from './src/knowledge-engine/hook-management.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            // Store data first
            const turtleData = `{}`;
            const store = await parseTurtle(turtleData);
            const quads = [];
            store.forEach(q => quads.push(q));
            await system.executeTransaction({{
                additions: quads,
                removals: [],
                actor: 'knhk-rust'
            }});
        
            // Then execute hook
            const hook = defineHook({{
                meta: {{
                    name: '{}',
                    description: 'KNHK hook'
                }},
                when: {{
                    kind: 'sparql-ask',
                    query: `{}`
                }},
                run: async (event) => {{
                    return {{ result: event.result ? 'Hook fired' : 'Hook not fired' }};
                }}
            }});
        
            await registerHook(hook);
            const receipt = await evaluateHook(hook, {{ persist: false }});
        
            console.log(JSON.stringify({{
                fired: receipt.fired || false,
                result: receipt.result || null,
                receipt: receipt.receipt || null
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ fired: false, result: null, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_data,
        hook_name,
        escaped_query
    );
    
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
    
    // Register hook in unrdf system
    let escaped_json = hook_json.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    let script = format!(
        r#"
        import {{ createDarkMatterCore, defineHook, registerHook }} from './src/knowledge-engine/index.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const hookDef = {};
            const hook = defineHook(hookDef);
            await registerHook(hook);
        
            console.log('SUCCESS');
        }}
        
        main().catch(err => {{
            console.error('ERROR:', err.message);
            process.exit(1);
        }});
        "#,
        escaped_json
    );
    
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


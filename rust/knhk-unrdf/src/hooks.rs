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
        let result: HookResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::HookFailed(format!("Failed to parse result: {}", e)))?;
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


// Hook management for unrdf integration

use crate::errors::{UnrdfError, UnrdfResult};
use crate::state::UNRDF_STATE;
use crate::types::HookListResult;
use crate::utils::{escape_js_string, execute_unrdf_script};

/// Register a hook with the system
pub fn register_hook(hook_json: &str) -> UnrdfResult<String> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let escaped_hook = escape_js_string(hook_json);
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore, defineHook, registerHook }} from './src/knowledge-engine/index.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        import fs from 'fs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            // Load existing state if available
            try {{
                if (fs.existsSync('{}')) {{
                    const stateData = fs.readFileSync('{}', 'utf8');
                    const state = JSON.parse(stateData);
                    if (state.store) {{
                        const store = await parseTurtle(state.store);
                        const quads = [];
                        store.forEach(q => quads.push(q));
                        await system.executeTransaction({{
                            additions: quads,
                            removals: [],
                            actor: 'knhk-rust'
                        }});
                    }}
                }}
            }} catch (e) {{
                // Ignore errors loading state
            }}
        
            const hookDef = JSON.parse(`{}`);
            const hook = defineHook(hookDef);
            await registerHook(hook);
        
            console.log(JSON.stringify({{ hookId: hook.meta.name }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ hookId: null, error: err.message }}));
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str, escaped_hook
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::HookManagementFailed(format!("Failed to parse result: {}", e)))?;
        
        if let Some(hook_id) = result.get("hookId").and_then(|v| v.as_str()) {
            Ok(hook_id.to_string())
        } else {
            Err(UnrdfError::HookManagementFailed(
                result.get("error").and_then(|v| v.as_str())
                    .unwrap_or("Unknown error").to_string()
            ))
        }
    })
}

/// Deregister a hook from the system
pub fn deregister_hook(hook_id: &str) -> UnrdfResult<()> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let escaped_id = escape_js_string(hook_id);
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        import {{ deregisterHook }} from './src/knowledge-engine/hook-management.mjs';
        import fs from 'fs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            // Load existing state if available
            try {{
                if (fs.existsSync('{}')) {{
                    const stateData = fs.readFileSync('{}', 'utf8');
                    const state = JSON.parse(stateData);
                    if (state.store) {{
                        const store = await parseTurtle(state.store);
                        const quads = [];
                        store.forEach(q => quads.push(q));
                        await system.executeTransaction({{
                            additions: quads,
                            removals: [],
                            actor: 'knhk-rust'
                        }});
                    }}
                }}
            }} catch (e) {{
                // Ignore errors loading state
            }}
        
            await deregisterHook('{}');
        
            console.log(JSON.stringify({{ success: true }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str, escaped_id
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::HookManagementFailed(format!("Failed to parse result: {}", e)))?;
        
        if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
            Ok(())
        } else {
            Err(UnrdfError::HookManagementFailed(
                result.get("error").and_then(|v| v.as_str())
                    .unwrap_or("Unknown error").to_string()
            ))
        }
    })
}

/// List all registered hooks
pub fn list_hooks() -> UnrdfResult<HookListResult> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        import {{ listHooks }} from './src/knowledge-engine/hook-management.mjs';
        import fs from 'fs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            // Load existing state if available
            try {{
                if (fs.existsSync('{}')) {{
                    const stateData = fs.readFileSync('{}', 'utf8');
                    const state = JSON.parse(stateData);
                    if (state.store) {{
                        const store = await parseTurtle(state.store);
                        const quads = [];
                        store.forEach(q => quads.push(q));
                        await system.executeTransaction({{
                            additions: quads,
                            removals: [],
                            actor: 'knhk-rust'
                        }});
                    }}
                }}
            }} catch (e) {{
                // Ignore errors loading state
            }}
        
            const hooks = await listHooks();
            const hooksList = hooks.map(h => ({{
                id: h.meta.name,
                description: h.meta.description || null,
                kind: h.when.kind || null
            }}));
        
            console.log(JSON.stringify({{ hooks: hooksList }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ hooks: [], error: err.message }}));
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: HookListResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::HookManagementFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}


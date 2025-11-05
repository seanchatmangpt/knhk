// Transaction management for unrdf integration

use crate::errors::{UnrdfError, UnrdfResult};
use crate::state::UNRDF_STATE;
use crate::types::TransactionResult;
use crate::utils::{escape_js_string, execute_unrdf_script};

/// Execute transaction with additions and removals
pub fn execute_transaction(additions_turtle: &str, removals_turtle: &str, actor: &str) -> UnrdfResult<TransactionResult> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let escaped_additions = escape_js_string(additions_turtle);
    let escaped_removals = escape_js_string(removals_turtle);
    let escaped_actor = escape_js_string(actor);
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        import {{ toTurtle }} from './src/knowledge-engine/serialize.mjs';
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
        
            const additionsTurtle = `{}`;
            const removalsTurtle = `{}`;
            const actor = '{}';
            
            const additionsStore = await parseTurtle(additionsTurtle);
            const removalsStore = removalsTurtle ? await parseTurtle(removalsTurtle) : null;
            
            const additions = [];
            additionsStore.forEach(q => additions.push(q));
            
            const removals = [];
            if (removalsStore) {{
                removalsStore.forEach(q => removals.push(q));
            }}
            
            const receipt = await system.executeTransaction({{
                additions: additions,
                removals: removals,
                actor: actor
            }});
            
            // Save state
            const currentStore = system.store;
            const serialized = await toTurtle(currentStore);
            fs.writeFileSync('{}', JSON.stringify({{ store: serialized }}));
        
            console.log(JSON.stringify({{
                success: true,
                receipt: receipt ? JSON.stringify(receipt) : null
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ success: false, receipt: null, error: err.message }}));
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str, escaped_additions, escaped_removals, escaped_actor, state_file_str
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: TransactionResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::TransactionFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

// Query operations for unrdf integration

use crate::errors::{UnrdfError, UnrdfResult};
use crate::state::UNRDF_STATE;
use crate::types::{HookResult, QueryResult};
use crate::utils::{detect_sparql_query_type, escape_js_string, execute_unrdf_script};
use crate::types::SparqlQueryType;

/// Store data in unrdf (uses persistent state)
pub fn store_turtle_data(turtle_data: &str) -> UnrdfResult<()> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    // Escape backticks and template literals in turtle data
    let escaped_data = escape_js_string(turtle_data);
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
        
            const turtleData = `{}`;
            const store = await parseTurtle(turtleData);
        
            const quads = [];
            store.forEach(q => quads.push(q));
        
            await system.executeTransaction({{
                additions: quads,
                removals: [],
                actor: 'knhk-rust'
            }});
        
            // Save state
            const currentStore = system.store;
            const serialized = await toTurtle(currentStore);
            fs.writeFileSync('{}', JSON.stringify({{ store: serialized }}));
        
            console.log('SUCCESS');
        }}
        
        main().catch(err => {{
            console.error('ERROR:', err.message);
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str, escaped_data, state_file_str
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        if output.contains("SUCCESS") {
            Ok(())
        } else {
            Err(UnrdfError::StoreFailed(output))
        }
    })
}

/// Execute SPARQL query via unrdf (supports all query types)
pub fn query_sparql(query: &str) -> UnrdfResult<QueryResult> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let query_type = detect_sparql_query_type(query);
    let escaped_query = escape_js_string(query);
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
    // Determine query type string for unrdf
    let query_type_str = match query_type {
        SparqlQueryType::Select => "sparql-select",
        SparqlQueryType::Ask => "sparql-ask",
        SparqlQueryType::Construct => "sparql-construct",
        SparqlQueryType::Describe => "sparql-describe",
        SparqlQueryType::Update => "sparql-update",
    };
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
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
        
            const query = `{}`;
            
            let results;
            if ('{}' === 'sparql-ask') {{
                results = await system.query({{
                    query: query,
                    type: 'sparql-ask'
                }});
                console.log(JSON.stringify({{ bindings: [{{ result: results }}], success: true }}));
            }} else if ('{}' === 'sparql-update') {{
                await system.query({{
                    query: query,
                    type: 'sparql-update'
                }});
                // Save state after update
                const currentStore = system.store;
                const {{ toTurtle }} = await import('./src/knowledge-engine/serialize.mjs');
                const serialized = await toTurtle(currentStore);
                fs.writeFileSync('{}', JSON.stringify({{ store: serialized }}));
                console.log(JSON.stringify({{ bindings: [{{ success: true }}], success: true }}));
            }} else {{
                results = await system.query({{
                    query: query,
                    type: '{}'
                }});
                console.log(JSON.stringify({{ bindings: results, success: true }}));
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ bindings: [], success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str, escaped_query, query_type_str, query_type_str, state_file_str, query_type_str
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: QueryResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

/// Execute knowledge hook via unrdf (uses persistent state)
pub fn execute_hook(hook_name: &str, hook_query: &str) -> UnrdfResult<HookResult> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let escaped_query = escape_js_string(hook_query);
    let escaped_hook_name = escape_js_string(hook_name);
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore, defineHook, registerHook }} from './src/knowledge-engine/index.mjs';
        import {{ evaluateHook }} from './src/knowledge-engine/hook-management.mjs';
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
        state_file_str, state_file_str, escaped_hook_name, escaped_query
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: HookResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::HookFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}


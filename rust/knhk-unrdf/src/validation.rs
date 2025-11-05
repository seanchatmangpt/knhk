// SHACL validation for unrdf integration

use crate::errors::{UnrdfError, UnrdfResult};
use crate::state::UNRDF_STATE;
use crate::types::ValidationResult;
use crate::utils::{escape_js_string, execute_unrdf_script};

/// Validate SHACL shapes against data graph
pub fn validate_shacl(data_turtle: &str, shapes_turtle: &str) -> UnrdfResult<ValidationResult> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let escaped_data = escape_js_string(data_turtle);
    let escaped_shapes = escape_js_string(shapes_turtle);
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
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
        
            const dataTurtle = `{}`;
            const shapesTurtle = `{}`;
            
            const dataStore = await parseTurtle(dataTurtle);
            const shapesStore = await parseTurtle(shapesTurtle);
            
            const validation = await system.validate({{
                dataGraph: dataStore,
                shapesGraph: shapesStore
            }});
        
            console.log(JSON.stringify({{
                conforms: validation.conforms || false,
                results: validation.results || []
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ conforms: false, results: [], error: err.message }}));
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str, escaped_data, escaped_shapes
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: ValidationResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::ValidationFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

// RDF serialization for unrdf integration

use crate::errors::{UnrdfError, UnrdfResult};
use crate::state::UNRDF_STATE;
use crate::types::SerializationResult;
use crate::utils::{escape_js_string, execute_unrdf_script};

/// Serialize store to Turtle format
pub fn serialize_to_turtle() -> UnrdfResult<SerializationResult> {
    serialize_store("turtle")
}

/// Serialize store to JSON-LD format
pub fn serialize_to_jsonld() -> UnrdfResult<SerializationResult> {
    serialize_store("jsonld")
}

/// Serialize store to N-Quads format
pub fn serialize_to_nquads() -> UnrdfResult<SerializationResult> {
    serialize_store("nquads")
}

/// Internal function to serialize store in various formats
fn serialize_store(format: &str) -> UnrdfResult<SerializationResult> {
    let state_lock = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let state = state_lock.lock()
        .map_err(|e| UnrdfError::StateManagementFailed(format!("Failed to acquire lock: {}", e)))?;
    
    let state_file_str = escape_js_string(&state.state_file.to_string_lossy());
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        import {{ toTurtle, toJsonLd, toNQuads }} from './src/knowledge-engine/serialize.mjs';
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
        
            const currentStore = system.store;
            let serialized;
            if ('{}' === 'turtle') {{
                serialized = await toTurtle(currentStore);
            }} else if ('{}' === 'jsonld') {{
                serialized = await toJsonLd(currentStore);
            }} else if ('{}' === 'nquads') {{
                serialized = await toNQuads(currentStore);
            }} else {{
                throw new Error('Unknown format: ' + '{}');
            }}
        
            console.log(JSON.stringify({{
                data: serialized,
                format: '{}'
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ data: '', format: '{}', error: err.message }}));
            process.exit(1);
        }});
        "#,
        state_file_str, state_file_str, format, format, format, format, format, format
    );
    
    let unrdf_path = state.unrdf_path.clone();
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script, &unrdf_path).await?;
        let result: SerializationResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::SerializationFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}


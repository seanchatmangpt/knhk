// knhk-unrdf: Rust integration layer for unrdf knowledge hook engine
// Provides FFI-safe interface for cold path integration

use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tokio::process::Command;

/// Error types for unrdf integration
#[derive(Debug, thiserror::Error)]
pub enum UnrdfError {
    #[error("Failed to initialize unrdf: {0}")]
    InitializationFailed(String),
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    #[error("Store operation failed: {0}")]
    StoreFailed(String),
    #[error("Hook execution failed: {0}")]
    HookFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for unrdf operations
pub type UnrdfResult<T> = Result<T, UnrdfError>;

/// Query result from unrdf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub bindings: Vec<serde_json::Value>,
    pub success: bool,
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub fired: bool,
    pub result: Option<serde_json::Value>,
    pub receipt: Option<String>,
}

/// Internal state for unrdf integration
struct UnrdfState {
    runtime: Runtime,
    unrdf_path: String,
}

static UNRDF_STATE: OnceLock<UnrdfState> = OnceLock::new();

/// Initialize unrdf integration layer
/// Must be called before any other operations
pub fn init_unrdf(unrdf_path: &str) -> Result<(), UnrdfError> {
    let runtime = Runtime::new()
        .map_err(|e| UnrdfError::InitializationFailed(format!("Failed to create runtime: {}", e)))?;
    
    let state = UnrdfState {
        runtime,
        unrdf_path: unrdf_path.to_string(),
    };
    
    UNRDF_STATE.set(state)
        .map_err(|_| UnrdfError::InitializationFailed("unrdf already initialized".to_string()))?;
    
    Ok(())
}

/// Execute a script using Node.js and unrdf
async fn execute_unrdf_script(script_content: &str) -> Result<String, UnrdfError> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    // Write script to temporary file
    let temp_file = std::env::temp_dir().join(format!("knhk_unrdf_{}.mjs", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::write(&temp_file, script_content)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to write script: {}", e)))?;
    
    // Execute via Node.js
    let output = Command::new("node")
        .arg(&temp_file)
        .current_dir(&state.unrdf_path)
        .output()
        .await
        .map_err(|e| UnrdfError::QueryFailed(format!("Failed to execute node: {}", e)))?;
    
    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(UnrdfError::QueryFailed(format!("Script failed: stderr={}, stdout={}", stderr, stdout)));
    }
    
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| UnrdfError::QueryFailed(format!("Invalid output: {}", e)))?;
    
    Ok(stdout)
}

/// Store data in unrdf
pub fn store_turtle_data(turtle_data: &str) -> UnrdfResult<()> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    // Escape backticks and template literals in turtle data
    let escaped_data = turtle_data.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const turtleData = `{}`;
            const store = await parseTurtle(turtleData);
        
            const quads = [];
            store.forEach(q => quads.push(q));
        
            await system.executeTransaction({{
                additions: quads,
                removals: [],
                actor: 'knhk-rust'
            }});
        
            console.log('SUCCESS');
        }}
        
        main().catch(err => {{
            console.error('ERROR:', err.message);
            process.exit(1);
        }});
        "#,
        escaped_data
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        if output.contains("SUCCESS") {
            Ok(())
        } else {
            Err(UnrdfError::StoreFailed(output))
        }
    })
}

/// Execute SPARQL query via unrdf
pub fn query_sparql(query: &str) -> UnrdfResult<QueryResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const query = `{}`;
        
            const results = await system.query({{
                query: query,
                type: 'sparql-select'
            }});
        
            console.log(JSON.stringify({{ bindings: results, success: true }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ bindings: [], success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        query.replace('`', "\\`").replace('$', "\\$")
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: QueryResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

/// Execute knowledge hook via unrdf
pub fn execute_hook(hook_name: &str, hook_query: &str) -> UnrdfResult<HookResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
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

// FFI exports for C integration

#[no_mangle]
pub extern "C" fn knhk_unrdf_init(unrdf_path: *const c_char) -> c_int {
    let path = unsafe {
        CStr::from_ptr(unrdf_path)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid path".to_string()))
    };
    
    match path {
        Ok(p) => {
            match init_unrdf(p) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_store_turtle(turtle_data: *const c_char) -> c_int {
    let data = unsafe {
        CStr::from_ptr(turtle_data)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data".to_string()))
    };
    
    match data {
        Ok(d) => {
            match store_turtle_data(d) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_query(query: *const c_char, result_json: *mut c_char, result_size: usize) -> c_int {
    let q = unsafe {
        CStr::from_ptr(query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid query".to_string()))
    };
    
    match q {
        Ok(query_str) => {
            match query_sparql(query_str) {
                Ok(result) => {
                    match serde_json::to_string(&result) {
                        Ok(json) => {
                            let json_bytes = json.as_bytes();
                            if json_bytes.len() < result_size {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        json_bytes.as_ptr(),
                                        result_json as *mut u8,
                                        json_bytes.len()
                                    );
                                    *result_json.add(json_bytes.len()) = 0;
                                }
                                0
                            } else {
                                -1
                            }
                        }
                        Err(_) => -1,
                    }
                }
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_execute_hook(
    hook_name: *const c_char,
    hook_query: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    let name = unsafe {
        CStr::from_ptr(hook_name)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook name".to_string()))
    };
    
    let query = unsafe {
        CStr::from_ptr(hook_query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook query".to_string()))
    };
    
    match (name, query) {
        (Ok(n), Ok(q)) => {
            match execute_hook(n, q) {
                Ok(result) => {
                    match serde_json::to_string(&result) {
                        Ok(json) => {
                            let json_bytes = json.as_bytes();
                            if json_bytes.len() < result_size {
                                unsafe {
                                    std::ptr::copy_nonoverlapping(
                                        json_bytes.as_ptr(),
                                        result_json as *mut u8,
                                        json_bytes.len()
                                    );
                                    *result_json.add(json_bytes.len()) = 0;
                                }
                                0
                            } else {
                                -1
                            }
                        }
                        Err(_) => -1,
                    }
                }
                Err(_) => -1,
            }
        }
        _ => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // Test would require unrdf path
        // This is a placeholder for integration tests
    }
}


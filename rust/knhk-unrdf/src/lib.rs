// knhk-unrdf: Rust integration layer for unrdf knowledge hook engine
// Provides FFI-safe interface for cold path integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::{Arc, Mutex, OnceLock};
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
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Error code enumeration for FFI
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnrdfErrorCode {
    Success = 0,
    InitializationFailed = -1,
    QueryFailed = -2,
    StoreFailed = -3,
    HookFailed = -4,
    TransactionFailed = -5,
    ValidationFailed = -6,
    SerializationFailed = -7,
    InvalidInput = -8,
}

impl From<&UnrdfError> for UnrdfErrorCode {
    fn from(err: &UnrdfError) -> Self {
        match err {
            UnrdfError::InitializationFailed(_) => UnrdfErrorCode::InitializationFailed,
            UnrdfError::QueryFailed(_) => UnrdfErrorCode::QueryFailed,
            UnrdfError::StoreFailed(_) => UnrdfErrorCode::StoreFailed,
            UnrdfError::HookFailed(_) => UnrdfErrorCode::HookFailed,
            UnrdfError::TransactionFailed(_) => UnrdfErrorCode::TransactionFailed,
            UnrdfError::ValidationFailed(_) => UnrdfErrorCode::ValidationFailed,
            UnrdfError::SerializationFailed(_) => UnrdfErrorCode::SerializationFailed,
            UnrdfError::InvalidInput(_) => UnrdfErrorCode::InvalidInput,
        }
    }
}

/// Result type for unrdf operations
pub type UnrdfResult<T> = Result<T, UnrdfError>;

/// SPARQL query types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SparqlQueryType {
    Select,
    Ask,
    Construct,
    Describe,
    Insert,
    Delete,
    Unknown,
}

/// Query result from unrdf
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub bindings: Option<Vec<serde_json::Value>>,
    pub boolean: Option<bool>,
    pub triples: Option<Vec<serde_json::Value>>,
    pub success: bool,
    pub query_type: Option<String>,
    pub error: Option<String>,
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub fired: bool,
    pub result: Option<serde_json::Value>,
    pub receipt: Option<String>,
}

/// Hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub id: String,
    pub name: String,
    pub hook_type: String,
    pub definition: serde_json::Value,
}

/// Hook registry entry
#[derive(Debug, Clone)]
pub struct HookRegistryEntry {
    pub hook: HookDefinition,
    pub registered: bool,
}

/// Transaction state
#[derive(Debug, Clone)]
pub enum TransactionState {
    Pending,
    Committed,
    RolledBack,
}

/// Transaction information
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: u32,
    pub state: TransactionState,
    pub additions: Vec<String>,
    pub removals: Vec<String>,
    pub actor: String,
    pub metadata: HashMap<String, String>,
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_id: u32,
    pub success: bool,
    pub receipt: Option<String>,
    pub error: Option<String>,
}

/// SHACL validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclValidationResult {
    pub conforms: bool,
    pub violations: Vec<ShaclViolation>,
    pub error: Option<String>,
}

/// SHACL violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaclViolation {
    pub path: Option<String>,
    pub message: String,
    pub severity: Option<String>,
    pub focus_node: Option<String>,
    pub value: Option<String>,
}

/// Internal state for unrdf integration
struct UnrdfState {
    runtime: Runtime,
    unrdf_path: String,
    transactions: Arc<Mutex<HashMap<u32, Transaction>>>,
    next_transaction_id: Arc<Mutex<u32>>,
    hooks: Arc<Mutex<HashMap<String, HookRegistryEntry>>>,
}

static UNRDF_STATE: OnceLock<UnrdfState> = OnceLock::new();

/// Initialize unrdf integration layer
/// Must be called before any other operations
pub fn init_unrdf(unrdf_path: &str) -> Result<(), UnrdfError> {
    // Verify unrdf directory exists
    let path = std::path::Path::new(unrdf_path);
    if !path.exists() {
        return Err(UnrdfError::InitializationFailed(format!("unrdf directory does not exist: {}", unrdf_path)));
    }
    if !path.is_dir() {
        return Err(UnrdfError::InitializationFailed(format!("unrdf path is not a directory: {}", unrdf_path)));
    }
    
    // Verify required files exist
    let knowledge_engine = path.join("src/knowledge-engine/knowledge-substrate-core.mjs");
    if !knowledge_engine.exists() {
        return Err(UnrdfError::InitializationFailed(format!("unrdf knowledge engine not found at: {}", knowledge_engine.display())));
    }
    
    let runtime = Runtime::new()
        .map_err(|e| UnrdfError::InitializationFailed(format!("Failed to create runtime: {}", e)))?;
    
    let state = UnrdfState {
        runtime,
        unrdf_path: unrdf_path.to_string(),
        transactions: Arc::new(Mutex::new(HashMap::new())),
        next_transaction_id: Arc::new(Mutex::new(1)),
        hooks: Arc::new(Mutex::new(HashMap::new())),
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
    
    // Trim whitespace and check for error messages
    let trimmed = stdout.trim();
    if trimmed.starts_with("ERROR:") || trimmed.contains("Error:") {
        return Err(UnrdfError::QueryFailed(format!("Script reported error: {}", trimmed)));
    }
    
    Ok(trimmed.to_string())
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

/// Detect SPARQL query type from query string
pub fn detect_query_type(query: &str) -> SparqlQueryType {
    let query_upper = query.trim().to_uppercase();
    
    // Check for UPDATE operations first (INSERT/DELETE)
    if query_upper.starts_with("INSERT") || query_upper.starts_with("DELETE") {
        if query_upper.contains("INSERT") {
            return SparqlQueryType::Insert;
        }
        if query_upper.contains("DELETE") {
            return SparqlQueryType::Delete;
        }
        return SparqlQueryType::Insert; // Default to Insert for UPDATE
    }
    
    // Check for query types
    if query_upper.starts_with("ASK") {
        return SparqlQueryType::Ask;
    }
    if query_upper.starts_with("CONSTRUCT") {
        return SparqlQueryType::Construct;
    }
    if query_upper.starts_with("DESCRIBE") {
        return SparqlQueryType::Describe;
    }
    if query_upper.starts_with("SELECT") {
        return SparqlQueryType::Select;
    }
    
    SparqlQueryType::Unknown
}

/// Execute SPARQL query via unrdf with automatic query type detection
pub fn query_sparql(query: &str) -> UnrdfResult<QueryResult> {
    let query_type = detect_query_type(query);
    query_sparql_with_type(query, query_type)
}

/// Execute SPARQL query via unrdf with explicit query type
pub fn query_sparql_with_type(query: &str, query_type: SparqlQueryType) -> UnrdfResult<QueryResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let query_type_str = match query_type {
        SparqlQueryType::Select => "sparql-select",
        SparqlQueryType::Ask => "sparql-ask",
        SparqlQueryType::Construct => "sparql-construct",
        SparqlQueryType::Describe => "sparql-describe",
        SparqlQueryType::Insert | SparqlQueryType::Delete => "sparql-update",
        SparqlQueryType::Unknown => {
            return Err(UnrdfError::InvalidInput("Unknown query type".to_string()));
        }
    };
    
    let escaped_query = query.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const query = `{}`;
            const queryType = '{}';
        
            let results;
            let resultData = {{ success: true, query_type: queryType }};
        
            try {{
                if (queryType === 'sparql-ask') {{
                    results = await system.query({{
                        query: query,
                        type: queryType
                    }});
                    resultData.boolean = results;
                }} else if (queryType === 'sparql-construct' || queryType === 'sparql-describe') {{
                    results = await system.query({{
                        query: query,
                        type: queryType
                    }});
                    const triples = [];
                    for await (const quad of results) {{
                        triples.push({{
                            subject: quad.subject.value,
                            predicate: quad.predicate.value,
                            object: quad.object.value,
                            graph: quad.graph ? quad.graph.value : null
                        }});
                    }}
                    resultData.triples = triples;
                }} else if (queryType === 'sparql-update') {{
                    await system.query({{
                        query: query,
                        type: queryType
                    }});
                    resultData.success = true;
                }} else {{
                    // SELECT query
                    results = await system.query({{
                        query: query,
                        type: queryType
                    }});
                    const bindings = [];
                    for await (const binding of results) {{
                        const bindingObj = {{}};
                        for (const [key, value] of binding) {{
                            bindingObj[key] = value.value;
                        }}
                        bindings.push(bindingObj);
                    }}
                    resultData.bindings = bindings;
                }}
        
                console.log(JSON.stringify(resultData));
            }} catch (err) {{
                console.error(JSON.stringify({{
                    success: false,
                    query_type: queryType,
                    error: err.message
                }}));
                process.exit(1);
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{
                success: false,
                error: err.message
            }}));
            process.exit(1);
        }});
        "#,
        escaped_query,
        query_type_str
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: QueryResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {} - output: {}", e, output)))?;
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

/// Register a hook with the system
pub fn register_hook(hook_json: &str) -> UnrdfResult<String> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
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
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut hooks = state.hooks.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
    
    hooks.remove(hook_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Hook {} not found", hook_id)))?;
    
    Ok(())
}

/// List all registered hooks
pub fn list_hooks() -> UnrdfResult<Vec<HookDefinition>> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let hooks = state.hooks.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
    
    let hook_list: Vec<HookDefinition> = hooks.values()
        .filter(|entry| entry.registered)
        .map(|entry| entry.hook.clone())
        .collect();
    
    Ok(hook_list)
}

/// Begin a new transaction
pub fn begin_transaction(actor: &str) -> UnrdfResult<u32> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut next_id = state.next_transaction_id.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transaction ID lock: {}", e)))?;
    
    let transaction_id = *next_id;
    *next_id += 1;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = Transaction {
        id: transaction_id,
        state: TransactionState::Pending,
        additions: Vec::new(),
        removals: Vec::new(),
        actor: actor.to_string(),
        metadata: HashMap::new(),
    };
    
    transactions.insert(transaction_id, transaction);
    
    Ok(transaction_id)
}

/// Add data to a transaction
pub fn transaction_add(transaction_id: u32, turtle_data: &str) -> UnrdfResult<()> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            transaction.additions.push(turtle_data.to_string());
            Ok(())
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}

/// Remove data from a transaction
pub fn transaction_remove(transaction_id: u32, turtle_data: &str) -> UnrdfResult<()> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            transaction.removals.push(turtle_data.to_string());
            Ok(())
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}

/// Commit a transaction
pub fn commit_transaction(transaction_id: u32) -> UnrdfResult<TransactionReceipt> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            // Clone transaction data for script execution
            let additions = transaction.additions.clone();
            let removals = transaction.removals.clone();
            let actor = transaction.actor.clone();
            
            // Escape turtle data
            let escaped_additions: Vec<String> = additions.iter()
                .map(|s| s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$"))
                .collect();
            let escaped_removals: Vec<String> = removals.iter()
                .map(|s| s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$"))
                .collect();
            
            // Build additions array
            let additions_js = escaped_additions.iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join(",\n                ");
            
            // Build removals array
            let removals_js = escaped_removals.iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join(",\n                ");
            
            let script = format!(
                r#"
                import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
                import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
                
                async function main() {{
                    const system = await createDarkMatterCore({{
                        enableKnowledgeHookManager: true,
                        enableLockchainWriter: false
                    }});
                
                    const additionsData = [
                        {}
                    ];
                    const removalsData = [
                        {}
                    ];
                
                    const additionsQuads = [];
                    for (const turtleData of additionsData) {{
                        const store = await parseTurtle(turtleData);
                        store.forEach(q => additionsQuads.push(q));
                    }}
                
                    const removalsQuads = [];
                    for (const turtleData of removalsData) {{
                        const store = await parseTurtle(turtleData);
                        store.forEach(q => removalsQuads.push(q));
                    }}
                
                    const receipt = await system.executeTransaction({{
                        additions: additionsQuads,
                        removals: removalsQuads,
                        actor: '{}'
                    }});
                
                    console.log(JSON.stringify({{
                        transaction_id: {},
                        success: true,
                        receipt: receipt ? JSON.stringify(receipt) : null
                    }}));
                }}
                
                main().catch(err => {{
                    console.error(JSON.stringify({{
                        transaction_id: {},
                        success: false,
                        error: err.message
                    }}));
                    process.exit(1);
                }});
                "#,
                additions_js,
                removals_js,
                actor,
                transaction_id,
                transaction_id
            );
            
            drop(transactions); // Release lock before async operation
            
            let receipt_result = state.runtime.block_on(async {
                let output = execute_unrdf_script(&script).await?;
                let receipt: TransactionReceipt = serde_json::from_str(&output)
                    .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse receipt: {} - output: {}", e, output)))?;
                Ok(receipt)
            })?;
            
            // Update transaction state
            let mut transactions = state.transactions.lock()
                .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
            if let Some(txn) = transactions.get_mut(&transaction_id) {
                if receipt_result.success {
                    txn.state = TransactionState::Committed;
                }
            }
            
            Ok(receipt_result)
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}

/// Rollback a transaction
pub fn rollback_transaction(transaction_id: u32) -> UnrdfResult<()> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            transaction.state = TransactionState::RolledBack;
            Ok(())
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}

/// Validate data graph against SHACL shapes graph
pub fn validate_shacl(data_turtle: &str, shapes_turtle: &str) -> UnrdfResult<ShaclValidationResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    // Escape turtle data
    let escaped_data = data_turtle.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    let escaped_shapes = shapes_turtle.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const dataTurtle = `{}`;
            const shapesTurtle = `{}`;
        
            try {{
                const dataStore = await parseTurtle(dataTurtle);
                const shapesStore = await parseTurtle(shapesTurtle);
        
                const validation = await system.validate({{
                    dataGraph: dataStore,
                    shapesGraph: shapesStore
                }});
        
                const violations = [];
                for (const report of validation.report.results) {{
                    violations.push({{
                        path: report.path ? report.path.value : null,
                        message: report.message ? report.message[0].value : '',
                        severity: report.severity ? report.severity.value : null,
                        focus_node: report.focusNode ? report.focusNode.value : null,
                        value: report.value ? report.value.value : null
                    }});
                }}
        
                console.log(JSON.stringify({{
                    conforms: validation.conforms,
                    violations: violations
                }}));
            }} catch (err) {{
                console.error(JSON.stringify({{
                    conforms: false,
                    violations: [],
                    error: err.message
                }}));
                process.exit(1);
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{
                conforms: false,
                violations: [],
                error: err.message
            }}));
            process.exit(1);
        }});
        "#,
        escaped_data,
        escaped_shapes
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: ShaclValidationResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse validation result: {} - output: {}", e, output)))?;
        Ok(result)
    })
}

/// RDF serialization format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdfFormat {
    Turtle,
    JsonLd,
    NQuads,
}

/// Serialize unrdf store to RDF format
pub fn serialize_rdf(format: RdfFormat) -> UnrdfResult<String> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let format_str = match format {
        RdfFormat::Turtle => "turtle",
        RdfFormat::JsonLd => "jsonld",
        RdfFormat::NQuads => "nquads",
    };
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ toTurtle, toJsonLd, toNQuads }} from './src/knowledge-engine/serialize.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const format = '{}';
            let result;
        
            try {{
                // Get the store from the system
                const store = system.store || system.getStore();
        
                if (format === 'turtle') {{
                    result = await toTurtle(store);
                }} else if (format === 'jsonld') {{
                    result = await toJsonLd(store);
                }} else if (format === 'nquads') {{
                    result = await toNQuads(store);
                }} else {{
                    throw new Error('Unknown format: ' + format);
                }}
        
                console.log(result);
            }} catch (err) {{
                console.error('ERROR:', err.message);
                process.exit(1);
            }}
        }}
        
        main().catch(err => {{
            console.error('ERROR:', err.message);
            process.exit(1);
        }});
        "#,
        format_str
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        Ok(output.trim().to_string())
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

// Transaction Management FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_begin(actor: *const c_char) -> c_int {
    let actor_str = unsafe {
        CStr::from_ptr(actor)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid actor".to_string()))
    };
    
    match actor_str {
        Ok(a) => {
            match begin_transaction(a) {
                Ok(id) => id as c_int,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_add(transaction_id: c_int, turtle_data: *const c_char) -> c_int {
    let data = unsafe {
        CStr::from_ptr(turtle_data)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data".to_string()))
    };
    
    match data {
        Ok(d) => {
            match transaction_add(transaction_id as u32, d) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_remove(transaction_id: c_int, turtle_data: *const c_char) -> c_int {
    let data = unsafe {
        CStr::from_ptr(turtle_data)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid turtle data".to_string()))
    };
    
    match data {
        Ok(d) => {
            match transaction_remove(transaction_id as u32, d) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_commit(
    transaction_id: c_int,
    receipt_json: *mut c_char,
    receipt_size: usize
) -> c_int {
    match commit_transaction(transaction_id as u32) {
        Ok(receipt) => {
            match serde_json::to_string(&receipt) {
                Ok(json) => {
                    let json_bytes = json.as_bytes();
                    if json_bytes.len() < receipt_size {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                json_bytes.as_ptr(),
                                receipt_json as *mut u8,
                                json_bytes.len()
                            );
                            *receipt_json.add(json_bytes.len()) = 0;
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_rollback(transaction_id: c_int) -> c_int {
    match rollback_transaction(transaction_id as u32) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

// SHACL Validation FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_validate_shacl(
    data_turtle: *const c_char,
    shapes_turtle: *const c_char,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    let data = unsafe {
        CStr::from_ptr(data_turtle)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid data turtle".to_string()))
    };
    
    let shapes = unsafe {
        CStr::from_ptr(shapes_turtle)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid shapes turtle".to_string()))
    };
    
    match (data, shapes) {
        (Ok(d), Ok(s)) => {
            match validate_shacl(d, s) {
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

// RDF Serialization FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_to_turtle(output: *mut c_char, output_size: usize) -> c_int {
    match serialize_rdf(RdfFormat::Turtle) {
        Ok(result) => {
            let result_bytes = result.as_bytes();
            if result_bytes.len() < output_size {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        result_bytes.as_ptr(),
                        output as *mut u8,
                        result_bytes.len()
                    );
                    *output.add(result_bytes.len()) = 0;
                }
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_to_jsonld(output: *mut c_char, output_size: usize) -> c_int {
    match serialize_rdf(RdfFormat::JsonLd) {
        Ok(result) => {
            let result_bytes = result.as_bytes();
            if result_bytes.len() < output_size {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        result_bytes.as_ptr(),
                        output as *mut u8,
                        result_bytes.len()
                    );
                    *output.add(result_bytes.len()) = 0;
                }
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_to_nquads(output: *mut c_char, output_size: usize) -> c_int {
    match serialize_rdf(RdfFormat::NQuads) {
        Ok(result) => {
            let result_bytes = result.as_bytes();
            if result_bytes.len() < output_size {
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        result_bytes.as_ptr(),
                        output as *mut u8,
                        result_bytes.len()
                    );
                    *output.add(result_bytes.len()) = 0;
                }
                0
            } else {
                -1
            }
        }
        Err(_) => -1,
    }
}

// Hook Management FFI

#[no_mangle]
pub extern "C" fn knhk_unrdf_register_hook(
    hook_json: *const c_char,
    hook_id: *mut c_char,
    id_size: usize
) -> c_int {
    let json_str = unsafe {
        CStr::from_ptr(hook_json)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook JSON".to_string()))
    };
    
    match json_str {
        Ok(json) => {
            match register_hook(json) {
                Ok(id) => {
                    let id_bytes = id.as_bytes();
                    if id_bytes.len() < id_size {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                id_bytes.as_ptr(),
                                hook_id as *mut u8,
                                id_bytes.len()
                            );
                            *hook_id.add(id_bytes.len()) = 0;
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_deregister_hook(hook_id: *const c_char) -> c_int {
    let id = unsafe {
        CStr::from_ptr(hook_id)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid hook ID".to_string()))
    };
    
    match id {
        Ok(i) => {
            match deregister_hook(i) {
                Ok(_) => 0,
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_list_hooks(hooks_json: *mut c_char, hooks_size: usize) -> c_int {
    match list_hooks() {
        Ok(hooks) => {
            match serde_json::to_string(&hooks) {
                Ok(json) => {
                    let json_bytes = json.as_bytes();
                    if json_bytes.len() < hooks_size {
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                json_bytes.as_ptr(),
                                hooks_json as *mut u8,
                                json_bytes.len()
                            );
                            *hooks_json.add(json_bytes.len()) = 0;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // Test would require unrdf path
        // This is a placeholder for integration tests
    }
}


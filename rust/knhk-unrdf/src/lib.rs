// knhk-unrdf: Rust integration layer for unrdf knowledge hook engine
// Provides FFI-safe interface for cold path integration

use serde::{Deserialize, Serialize};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::{OnceLock, Mutex};
use std::collections::HashMap;
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
    #[error("Query parse failed: {0}")]
    QueryParseError(String),
    #[error("Validation failed: {0}")]
    ValidationError(String),
    #[error("Transaction failed: {0}")]
    TransactionError(String),
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
    Update,
}

/// Detect SPARQL query type from query string
fn detect_query_type(query: &str) -> Result<SparqlQueryType, UnrdfError> {
    let query_upper = query.trim().to_uppercase();
    
    if query_upper.starts_with("SELECT") {
        Ok(SparqlQueryType::Select)
    } else if query_upper.starts_with("ASK") {
        Ok(SparqlQueryType::Ask)
    } else if query_upper.starts_with("CONSTRUCT") {
        Ok(SparqlQueryType::Construct)
    } else if query_upper.starts_with("DESCRIBE") {
        Ok(SparqlQueryType::Describe)
    } else if query_upper.starts_with("INSERT") || query_upper.starts_with("DELETE") || query_upper.starts_with("LOAD") || query_upper.starts_with("CLEAR") || query_upper.starts_with("CREATE") || query_upper.starts_with("DROP") || query_upper.starts_with("COPY") || query_upper.starts_with("MOVE") || query_upper.starts_with("ADD") {
        Ok(SparqlQueryType::Update)
    } else {
        Err(UnrdfError::QueryParseError(format!("Unknown query type: {}", query.chars().take(20).collect::<String>())))
    }
}

/// Query result from unrdf (SELECT queries)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub bindings: Vec<serde_json::Value>,
    pub success: bool,
}

/// ASK query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskResult {
    pub result: bool,
    pub success: bool,
}

/// CONSTRUCT/DESCRIBE query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructResult {
    pub triples: Vec<serde_json::Value>,
    pub success: bool,
}

/// UPDATE query changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changes {
    pub additions: Vec<serde_json::Value>,
    pub removals: Vec<serde_json::Value>,
}

/// UPDATE query result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    pub changes: Changes,
    pub success: bool,
}

/// SHACL validation violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub path: Option<String>,
    pub message: Option<String>,
    pub severity: Option<String>,
    pub focus_node: Option<String>,
}

/// SHACL validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub conforms: bool,
    pub violations: Vec<Violation>,
    pub success: bool,
}

/// Transaction receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    pub transaction_id: u32,
    pub committed: bool,
    pub receipt: Option<String>,
    pub success: bool,
}

/// Hook execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub fired: bool,
    pub result: Option<serde_json::Value>,
    pub receipt: Option<String>,
}

/// Transaction state
#[derive(Debug, Clone)]
struct TransactionState {
    additions: Vec<String>,
    removals: Vec<String>,
    actor: String,
}

/// Transaction ID type
pub type TransactionId = u32;

/// Internal state for unrdf integration
struct UnrdfState {
    runtime: Runtime,
    unrdf_path: String,
    transactions: Mutex<HashMap<TransactionId, TransactionState>>,
    next_transaction_id: Mutex<TransactionId>,
}

static UNRDF_STATE: OnceLock<UnrdfState> = OnceLock::new();

/// Initialize unrdf integration layer
/// Must be called before any other operations
pub fn init_unrdf(unrdf_path: &str) -> Result<(), UnrdfError> {
    // Validate unrdf path exists
    let path = std::path::Path::new(unrdf_path);
    if !path.exists() {
        return Err(UnrdfError::InitializationFailed(
            format!("unrdf path does not exist: {}", unrdf_path)
        ));
    }
    
    if !path.is_dir() {
        return Err(UnrdfError::InitializationFailed(
            format!("unrdf path is not a directory: {}", unrdf_path)
        ));
    }
    
    // Check for key unrdf files
    let knowledge_engine_path = path.join("src/knowledge-engine/knowledge-substrate-core.mjs");
    if !knowledge_engine_path.exists() {
        return Err(UnrdfError::InitializationFailed(
            format!("unrdf knowledge engine not found at: {}. Please ensure unrdf is properly installed.", 
                knowledge_engine_path.display())
        ));
    }
    
    let runtime = Runtime::new()
        .map_err(|e| UnrdfError::InitializationFailed(format!("Failed to create runtime: {}", e)))?;
    
    let state = UnrdfState {
        runtime,
        unrdf_path: unrdf_path.to_string(),
        transactions: Mutex::new(HashMap::new()),
        next_transaction_id: Mutex::new(1),
    };
    
    UNRDF_STATE.set(state)
        .map_err(|_| UnrdfError::InitializationFailed("unrdf already initialized".to_string()))?;
    
    Ok(())
}

/// Execute a script using Node.js and unrdf
async fn execute_unrdf_script(script_content: &str) -> Result<String, UnrdfError> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    // Write script to temporary file INSIDE unrdf directory so relative imports work
    let unrdf_path = std::path::Path::new(&state.unrdf_path);
    let temp_file = unrdf_path.join(format!("knhk_unrdf_temp_{}.mjs", 
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| UnrdfError::InvalidInput(format!("System time error: {}", e)))?
            .as_nanos()));
    
    std::fs::write(&temp_file, script_content)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to write script: {}", e)))?;
    
    // Execute via Node.js from unrdf directory
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
        let error_msg = if !stderr.trim().is_empty() {
            format!("Script failed: stderr={}", stderr)
        } else if !stdout.trim().is_empty() {
            format!("Script failed: stdout={}", stdout)
        } else {
            format!("Script failed with exit code: {:?}", output.status.code())
        };
        return Err(UnrdfError::QueryFailed(error_msg));
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
            try {{
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
            }} catch (err) {{
                console.error('ERROR:', err.message);
                if (err.stack) console.error(err.stack);
                process.exit(1);
            }}
        }}
        
        main();
        "#,
        escaped_data
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Check for SUCCESS in output (may have initialization messages before it)
        let output_trimmed = output.trim();
        if output_trimmed.contains("SUCCESS") || output_trimmed.ends_with("SUCCESS") {
            Ok(())
        } else {
            Err(UnrdfError::StoreFailed(format!("Script did not return SUCCESS. Output: {}", 
                output_trimmed.chars().take(500).collect::<String>())))
        }
    })
}

/// Execute SPARQL query via unrdf (SELECT queries - backward compatibility)
pub fn query_sparql(query: &str) -> UnrdfResult<QueryResult> {
    query_sparql_select(query)
}

/// Execute SPARQL SELECT query via unrdf
pub fn query_sparql_select(query: &str) -> UnrdfResult<QueryResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let escaped_query = query.replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const query = `{}`;
        
            const result = await system.query(query);
            
            if (result.type === 'select') {{
                const bindings = result.rows || [];
                console.log(JSON.stringify({{ bindings: bindings, success: true }}));
            }} else {{
                throw new Error('Expected SELECT query, got ' + result.type);
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ bindings: [], success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_query
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: QueryResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

/// Execute SPARQL ASK query via unrdf
pub fn query_sparql_ask(query: &str) -> UnrdfResult<AskResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let escaped_query = query.replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const query = `{}`;
        
            const result = await system.query(query);
            
            if (result.type === 'ask') {{
                console.log(JSON.stringify({{ result: result.boolean || false, success: true }}));
            }} else {{
                throw new Error('Expected ASK query, got ' + result.type);
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ result: false, success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_query
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: AskResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

/// Execute SPARQL CONSTRUCT query via unrdf
pub fn query_sparql_construct(query: &str) -> UnrdfResult<ConstructResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let escaped_query = query.replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const query = `{}`;
        
            const result = await system.query(query);
            
            if (result.type === 'construct') {{
                const triples = [];
                if (result.store) {{
                    result.store.forEach(q => triples.push({{
                        subject: q.subject.value,
                        predicate: q.predicate.value,
                        object: q.object.value
                    }}));
                }}
                console.log(JSON.stringify({{ triples: triples, success: true }}));
            }} else {{
                throw new Error('Expected CONSTRUCT query, got ' + result.type);
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ triples: [], success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_query
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: ConstructResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

/// Epistemology generation: Synthesize new knowledge from observations using CONSTRUCT
/// This implements A = μ(O) - converting observations O into actions/knowledge A via transformation μ
/// 
/// # Arguments
/// * `construct_query` - SPARQL CONSTRUCT query that synthesizes new triples from existing data
/// * `store_triples` - If true, store the generated triples back into unrdf store
/// 
/// # Returns
/// Generated triples as knowledge synthesis result
/// 
/// # Example
/// ```rust
/// // Generate authorization knowledge from role assignments
/// let query = r#"
///   PREFIX ex: <http://example.org/>
///   CONSTRUCT { ?user ex:hasAccess ?resource }
///   WHERE { ?user ex:role ?role . ?role ex:grants ?resource }
/// "#;
/// 
/// let knowledge = generate_epistemology(query, false)?;
/// // knowledge.triples contains synthesized authorization triples
/// ```
pub fn generate_epistemology(construct_query: &str, store_triples: bool) -> UnrdfResult<ConstructResult> {
    let result = query_sparql_construct(construct_query)?;
    
    // If requested, store the generated triples back into unrdf
    if store_triples && !result.triples.is_empty() {
        // Convert triples back to Turtle format and store
        let mut turtle_data = String::new();
        for triple in &result.triples {
            if let serde_json::Value::Object(obj) = triple {
                if let (Some(s), Some(p), Some(o)) = (
                    obj.get("subject").and_then(|v| v.as_str()),
                    obj.get("predicate").and_then(|v| v.as_str()),
                    obj.get("object").and_then(|v| v.as_str()),
                ) {
                    turtle_data.push_str(&format!("<{}> <{}> <{}> .\n", s, p, o));
                }
            }
        }
        
        if !turtle_data.is_empty() {
            store_turtle_data(&turtle_data)?;
        }
    }
    
    Ok(result)
}

/// Execute SPARQL DESCRIBE query via unrdf
pub fn query_sparql_describe(query: &str) -> UnrdfResult<ConstructResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let escaped_query = query.replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const query = `{}`;
        
            const result = await system.query(query);
            
            if (result.type === 'describe') {{
                const triples = [];
                if (result.store) {{
                    result.store.forEach(q => triples.push({{
                        subject: q.subject.value,
                        predicate: q.predicate.value,
                        object: q.object.value
                    }}));
                }}
                console.log(JSON.stringify({{ triples: triples, success: true }}));
            }} else {{
                throw new Error('Expected DESCRIBE query, got ' + result.type);
            }}
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ triples: [], success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_query
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: ConstructResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

/// Execute SPARQL UPDATE query via unrdf
pub fn query_sparql_update(query: &str) -> UnrdfResult<UpdateResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let escaped_query = query.replace('`', "\\`").replace('$', "\\$");
    
    // UPDATE queries use system.query() which auto-detects UPDATE type
    // and returns changes in the result
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const query = `{}`;
        
            const result = await system.query(query);
            
            const additions = [];
            const removals = [];
            
            // UPDATE queries return changes
            if (result.additions) {{
                result.additions.forEach(q => additions.push({{
                    subject: q.subject.value,
                    predicate: q.predicate.value,
                    object: q.object.value
                }}));
            }}
            
            if (result.removals) {{
                result.removals.forEach(q => removals.push({{
                    subject: q.subject.value,
                    predicate: q.predicate.value,
                    object: q.object.value
                }}));
            }}
        
            console.log(JSON.stringify({{
                changes: {{
                    additions: additions,
                    removals: removals
                }},
                success: true
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ changes: {{ additions: [], removals: [] }}, success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_query
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: UpdateResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse result: {}", e)))?;
        Ok(result)
    })
}

/// Validate SHACL shapes against data graph
pub fn validate_shacl(data_turtle: &str, shapes_turtle: &str) -> UnrdfResult<ValidationResult> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    // Escape backticks and template literals
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
        
            const dataStore = await parseTurtle(dataTurtle);
            const shapesStore = await parseTurtle(shapesTurtle);
        
            // Use system.validate() if available, otherwise use engine.validateShacl()
            let validation;
            if (typeof system.validate === 'function') {{
                validation = await system.validate({{
                    dataGraph: dataStore,
                    shapesGraph: shapesStore
                }});
            }} else {{
                // Fallback to engine method
                const engine = system.engine || system.context?.engine;
                if (engine && typeof engine.validateShacl === 'function') {{
                    validation = engine.validateShacl(dataStore, shapesStore);
                }} else {{
                    throw new Error('SHACL validation not available');
                }}
            }}
        
            const violations = (validation.results || []).map(r => ({{
                path: r.path || null,
                message: r.message || null,
                severity: r.severity || null,
                focus_node: r.focusNode || null
            }}));
        
            console.log(JSON.stringify({{
                conforms: validation.conforms || false,
                violations: violations,
                success: true
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ conforms: false, violations: [], success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_data,
        escaped_shapes
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: ValidationResult = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::ValidationError(format!("Failed to parse validation result: {}", e)))?;
        Ok(result)
    })
}
/// Begin a new transaction
pub fn transaction_begin(actor: &str) -> UnrdfResult<TransactionId> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut next_id = state.next_transaction_id.lock()
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to lock transaction ID: {}", e)))?;
    
    let transaction_id = *next_id;
    *next_id += 1;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to lock transactions: {}", e)))?;
    
    transactions.insert(transaction_id, TransactionState {
        additions: Vec::new(),
        removals: Vec::new(),
        actor: actor.to_string(),
    });
    
    Ok(transaction_id)
}

/// Add quads to transaction
pub fn transaction_add(transaction_id: TransactionId, turtle_data: &str) -> UnrdfResult<()> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to lock transactions: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::TransactionError(format!("Transaction {} not found", transaction_id)))?;
    
    transaction.additions.push(turtle_data.to_string());
    Ok(())
}

/// Remove quads from transaction
pub fn transaction_remove(transaction_id: TransactionId, turtle_data: &str) -> UnrdfResult<()> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to lock transactions: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::TransactionError(format!("Transaction {} not found", transaction_id)))?;
    
    transaction.removals.push(turtle_data.to_string());
    Ok(())
}

/// Commit transaction
pub fn transaction_commit(transaction_id: TransactionId) -> UnrdfResult<TransactionReceipt> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to lock transactions: {}", e)))?;
    
    let transaction = transactions.remove(&transaction_id)
        .ok_or_else(|| UnrdfError::TransactionError(format!("Transaction {} not found", transaction_id)))?;
    
    // Build script to execute transaction
    let escaped_additions: Vec<String> = transaction.additions.iter()
        .map(|s| s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$"))
        .collect();
    let escaped_removals: Vec<String> = transaction.removals.iter()
        .map(|s| s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$"))
        .collect();
    
    let additions_json = serde_json::to_string(&escaped_additions)
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to serialize additions: {}", e)))?;
    let removals_json = serde_json::to_string(&escaped_removals)
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to serialize removals: {}", e)))?;
    
    let escaped_actor = transaction.actor.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
        import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const additionsTurtle = {};
            const removalsTurtle = {};
            const actor = `{}`;
        
            const additions = [];
            const removals = [];
        
            for (const turtle of additionsTurtle) {{
                const store = await parseTurtle(turtle);
                store.forEach(q => additions.push(q));
            }}
        
            for (const turtle of removalsTurtle) {{
                const store = await parseTurtle(turtle);
                store.forEach(q => removals.push(q));
            }}
        
            const receipt = await system.executeTransaction({{
                additions: additions,
                removals: removals,
                actor: actor
            }});
        
            console.log(JSON.stringify({{
                transaction_id: {},
                committed: receipt.committed || false,
                receipt: receipt.receipt || null,
                success: true
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ transaction_id: {}, committed: false, receipt: null, success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        additions_json,
        removals_json,
        escaped_actor,
        transaction_id,
        transaction_id
    );
    
    let result = state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let receipt: TransactionReceipt = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::TransactionError(format!("Failed to parse receipt: {}", e)))?;
        Ok(receipt)
    })?;
    
    Ok(result)
}

/// Rollback transaction
pub fn transaction_rollback(transaction_id: TransactionId) -> UnrdfResult<()> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::TransactionError(format!("Failed to lock transactions: {}", e)))?;
    
    transactions.remove(&transaction_id)
        .ok_or_else(|| UnrdfError::TransactionError(format!("Transaction {} not found", transaction_id)))?;
    
    Ok(())
}

/// Autonomous epistemology hook definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutonomousEpistemologyHook {
    pub name: String,
    pub description: String,
    pub when: HookCondition,
    pub construct_query: String,
    pub store_results: bool,
    pub before: Option<String>, // Optional before hook logic
    pub after: Option<String>,  // Optional after hook logic
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCondition {
    pub kind: String, // "sparql-ask", "shacl", etc.
    pub query: String, // Condition query
}

/// Register an autonomous epistemology hook that automatically generates knowledge
/// This implements autonomic epistemology generation: A = μ(O) triggered by conditions
/// 
/// # Arguments
/// * `hook` - Hook definition with condition and CONSTRUCT query
/// 
/// # Returns
/// Hook ID on success
/// 
/// # Example
/// ```rust
/// let hook = AutonomousEpistemologyHook {
///     name: "authorization-reflex".to_string(),
///     description: "Automatically generate access permissions from roles".to_string(),
///     when: HookCondition {
///         kind: "sparql-ask".to_string(),
///         query: "ASK { ?user ex:role ?role }".to_string(),
///     },
///     construct_query: r#"
///         CONSTRUCT { ?user ex:hasAccess ?resource }
///         WHERE { ?user ex:role ?role . ?role ex:grants ?resource }
///     "#.to_string(),
///     store_results: true,
///     before: None,
///     after: None,
/// };
/// 
/// let hook_id = register_autonomous_epistemology(hook)?;
/// ```
pub fn register_autonomous_epistemology(hook: AutonomousEpistemologyHook) -> UnrdfResult<String> {
    let state = UNRDF_STATE.get()
        .ok_or_else(|| UnrdfError::InitializationFailed("unrdf not initialized".to_string()))?;
    
    let escaped_name = hook.name.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    let escaped_desc = hook.description.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    let escaped_when_query = hook.when.query.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    let escaped_construct = hook.construct_query.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
    
    let before_logic = hook.before.as_ref()
        .map(|s| format!("async (event) => {{ {} }}", s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$")))
        .unwrap_or_else(|| "undefined".to_string());
    
    let after_logic = hook.after.as_ref()
        .map(|s| format!("async (event) => {{ {} }}", s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$")))
        .unwrap_or_else(|| "undefined".to_string());
    
    let script = format!(
        r#"
        import {{ createDarkMatterCore, defineHook, registerHook }} from './src/knowledge-engine/index.mjs';
        
        async function main() {{
            const system = await createDarkMatterCore({{
                enableKnowledgeHookManager: true,
                enableLockchainWriter: false
            }});
        
            const whenQuery = `{}`;
            const constructQuery = `{}`;
            
            const hook = defineHook({{
                meta: {{
                    name: '{}',
                    description: '{}'
                }},
                when: {{
                    kind: '{}',
                    query: whenQuery
                }},
                before: {},
                run: async (event) => {{
                    // Autonomic epistemology generation: A = μ(O)
                    const result = await system.query(constructQuery);
                    
                    if (result.type === 'construct' && result.store) {{
                        const triples = [];
                        result.store.forEach(q => triples.push(q));
                        
                        if ({}) {{
                            // Store generated knowledge back into system
                            await system.executeTransaction({{
                                additions: triples,
                                removals: [],
                                actor: 'autonomic-epistemology'
                            }});
                        }}
                        
                        return {{
                            epistemology: true,
                            triplesGenerated: triples.length,
                            triples: triples.map(q => ({{
                                subject: q.subject.value,
                                predicate: q.predicate.value,
                                object: q.object.value
                            }}))
                        }};
                    }}
                    
                    return {{ epistemology: false, error: 'Failed to generate epistemology' }};
                }},
                after: {}
            }});
        
            await registerHook(hook);
            
            console.log(JSON.stringify({{ 
                hookId: hook.meta.name, 
                success: true 
            }}));
        }}
        
        main().catch(err => {{
            console.error(JSON.stringify({{ hookId: null, success: false, error: err.message }}));
            process.exit(1);
        }});
        "#,
        escaped_when_query,
        escaped_construct,
        escaped_name,
        escaped_desc,
        hook.when.kind,
        before_logic,
        if hook.store_results { "true" } else { "false" },
        after_logic
    );
    
    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        let result: serde_json::Value = serde_json::from_str(&output)
            .map_err(|e| UnrdfError::HookFailed(format!("Failed to parse hook registration: {}", e)))?;
        
        if let Some(hook_id) = result.get("hookId").and_then(|v| v.as_str()) {
            Ok(hook_id.to_string())
        } else {
            Err(UnrdfError::HookFailed("Failed to register hook".to_string()))
        }
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
pub extern "C" fn knhk_unrdf_query_ask(query: *const c_char, result: *mut c_int) -> c_int {
    let q = unsafe {
        CStr::from_ptr(query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid query".to_string()))
    };
    
    match q {
        Ok(query_str) => {
            match query_sparql_ask(query_str) {
                Ok(ask_result) => {
                    unsafe {
                        *result = if ask_result.result { 1 } else { 0 };
                    }
                    0
                }
                Err(_) => -1,
            }
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_query_construct(query: *const c_char, result_json: *mut c_char, result_size: usize) -> c_int {
    let q = unsafe {
        CStr::from_ptr(query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid query".to_string()))
    };
    
    match q {
        Ok(query_str) => {
            match query_sparql_construct(query_str) {
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
pub extern "C" fn knhk_unrdf_query_describe(query: *const c_char, result_json: *mut c_char, result_size: usize) -> c_int {
    let q = unsafe {
        CStr::from_ptr(query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid query".to_string()))
    };
    
    match q {
        Ok(query_str) => {
            match query_sparql_describe(query_str) {
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
pub extern "C" fn knhk_unrdf_query_update(query: *const c_char, result_json: *mut c_char, result_size: usize) -> c_int {
    let q = unsafe {
        CStr::from_ptr(query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid query".to_string()))
    };
    
    match q {
        Ok(query_str) => {
            match query_sparql_update(query_str) {
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

#[no_mangle]
pub extern "C" fn knhk_unrdf_transaction_begin(actor: *const c_char) -> c_int {
    let a = unsafe {
        CStr::from_ptr(actor)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid actor".to_string()))
    };
    
    match a {
        Ok(actor_str) => {
            match transaction_begin(actor_str) {
                Ok(transaction_id) => transaction_id as c_int,
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
            match transaction_add(transaction_id as TransactionId, d) {
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
            match transaction_remove(transaction_id as TransactionId, d) {
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
    match transaction_commit(transaction_id as TransactionId) {
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
    match transaction_rollback(transaction_id as TransactionId) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn knhk_unrdf_generate_epistemology(
    construct_query: *const c_char,
    store_triples: c_int,
    result_json: *mut c_char,
    result_size: usize
) -> c_int {
    let query = unsafe {
        CStr::from_ptr(construct_query)
            .to_str()
            .map_err(|_| UnrdfError::InvalidInput("Invalid construct query".to_string()))
    };
    
    match query {
        Ok(query_str) => {
            match generate_epistemology(query_str, store_triples != 0) {
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
pub extern "C" fn knhk_unrdf_register_autonomous_epistemology(
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
            match serde_json::from_str::<AutonomousEpistemologyHook>(json) {
                Ok(hook) => {
                    match register_autonomous_epistemology(hook) {
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


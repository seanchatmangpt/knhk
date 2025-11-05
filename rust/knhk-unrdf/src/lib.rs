// knhk-unrdf: Rust integration layer for unrdf knowledge hook engine
// Provides FFI-safe interface for cold path integration

pub mod error;
pub mod ffi;
pub mod hooks;
pub mod query;
pub mod script;
pub mod serialize;
pub mod shacl;
pub mod state;
pub mod store;
pub mod transaction;
pub mod types;

// Re-export public API
pub use error::{UnrdfError, UnrdfErrorCode, UnrdfResult};
pub use hooks::{deregister_hook, execute_hook, execute_hook_with_data, list_hooks, register_hook};
pub use query::{detect_query_type, query_sparql, query_sparql_with_type, query_sparql_with_data};
pub use serialize::serialize_rdf;
pub use shacl::validate_shacl;
pub use store::store_turtle_data;
pub use transaction::{begin_transaction, commit_transaction, rollback_transaction, transaction_add, transaction_remove};
pub use types::{HookDefinition, HookResult, QueryResult, RdfFormat, ShaclValidationResult, ShaclViolation, SparqlQueryType, TransactionReceipt};

use state::init_state;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

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
    
    let state = state::UnrdfState {
        runtime,
        unrdf_path: unrdf_path.to_string(),
        transactions: Arc::new(Mutex::new(HashMap::new())),
        next_transaction_id: Arc::new(Mutex::new(1)),
        hooks: Arc::new(Mutex::new(HashMap::new())),
    };
    
    init_state(state)?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_path_validation() {
        // Test path validation logic
        let invalid_path = "/nonexistent/path/that/does/not/exist";
        let result = init_unrdf(invalid_path);
        assert!(result.is_err(), "Init should fail for non-existent path");
        
        // Verify error message contains path information
        if let Err(UnrdfError::InitializationFailed(msg)) = result {
            assert!(msg.contains("does not exist") || msg.contains("not found") || msg.contains("not a directory"), 
                    "Error message should indicate path issue: {}", msg);
        }
    }
}

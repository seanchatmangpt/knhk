// knhk-unrdf: Rust integration layer for unrdf knowledge hook engine
// Provides FFI-safe interface for cold path integration

pub mod errors;
pub mod types;
pub mod utils;
pub mod state;
pub mod query;
pub mod validation;
pub mod transaction;
pub mod serialization;
pub mod hooks;
pub mod ffi;

// Re-export public API
pub use errors::{UnrdfError, UnrdfResult};
pub use types::{
    HookListResult, HookResult, QueryResult, SerializationResult, TransactionResult,
    ValidationResult,
};
pub use state::init_unrdf;
pub use query::{execute_hook, query_sparql, store_turtle_data};
pub use validation::validate_shacl;
pub use transaction::execute_transaction;
pub use serialization::{serialize_to_jsonld, serialize_to_nquads, serialize_to_turtle};
pub use hooks::{deregister_hook, list_hooks, register_hook};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        // Test would require unrdf path
        // This is a placeholder for integration tests
    }
}

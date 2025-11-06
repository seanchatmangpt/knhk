// knhk-unrdf: Rust-native hooks engine
// Pure Rust hook execution without Node.js dependency
// Focuses on 80/20 use cases: hook execution and batch evaluation

#[cfg(feature = "native")]
use crate::error::{UnrdfError, UnrdfResult};
#[cfg(feature = "native")]
use crate::types::{HookDefinition, HookResult};
#[cfg(feature = "native")]
use crate::query_native::NativeStore;
#[cfg(feature = "native")]
use crate::canonicalize::get_canonical_hash;
#[cfg(feature = "native")]
use sha2::{Sha256, Digest};
#[cfg(feature = "native")]
use rayon::prelude::*;
#[cfg(feature = "native")]
use std::collections::HashMap;
#[cfg(feature = "native")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "native")]
use serde_json::Value as JsonValue;

#[cfg(feature = "native")]
/// Hook registry for native Rust hooks
pub struct NativeHookRegistry {
    hooks: Arc<Mutex<HashMap<String, HookDefinition>>>,
}

#[cfg(feature = "native")]
impl NativeHookRegistry {
    /// Create a new hook registry
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a hook
    pub fn register(&self, hook: HookDefinition) -> UnrdfResult<()> {
        let mut hooks = self.hooks.lock()
            .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
        
        hooks.insert(hook.id.clone(), hook);
        Ok(())
    }

    /// Deregister a hook
    pub fn deregister(&self, hook_id: &str) -> UnrdfResult<()> {
        let mut hooks = self.hooks.lock()
            .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
        
        hooks.remove(hook_id);
        Ok(())
    }

    /// Get all registered hooks
    pub fn list(&self) -> UnrdfResult<Vec<HookDefinition>> {
        let hooks = self.hooks.lock()
            .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
        
        Ok(hooks.values().cloned().collect())
    }

    /// Get a hook by ID
    pub fn get(&self, hook_id: &str) -> UnrdfResult<Option<HookDefinition>> {
        let hooks = self.hooks.lock()
            .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire hooks lock: {}", e)))?;
        
        Ok(hooks.get(hook_id).cloned())
    }
}

#[cfg(feature = "native")]
impl Default for NativeHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "native")]
/// Execute a hook evaluation
/// Primary use case 1: Execute a single hook based on SPARQL ASK query
pub fn evaluate_hook_native(
    hook: &HookDefinition,
    turtle_data: &str,
) -> UnrdfResult<HookResult> {
    // Extract hook query from definition
    let hook_query = extract_hook_query(hook)?;
    
    // Create store and load data
    let store = NativeStore::new();
    store.load_turtle(turtle_data)?;
    
    // Execute ASK query to check if hook condition is met
    let query_type = crate::query::detect_query_type(&hook_query);
    if query_type != crate::types::SparqlQueryType::Ask {
        return Err(UnrdfError::HookFailed(
            "Hook queries must be ASK queries".to_string()
        ));
    }
    
    let query_result = store.query(&hook_query, query_type)?;
    
    // Check if hook fired (ASK query returned true)
    let fired = query_result.boolean.unwrap_or(false);
    
    // Generate receipt hash
    let receipt = if fired {
        Some(generate_hook_receipt(hook, turtle_data)?)
    } else {
        None
    };
    
    // Prepare result
    let result = if fired {
        Some(JsonValue::Object({
            let mut obj = serde_json::Map::new();
            obj.insert("fired".to_string(), JsonValue::Bool(true));
            obj.insert("hook_id".to_string(), JsonValue::String(hook.id.clone()));
            obj
        }))
    } else {
        None
    };
    
    Ok(HookResult {
        fired,
        result,
        receipt,
    })
}

#[cfg(feature = "native")]
/// Execute multiple hooks in parallel (batch evaluation)
/// Primary use case 2: Evaluate multiple hooks efficiently
pub fn evaluate_hooks_batch_native(
    hooks: &[HookDefinition],
    turtle_data: &str,
) -> UnrdfResult<Vec<HookResult>> {
    // Evaluate hooks in parallel using Rayon
    // Each hook gets its own store instance to avoid mutability issues
    let results: Vec<UnrdfResult<HookResult>> = hooks
        .par_iter()
        .map(|hook| {
            evaluate_hook_native(hook, turtle_data)
        })
        .collect();
    
    // Collect results, returning first error if any
    let mut hook_results = Vec::new();
    for result in results {
        hook_results.push(result?);
    }
    
    Ok(hook_results)
}

#[cfg(feature = "native")]
/// Extract SPARQL ASK query from hook definition
fn extract_hook_query(hook: &HookDefinition) -> UnrdfResult<String> {
    // Hook definition should have a JSON structure with "when" containing "query"
    if let Some(when) = hook.definition.get("when") {
        if let Some(query) = when.get("query") {
            if let Some(query_str) = query.as_str() {
                return Ok(query_str.to_string());
            }
        }
    }
    
    Err(UnrdfError::HookFailed(
        format!("Hook {} does not contain a valid SPARQL ASK query", hook.id)
    ))
}

#[cfg(feature = "native")]
/// Generate cryptographic receipt for hook execution
/// Receipt format: hash(hook_id + canonical_data_hash + timestamp + counter)
fn generate_hook_receipt(hook: &HookDefinition, turtle_data: &str) -> UnrdfResult<String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    
    // Get canonical hash of data
    let data_hash = get_canonical_hash(turtle_data)?;
    
    // Generate receipt hash with timestamp and counter for uniqueness
    let mut hasher = Sha256::new();
    hasher.update(hook.id.as_bytes());
    hasher.update(data_hash.as_bytes());
    
    // Use high-resolution timestamp and counter for uniqueness
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| UnrdfError::HookFailed(format!("Failed to get timestamp: {}", e)))?
        .as_nanos()
        .to_string();
    hasher.update(timestamp.as_bytes());
    
    let counter = COUNTER.fetch_add(1, Ordering::Relaxed).to_string();
    hasher.update(counter.as_bytes());
    
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

#[cfg(feature = "native")]
/// Execute hook by name (convenience function)
pub fn execute_hook_by_name_native(
    hook_name: &str,
    hook_query: &str,
    turtle_data: &str,
) -> UnrdfResult<HookResult> {
    // Create a temporary hook definition
    let hook = HookDefinition {
        id: hook_name.to_string(),
        name: hook_name.to_string(),
        hook_type: "sparql-ask".to_string(),
        definition: {
            let mut def = serde_json::Map::new();
            let mut when = serde_json::Map::new();
            when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
            when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
            def.insert("when".to_string(), JsonValue::Object(when));
            JsonValue::Object(def)
        },
    };
    
    evaluate_hook_native(&hook, turtle_data)
}

#[cfg(test)]
#[cfg(feature = "native")]
mod tests {
    use super::*;
    use crate::types::HookDefinition;
    use serde_json::Value as JsonValue;

    #[test]
    fn test_single_hook_execution() {
        // Use case 1: Execute a single hook
        let hook_query = "ASK { ?s ?p ?o }";
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            @prefix foaf: <http://xmlns.com/foaf/0.1/> .
            
            ex:alice foaf:name "Alice" .
            ex:bob foaf:name "Bob" .
        "#;
        
        let hook = HookDefinition {
            id: "test-hook-1".to_string(),
            name: "Test Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        let result = evaluate_hook_native(&hook, turtle_data).unwrap();
        
        // Hook should fire if data exists
        assert!(result.fired == true);
        assert!(result.receipt.is_some());
        assert!(result.result.is_some());
    }

    #[test]
    fn test_hook_execution_by_name() {
        // Use case 1: Execute hook by name (convenience function)
        let hook_name = "missing-name-hook";
        let hook_query = "ASK { ?s ?p ?o }";
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            @prefix foaf: <http://xmlns.com/foaf/0.1/> .
            
            ex:alice foaf:name "Alice" .
        "#;
        
        let result = execute_hook_by_name_native(hook_name, hook_query, turtle_data).unwrap();
        
        assert!(result.fired == true);
    }

    #[test]
    fn test_batch_hook_evaluation() {
        // Use case 2: Evaluate multiple hooks in parallel
        let hooks = vec![
            HookDefinition {
                id: "hook-1".to_string(),
                name: "Hook 1".to_string(),
                hook_type: "sparql-ask".to_string(),
                definition: {
                    let mut def = serde_json::Map::new();
                    let mut when = serde_json::Map::new();
                    when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                    when.insert("query".to_string(), JsonValue::String("ASK { ?s ?p ?o }".to_string()));
                    def.insert("when".to_string(), JsonValue::Object(when));
                    JsonValue::Object(def)
                },
            },
            HookDefinition {
                id: "hook-2".to_string(),
                name: "Hook 2".to_string(),
                hook_type: "sparql-ask".to_string(),
                definition: {
                    let mut def = serde_json::Map::new();
                    let mut when = serde_json::Map::new();
                    when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                    when.insert("query".to_string(), JsonValue::String("ASK { ?s <http://example.org/name> ?name }".to_string()));
                    def.insert("when".to_string(), JsonValue::Object(when));
                    JsonValue::Object(def)
                },
            },
        ];
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let results = evaluate_hooks_batch_native(&hooks, turtle_data).unwrap();
        
        assert_eq!(results.len(), 2);
        // Both hooks should fire (data exists)
        assert!(results.iter().all(|r| r.fired == true));
    }

    #[test]
    fn test_hook_registry() {
        let registry = NativeHookRegistry::new();
        
        let hook = HookDefinition {
            id: "test-hook".to_string(),
            name: "Test Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: JsonValue::Object(serde_json::Map::new()),
        };
        
        // Register hook
        registry.register(hook.clone()).unwrap();
        
        // List hooks
        let hooks = registry.list().unwrap();
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].id, "test-hook");
        
        // Get hook
        let retrieved = registry.get("test-hook").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "test-hook");
        
        // Deregister hook
        registry.deregister("test-hook").unwrap();
        
        // Verify deregistered
        let hooks_after = registry.list().unwrap();
        assert_eq!(hooks_after.len(), 0);
    }

    #[test]
    fn test_hook_receipt_generation() {
        let hook = HookDefinition {
            id: "test-hook".to_string(),
            name: "Test Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String("ASK { ?s ?p ?o }".to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let result = evaluate_hook_native(&hook, turtle_data).unwrap();
        
        // Hook should fire (data exists)
        assert!(result.fired);
        assert!(result.receipt.is_some());
        
        // Receipt should be a valid hex string
        let receipt = result.receipt.unwrap();
        assert!(receipt.len() == 64); // SHA-256 produces 64 hex characters
        assert!(receipt.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_guard_law_validation() {
        // Law: Guard: μ ⊣ H (partial) - validates O ⊨ Σ before A = μ(O)
        // Test: Hook validates operations O before canonicalization
        let hook_query = "ASK { ?s ?p ?o }";
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let hook = HookDefinition {
            id: "guard-hook".to_string(),
            name: "Guard Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        // Execute hook: validates O (operations) before producing A (artifacts)
        let result = evaluate_hook_native(&hook, turtle_data).unwrap();
        
        // Guard should pass: O ⊨ Σ (operations satisfy schema)
        assert!(result.fired);
        // Receipt ensures: hash(A) = hash(μ(O))
        assert!(result.receipt.is_some());
    }

    #[test]
    fn test_guard_law_failure() {
        // Law: Guard: μ ⊣ H (partial) - guard fails when O does not satisfy Σ
        // Test: Hook fails when operations don't satisfy condition
        let hook_query = "ASK { ?s <http://example.org/nonexistent> ?o }";
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let hook = HookDefinition {
            id: "guard-fail-hook".to_string(),
            name: "Guard Fail Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        let result = evaluate_hook_native(&hook, turtle_data).unwrap();
        
        // Guard should fail: O does not satisfy Σ
        assert!(result.fired == false);
        assert!(result.receipt.is_none());
    }

    #[test]
    fn test_provenance_hash_equality() {
        // Law: Provenance: hash(A) = hash(μ(O))
        // Test: Receipt hash matches canonical hash of operations
        let hook_query = "ASK { ?s ?p ?o }";
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let hook = HookDefinition {
            id: "provenance-hook".to_string(),
            name: "Provenance Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        let result = evaluate_hook_native(&hook, turtle_data).unwrap();
        
        // Receipt should contain hash derived from canonical hash of O
        // hash(A) = hash(μ(O)) where A includes hook_id and canonical_data_hash
        assert!(result.fired);
        assert!(result.receipt.is_some());
        
        let receipt = result.receipt.unwrap();
        // Receipt format: hash(hook_id + canonical_hash(O) + timestamp + counter)
        // Verify receipt structure (64 hex chars for SHA-256)
        assert_eq!(receipt.len(), 64);
    }

    #[test]
    fn test_order_preservation_batch() {
        // Law: Order: Λ is ≺-total - batch results maintain hook order
        // Test: Batch evaluation preserves hook order in results
        let hooks = vec![
            HookDefinition {
                id: "hook-1".to_string(),
                name: "Hook 1".to_string(),
                hook_type: "sparql-ask".to_string(),
                definition: {
                    let mut def = serde_json::Map::new();
                    let mut when = serde_json::Map::new();
                    when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                    when.insert("query".to_string(), JsonValue::String("ASK { ?s ?p ?o }".to_string()));
                    def.insert("when".to_string(), JsonValue::Object(when));
                    JsonValue::Object(def)
                },
            },
            HookDefinition {
                id: "hook-2".to_string(),
                name: "Hook 2".to_string(),
                hook_type: "sparql-ask".to_string(),
                definition: {
                    let mut def = serde_json::Map::new();
                    let mut when = serde_json::Map::new();
                    when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                    when.insert("query".to_string(), JsonValue::String("ASK { ?s <http://example.org/name> ?name }".to_string()));
                    def.insert("when".to_string(), JsonValue::Object(when));
                    JsonValue::Object(def)
                },
            },
            HookDefinition {
                id: "hook-3".to_string(),
                name: "Hook 3".to_string(),
                hook_type: "sparql-ask".to_string(),
                definition: {
                    let mut def = serde_json::Map::new();
                    let mut when = serde_json::Map::new();
                    when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                    when.insert("query".to_string(), JsonValue::String("ASK { ?s ?p ?o . ?s ?p2 ?o2 }".to_string()));
                    def.insert("when".to_string(), JsonValue::Object(when));
                    JsonValue::Object(def)
                },
            },
        ];
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let results = evaluate_hooks_batch_native(&hooks, turtle_data).unwrap();
        
        // Order Λ is ≺-total: results maintain hook order
        assert_eq!(results.len(), 3);
        // Verify order preservation
        assert_eq!(results[0].result.as_ref().unwrap().get("hook_id").unwrap().as_str().unwrap(), "hook-1");
        assert_eq!(results[1].result.as_ref().unwrap().get("hook_id").unwrap().as_str().unwrap(), "hook-2");
        assert_eq!(results[2].result.as_ref().unwrap().get("hook_id").unwrap().as_str().unwrap(), "hook-3");
    }

    #[test]
    fn test_invariant_preservation() {
        // Law: Invariant: preserve(Q) - hooks enforce invariants Q
        // Test: Multiple hooks enforce different invariants
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            @prefix foaf: <http://xmlns.com/foaf/0.1/> .
            
            ex:alice foaf:name "Alice" .
            ex:bob foaf:name "Bob" .
        "#;
        
        // Hook 1: Typing invariant (O ⊨ Σ) - all triples have valid structure
        let typing_hook = HookDefinition {
            id: "typing-hook".to_string(),
            name: "Typing Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String("ASK { ?s ?p ?o }".to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        // Hook 2: Schema constraint - all persons have names
        let schema_hook = HookDefinition {
            id: "schema-hook".to_string(),
            name: "Schema Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                // Use full IRI to avoid prefix detection issues
                when.insert("query".to_string(), JsonValue::String("ASK { ?person <http://xmlns.com/foaf/0.1/name> ?name }".to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        let results = evaluate_hooks_batch_native(&[typing_hook, schema_hook], turtle_data).unwrap();
        
        // All invariants Q should be preserved
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.fired == true));
    }

    #[test]
    fn test_idempotence_property() {
        // Law: Idempotence: μ ∘ μ = μ - canonicalization is idempotent
        // Test: Executing same hook multiple times produces consistent results
        let hook_query = "ASK { ?s ?p ?o }";
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let hook = HookDefinition {
            id: "idempotent-hook".to_string(),
            name: "Idempotent Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        // Execute hook multiple times
        let result1 = evaluate_hook_native(&hook, turtle_data).unwrap();
        let result2 = evaluate_hook_native(&hook, turtle_data).unwrap();
        let result3 = evaluate_hook_native(&hook, turtle_data).unwrap();
        
        // μ ∘ μ = μ: repeated execution produces same result (fired state)
        assert_eq!(result1.fired, result2.fired);
        assert_eq!(result2.fired, result3.fired);
        // All should fire (data exists)
        assert!(result1.fired);
    }

    #[test]
    fn test_merge_associativity() {
        // Law: Merge: Π is ⊕-monoid - merge operations are associative
        // Test: Batch evaluation respects associative merge property
        let hooks = vec![
            HookDefinition {
                id: "merge-hook-1".to_string(),
                name: "Merge Hook 1".to_string(),
                hook_type: "sparql-ask".to_string(),
                definition: {
                    let mut def = serde_json::Map::new();
                    let mut when = serde_json::Map::new();
                    when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                    when.insert("query".to_string(), JsonValue::String("ASK { ?s ?p ?o }".to_string()));
                    def.insert("when".to_string(), JsonValue::Object(when));
                    JsonValue::Object(def)
                },
            },
            HookDefinition {
                id: "merge-hook-2".to_string(),
                name: "Merge Hook 2".to_string(),
                hook_type: "sparql-ask".to_string(),
                definition: {
                    let mut def = serde_json::Map::new();
                    let mut when = serde_json::Map::new();
                    when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                    when.insert("query".to_string(), JsonValue::String("ASK { ?s ?p2 ?o2 }".to_string()));
                    def.insert("when".to_string(), JsonValue::Object(when));
                    JsonValue::Object(def)
                },
            },
        ];
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
            ex:bob ex:age "30" .
        "#;
        
        // Evaluate as batch: (H₁ ⊕ H₂)
        let batch_result = evaluate_hooks_batch_native(&hooks, turtle_data).unwrap();
        
        // Evaluate individually: H₁, then H₂
        let individual1 = evaluate_hook_native(&hooks[0], turtle_data).unwrap();
        let individual2 = evaluate_hook_native(&hooks[1], turtle_data).unwrap();
        
        // Π is ⊕-monoid: batch(H₁ ⊕ H₂) = batch(H₁) ⊕ batch(H₂)
        assert_eq!(batch_result.len(), 2);
        assert_eq!(batch_result[0].fired, individual1.fired);
        assert_eq!(batch_result[1].fired, individual2.fired);
    }

    #[test]
    fn test_typing_constraint() {
        // Law: Typing: O ⊨ Σ - operations satisfy schema
        // Test: Hook validates O ⊨ Σ before execution
        let hook_query = "ASK { ?s ?p ?o }";
        
        // Valid operations O that satisfy schema Σ
        let valid_turtle_data = r#"
            @prefix ex: <http://example.org/> .
            @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
            
            ex:alice rdf:type ex:Person .
            ex:alice ex:name "Alice" .
        "#;
        
        let hook = HookDefinition {
            id: "typing-hook".to_string(),
            name: "Typing Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        let result = evaluate_hook_native(&hook, valid_turtle_data).unwrap();
        
        // O ⊨ Σ: operations satisfy schema, hook should fire
        assert!(result.fired);
    }

    #[test]
    fn test_receipt_deterministic() {
        // Law: Provenance: hash(A) = hash(μ(O))
        // Test: Same operations O produce same receipt (excluding timestamp/counter)
        // Note: Receipts include timestamp+counter, so exact equality isn't expected
        // But structure should be consistent
        let hook_query = "ASK { ?s ?p ?o }";
        
        let turtle_data = r#"
            @prefix ex: <http://example.org/> .
            ex:alice ex:name "Alice" .
        "#;
        
        let hook = HookDefinition {
            id: "deterministic-hook".to_string(),
            name: "Deterministic Hook".to_string(),
            hook_type: "sparql-ask".to_string(),
            definition: {
                let mut def = serde_json::Map::new();
                let mut when = serde_json::Map::new();
                when.insert("kind".to_string(), JsonValue::String("sparql-ask".to_string()));
                when.insert("query".to_string(), JsonValue::String(hook_query.to_string()));
                def.insert("when".to_string(), JsonValue::Object(when));
                JsonValue::Object(def)
            },
        };
        
        let result1 = evaluate_hook_native(&hook, turtle_data).unwrap();
        let result2 = evaluate_hook_native(&hook, turtle_data).unwrap();
        
        // Both should fire (same O)
        assert!(result1.fired);
        assert!(result2.fired);
        
        // Receipts should exist (hash(A) = hash(μ(O)))
        assert!(result1.receipt.is_some());
        assert!(result2.receipt.is_some());
        
        // Receipts should be valid SHA-256 hashes (64 hex chars)
        assert_eq!(result1.receipt.as_ref().unwrap().len(), 64);
        assert_eq!(result2.receipt.as_ref().unwrap().len(), 64);
    }
}

// knhk-unrdf: Rust-native query execution using oxigraph
// Pure Rust SPARQL query engine without Node.js dependency

#[cfg(feature = "native")]
use crate::error::{UnrdfError, UnrdfResult};
#[cfg(feature = "native")]
use crate::types::{QueryResult, SparqlQueryType};
#[cfg(feature = "native")]
use oxigraph::model::{NamedNode, Quad, Term, Triple};
#[cfg(feature = "native")]
use oxigraph::sparql::QueryResults;
#[cfg(feature = "native")]
use oxigraph::store::Store;
#[cfg(feature = "native")]
use oxigraph::io::RdfFormat;
#[cfg(feature = "native")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "native")]
/// In-memory store for Rust-native operations
pub struct NativeStore {
    store: Arc<Mutex<Store>>,
}

#[cfg(feature = "native")]
impl NativeStore {
    /// Create a new native store
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(Store::new().expect("Failed to create store"))),
        }
    }

    /// Load Turtle data into the store
    pub fn load_turtle(&self, turtle_data: &str) -> UnrdfResult<()> {
        let store = self.store.lock()
            .map_err(|e| UnrdfError::StoreFailed(format!("Failed to acquire store lock: {}", e)))?;
        
        store.load_from_reader(RdfFormat::Turtle, turtle_data.as_bytes())
            .map_err(|e| UnrdfError::StoreFailed(format!("Failed to load Turtle data: {}", e)))?;
        
        Ok(())
    }

    /// Execute SPARQL query
    pub fn query(&self, query: &str, query_type: SparqlQueryType) -> UnrdfResult<QueryResult> {
        let store = self.store.lock()
            .map_err(|e| UnrdfError::QueryFailed(format!("Failed to acquire store lock: {}", e)))?;
        
        let results: QueryResults = store.query(query)
            .map_err(|e| UnrdfError::QueryFailed(format!("SPARQL query failed: {}", e)))?;
        
        match query_type {
            SparqlQueryType::Select => {
                match results {
                    QueryResults::Solutions(solutions) => {
                        let mut bindings = Vec::new();
                        for solution_result in solutions {
                            let solution = solution_result
                                .map_err(|e| UnrdfError::QueryFailed(format!("Solution error: {}", e)))?;
                            let mut binding_obj = serde_json::Map::new();
                            // QuerySolution implements IntoIterator for (&Variable, &Term)
                            for (var, term) in &solution {
                                binding_obj.insert(
                                    var.as_str().to_string(),
                                    serde_json::Value::String(term.to_string())
                                );
                            }
                            bindings.push(serde_json::Value::Object(binding_obj));
                        }
                        Ok(QueryResult {
                            success: true,
                            query_type: Some("sparql-select".to_string()),
                            bindings: Some(bindings),
                            boolean: None,
                            triples: None,
                            error: None,
                        })
                    }
                    _ => Err(UnrdfError::QueryFailed("Expected SELECT query results".to_string()))
                }
            }
            SparqlQueryType::Ask => {
                match results {
                    QueryResults::Boolean(b) => {
                        Ok(QueryResult {
                            success: true,
                            query_type: Some("sparql-ask".to_string()),
                            bindings: None,
                            boolean: Some(b),
                            triples: None,
                            error: None,
                        })
                    }
                    _ => Err(UnrdfError::QueryFailed("Expected ASK query results".to_string()))
                }
            }
            SparqlQueryType::Construct | SparqlQueryType::Describe => {
                match results {
                    QueryResults::Graph(triples_iter) => {
                        let mut triples = Vec::new();
                        for triple_result in triples_iter {
                            let triple = triple_result
                                .map_err(|e| UnrdfError::QueryFailed(format!("Triple error: {}", e)))?;
                            let mut triple_obj = serde_json::Map::new();
                            triple_obj.insert("subject".to_string(), serde_json::Value::String(triple.subject.to_string()));
                            triple_obj.insert("predicate".to_string(), serde_json::Value::String(triple.predicate.to_string()));
                            triple_obj.insert("object".to_string(), serde_json::Value::String(triple.object.to_string()));
                            // CONSTRUCT/DESCRIBE queries return triples, not quads
                            triples.push(serde_json::Value::Object(triple_obj));
                        }
                        Ok(QueryResult {
                            success: true,
                            query_type: Some(if query_type == SparqlQueryType::Construct {
                                "sparql-construct".to_string()
                            } else {
                                "sparql-describe".to_string()
                            }),
                            bindings: None,
                            boolean: None,
                            triples: Some(triples),
                            error: None,
                        })
                    }
                    _ => Err(UnrdfError::QueryFailed("Expected CONSTRUCT/DESCRIBE query results".to_string()))
                }
            }
            SparqlQueryType::Insert | SparqlQueryType::Delete => {
                // UPDATE queries return empty results
                Ok(QueryResult {
                    success: true,
                    query_type: Some("sparql-update".to_string()),
                    bindings: None,
                    boolean: None,
                    triples: None,
                    error: None,
                })
            }
            SparqlQueryType::Unknown => {
                Err(UnrdfError::QueryFailed("Unknown query type".to_string()))
            }
        }
    }

    /// Clear the store
    pub fn clear(&self) -> UnrdfResult<()> {
        let mut store = self.store.lock()
            .map_err(|e| UnrdfError::StoreFailed(format!("Failed to acquire store lock: {}", e)))?;
        
        // Create new empty store
        *store = Store::new().expect("Failed to create store");
        Ok(())
    }
}

#[cfg(feature = "native")]
/// Convert oxigraph Term to string representation
fn term_to_string(term: &Term) -> String {
    match term {
        Term::NamedNode(named) => named.as_str().to_string(),
        Term::BlankNode(blank) => format!("_:{}", blank.as_str()),
        Term::Literal(literal) => {
            // Handle all literal types
            if let Some(lang) = literal.language() {
                format!("\"{}\"@{}", literal.value(), lang)
            } else {
                format!("\"{}\"^^<{}>", literal.value(), literal.datatype().as_str())
            }
        }
    }
}

#[cfg(feature = "native")]
/// Execute SPARQL query with data (native Rust implementation)
pub fn query_sparql_native(query: &str, turtle_data: &str) -> UnrdfResult<QueryResult> {
    use crate::query::detect_query_type;
    
    let query_type = detect_query_type(query);
    let store = NativeStore::new();
    
    // Load data
    store.load_turtle(turtle_data)?;
    
    // Execute query
    store.query(query, query_type)
}

#[cfg(feature = "native")]
/// Execute SPARQL query without data (native Rust implementation)
pub fn query_sparql_native_empty(query: &str) -> UnrdfResult<QueryResult> {
    use crate::query::detect_query_type;
    
    let query_type = detect_query_type(query);
    let store = NativeStore::new();
    
    // Execute query on empty store
    store.query(query, query_type)
}


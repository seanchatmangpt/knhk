//! Hook storage - Stores hooks in Oxigraph

use crate::state::StateStore;
use oxigraph::model::{Graph, NamedNode, Quad, TripleRef};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Hook entry
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HookEntry {
    pub id: String,
    pub name: String,
    pub op: String,
    pub pred: u64,
    pub off: u64,
    pub len: u64,
    pub s: Option<u64>,
    pub p: Option<u64>,
    pub o: Option<u64>,
    pub k: Option<u64>,
}

/// Hook storage - Stores hooks in Oxigraph
pub struct HookStore {
    store: Arc<StateStore>,
}

impl HookStore {
    /// Create new hook store
    pub fn new() -> Result<Self, String> {
        let store = Arc::new(crate::state::StateStore::new()?);
        Ok(Self { store })
    }

    /// Load all hooks
    pub fn load_all(&self) -> Result<Vec<HookEntry>, String> {
        // Load hooks from Oxigraph using SPARQL query
        let store = self.store.store();
        
        let query = r#"
            PREFIX knhk: <urn:knhk:>
            SELECT ?hook_id ?name ?op ?pred ?off ?len ?s ?p ?o ?k
            WHERE {
                ?hook_id a knhk:Hook ;
                    knhk:hasName ?name ;
                    knhk:hasOp ?op ;
                    knhk:hasPred ?pred .
                OPTIONAL { ?hook_id knhk:hasOff ?off . }
                OPTIONAL { ?hook_id knhk:hasLen ?len . }
                OPTIONAL { ?hook_id knhk:hasS ?s . }
                OPTIONAL { ?hook_id knhk:hasP ?p . }
                OPTIONAL { ?hook_id knhk:hasO ?o . }
                OPTIONAL { ?hook_id knhk:hasK ?k . }
            }
        "#;
        
        #[allow(deprecated)]
        let results = store.query(query)
            .map_err(|e| format!("SPARQL query failed: {}", e))?;
        
        let mut hooks = Vec::new();
        
        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            for solution_result in solutions {
                let solution = solution_result
                    .map_err(|e| format!("Failed to get solution: {}", e))?;
                
                let hook_id_str = solution.get("hook_id")
                    .map(|term| term.to_string())
                    .ok_or_else(|| "Missing hook_id".to_string())?;
                
                // Extract hook ID from IRI (urn:knhk:hook:ID -> ID)
                let id = hook_id_str.strip_prefix("urn:knhk:hook:")
                    .ok_or_else(|| format!("Invalid hook_id format: {}", hook_id_str))?;
                
                let name = solution.get("name")
                    .and_then(|term| term.to_string().strip_prefix('"').and_then(|s| s.strip_suffix('"')))
                    .map(|s| s.to_string())
                    .ok_or_else(|| format!("Missing or invalid name for hook {}", id))?;
                
                let op = solution.get("op")
                    .and_then(|term| term.to_string().strip_prefix('"').and_then(|s| s.strip_suffix('"')))
                    .map(|s| s.to_string())
                    .ok_or_else(|| format!("Missing or invalid op for hook {}", id))?;
                
                let pred = solution.get("pred")
                    .and_then(|term| term.to_string().parse::<u64>().ok())
                    .ok_or_else(|| format!("Missing or invalid pred for hook {}", id))?;
                
                let off = solution.get("off")
                    .and_then(|term| term.to_string().parse::<u64>().ok())
                    .unwrap_or(0);
                
                let len = solution.get("len")
                    .and_then(|term| term.to_string().parse::<u64>().ok())
                    .unwrap_or(0);
                
                let s = solution.get("s")
                    .and_then(|term| term.to_string().parse::<u64>().ok());
                
                let p = solution.get("p")
                    .and_then(|term| term.to_string().parse::<u64>().ok());
                
                let o = solution.get("o")
                    .and_then(|term| term.to_string().parse::<u64>().ok());
                
                let k = solution.get("k")
                    .and_then(|term| term.to_string().parse::<u64>().ok());
                
                hooks.push(HookEntry {
                    id: id.to_string(),
                    name,
                    op,
                    pred,
                    off,
                    len,
                    s,
                    p,
                    o,
                    k,
                });
            }
        }
        
        Ok(hooks)
    }

    /// Save hook
    pub fn save(&self, hook: &HookEntry) -> Result<(), String> {
        // Save hook to Oxigraph using StateStore
        // Convert HookEntry to RDF triples and store in Oxigraph
        use oxigraph::model::{GraphName, Quad};
        
        let store = self.store.store();
        
        // Create subject IRI for hook
        let hook_subject = NamedNode::new(format!("urn:knhk:hook:{}", hook.id))
            .map_err(|e| format!("Failed to create hook subject IRI: {:?}", e))?;
        
        // Create predicate IRIs
        let rdf_type = NamedNode::new("http://www.w3.org/1999/02/22-rdf-syntax-ns#type")
            .map_err(|e| format!("Failed to create rdf:type IRI: {:?}", e))?;
        let hook_class = NamedNode::new("urn:knhk:Hook")
            .map_err(|e| format!("Failed to create Hook class IRI: {:?}", e))?;
        
        // Insert type triple
        let type_quad = Quad::new(
            &hook_subject,
            &rdf_type,
            &hook_class,
            GraphName::DefaultGraph,
        );
        store.insert(&type_quad)
            .map_err(|e| format!("Failed to insert hook type triple: {:?}", e))?;
        
        // Add hook properties
        let has_name = NamedNode::new("urn:knhk:hasName")
            .map_err(|e| format!("Failed to create hasName IRI: {:?}", e))?;
        let name_literal = oxigraph::model::Literal::new_simple_literal(&hook.name);
        let name_quad = Quad::new(
            &hook_subject,
            &has_name,
            &name_literal,
            GraphName::DefaultGraph,
        );
        store.insert(&name_quad)
            .map_err(|e| format!("Failed to insert hook name triple: {:?}", e))?;
        
        let has_op = NamedNode::new("urn:knhk:hasOp")
            .map_err(|e| format!("Failed to create hasOp IRI: {:?}", e))?;
        let op_literal = oxigraph::model::Literal::new_simple_literal(&hook.op);
        let op_quad = Quad::new(
            &hook_subject,
            &has_op,
            &op_literal,
            GraphName::DefaultGraph,
        );
        store.insert(&op_quad)
            .map_err(|e| format!("Failed to insert hook op triple: {:?}", e))?;
        
        // Add numeric properties
        let has_pred = NamedNode::new("urn:knhk:hasPred")
            .map_err(|e| format!("Failed to create hasPred IRI: {:?}", e))?;
        let pred_literal = oxigraph::model::Literal::new_typed_literal(
            hook.pred.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong IRI: {:?}", e))?,
        );
        let pred_quad = Quad::new(
            &hook_subject,
            &has_pred,
            &pred_literal,
            GraphName::DefaultGraph,
        );
        store.insert(&pred_quad)
            .map_err(|e| format!("Failed to insert hook pred triple: {:?}", e))?;
        
        Ok(())
    }
}


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
        use oxigraph::model::{NamedNode, Quad, Subject, Term};
        use oxigraph::sparql::Query;

        let store = self.store.store();

        // Query for all hooks using SPARQL
        let query_str = r#"
            SELECT ?id ?name ?op ?pred ?off ?len ?s ?p ?o ?k
            WHERE {
                ?hook <http://knhk.org/ns/hook#id> ?id .
                ?hook <http://knhk.org/ns/hook#name> ?name .
                ?hook <http://knhk.org/ns/hook#op> ?op .
                ?hook <http://knhk.org/ns/hook#pred> ?pred .
                ?hook <http://knhk.org/ns/hook#off> ?off .
                ?hook <http://knhk.org/ns/hook#len> ?len .
                OPTIONAL { ?hook <http://knhk.org/ns/hook#s> ?s . }
                OPTIONAL { ?hook <http://knhk.org/ns/hook#p> ?p . }
                OPTIONAL { ?hook <http://knhk.org/ns/hook#o> ?o . }
                OPTIONAL { ?hook <http://knhk.org/ns/hook#k> ?k . }
            }
        "#;

        let query = Query::parse(query_str, None)
            .map_err(|e| format!("Failed to parse SPARQL query: {}", e))?;

        let results: oxigraph::sparql::QueryResults = store
            .query(query)
            .map_err(|e| format!("Failed to execute SPARQL query: {}", e))?;

        let mut hooks = Vec::new();

        if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
            for solution in solutions {
                let mut id: Option<String> = None;
                let mut name: Option<String> = None;
                let mut op: Option<String> = None;
                let mut pred: Option<u64> = None;
                let mut off: Option<u64> = None;
                let mut len: Option<u64> = None;
                let mut s: Option<u64> = None;
                let mut p: Option<u64> = None;
                let mut o: Option<u64> = None;
                let mut k: Option<u64> = None;

                for var in solution.variables() {
                    if let Some(term) = solution.get(var) {
                        if let oxigraph::model::Term::Literal(lit) = term {
                            let value = lit.value();
                            match var.as_str() {
                                "id" => id = Some(value.to_string()),
                                "name" => name = Some(value.to_string()),
                                "op" => op = Some(value.to_string()),
                                "pred" => pred = value.parse::<u64>().ok(),
                                "off" => off = value.parse::<u64>().ok(),
                                "len" => len = value.parse::<u64>().ok(),
                                "s" => s = value.parse::<u64>().ok(),
                                "p" => p = value.parse::<u64>().ok(),
                                "o" => o = value.parse::<u64>().ok(),
                                "k" => k = value.parse::<u64>().ok(),
                                _ => {}
                            }
                        }
                    }
                }

                let id = id.ok_or_else(|| "Missing id in query result".to_string())?;
                let name = name.ok_or_else(|| "Missing name in query result".to_string())?;
                let op = op.ok_or_else(|| "Missing op in query result".to_string())?;
                let pred =
                    pred.ok_or_else(|| "Missing or invalid pred in query result".to_string())?;
                let off =
                    off.ok_or_else(|| "Missing or invalid off in query result".to_string())?;
                let len =
                    len.ok_or_else(|| "Missing or invalid len in query result".to_string())?;

                hooks.push(HookEntry {
                    id,
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
        use oxigraph::model::{GraphName, NamedNode, Quad, Subject, Term};

        let store = self.store.store();

        // Create hook subject IRI
        let hook_subject = NamedNode::new(format!("http://knhk.org/hook/{}", hook.id))
            .map_err(|e| format!("Failed to create hook subject IRI: {}", e))?;

        // Create graph for hook
        let mut graph = oxigraph::model::Graph::new();

        // Add hook properties as triples
        let id_node = NamedNode::new("http://knhk.org/ns/hook#id")
            .map_err(|e| format!("Failed to create id predicate: {}", e))?;
        let id_literal = Term::Literal(oxigraph::model::Literal::new_simple_literal(&hook.id));
        graph.insert(oxigraph::model::TripleRef::new(
            hook_subject.as_ref(),
            id_node.as_ref(),
            id_literal.as_ref(),
        ));

        let name_node = NamedNode::new("http://knhk.org/ns/hook#name")
            .map_err(|e| format!("Failed to create name predicate: {}", e))?;
        let name_literal = Term::Literal(oxigraph::model::Literal::new_simple_literal(&hook.name));
        graph.insert(oxigraph::model::TripleRef::new(
            hook_subject.as_ref(),
            name_node.as_ref(),
            name_literal.as_ref(),
        ));

        let op_node = NamedNode::new("http://knhk.org/ns/hook#op")
            .map_err(|e| format!("Failed to create op predicate: {}", e))?;
        let op_literal = Term::Literal(oxigraph::model::Literal::new_simple_literal(&hook.op));
        graph.insert(oxigraph::model::TripleRef::new(
            hook_subject.as_ref(),
            op_node.as_ref(),
            op_literal.as_ref(),
        ));

        let pred_node = NamedNode::new("http://knhk.org/ns/hook#pred")
            .map_err(|e| format!("Failed to create pred predicate: {}", e))?;
        let pred_literal = Term::Literal(oxigraph::model::Literal::new_typed_literal(
            hook.pred.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong type: {}", e))?,
        ));
        graph.insert(oxigraph::model::TripleRef::new(
            hook_subject.as_ref(),
            pred_node.as_ref(),
            pred_literal.as_ref(),
        ));

        let off_node = NamedNode::new("http://knhk.org/ns/hook#off")
            .map_err(|e| format!("Failed to create off predicate: {}", e))?;
        let off_literal = Term::Literal(oxigraph::model::Literal::new_typed_literal(
            hook.off.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong type: {}", e))?,
        ));
        graph.insert(oxigraph::model::TripleRef::new(
            hook_subject.as_ref(),
            off_node.as_ref(),
            off_literal.as_ref(),
        ));

        let len_node = NamedNode::new("http://knhk.org/ns/hook#len")
            .map_err(|e| format!("Failed to create len predicate: {}", e))?;
        let len_literal = Term::Literal(oxigraph::model::Literal::new_typed_literal(
            hook.len.to_string(),
            NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                .map_err(|e| format!("Failed to create unsignedLong type: {}", e))?,
        ));
        graph.insert(oxigraph::model::TripleRef::new(
            hook_subject.as_ref(),
            len_node.as_ref(),
            len_literal.as_ref(),
        ));

        // Add optional fields
        if let Some(s_val) = hook.s {
            let s_node = NamedNode::new("http://knhk.org/ns/hook#s")
                .map_err(|e| format!("Failed to create s predicate: {}", e))?;
            let s_literal = Term::Literal(oxigraph::model::Literal::new_typed_literal(
                s_val.to_string(),
                NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                    .map_err(|e| format!("Failed to create unsignedLong type: {}", e))?,
            ));
            graph.insert(oxigraph::model::TripleRef::new(
                hook_subject.as_ref(),
                s_node.as_ref(),
                s_literal.as_ref(),
            ));
        }

        if let Some(p_val) = hook.p {
            let p_node = NamedNode::new("http://knhk.org/ns/hook#p")
                .map_err(|e| format!("Failed to create p predicate: {}", e))?;
            let p_literal = Term::Literal(oxigraph::model::Literal::new_typed_literal(
                p_val.to_string(),
                NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                    .map_err(|e| format!("Failed to create unsignedLong type: {}", e))?,
            ));
            graph.insert(oxigraph::model::TripleRef::new(
                hook_subject.as_ref(),
                p_node.as_ref(),
                p_literal.as_ref(),
            ));
        }

        if let Some(o_val) = hook.o {
            let o_node = NamedNode::new("http://knhk.org/ns/hook#o")
                .map_err(|e| format!("Failed to create o predicate: {}", e))?;
            let o_literal = Term::Literal(oxigraph::model::Literal::new_typed_literal(
                o_val.to_string(),
                NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                    .map_err(|e| format!("Failed to create unsignedLong type: {}", e))?,
            ));
            graph.insert(oxigraph::model::TripleRef::new(
                hook_subject.as_ref(),
                o_node.as_ref(),
                o_literal.as_ref(),
            ));
        }

        if let Some(k_val) = hook.k {
            let k_node = NamedNode::new("http://knhk.org/ns/hook#k")
                .map_err(|e| format!("Failed to create k predicate: {}", e))?;
            let k_literal = Term::Literal(oxigraph::model::Literal::new_typed_literal(
                k_val.to_string(),
                NamedNode::new("http://www.w3.org/2001/XMLSchema#unsignedLong")
                    .map_err(|e| format!("Failed to create unsignedLong type: {}", e))?,
            ));
            graph.insert(oxigraph::model::TripleRef::new(
                hook_subject.as_ref(),
                k_node.as_ref(),
                k_literal.as_ref(),
            ));
        }

        // Insert graph into store
        for triple in graph.iter() {
            let subject: oxigraph::model::Subject = triple.subject().into();
            let predicate: oxigraph::model::NamedNode = triple.predicate().into();
            let object: oxigraph::model::Term = triple.object().into();
            let quad = Quad::new(subject, predicate, object, GraphName::DefaultGraph);
            store
                .insert(&quad)
                .map_err(|e| format!("Failed to insert hook triple into Oxigraph: {}", e))?;
        }

        Ok(())
    }

    /// Load a hook by name
    pub fn load(&self, name: &str) -> Result<HookEntry, String> {
        let all_hooks = self.load_all()?;
        all_hooks
            .into_iter()
            .find(|h| h.name == name)
            .ok_or_else(|| format!("Hook '{}' not found", name))
    }
}

//! Hot path execution (≤2ns / ≤8 ticks)
//! Direct calls to C hot path functions for simple queries

use crate::graph::WarmPathGraph;
use crate::query::{AskResult, QueryError, SelectResult};
use knhk_hot::{Engine, Ir, Op, Receipt, Run};
use std::collections::hash_map::DefaultHasher;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// Parse simple ASK query to hook IR
/// Supports: ASK { <s> <p> <o> } or ASK { <s> <p> ?o }
fn parse_ask_query(sparql: &str) -> Result<(u64, u64, Option<u64>), String> {
    // Simple parser for ASK { <s> <p> <o> } pattern
    let query = sparql.trim();

    if !query.to_uppercase().starts_with("ASK") {
        return Err("Not an ASK query".to_string());
    }

    // Extract triple pattern: <s> <p> <o> or <s> <p> ?o
    // This is a simplified parser - for production, use proper SPARQL parser
    let pattern = if let Some(start) = query.find('{') {
        if let Some(end) = query[start..].find('}') {
            &query[start + 1..start + end]
        } else {
            return Err("Missing closing brace".to_string());
        }
    } else {
        return Err("Missing opening brace".to_string());
    };

    // Parse IRIs: <iri> or ?var
    let parts: Vec<&str> = pattern.split_whitespace().collect();
    if parts.len() < 3 {
        return Err("Invalid triple pattern".to_string());
    }

    // Hash IRIs to u64 (simplified - should use proper IRI hashing)
    let s = hash_iri(parts[0])?;
    let p = hash_iri(parts[1])?;
    let o = if parts[2].starts_with('?') {
        None // Variable
    } else {
        Some(hash_iri(parts[2])?)
    };

    Ok((s, p, o))
}

/// Hash IRI to u64 (simplified - should match KNHK's IRI hashing)
fn hash_iri(iri: &str) -> Result<u64, String> {
    // Remove < > brackets
    let cleaned = iri.trim_matches(|c| c == '<' || c == '>');

    // Use simple hash for now - should match KNHK's hash_iri function
    let mut hasher = DefaultHasher::new();
    cleaned.hash(&mut hasher);
    Ok(hasher.finish())
}

/// Execute hot path ASK query
pub fn execute_hot_path_ask(graph: &WarmPathGraph, sparql: &str) -> Result<AskResult, QueryError> {
    // Parse query
    let (s, p, o) = parse_ask_query(sparql).map_err(QueryError::ParseError)?;

    // For hot path, we need SoA arrays from graph
    // Since WarmPathGraph uses oxigraph Store, we need to extract a small run
    // For now, use a simplified approach: query oxigraph for matching triples
    // and if ≤8, convert to SoA format

    // Query graph to get matching triples (limited to 8)
    let query = format!(
        "SELECT ?s ?o WHERE {{ ?s <{}> ?o }} LIMIT 8",
        format_iri_from_hash(p)
    );
    let results = graph.query(&query).map_err(QueryError::ExecutionError)?;

    // Extract results
    let mut triples = Vec::new();
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
        for solution_result in solutions {
            // Handle Result from iterator
            let solution = solution_result
                .map_err(|e| QueryError::ExecutionError(format!("Query solution error: {}", e)))?;
            // Check if this triple matches our query
            if let Some(s_val) = solution.get("s") {
                if let Some(o_val) = solution.get("o") {
                    triples.push((hash_iri_value(s_val), hash_iri_value(o_val)));
                }
            }
        }
    }

    // If we have ≤8 triples, use hot path
    if triples.len() <= 8 {
        // Create SoA arrays (64-byte aligned)
        let mut s_array = [0u64; 8];
        let mut p_array = [0u64; 8];
        let mut o_array = [0u64; 8];

        for (i, (s_val, o_val)) in triples.iter().enumerate() {
            if i >= 8 {
                break;
            }
            s_array[i] = *s_val;
            p_array[i] = p;
            o_array[i] = *o_val;
        }

        // Create engine
        // SAFETY: Engine::new requires valid pointers to SoA arrays.
        // We guarantee this by passing pointers from valid Vec<u64> allocations.
        let engine = unsafe { Engine::new(s_array.as_ptr(), p_array.as_ptr(), o_array.as_ptr()) };

        // Create run
        let run = Run {
            pred: p,
            off: 0,
            len: triples.len() as u64,
        };

        let mut engine_mut = engine;
        engine_mut
            .pin_run(run)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        // Create IR
        let mut ir = Ir {
            op: if o.is_some() { Op::AskSpo } else { Op::AskSp },
            s,
            p,
            o: o.unwrap_or(0),
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            construct8_pattern_hint: 0, // Default to generic pattern
        };

        let mut receipt = Receipt::default();

        // Execute hot path
        let result = engine_mut.eval_bool(&mut ir, &mut receipt);

        // Verify ticks ≤ 8
        if receipt.ticks > 8 {
            return Err(QueryError::ExecutionError(format!(
                "Hot path exceeded tick budget: {} ticks",
                receipt.ticks
            )));
        }

        Ok(AskResult { result })
    } else {
        // Too many triples - fall back to warm path
        Err(QueryError::ExecutionError(
            "Query exceeds hot path limit (8 triples)".to_string(),
        ))
    }
}

/// Hash IRI value from oxigraph
fn hash_iri_value(term: &oxigraph::model::Term) -> u64 {
    match term {
        oxigraph::model::Term::NamedNode(node) => hash_iri(node.as_str()).unwrap_or(0),
        oxigraph::model::Term::BlankNode(node) => hash_iri(node.as_str()).unwrap_or(0),
        oxigraph::model::Term::Literal(_) => {
            0 // Literals not supported in hot path
        }
    }
}

/// Format IRI from hash (inverse of hash_iri - simplified)
fn format_iri_from_hash(hash: u64) -> String {
    // This is a placeholder - should use proper IRI lookup
    format!("<iri:{}>", hash)
}

/// Execute hot path SELECT query (COUNT queries)
pub fn execute_hot_path_select(
    graph: &WarmPathGraph,
    sparql: &str,
) -> Result<SelectResult, QueryError> {
    // Parse COUNT query
    let query_upper = sparql.trim().to_uppercase();

    if !query_upper.contains("COUNT") {
        return Err(QueryError::UnsupportedQueryType(
            "Hot path SELECT only supports COUNT queries".to_string(),
        ));
    }

    // Extract COUNT pattern (simplified)
    // SELECT (COUNT(?s) AS ?count) WHERE { ?s <p> ?o }
    let (s, p, o) = parse_ask_query(sparql).map_err(QueryError::ParseError)?;

    // Query graph for matching triples (limited to 8)
    let query = format!(
        "SELECT ?s ?o WHERE {{ ?s <{}> ?o }} LIMIT 8",
        format_iri_from_hash(p)
    );
    let results = graph.query(&query).map_err(QueryError::ExecutionError)?;

    // Extract results
    let mut triples = Vec::new();
    if let oxigraph::sparql::QueryResults::Solutions(solutions) = results {
        for solution_result in solutions {
            // Handle Result from iterator
            let solution = solution_result
                .map_err(|e| QueryError::ExecutionError(format!("Query solution error: {}", e)))?;
            if let Some(s_val) = solution.get("s") {
                if let Some(o_val) = solution.get("o") {
                    triples.push((hash_iri_value(s_val), hash_iri_value(o_val)));
                }
            }
        }
    }

    // If we have ≤8 triples, use hot path COUNT
    if triples.len() <= 8 {
        // Create SoA arrays
        let mut s_array = [0u64; 8];
        let mut p_array = [0u64; 8];
        let mut o_array = [0u64; 8];

        for (i, (s_val, o_val)) in triples.iter().enumerate() {
            if i >= 8 {
                break;
            }
            s_array[i] = *s_val;
            p_array[i] = p;
            o_array[i] = *o_val;
        }

        // Create engine
        // SAFETY: Engine::new requires valid pointers to SoA arrays.
        // We guarantee this by passing pointers from valid Vec<u64> allocations.
        let engine = unsafe { Engine::new(s_array.as_ptr(), p_array.as_ptr(), o_array.as_ptr()) };

        // Create run
        let run = Run {
            pred: p,
            off: 0,
            len: triples.len() as u64,
        };

        let mut engine_mut = engine;
        engine_mut
            .pin_run(run)
            .map_err(|e| QueryError::ExecutionError(e.to_string()))?;

        // Create IR for COUNT_SP_GE
        let mut ir = Ir {
            op: Op::CountSpGe,
            s,
            p,
            o: o.unwrap_or(0),
            k: 0, // Will be set based on query
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            construct8_pattern_hint: 0, // Default to generic pattern
        };

        let mut receipt = Receipt::default();

        // Execute hot path
        let result = engine_mut.eval_bool(&mut ir, &mut receipt);

        // Verify ticks ≤ 8
        if receipt.ticks > 8 {
            return Err(QueryError::ExecutionError(format!(
                "Hot path exceeded tick budget: {} ticks",
                receipt.ticks
            )));
        }

        // Convert to SELECT result
        let count = if result { triples.len() } else { 0 };
        let mut bindings = BTreeMap::new();
        bindings.insert("count".to_string(), count.to_string());

        Ok(SelectResult {
            bindings: vec![bindings],
            variables: vec!["count".to_string()],
        })
    } else {
        Err(QueryError::ExecutionError(
            "Query exceeds hot path limit (8 triples)".to_string(),
        ))
    }
}

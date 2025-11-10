//! Path selection logic for routing queries to hot/warm/cold paths
//! Analyzes query complexity and data size to determine optimal execution path

use crate::runtime_class::RuntimeClass;

/// Query execution path
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryPath {
    /// Hot path: ≤8 ticks, simple operations, data size ≤8
    Hot,
    /// Warm path: ≤500µs budget, ≤1ms SLO, SPARQL queries, data size ≤10K
    Warm,
    /// Cold path: ≤200ms budget, ≤500ms SLO, complex operations, SHACL, reasoning
    Cold,
}

/// Path selection result with runtime class
#[derive(Debug, Clone)]
pub struct PathSelectionResult {
    /// Query execution path
    pub path: QueryPath,
    /// Runtime class (R1/W1/C1)
    pub runtime_class: RuntimeClass,
}

/// Select execution path based on query and data characteristics
/// 
/// # Arguments
/// * `query` - SPARQL query string
/// * `data_size` - Estimated data size (number of triples)
/// 
/// # Returns
/// `QueryPath` indicating which path should execute the query
pub fn select_path(query: &str, data_size: usize) -> QueryPath {
    // Check if query fits hot path constraints
    if is_hot_path_query(query) && data_size <= 8 {
        return QueryPath::Hot;
    }
    
    // Check if query fits warm path (oxigraph)
    if is_warm_path_query(query) && data_size <= 10000 {
        return QueryPath::Warm;
    }
    
    // Otherwise, use warm path (oxigraph)
    QueryPath::Cold
}

/// Select execution path with runtime class classification
/// 
/// # Arguments
/// * `query` - SPARQL query string
/// * `data_size` - Estimated data size (number of triples)
/// * `operation_type` - Operation type (e.g., "ASK_SP", "CONSTRUCT8", "SPARQL_SELECT")
/// 
/// # Returns
/// `PathSelectionResult` with path and runtime class
pub fn select_path_with_class(query: &str, data_size: usize, operation_type: &str) -> Result<PathSelectionResult, String> {
    // Classify operation using RuntimeClass
    let runtime_class = RuntimeClass::classify_operation(operation_type, data_size)?;
    
    // Map runtime class to query path
    let path = match runtime_class {
        RuntimeClass::R1 => QueryPath::Hot,
        RuntimeClass::W1 => QueryPath::Warm,
        RuntimeClass::C1 => QueryPath::Cold,
    };
    
    Ok(PathSelectionResult {
        path,
        runtime_class,
    })
}

/// Check if query fits hot path constraints
/// 
/// Hot path supports:
/// - Simple ASK queries (no OPTIONAL, FILTER, UNION)
/// - COUNT queries (≤8 elements)
/// - Single predicate queries
pub fn is_hot_path_query(query: &str) -> bool {
    let query_upper = query.trim().to_uppercase();
    
    // Must be ASK or simple SELECT with COUNT
    if !query_upper.starts_with("ASK") && !query_upper.contains("COUNT") {
        return false;
    }
    
    // Must not contain complex features
    if query_upper.contains("OPTIONAL") {
        return false;
    }
    
    if query_upper.contains("FILTER") {
        return false;
    }
    
    if query_upper.contains("UNION") {
        return false;
    }
    
    if query_upper.contains("GROUP BY") {
        return false;
    }
    
    if query_upper.contains("ORDER BY") {
        return false;
    }
    
    if query_upper.contains("LIMIT") || query_upper.contains("OFFSET") {
        return false;
    }
    
    true
}

/// Check if query fits warm path (oxigraph)
/// 
/// Warm path supports:
/// - SELECT, ASK, CONSTRUCT, DESCRIBE queries
/// - Basic SPARQL features (FILTER, OPTIONAL, UNION)
/// - Property paths (limited)
/// - Does NOT support: UPDATE, SHACL validation, reasoning
pub fn is_warm_path_query(query: &str) -> bool {
    let query_upper = query.trim().to_uppercase();
    
    // Check for unsupported query types
    if query_upper.starts_with("INSERT") {
        return false;
    }
    
    if query_upper.starts_with("DELETE") {
        return false;
    }
    
    if query_upper.starts_with("LOAD") {
        return false;
    }
    
    if query_upper.starts_with("CLEAR") {
        return false;
    }
    
    if query_upper.starts_with("CREATE") {
        return false;
    }
    
    if query_upper.starts_with("DROP") {
        return false;
    }
    
    // Check for SHACL-specific keywords
    if query_upper.contains("SHACL") || query_upper.contains("SH:") {
        return false;
    }
    
    // Check for reasoning-specific patterns
    if query_upper.contains("REASONING") || query_upper.contains("INFER") {
        return false;
    }
    
    // Check for very complex property paths
    if query_upper.contains("+") && query_upper.matches("+").count() > 2 {
        return false;
    }
    
    // Check for very complex UNION patterns
    if query_upper.matches("UNION").count() > 5 {
        return false;
    }
    
    true
}

/// Check if query requires warm path (oxigraph)
/// 
/// Cold path required for:
/// - UPDATE queries
/// - SHACL validation
/// - OWL reasoning
/// - Complex property paths
/// - Very large result sets
pub fn is_cold_path_query(query: &str) -> bool {
    !is_hot_path_query(query) && !is_warm_path_query(query)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_path_simple_ask() {
        let query = "ASK { <s> <p> <o> }";
        assert!(is_hot_path_query(query));
        assert_eq!(select_path(query, 5), QueryPath::Hot);
    }

    #[test]
    fn test_hot_path_with_filter() {
        let query = "ASK { <s> <p> <o> FILTER (?o > 10) }";
        assert!(!is_hot_path_query(query));
        assert_eq!(select_path(query, 5), QueryPath::Warm);
    }

    #[test]
    fn test_warm_path_select() {
        let query = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }";
        assert!(is_warm_path_query(query));
        assert_eq!(select_path(query, 100), QueryPath::Warm);
    }

    #[test]
    fn test_warm_path_construct() {
        let query = "CONSTRUCT { ?s <p> ?o } WHERE { ?s <p> ?o }";
        assert!(is_warm_path_query(query));
        assert_eq!(select_path(query, 1000), QueryPath::Warm);
    }

    #[test]
    fn test_cold_path_update() {
        let query = "INSERT { <s> <p> <o> } WHERE {}";
        assert!(!is_warm_path_query(query));
        assert_eq!(select_path(query, 100), QueryPath::Cold);
    }

    #[test]
    fn test_cold_path_shacl() {
        let query = "ASK { <s> sh:hasValue <o> }";
        assert!(!is_warm_path_query(query));
        assert_eq!(select_path(query, 100), QueryPath::Cold);
    }

    #[test]
    fn test_data_size_hot_path() {
        let query = "ASK { <s> <p> <o> }";
        assert_eq!(select_path(query, 8), QueryPath::Hot);
        assert_eq!(select_path(query, 9), QueryPath::Warm);
    }

    #[test]
    fn test_data_size_warm_path() {
        let query = "SELECT ?s WHERE { ?s <p> <o> }";
        assert_eq!(select_path(query, 10000), QueryPath::Warm);
        assert_eq!(select_path(query, 10001), QueryPath::Cold);
    }
}

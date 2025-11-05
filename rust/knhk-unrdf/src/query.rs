// knhk-unrdf: SPARQL query execution
// Execute SPARQL queries via unrdf

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::types::{QueryResult, SparqlQueryType};

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
    let state = get_state()?;
    
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


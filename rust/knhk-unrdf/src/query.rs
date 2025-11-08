// knhk-unrdf: SPARQL query execution
// Execute SPARQL queries via unrdf

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::template::TemplateEngine;
use crate::types::{QueryResult, SparqlQueryType};
use tera::Context;

/// Detect SPARQL query type from query string
pub fn detect_query_type(query: &str) -> SparqlQueryType {
    let query_upper = query.trim().to_uppercase();

    // Remove PREFIX declarations for detection
    let query_without_prefix = query_upper
        .lines()
        .filter(|line| !line.trim().starts_with("PREFIX"))
        .collect::<Vec<_>>()
        .join(" ");

    // Check for UPDATE operations first (INSERT/DELETE)
    if query_without_prefix.starts_with("INSERT") || query_without_prefix.starts_with("DELETE") {
        if query_without_prefix.contains("INSERT") {
            return SparqlQueryType::Insert;
        }
        if query_without_prefix.contains("DELETE") {
            return SparqlQueryType::Delete;
        }
        return SparqlQueryType::Insert; // Default to Insert for UPDATE
    }

    // Check for query types
    if query_without_prefix.starts_with("ASK") {
        return SparqlQueryType::Ask;
    }
    if query_without_prefix.starts_with("CONSTRUCT") {
        return SparqlQueryType::Construct;
    }
    if query_without_prefix.starts_with("DESCRIBE") {
        return SparqlQueryType::Describe;
    }
    if query_without_prefix.starts_with("SELECT") {
        return SparqlQueryType::Select;
    }

    SparqlQueryType::Unknown
}

/// Execute SPARQL query via unrdf with automatic query type detection
pub fn query_sparql(query: &str) -> UnrdfResult<QueryResult> {
    let query_type = detect_query_type(query);
    query_sparql_with_type(query, query_type)
}

/// Execute SPARQL query with data to store first (for stateful operations)
/// This combines store and query in a single script so data persists
pub fn query_sparql_with_data(query: &str, turtle_data: &str) -> UnrdfResult<QueryResult> {
    let query_type = detect_query_type(query);
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

    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    context.insert("turtle_data", turtle_data);
    context.insert("query", query);
    context.insert("query_type", &query_type_str);

    let script = template_engine
        .lock()
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e))
        })?
        .render("query-with-data", &context)
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to render query-with-data template: {}", e))
        })?;

    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Extract JSON from output (unrdf prints initialization messages to stdout)
        // Find the last line that looks like JSON (starts with { or [)
        let json_line = output
            .lines()
            .rev()
            .find(|line| line.trim().starts_with('{') || line.trim().starts_with('['))
            .ok_or_else(|| {
                UnrdfError::QueryFailed(format!("No JSON found in output. Full output: {}", output))
            })?;

        let result: QueryResult = serde_json::from_str(json_line.trim()).map_err(|e| {
            UnrdfError::QueryFailed(format!(
                "Failed to parse result: {} - JSON line: {}",
                e, json_line
            ))
        })?;
        Ok(result)
    })
}

/// Execute SPARQL query via unrdf with explicit query type
pub fn query_sparql_with_type(
    query: &str,
    query_type: SparqlQueryType,
) -> UnrdfResult<QueryResult> {
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

    // Use Tera template engine
    let template_engine = TemplateEngine::get()?;
    let mut context = Context::new();
    context.insert("query", query);
    context.insert("query_type", &query_type_str);

    let script = template_engine
        .lock()
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e))
        })?
        .render("query-only", &context)
        .map_err(|e| {
            UnrdfError::InvalidInput(format!("Failed to render query-only template: {}", e))
        })?;

    state.runtime.block_on(async {
        let output = execute_unrdf_script(&script).await?;
        // Extract JSON from output (unrdf prints initialization messages to stdout)
        // Find the last line that looks like JSON (starts with { or [)
        let json_line = output
            .lines()
            .rev()
            .find(|line| line.trim().starts_with('{') || line.trim().starts_with('['))
            .ok_or_else(|| {
                UnrdfError::QueryFailed(format!("No JSON found in output. Full output: {}", output))
            })?;

        let result: QueryResult = serde_json::from_str(json_line.trim()).map_err(|e| {
            UnrdfError::QueryFailed(format!(
                "Failed to parse result: {} - JSON line: {}",
                e, json_line
            ))
        })?;
        Ok(result)
    })
}

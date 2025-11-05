// Utility functions for unrdf integration

use crate::errors::UnrdfError;
use crate::types::SparqlQueryType;
use tokio::process::Command;

/// Escape a string for use in JavaScript template literals
pub(crate) fn escape_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Detect SPARQL query type from query string
pub(crate) fn detect_sparql_query_type(query: &str) -> SparqlQueryType {
    let query_upper = query.trim_start().to_uppercase();
    
    if query_upper.starts_with("SELECT") {
        SparqlQueryType::Select
    } else if query_upper.starts_with("ASK") {
        SparqlQueryType::Ask
    } else if query_upper.starts_with("CONSTRUCT") {
        SparqlQueryType::Construct
    } else if query_upper.starts_with("DESCRIBE") {
        SparqlQueryType::Describe
    } else if query_upper.starts_with("INSERT") || query_upper.starts_with("DELETE") || query_upper.starts_with("WITH") {
        SparqlQueryType::Update
    } else {
        // Default to SELECT for unknown queries
        SparqlQueryType::Select
    }
}

/// Execute a script using Node.js and unrdf
pub(crate) async fn execute_unrdf_script(script_content: &str, unrdf_path: &str) -> Result<String, UnrdfError> {
    // Write script to temporary file
    let temp_file = std::env::temp_dir().join(format!("knhk_unrdf_{}.mjs", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::write(&temp_file, script_content)
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to write script: {}", e)))?;
    
    // Execute via Node.js
    let output = Command::new("node")
        .arg(&temp_file)
        .current_dir(unrdf_path)
        .output()
        .await
        .map_err(|e| UnrdfError::QueryFailed(format!("Failed to execute node: {}", e)))?;
    
    // Cleanup
    let _ = std::fs::remove_file(&temp_file);
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(UnrdfError::QueryFailed(format!("Script failed: stderr={}, stdout={}", stderr, stdout)));
    }
    
    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| UnrdfError::QueryFailed(format!("Invalid output: {}", e)))?;
    
    Ok(stdout)
}


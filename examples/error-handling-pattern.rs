// Error Handling Pattern Example
// Demonstrates proper Result<T, E> usage in KNHK
//
// Key Concepts:
// - No unwrap() or expect() in production code
// - Descriptive error types and messages
// - Error context propagation
// - Recovery strategies
// - Telemetry for errors

use std::fmt;

// ============================================================================
// Error Types (Production-Ready)
// ============================================================================

/// Query execution errors
#[derive(Debug, Clone, PartialEq)]
pub enum QueryError {
    /// Query parsing failed
    ParseError(String),
    /// Query execution failed
    ExecutionError(String),
    /// Invalid input
    InvalidInput(String),
    /// Resource not found
    NotFound(String),
    /// Operation timeout
    Timeout(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::ParseError(msg) => write!(f, "Query parse error: {}", msg),
            QueryError::ExecutionError(msg) => write!(f, "Query execution error: {}", msg),
            QueryError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            QueryError::NotFound(msg) => write!(f, "Not found: {}", msg),
            QueryError::Timeout(msg) => write!(f, "Operation timed out: {}", msg),
        }
    }
}

impl std::error::Error for QueryError {}

/// Workflow execution errors
#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowError {
    /// Workflow not found
    NotFound(String),
    /// Invalid workflow state
    InvalidState(String),
    /// Step execution failed
    StepFailed { step: String, error: String },
    /// Query error (wraps QueryError)
    QueryError(QueryError),
}

impl fmt::Display for WorkflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkflowError::NotFound(msg) => write!(f, "Workflow not found: {}", msg),
            WorkflowError::InvalidState(msg) => write!(f, "Invalid workflow state: {}", msg),
            WorkflowError::StepFailed { step, error } => {
                write!(f, "Step '{}' failed: {}", step, error)
            }
            WorkflowError::QueryError(e) => write!(f, "Query error: {}", e),
        }
    }
}

impl std::error::Error for WorkflowError {}

// Convert QueryError → WorkflowError
impl From<QueryError> for WorkflowError {
    fn from(err: QueryError) -> Self {
        WorkflowError::QueryError(err)
    }
}

// ============================================================================
// ❌ WRONG: Bad Error Handling Patterns
// ============================================================================

mod bad_patterns {
    use super::*;

    // ❌ WRONG: Using unwrap() in production
    pub fn parse_query_unsafe(sparql: &str) -> String {
        if sparql.is_empty() {
            panic!("Empty query!"); // Crashes production!
        }
        sparql.to_uppercase() // What if this fails?
    }

    // ❌ WRONG: Silently ignoring errors
    pub fn execute_workflow_silent(workflow_id: &str) -> bool {
        let result = load_workflow(workflow_id);
        if result.is_err() {
            return false; // Lost error context!
        }
        true
    }

    // ❌ WRONG: Generic error messages
    pub fn validate_input_vague(input: &str) -> Result<(), String> {
        if input.is_empty() {
            Err("Invalid".to_string()) // What's invalid? Why?
        } else {
            Ok(())
        }
    }

    fn load_workflow(_id: &str) -> Result<(), WorkflowError> {
        Err(WorkflowError::NotFound("test".to_string()))
    }
}

// ============================================================================
// ✅ CORRECT: Proper Error Handling Patterns
// ============================================================================

/// Parse SPARQL query
/// Returns parsed AST or descriptive error
pub fn parse_query(sparql: &str) -> Result<String, QueryError> {
    // Validate input
    if sparql.is_empty() {
        return Err(QueryError::InvalidInput(
            "Query cannot be empty".to_string(),
        ));
    }

    if sparql.len() > 10_000 {
        return Err(QueryError::InvalidInput(format!(
            "Query too long: {} bytes (max 10,000)",
            sparql.len()
        )));
    }

    // Parse query (simplified for demo)
    if !sparql.to_uppercase().starts_with("ASK")
        && !sparql.to_uppercase().starts_with("SELECT")
    {
        return Err(QueryError::ParseError(format!(
            "Unsupported query type. Query must start with ASK or SELECT, got: {}",
            &sparql[..10.min(sparql.len())]
        )));
    }

    Ok(sparql.to_uppercase())
}

/// Execute parsed query
pub fn execute_query(parsed_query: &str) -> Result<bool, QueryError> {
    // Simulate execution
    if parsed_query.contains("INVALID") {
        return Err(QueryError::ExecutionError(
            "Query contains INVALID keyword".to_string(),
        ));
    }

    // Simulate timeout
    if parsed_query.len() > 1000 {
        return Err(QueryError::Timeout(format!(
            "Query execution exceeded 5s timeout (query length: {} bytes)",
            parsed_query.len()
        )));
    }

    Ok(true)
}

/// Execute query end-to-end with error context
pub fn execute_query_pipeline(sparql: &str) -> Result<bool, QueryError> {
    // Parse query (propagate error with context)
    let parsed = parse_query(sparql).map_err(|e| {
        // Add context to error
        QueryError::ParseError(format!("Failed to parse query: {}", e))
    })?;

    // Execute query (propagate error with ? operator)
    let result = execute_query(&parsed)?;

    Ok(result)
}

/// Execute workflow step with recovery
pub fn execute_workflow_step(
    workflow_id: &str,
    step_name: &str,
    query: &str,
) -> Result<bool, WorkflowError> {
    // Validate workflow exists
    if workflow_id.is_empty() {
        return Err(WorkflowError::InvalidState(
            "Workflow ID cannot be empty".to_string(),
        ));
    }

    // Execute query (QueryError auto-converts to WorkflowError via From trait)
    let result = execute_query_pipeline(query).map_err(|e| WorkflowError::StepFailed {
        step: step_name.to_string(),
        error: e.to_string(),
    })?;

    Ok(result)
}

/// Execute workflow with retry on transient errors
pub fn execute_workflow_with_retry(
    workflow_id: &str,
    step_name: &str,
    query: &str,
    max_retries: usize,
) -> Result<bool, WorkflowError> {
    let mut attempts = 0;

    loop {
        attempts += 1;

        match execute_workflow_step(workflow_id, step_name, query) {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Determine if error is retryable
                let is_retryable = matches!(
                    &e,
                    WorkflowError::QueryError(QueryError::Timeout(_))
                        | WorkflowError::StepFailed { .. }
                );

                if !is_retryable || attempts >= max_retries {
                    return Err(e); // Give up
                }

                // Log retry (in production, use tracing::warn!)
                eprintln!("Retry {}/{}: {}", attempts, max_retries, e);

                // Exponential backoff (simplified)
                std::thread::sleep(std::time::Duration::from_millis(100 * attempts as u64));
            }
        }
    }
}

// ============================================================================
// Main: Demonstrate Error Handling
// ============================================================================

fn main() {
    println!("=== Error Handling Pattern Example ===\n");

    // Example 1: Successful execution
    println!("--- Example 1: Success Case ---");
    match execute_query_pipeline("ASK { ?s ?p ?o }") {
        Ok(result) => println!("✅ Query executed successfully: {}", result),
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    // Example 2: Parse error (empty query)
    println!("--- Example 2: Parse Error (Empty Query) ---");
    match execute_query_pipeline("") {
        Ok(result) => println!("✅ Query executed: {}", result),
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("   Type: {:?}", e); // Debug output for error type
        }
    }
    println!();

    // Example 3: Parse error (unsupported query type)
    println!("--- Example 3: Parse Error (Unsupported Query) ---");
    match execute_query_pipeline("DELETE { ?s ?p ?o }") {
        Ok(result) => println!("✅ Query executed: {}", result),
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("   Type: {:?}", e);
        }
    }
    println!();

    // Example 4: Execution error
    println!("--- Example 4: Execution Error ---");
    match execute_query_pipeline("ASK { INVALID }") {
        Ok(result) => println!("✅ Query executed: {}", result),
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("   Type: {:?}", e);
        }
    }
    println!();

    // Example 5: Workflow execution with error context
    println!("--- Example 5: Workflow Step Execution ---");
    match execute_workflow_step("workflow_123", "validate_user", "ASK { ?user a :Person }") {
        Ok(result) => println!("✅ Workflow step executed: {}", result),
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("   Type: {:?}", e);
        }
    }
    println!();

    // Example 6: Workflow step failure
    println!("--- Example 6: Workflow Step Failure ---");
    match execute_workflow_step("workflow_123", "invalid_step", "") {
        Ok(result) => println!("✅ Workflow step executed: {}", result),
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("   Type: {:?}", e);
        }
    }
    println!();

    // Example 7: Retry on transient error
    println!("--- Example 7: Retry on Transient Error ---");
    // This will fail (empty query), but demonstrates retry logic
    match execute_workflow_with_retry("workflow_123", "retry_step", "", 3) {
        Ok(result) => println!("✅ Succeeded after retry: {}", result),
        Err(e) => {
            println!("❌ Failed after 3 retries: {}", e);
            println!("   Type: {:?}", e);
        }
    }
    println!();

    println!("=== Error Handling Best Practices ===");
    println!("1. ✅ Use Result<T, E> for all fallible operations");
    println!("2. ✅ Define descriptive error types (enum with context)");
    println!("3. ✅ Implement Display for user-friendly error messages");
    println!("4. ✅ Implement std::error::Error for interoperability");
    println!("5. ✅ Use ? operator for error propagation");
    println!("6. ✅ Add context when propagating errors (map_err)");
    println!("7. ✅ Implement From trait for error conversions");
    println!("8. ✅ Use pattern matching for error handling");
    println!("9. ✅ Log errors with tracing (not println)");
    println!("10. ✅ Implement recovery strategies (retry, fallback)");
    println!();

    println!("=== Anti-Patterns to Avoid ===");
    println!("1. ❌ unwrap() or expect() in production code");
    println!("2. ❌ Silently ignoring errors (if let Err(_) = ...)");
    println!("3. ❌ Generic error messages (\"Error\", \"Invalid\")");
    println!("4. ❌ Losing error context when propagating");
    println!("5. ❌ Using panic! for recoverable errors");
    println!("6. ❌ Catching errors too broadly (catches bugs)");
    println!("7. ❌ Not logging errors (lost debugging info)");
    println!("8. ❌ Exposing internal error details to users");
    println!();

    println!("=== Error Handling Checklist ===");
    println!("- [ ] All public functions return Result<T, E>");
    println!("- [ ] No unwrap() or expect() in production code");
    println!("- [ ] Descriptive error messages (what, why, how to fix)");
    println!("- [ ] Error types implement Display and std::error::Error");
    println!("- [ ] Errors logged with tracing (not println)");
    println!("- [ ] Error context preserved when propagating");
    println!("- [ ] Recovery strategies for transient errors");
    println!("- [ ] Error telemetry (spans with error status)");
}

// Key Takeaways:
//
// 1. **Result<T, E> Everywhere**: Never use unwrap() in production
//    - ✅ fn parse(s: &str) -> Result<T, E>
//    - ❌ fn parse(s: &str) -> T  // Panics on error!
//
// 2. **Descriptive Error Types**: Use enums with context
//    - ✅ QueryError::ParseError("Expected SELECT, got DELETE")
//    - ❌ Error::Generic("Invalid")  // What's invalid?
//
// 3. **Error Propagation**: Use ? operator
//    - ✅ let x = parse(s)?;
//    - ❌ let x = parse(s).unwrap();
//
// 4. **Error Context**: Add context when propagating
//    - ✅ .map_err(|e| format!("Failed to parse: {}", e))?
//    - ❌ ?  // Loses context
//
// 5. **Error Conversion**: Implement From trait
//    - ✅ impl From<QueryError> for WorkflowError
//    - Auto-conversion with ? operator
//
// 6. **Recovery Strategies**: Retry, fallback, default
//    - Transient errors: Retry with exponential backoff
//    - Permanent errors: Fail fast
//    - Optional data: Use Option or default value
//
// 7. **Error Telemetry**: Log and trace errors
//    - Span status: SpanStatus::Error
//    - Error attribute: Add error message
//    - Metrics: Count error types
//
// See also:
// - /home/user/knhk/rust/knhk-warm/src/warm_path.rs (WarmPathError)
// - /home/user/knhk/docs/reference/cards/PRODUCTION_READINESS_CHECKLIST.md

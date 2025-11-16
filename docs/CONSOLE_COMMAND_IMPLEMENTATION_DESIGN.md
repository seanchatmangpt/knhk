# Console Command Implementation Design

## Executive Summary

This document provides a comprehensive design for implementing the four console commands:
- `validate` - Validate loaded workflow using ValidationFramework
- `create-case` - Create new workflow case using CaseService
- `list-cases` - List all cases for loaded workflow
- `query` - Execute SPARQL queries against RDF stores

## Architecture Overview

### Current State Analysis

**Console Context Structure:**
```rust
struct ConsoleContext {
    workflow_path: Option<String>,      // Path to loaded Turtle file
    workflow_id: Option<String>,        // Parsed WorkflowSpecId
    state_store_path: Option<String>,   // Path to state store (default: ./workflow_db)
}
```

**Available APIs:**
- `WorkflowEngine`: Core engine with case management and RDF query capabilities
- `CaseService`: Service layer for case operations (create, list, get, etc.)
- `WorkflowService`: Service layer for workflow operations (register, get, list)
- `ValidationFramework`: Van der Aalst validation framework (fitness, precision, etc.)
- `WorkflowParser`: Turtle/RDF parser with deadlock detection

**Reference Implementation:**
- `rust/knhk-cli/src/workflow.rs` demonstrates the pattern for using service layer
- All commands use the service layer for consistency and proper error handling

## Command Implementations

### 1. `validate` Command

**Purpose:** Validate the loaded workflow using the ValidationFramework

**API Calls:**
```rust
// Get engine instance
let engine = get_engine(ctx.state_store_path.as_deref())?;

// Parse workflow_id from context
let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.unwrap())?;

// Create ValidationFramework
let framework = ValidationFramework::new(engine);

// Run complete validation (all 6 phases)
let report = framework.run_complete_validation(spec_id).await?;

// Or run specific phase:
// let report = framework.run_phase("fitness", spec_id).await?;
```

**Implementation Pattern:**
```rust
"validate" => {
    let runtime = get_runtime();
    let engine = get_engine(ctx.state_store_path.as_deref())?;

    let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())
        .map_err(|e| {
            error!(error = %e, "console.validate.parse_id_failed");
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Invalid workflow ID: {}", e
            ))
        })?;

    runtime.block_on(async {
        let framework = ValidationFramework::new(engine);
        let report = framework.run_complete_validation(spec_id).await
            .map_err(|e| {
                error!(error = %e, "console.validate.failed");
                clap_noun_verb::NounVerbError::execution_error(format!(
                    "Validation failed: {}", e
                ))
            })?;

        // Format report
        vec![
            format!("Validation Status: {:?}", report.summary.overall_status),
            format!("Passed: {} / {}", report.summary.passed_phases, report.summary.total_phases),
            format!("Failed: {}", report.summary.failed_phases),
            format!("Warnings: {}", report.summary.warnings),
        ]
    })
}
```

**Result Format:**
```
Validation Status: Pass
Passed: 6 / 6
Failed: 0
Warnings: 0
```

**Error Handling:**
- Invalid workflow_id: Parse error
- Workflow not loaded: Check ctx.workflow_id.is_none()
- Validation failure: ValidationFramework error
- Engine creation failure: StateStore error

**Telemetry:**
```rust
#[cfg(feature = "otel")]
{
    info!(
        duration_ms = duration.as_millis(),
        workflow_id = %spec_id,
        status = ?report.summary.overall_status,
        "console.validate.success"
    );
}
```

### 2. `create-case` Command

**Purpose:** Create a new workflow case for the loaded workflow

**API Calls:**
```rust
// Get engine instance
let engine = get_engine(ctx.state_store_path.as_deref())?;

// Parse workflow_id from context
let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.unwrap())?;

// Create CaseService
let service = CaseService::new(engine);

// Create case with empty or user-provided data
let request = CreateCaseRequest {
    spec_id,
    data: serde_json::json!({}), // Or parse from user input
};

let response = service.create_case(request).await?;
```

**Implementation Pattern:**
```rust
"create-case" => {
    let runtime = get_runtime();
    let engine = get_engine(ctx.state_store_path.as_deref())?;

    let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())
        .map_err(|e| {
            error!(error = %e, "console.create_case.parse_id_failed");
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Invalid workflow ID: {}", e
            ))
        })?;

    runtime.block_on(async {
        let service = CaseService::new(engine);
        let request = CreateCaseRequest {
            spec_id,
            data: serde_json::json!({"console_created": true}),
        };

        let response = service.create_case(request).await
            .map_err(|e| {
                error!(error = ?e, "console.create_case.failed");
                clap_noun_verb::NounVerbError::execution_error(
                    CliAdapter::format_error(&e)
                )
            })?;

        vec![
            format!("Case created: {}", response.case_id),
            format!("Workflow: {}", spec_id),
        ]
    })
}
```

**Result Format:**
```
Case created: 550e8400-e29b-41d4-a716-446655440000
Workflow: 123e4567-e89b-12d3-a456-426614174000
```

**Enhanced Version (with optional data parameter):**
```rust
// Allow user to provide JSON data via console
// Example: create-case --data '{"customer_id": 123}'

// Parse data from command or use default
let case_data = if let Some(data_str) = /* extract from command */ {
    serde_json::from_str(&data_str).map_err(|e| {
        error!(error = %e, "console.create_case.invalid_json");
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Invalid JSON data: {}", e
        ))
    })?
} else {
    serde_json::json!({})
};
```

**Error Handling:**
- Invalid workflow_id: Parse error
- Workflow not loaded: Check ctx.workflow_id.is_none()
- Workflow not found: WorkflowError::InvalidSpecification
- Invalid JSON data: serde_json::Error
- Service layer error: ApiError conversion

**Telemetry:**
```rust
#[cfg(feature = "otel")]
{
    info!(
        duration_ms = duration.as_millis(),
        workflow_id = %spec_id,
        case_id = %response.case_id,
        "console.create_case.success"
    );
}
```

### 3. `list-cases` Command

**Purpose:** List all workflow cases for the loaded workflow

**API Calls:**
```rust
// Get engine instance
let engine = get_engine(ctx.state_store_path.as_deref())?;

// Parse workflow_id from context
let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.unwrap())?;

// Create CaseService
let service = CaseService::new(engine);

// List cases for this workflow
let request = ListCasesRequest {
    spec_id: Some(spec_id),
};

let response = service.list_cases(request).await?;
```

**Implementation Pattern:**
```rust
"list-cases" => {
    let runtime = get_runtime();
    let engine = get_engine(ctx.state_store_path.as_deref())?;

    let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())
        .map_err(|e| {
            error!(error = %e, "console.list_cases.parse_id_failed");
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Invalid workflow ID: {}", e
            ))
        })?;

    runtime.block_on(async {
        let service = CaseService::new(engine);
        let request = ListCasesRequest {
            spec_id: Some(spec_id),
        };

        let response = service.list_cases(request).await
            .map_err(|e| {
                error!(error = ?e, "console.list_cases.failed");
                clap_noun_verb::NounVerbError::execution_error(
                    CliAdapter::format_error(&e)
                )
            })?;

        if response.cases.is_empty() {
            vec!["No cases found for this workflow.".to_string()]
        } else {
            let mut output = vec![
                format!("Cases for workflow {} ({} total):", spec_id, response.cases.len()),
            ];
            for case_id in response.cases {
                output.push(format!("  - {}", case_id));
            }
            output
        }
    })
}
```

**Result Format:**
```
Cases for workflow 123e4567-e89b-12d3-a456-426614174000 (3 total):
  - 550e8400-e29b-41d4-a716-446655440000
  - 660e8400-e29b-41d4-a716-446655440001
  - 770e8400-e29b-41d4-a716-446655440002
```

**Enhanced Version (with case status):**
```rust
// Get detailed case info for each case
let mut output = vec![
    format!("Cases for workflow {} ({} total):", spec_id, response.cases.len()),
];

for case_id in &response.cases {
    // Get case details
    let case_request = GetCaseRequest { case_id: *case_id };
    if let Ok(case_response) = service.get_case(case_request).await {
        output.push(format!(
            "  - {} (state: {:?})",
            case_id,
            case_response.case.state
        ));
    } else {
        output.push(format!("  - {}", case_id));
    }
}
```

**Error Handling:**
- Invalid workflow_id: Parse error
- Workflow not loaded: Check ctx.workflow_id.is_none()
- Service layer error: ApiError conversion

**Telemetry:**
```rust
#[cfg(feature = "otel")]
{
    info!(
        duration_ms = duration.as_millis(),
        workflow_id = %spec_id,
        case_count = response.cases.len(),
        "console.list_cases.success"
    );
}
```

### 4. `query` Command (Enhanced from console.rs placeholder)

**Purpose:** Execute SPARQL queries against loaded workflow RDF stores

**API Calls:**
```rust
// Get engine instance (already created in console.rs query function)
let engine = get_engine(ctx.state_store_path.as_deref())?;

// Parse workflow_id from context
let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.unwrap())?;

// Execute SPARQL query against workflow spec RDF store
#[cfg(feature = "rdf")]
let results = engine.query_rdf(&spec_id, &query).await?;

// Or query pattern metadata
#[cfg(feature = "rdf")]
let results = engine.query_pattern_metadata(&query).await?;

// Or query case runtime state (if case_id is provided)
#[cfg(feature = "rdf")]
let results = engine.query_case_rdf(&case_id, &query).await?;
```

**Implementation Pattern:**
```rust
// In console.rs, replace the placeholder in query() function:

#[cfg(feature = "rdf")]
{
    let engine = get_engine(ctx.state_store_path.as_deref())?;

    let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())
        .map_err(|e| {
            error!(error = %e, "console.query.parse_id_failed");
            clap_noun_verb::NounVerbError::execution_error(format!(
                "Invalid workflow ID: {}", e
            ))
        })?;

    let runtime = get_runtime();
    let query_results = runtime.block_on(async {
        engine.query_rdf(&spec_id, &query).await
    }).map_err(|e| {
        error!(error = %e, "console.query.failed");
        clap_noun_verb::NounVerbError::execution_error(format!(
            "SPARQL query failed: {}", e
        ))
    })?;

    // Format results
    let results_formatted = if query_results.is_empty() {
        "No results found.".to_string()
    } else {
        let mut output = vec![format!("Query results ({} rows):", query_results.len())];
        for (i, binding) in query_results.iter().enumerate() {
            output.push(format!("  Row {}:", i + 1));
            for (var, value) in binding {
                output.push(format!("    {}: {}", var, value));
            }
        }
        output.join("\n")
    };

    Ok(QueryResult {
        status: "success".to_string(),
        results: results_formatted,
        query,
    })
}

#[cfg(not(feature = "rdf"))]
{
    Err(clap_noun_verb::NounVerbError::execution_error(
        "RDF feature not enabled. Rebuild with --features rdf".to_string()
    ))
}
```

**Example SPARQL Queries:**

1. **List all tasks in workflow:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    ?task yawl:name ?name .
}
```

2. **List all patterns used:**
```sparql
PREFIX knhk: <http://knhk.io/pattern#>
SELECT ?pattern ?name WHERE {
    ?task knhk:pattern ?pattern .
    ?pattern knhk:name ?name .
}
```

3. **Query pattern metadata:**
```sparql
PREFIX knhk: <http://knhk.io/pattern#>
SELECT ?id ?name ?category WHERE {
    ?pattern knhk:id ?id .
    ?pattern knhk:name ?name .
    ?pattern knhk:category ?category .
}
```

**Result Format:**
```
Query results (3 rows):
  Row 1:
    task: http://example.org/workflow#task1
    name: Approve Request
  Row 2:
    task: http://example.org/workflow#task2
    name: Process Payment
  Row 3:
    task: http://example.org/workflow#task3
    name: Send Notification
```

**Error Handling:**
- Invalid workflow_id: Parse error
- Workflow not loaded: Check ctx.workflow_id.is_none()
- Invalid SPARQL query: oxigraph parse error
- RDF feature not enabled: Feature gate check
- Query execution error: WorkflowEngine error

**Telemetry:**
```rust
#[cfg(feature = "otel")]
{
    info!(
        duration_ms = duration.as_millis(),
        workflow_id = %spec_id,
        query_length = query.len(),
        result_count = query_results.len(),
        "console.query.success"
    );
}
```

## Architecture Decisions

### 1. Engine Instance Management

**Decision:** Create engine instance per command (not cached globally)

**Rationale:**
- Avoids Sync issues with LockchainStorage (see workflow.rs line 62)
- Follows existing pattern in workflow.rs
- StateStore path can change between commands
- Simpler lifecycle management

**Implementation:**
```rust
fn get_engine(state_store_path: Option<&str>) -> CnvResult<Arc<WorkflowEngine>> {
    let path = state_store_path.unwrap_or("./workflow_db");
    let state_store = StateStore::new(path).map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Failed to create state store: {}", e
        ))
    })?;
    Ok(Arc::new(WorkflowEngine::new(state_store)))
}
```

### 2. Workflow ID from Context

**Decision:** Parse workflow_id from context string to WorkflowSpecId

**Rationale:**
- Context stores workflow_id as String (for simplicity)
- APIs require WorkflowSpecId (UUID type)
- Parse at command execution time with proper error handling

**Implementation:**
```rust
let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())
    .map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(format!(
            "Invalid workflow ID: {}", e
        ))
    })?;
```

### 3. RDF Store Access

**Decision:** Use WorkflowEngine's built-in RDF query methods

**Available Methods:**
- `query_rdf(&spec_id, sparql)` - Query workflow specification store
- `query_case_rdf(&case_id, sparql)` - Query case runtime state store
- `query_pattern_metadata(sparql)` - Query pattern metadata store

**Rationale:**
- Engine manages RDF stores internally
- Proper error handling and validation
- Consistent with architecture
- No direct Store access needed

**Implementation:**
```rust
#[cfg(feature = "rdf")]
{
    let results = engine.query_rdf(&spec_id, &query).await?;
    // results: Vec<HashMap<String, String>>
}
```

### 4. Result Formatting

**Decision:** Format as human-readable multi-line strings

**Rationale:**
- Console commands are for human interaction
- Structured JSON available via workflow.rs commands
- Easier to read in terminal
- Consistent with existing console patterns

**Format Pattern:**
```rust
vec![
    "Header line".to_string(),
    "  - Item 1".to_string(),
    "  - Item 2".to_string(),
].join("\n")
```

### 5. Caching Strategy

**Decision:** No workflow spec caching in console context

**Rationale:**
- WorkflowEngine already caches specs in memory (DashMap)
- Load command re-parses for validation
- Simpler context management
- Avoid sync issues

**Current Flow:**
1. `load` command: Parse workflow, extract ID, store ID in context
2. Commands: Re-create engine, use cached spec from engine's DashMap

### 6. Error Handling Pattern

**Decision:** Use consistent error handling with telemetry

**Pattern:**
```rust
.map_err(|e| {
    #[cfg(feature = "otel")]
    error!(error = %e, "console.{command}.failed");

    clap_noun_verb::NounVerbError::execution_error(format!(
        "Operation failed: {}", e
    ))
})?
```

**Rationale:**
- Consistent with workflow.rs pattern
- Proper telemetry emission
- User-friendly error messages
- Proper error conversion chain

## Code Patterns

### 1. Telemetry Instrumentation

**Pattern from workflow.rs:**
```rust
#[cfg(feature = "otel")]
{
    use std::time::Instant;
    use tracing::{error, info};

    let start_time = Instant::now();

    // ... operation ...

    let duration = start_time.elapsed();
    info!(
        duration_ms = duration.as_millis(),
        workflow_id = %spec_id,
        "console.{command}.success"
    );
}
```

**Apply to all commands:**
- Start timer at beginning
- Log errors with `error!` macro
- Log success with `info!` macro
- Include relevant context (IDs, counts, etc.)

### 2. Service Layer Usage

**Pattern from workflow.rs:**
```rust
// Create service with engine
let service = CaseService::new(engine.clone());

// Create request
let request = CreateCaseRequest {
    spec_id,
    data,
};

// Execute via service
let response = service.create_case(request).await
    .map_err(|e| {
        clap_noun_verb::NounVerbError::execution_error(
            CliAdapter::format_error(&e)
        )
    })?;
```

**Benefits:**
- Consistent API
- Proper validation (guard constraints)
- Error conversion
- Future-proof (service layer can evolve)

### 3. Runtime Management

**Pattern from workflow.rs:**
```rust
fn get_runtime() -> &'static Runtime {
    static RUNTIME: std::sync::OnceLock<Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        Runtime::new().unwrap_or_else(|e| {
            panic!("Failed to create tokio runtime: {}", e);
        })
    })
}

// Usage:
let runtime = get_runtime();
runtime.block_on(async {
    // async operations
})
```

**Apply to console.rs:**
- Use existing get_runtime() function
- block_on for async operations
- Consistent with workflow.rs pattern

## Required Imports

### For console.rs

```rust
// Add to existing imports at top of file
#[cfg(feature = "workflow")]
use knhk_workflow_engine::{
    api::{
        models::requests::{
            CreateCaseRequest, GetCaseRequest, ListCasesRequest,
        },
        service::CaseService,
        transport::CliAdapter,
    },
    case::CaseId,
    parser::WorkflowSpecId,
    validation::ValidationFramework,
    WorkflowEngine,
};

#[cfg(feature = "workflow")]
use std::sync::Arc;
```

### Conditional Compilation

**Pattern:**
```rust
#[cfg(feature = "workflow")]
{
    // Implementation with WorkflowEngine
}

#[cfg(not(feature = "workflow"))]
{
    // Fallback or error message
    Err(clap_noun_verb::NounVerbError::execution_error(
        "Workflow feature not enabled. Rebuild with --features workflow".to_string()
    ))
}
```

## Implementation Checklist

### validate Command
- [ ] Add ValidationFramework import
- [ ] Parse spec_id from context
- [ ] Create ValidationFramework instance
- [ ] Execute run_complete_validation()
- [ ] Format validation report
- [ ] Add telemetry instrumentation
- [ ] Handle errors (invalid ID, validation failure)
- [ ] Test with sample workflow

### create-case Command
- [ ] Add CaseService and CreateCaseRequest imports
- [ ] Parse spec_id from context
- [ ] Create CaseService instance
- [ ] Execute create_case()
- [ ] Format case creation result
- [ ] Add telemetry instrumentation
- [ ] Handle errors (invalid ID, workflow not found)
- [ ] Test with sample workflow
- [ ] (Optional) Add JSON data parameter

### list-cases Command
- [ ] Add CaseService and ListCasesRequest imports
- [ ] Parse spec_id from context
- [ ] Create CaseService instance
- [ ] Execute list_cases()
- [ ] Format case list (handle empty)
- [ ] Add telemetry instrumentation
- [ ] Handle errors (invalid ID)
- [ ] Test with sample workflow
- [ ] (Optional) Add case status details

### query Command
- [ ] Add RDF query imports
- [ ] Parse spec_id from context
- [ ] Create WorkflowEngine instance
- [ ] Execute query_rdf()
- [ ] Format SPARQL results
- [ ] Add telemetry instrumentation
- [ ] Handle errors (invalid query, RDF not enabled)
- [ ] Add feature gate check
- [ ] Test with sample SPARQL queries

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_command() {
        // Create test workflow
        // Load into console context
        // Execute validate command
        // Assert validation passes
    }

    #[tokio::test]
    async fn test_create_case_command() {
        // Create test workflow
        // Load into console context
        // Execute create-case command
        // Assert case created
        // Verify case exists
    }

    #[tokio::test]
    async fn test_list_cases_command() {
        // Create test workflow
        // Create test cases
        // Execute list-cases command
        // Assert correct case count
    }

    #[tokio::test]
    #[cfg(feature = "rdf")]
    async fn test_query_command() {
        // Create test workflow
        // Execute SPARQL query
        // Assert results returned
    }
}
```

### Integration Tests
```bash
# Create sample workflow
cat > /tmp/test_workflow.ttl <<EOF
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix : <http://example.org/workflow#> .

:workflow1 a yawl:Workflow ;
    yawl:name "Test Workflow" ;
    yawl:task :task1 .

:task1 a yawl:AtomicTask ;
    yawl:name "Test Task" .
EOF

# Test console commands
knhk console start
knhk console load /tmp/test_workflow.ttl
knhk console run validate
knhk console run create-case
knhk console run list-cases
knhk console run quit
```

## Pseudo-Code Summary

### Complete Implementation

```rust
// In console.rs run() function, replace placeholders:

match command.trim() {
    "validate" => {
        #[cfg(feature = "workflow")]
        {
            let runtime = get_runtime();
            let engine = get_engine(ctx.state_store_path.as_deref())?;
            let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())?;

            runtime.block_on(async {
                let framework = ValidationFramework::new(engine);
                let report = framework.run_complete_validation(spec_id).await?;

                vec![
                    format!("Status: {:?}", report.summary.overall_status),
                    format!("Passed: {}/{}", report.summary.passed_phases, report.summary.total_phases),
                ]
            })
        }
        #[cfg(not(feature = "workflow"))]
        vec!["Workflow feature not enabled.".to_string()]
    },

    "create-case" => {
        #[cfg(feature = "workflow")]
        {
            let runtime = get_runtime();
            let engine = get_engine(ctx.state_store_path.as_deref())?;
            let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())?;

            runtime.block_on(async {
                let service = CaseService::new(engine);
                let request = CreateCaseRequest {
                    spec_id,
                    data: serde_json::json!({}),
                };
                let response = service.create_case(request).await?;

                vec![format!("Case created: {}", response.case_id)]
            })
        }
        #[cfg(not(feature = "workflow"))]
        vec!["Workflow feature not enabled.".to_string()]
    },

    "list-cases" => {
        #[cfg(feature = "workflow")]
        {
            let runtime = get_runtime();
            let engine = get_engine(ctx.state_store_path.as_deref())?;
            let spec_id = WorkflowSpecId::parse_str(&ctx.workflow_id.clone().unwrap())?;

            runtime.block_on(async {
                let service = CaseService::new(engine);
                let request = ListCasesRequest { spec_id: Some(spec_id) };
                let response = service.list_cases(request).await?;

                if response.cases.is_empty() {
                    vec!["No cases found.".to_string()]
                } else {
                    let mut output = vec![format!("Cases ({}):", response.cases.len())];
                    for case_id in response.cases {
                        output.push(format!("  - {}", case_id));
                    }
                    output
                }
            })
        }
        #[cfg(not(feature = "workflow"))]
        vec!["Workflow feature not enabled.".to_string()]
    },

    // ... other commands ...
}
```

## Performance Considerations

### Hot Path Performance (Chatman Constant)
- **Target:** ≤8 ticks for critical operations
- **Applies to:** Case creation, list operations
- **Not applicable to:** Validation (complex, multi-phase)
- **Not applicable to:** SPARQL queries (variable complexity)

### Optimizations
- Use engine's DashMap cache for specs (lock-free)
- Reuse tokio runtime (static OnceLock)
- Avoid unnecessary clones
- Stream large result sets

## Security Considerations

### SPARQL Injection Prevention
- Validate query syntax before execution
- Use oxigraph's built-in parser (safe)
- No dynamic query construction
- Read-only queries (no UPDATE/DELETE)

### Error Message Sanitization
- Don't expose internal paths
- Sanitize error messages
- Use generic messages for security errors

## Future Enhancements

### 1. Interactive Query Builder
```rust
// Provide query templates
"query-tasks" => execute_query(QUERY_ALL_TASKS),
"query-patterns" => execute_query(QUERY_ALL_PATTERNS),
```

### 2. Case Data Input
```rust
// Allow JSON data input for create-case
"create-case --data '{\"customer_id\": 123}'"
```

### 3. Validation Phase Selection
```rust
// Run specific validation phase
"validate --phase fitness"
"validate --phase precision"
```

### 4. Export Results
```rust
// Export query results to file
"query --output results.json"
```

## Conclusion

This design provides a comprehensive implementation strategy for the four console commands, following the established patterns from `workflow.rs` and integrating cleanly with the WorkflowEngine architecture. The implementation prioritizes:

1. **Consistency**: Uses service layer like workflow.rs
2. **Correctness**: Proper error handling and validation
3. **Observability**: Telemetry instrumentation throughout
4. **Maintainability**: Clear patterns and architecture decisions
5. **Performance**: Follows hot path constraints where applicable

All commands are designed to be production-ready with proper:
- Error handling
- Telemetry
- Documentation
- Testing strategy
- Security considerations

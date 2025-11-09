# ggen Integration - Use Cases

**Status**: Production-Ready

---

## Overview

RDF-driven template generation for workflow engine code, tests, and documentation.

---

## Use Case 1: Generate Workflow Specs from RDF

Generate Rust workflow specifications from RDF/Turtle ontologies.

### RDF Input (`payment-workflow.ttl`)

```turtle
@prefix ex: <http://example.org/workflow/> .
@prefix yawl: <http://yawlfoundation.org/yawlschema/> .

ex:PaymentWorkflow
    a yawl:WorkflowSpecification ;
    yawl:name "Payment Processing Workflow" ;
    yawl:hasTask ex:ValidatePayment, ex:ProcessPayment, ex:SendConfirmation .

ex:ValidatePayment
    a yawl:AtomicTask ;
    yawl:name "Validate Payment" ;
    yawl:maxTicks "8" .
```

### Usage

```rust
use knhk_workflow_engine::ggen::generate_workflow_from_rdf;

// Generate workflow spec from RDF
generate_workflow_from_rdf(
    "templates/workflow-spec.tmpl",
    "domain/payment-workflow.ttl",
    "src/generated/payment_workflow.rs",
)?;
```

### Generated Output

```rust
//! Generated Workflow: Payment Processing Workflow

use knhk_workflow_engine::parser::{WorkflowSpec, Task, TaskType};

pub fn create_payment_processing_workflow() -> WorkflowSpec {
    let mut tasks = HashMap::new();
    
    tasks.insert("ValidatePayment".to_string(), Task {
        name: "Validate Payment".to_string(),
        task_type: TaskType::Atomic,
        max_ticks: Some(8),
        // ... other fields
    });
    
    // ... more tasks
    
    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Payment Processing Workflow".to_string(),
        tasks,
        // ... other fields
    }
}
```

---

## Use Case 2: Generate Chicago TDD Tests

Generate comprehensive Chicago TDD tests from workflow specs.

### Usage

```rust
use knhk_workflow_engine::ggen::generate_tests_from_workflow;

// Generate test suite from workflow spec
generate_tests_from_workflow(
    "templates/test-suite.tmpl",
    "workflows/approval.ttl",
    "tests/generated/approval_tests.rs",
)?;
```

### Generated Output

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use knhk_workflow_engine::WorkflowEngine;
    
    #[tokio::test]
    async fn test_approval_workflow_sequence() {
        // Arrange
        let engine = create_test_engine();
        let spec = create_approval_workflow();
        
        // Act
        let case_id = engine.create_case(spec.id, json!({})).await?;
        engine.start_case(case_id).await?;
        engine.execute_case(case_id).await?;
        
        // Assert
        let case = engine.get_case(case_id).await?;
        assert_eq!(case.state, CaseState::Completed);
    }
}
```

---

## Use Case 3: Generate Documentation

Generate documentation from workflow specs.

### Usage

```rust
use knhk_workflow_engine::ggen::generate_docs_from_workflow;

// Generate documentation from workflow spec
generate_docs_from_workflow(
    "templates/workflow-docs.tmpl",
    "workflows/approval.ttl",
    "docs/generated/approval.md",
)?;
```

---

## Additional Use Cases

- **Use Case 4**: Generate API client code from RDF service definitions
- **Use Case 5**: Generate configuration files from RDF deployment specs
- **Use Case 6**: Generate database migrations from RDF schema definitions
- **Use Case 7**: Generate OpenAPI specs from RDF API definitions
- **Use Case 8**: Generate Fortune 5 configuration from RDF enterprise specs
- **Use Case 9**: Generate workflow visualization code from RDF
- **Use Case 10**: Generate CLI commands from RDF command definitions

See source code for implementation details.

---

## Template Format

Templates use Tera syntax with RDF query results:

```tera
//! Generated: {{ workflow_name }}

{% for task in tasks %}
pub fn {{ task.name | slugify }}() -> Task {
    Task {
        name: "{{ task.name }}".to_string(),
        max_ticks: Some({{ task.max_ticks }}),
    }
}
{% endfor %}
```

---

## Quick Reference

```rust
// Generate from RDF
generate_workflow_from_rdf(template, rdf_input, output_path)?;

// Generate tests
generate_tests_from_workflow(template, workflow_spec, output_path)?;

// Generate docs
generate_docs_from_workflow(template, workflow_spec, output_path)?;
```

---

**License**: MIT

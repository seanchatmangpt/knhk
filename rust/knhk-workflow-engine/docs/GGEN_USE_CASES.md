# ggen Integration - Use Cases

**Date**: 2025-01-XX  
**Status**: ✅ **USE CASES DOCUMENTED**

---

## Overview

This document provides comprehensive use cases for the ggen integration in the workflow engine, demonstrating practical applications of RDF-driven template generation.

---

## Use Case 1: Generate Workflow Specs from RDF Ontologies

### Scenario

You have a domain ontology in RDF/Turtle format that defines workflows, tasks, and their relationships. You want to generate Rust workflow specifications automatically.

### RDF Input (`domain/payment-workflow.ttl`)

```turtle
@prefix ex: <http://example.org/workflow/> .
@prefix yawl: <http://yawlfoundation.org/yawlschema/> .

ex:PaymentWorkflow
    a yawl:WorkflowSpecification ;
    yawl:name "Payment Processing Workflow" ;
    yawl:hasTask ex:ValidatePayment ;
    yawl:hasTask ex:ProcessPayment ;
    yawl:hasTask ex:SendConfirmation .

ex:ValidatePayment
    a yawl:AtomicTask ;
    yawl:name "Validate Payment" ;
    yawl:taskType "Atomic" ;
    yawl:maxTicks "8" ;
    yawl:hasOutgoingFlow ex:Flow1 .

ex:ProcessPayment
    a yawl:AtomicTask ;
    yawl:name "Process Payment" ;
    yawl:taskType "Atomic" ;
    yawl:maxTicks "100" ;
    yawl:hasIncomingFlow ex:Flow1 ;
    yawl:hasOutgoingFlow ex:Flow2 .

ex:SendConfirmation
    a yawl:AtomicTask ;
    yawl:name "Send Confirmation" ;
    yawl:taskType "Atomic" ;
    yawl:maxTicks "50" ;
    yawl:hasIncomingFlow ex:Flow2 .
```

### Template (`templates/workflow-spec.tmpl`)

```tera
//! Generated Workflow: {{ workflow_name }}
//! Generated from RDF ontology

use knhk_workflow_engine::parser::{WorkflowSpec, WorkflowSpecId, Task, TaskType};
use std::collections::HashMap;

pub fn create_{{ workflow_name | slugify }}_workflow() -> WorkflowSpec {
    let mut tasks = HashMap::new();
    
    {% for task in tasks %}
    tasks.insert("{{ task.id }}".to_string(), Task {
        name: "{{ task.name }}".to_string(),
        task_type: TaskType::{{ task.task_type }},
        max_ticks: Some({{ task.max_ticks }}),
        // ... other fields
    });
    {% endfor %}
    
    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "{{ workflow_name }}".to_string(),
        tasks,
        // ... other fields
    }
}
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

### Generated Output (`src/generated/payment_workflow.rs`)

```rust
//! Generated Workflow: Payment Processing Workflow
//! Generated from RDF ontology

use knhk_workflow_engine::parser::{WorkflowSpec, WorkflowSpecId, Task, TaskType};
use std::collections::HashMap;

pub fn create_payment_processing_workflow() -> WorkflowSpec {
    let mut tasks = HashMap::new();
    
    tasks.insert("ValidatePayment".to_string(), Task {
        name: "Validate Payment".to_string(),
        task_type: TaskType::Atomic,
        max_ticks: Some(8),
        // ... other fields
    });
    
    tasks.insert("ProcessPayment".to_string(), Task {
        name: "Process Payment".to_string(),
        task_type: TaskType::Atomic,
        max_ticks: Some(100),
        // ... other fields
    });
    
    tasks.insert("SendConfirmation".to_string(), Task {
        name: "Send Confirmation".to_string(),
        task_type: TaskType::Atomic,
        max_ticks: Some(50),
        // ... other fields
    });
    
    WorkflowSpec {
        id: WorkflowSpecId::new(),
        name: "Payment Processing Workflow".to_string(),
        tasks,
        // ... other fields
    }
}
```

---

## Use Case 2: Generate Chicago TDD Tests from Workflow Specs

### Scenario

You have a workflow spec and want to automatically generate comprehensive Chicago TDD tests for it.

### Template (`templates/test-suite.tmpl`)

```tera
//! Generated Chicago TDD Tests for {{ workflow_name }}
//! Generated from workflow specification

use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;
use knhk_workflow_engine::case::CaseState;
use chicago_tdd_tools::builders::TestDataBuilder;

#[tokio::test]
async fn test_{{ workflow_name | slugify }}_registration() {
    // Arrange: Set up test fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    // Act: Register workflow
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Assert: Workflow is registered
    assert!(fixture.specs.contains_key(&spec_id));
}

#[tokio::test]
async fn test_{{ workflow_name | slugify }}_case_creation() {
    // Arrange: Set up test fixture and register workflow
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Act: Create case
    let data = TestDataBuilder::new().build_json();
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    
    // Assert: Case is created
    assert!(fixture.cases.contains(&case_id));
}

#[tokio::test]
async fn test_{{ workflow_name | slugify }}_execution() {
    // Arrange: Set up test fixture and create case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let data = TestDataBuilder::new().build_json();
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    
    // Act: Execute case
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Case completes successfully
    fixture.assert_case_completed(&case);
}

{% for task in tasks %}
#[tokio::test]
async fn test_{{ workflow_name | slugify }}_{{ task.id | slugify }}() {
    // Arrange: Set up test fixture and create case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let data = TestDataBuilder::new()
        .with_var("task_id", "{{ task.id }}")
        .build_json();
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    
    // Act: Execute case (task {{ task.name }} will be executed)
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Case completes successfully
    fixture.assert_case_completed(&case);
}
{% endfor %}
```

### Usage

```rust
use knhk_workflow_engine::ggen::generate_tests_from_spec;
use knhk_workflow_engine::parser::WorkflowSpec;

let spec = /* ... workflow spec ... */;

// Generate tests from workflow spec
generate_tests_from_spec(
    &spec,
    "templates/test-suite.tmpl",
    "tests/generated/payment_workflow_tests.rs",
)?;
```

### Generated Output (`tests/generated/payment_workflow_tests.rs`)

```rust
//! Generated Chicago TDD Tests for Payment Processing Workflow
//! Generated from workflow specification

use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;
use knhk_workflow_engine::case::CaseState;
use chicago_tdd_tools::builders::TestDataBuilder;

#[tokio::test]
async fn test_payment_processing_workflow_registration() {
    // Arrange: Set up test fixture
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    // Act: Register workflow
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Assert: Workflow is registered
    assert!(fixture.specs.contains_key(&spec_id));
}

#[tokio::test]
async fn test_payment_processing_workflow_validate_payment() {
    // Arrange: Set up test fixture and create case
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    let data = TestDataBuilder::new()
        .with_var("task_id", "ValidatePayment")
        .build_json();
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    
    // Act: Execute case (task Validate Payment will be executed)
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert: Case completes successfully
    fixture.assert_case_completed(&case);
}

// ... more tests for each task ...
```

---

## Use Case 3: Generate Documentation from Workflow Specs

### Scenario

You want to automatically generate Markdown documentation for your workflows.

### Template (`templates/documentation.tmpl`)

```tera
# {{ workflow_name }}

**Workflow ID**: `{{ workflow_id }}`  
**Generated**: {{ "now" | date(format="%Y-%m-%d") }}

## Overview

This workflow contains {{ task_count }} tasks and implements a {{ workflow_category }} pattern.

## Tasks

{% for task in tasks %}
### {{ task.name }} (`{{ task.id }}`)

- **Type**: {{ task.task_type }}
- **ID**: `{{ task.id }}`
- **Max Ticks**: {{ task.max_ticks | default(value="N/A") }}

{% endfor %}

## Usage

```rust
use knhk_workflow_engine::{WorkflowEngine, StateStore};

let state_store = StateStore::new("./workflow_db")?;
let engine = WorkflowEngine::new(state_store);
let spec = create_{{ workflow_name | slugify }}_workflow();
let spec_id = engine.register_workflow(spec).await?;
```

## Testing

See `tests/generated/{{ workflow_name | slugify }}_tests.rs` for generated test suite.
```

### Usage

```rust
use knhk_workflow_engine::ggen::generate_documentation_from_spec;

let spec = /* ... workflow spec ... */;

// Generate documentation from workflow spec
generate_documentation_from_spec(
    &spec,
    "templates/documentation.tmpl",
    "docs/generated/payment_workflow.md",
)?;
```

### Generated Output (`docs/generated/payment_workflow.md`)

```markdown
# Payment Processing Workflow

**Workflow ID**: `550e8400-e29b-41d4-a716-446655440000`  
**Generated**: 2025-01-XX

## Overview

This workflow contains 3 tasks and implements a payment processing pattern.

## Tasks

### Validate Payment (`ValidatePayment`)

- **Type**: Atomic
- **ID**: `ValidatePayment`
- **Max Ticks**: 8

### Process Payment (`ProcessPayment`)

- **Type**: Atomic
- **ID**: `ProcessPayment`
- **Max Ticks**: 100

### Send Confirmation (`SendConfirmation`)

- **Type**: Atomic
- **ID**: `SendConfirmation`
- **Max Ticks**: 50

## Usage

```rust
use knhk_workflow_engine::{WorkflowEngine, StateStore};

let state_store = StateStore::new("./workflow_db")?;
let engine = WorkflowEngine::new(state_store);
let spec = create_payment_processing_workflow();
let spec_id = engine.register_workflow(spec).await?;
```

## Testing

See `tests/generated/payment_processing_workflow_tests.rs` for generated test suite.
```

---

## Use Case 4: Generate API Client Code from RDF Service Definitions

### Scenario

You have service definitions in RDF and want to generate REST API client code.

### RDF Input (`domain/api-services.ttl`)

```turtle
@prefix ex: <http://example.org/api/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:WorkflowService
    a ex:Service ;
    ex:name "Workflow Service" ;
    ex:baseUrl "https://api.example.com/workflows" ;
    ex:hasEndpoint ex:CreateWorkflowEndpoint ;
    ex:hasEndpoint ex:GetWorkflowEndpoint .

ex:CreateWorkflowEndpoint
    a ex:Endpoint ;
    ex:method "POST" ;
    ex:path "/workflows" ;
    ex:requestBody ex:WorkflowRequest ;
    ex:responseBody ex:WorkflowResponse .

ex:GetWorkflowEndpoint
    a ex:Endpoint ;
    ex:method "GET" ;
    ex:path "/workflows/{id}" ;
    ex:responseBody ex:WorkflowResponse .
```

### Template (`templates/api-client.tmpl`)

```tera
//! Generated API Client for {{ service_name }}
//! Generated from RDF service definitions

use reqwest::Client;
use serde::{Deserialize, Serialize};

pub struct {{ service_name | pascal_case }}Client {
    client: Client,
    base_url: String,
}

impl {{ service_name | pascal_case }}Client {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
    
    {% for endpoint in endpoints %}
    pub async fn {{ endpoint.name | snake_case }}(&self{% if endpoint.has_params %}, params: {{ endpoint.name | pascal_case }}Params{% endif %}) -> Result<{{ endpoint.response_type }}, String> {
        let url = format!("{}{}", self.base_url, "{{ endpoint.path }}");
        let response = self.client
            .{{ endpoint.method | lower }}("{{ endpoint.path }}")
            {% if endpoint.has_body %}
            .json(&params)
            {% endif %}
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;
        
        response.json().await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }
    {% endfor %}
}
```

### Usage

```rust
use knhk_workflow_engine::ggen::GgenGenerator;
use std::collections::HashMap;

let mut generator = GgenGenerator::new("templates")?;
generator.load_rdf("domain/api-services.ttl")?;

// Extract service data via SPARQL (when fully implemented)
let context = HashMap::new();
let generated = generator.generate_workflow_spec("api-client.tmpl", context)?;
```

---

## Use Case 5: Generate Configuration Files from RDF Deployment Specs

### Scenario

You have deployment configurations in RDF and want to generate Kubernetes YAML or Docker Compose files.

### RDF Input (`domain/deployment.ttl`)

```turtle
@prefix ex: <http://example.org/deployment/> .
@prefix k8s: <http://kubernetes.io/schema/> .

ex:WorkflowEngineDeployment
    a k8s:Deployment ;
    k8s:name "workflow-engine" ;
    k8s:replicas "3" ;
    k8s:image "knhk/workflow-engine:latest" ;
    k8s:port "8080" ;
    k8s:hasEnvVar ex:DatabaseUrl ;
    k8s:hasEnvVar ex:OtelEndpoint .

ex:DatabaseUrl
    a k8s:EnvVar ;
    k8s:name "DATABASE_URL" ;
    k8s:value "postgresql://localhost:5432/workflows" .

ex:OtelEndpoint
    a k8s:EnvVar ;
    k8s:name "OTEL_ENDPOINT" ;
    k8s:value "http://otel-collector:4317" .
```

### Template (`templates/kubernetes-deployment.tmpl`)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: {{ deployment_name }}
spec:
  replicas: {{ replicas }}
  selector:
    matchLabels:
      app: {{ deployment_name }}
  template:
    metadata:
      labels:
        app: {{ deployment_name }}
    spec:
      containers:
      - name: {{ deployment_name }}
        image: {{ image }}
        ports:
        - containerPort: {{ port }}
        env:
        {% for env_var in env_vars %}
        - name: {{ env_var.name }}
          value: "{{ env_var.value }}"
        {% endfor %}
```

### Usage

```rust
use knhk_workflow_engine::ggen::GgenGenerator;
use std::collections::HashMap;

let mut generator = GgenGenerator::new("templates")?;
generator.load_rdf("domain/deployment.ttl")?;

let context = HashMap::new();
let generated = generator.generate_workflow_spec("kubernetes-deployment.tmpl", context)?;

std::fs::write("k8s/deployment.yaml", generated)?;
```

---

## Use Case 6: Generate Database Migrations from RDF Schema Definitions

### Scenario

You have database schemas defined in RDF and want to generate SQL migration scripts.

### RDF Input (`domain/schema.ttl`)

```turtle
@prefix ex: <http://example.org/schema/> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

ex:WorkflowTable
    a ex:Table ;
    ex:name "workflows" ;
    ex:hasColumn ex:WorkflowIdColumn ;
    ex:hasColumn ex:WorkflowNameColumn ;
    ex:hasColumn ex:WorkflowStateColumn .

ex:WorkflowIdColumn
    a ex:Column ;
    ex:name "id" ;
    ex:type "UUID" ;
    ex:primaryKey true ;
    ex:nullable false .

ex:WorkflowNameColumn
    a ex:Column ;
    ex:name "name" ;
    ex:type "VARCHAR(255)" ;
    ex:nullable false .

ex:WorkflowStateColumn
    a ex:Column ;
    ex:name "state" ;
    ex:type "VARCHAR(50)" ;
    ex:nullable false .
```

### Template (`templates/sql-migration.tmpl`)

```sql
-- Generated migration for {{ table_name }}
-- Generated from RDF schema definition

CREATE TABLE {{ table_name }} (
    {% for column in columns %}
    {{ column.name }} {{ column.type }}{% if column.primary_key %} PRIMARY KEY{% endif %}{% if not column.nullable %} NOT NULL{% endif %}{% if not loop.last %},{% endif %}
    {% endfor %}
);

{% if has_indexes %}
{% for index in indexes %}
CREATE INDEX {{ index.name }} ON {{ table_name }}({{ index.columns | join(", ") }});
{% endfor %}
{% endif %}
```

### Usage

```rust
use knhk_workflow_engine::ggen::GgenGenerator;
use std::collections::HashMap;

let mut generator = GgenGenerator::new("templates")?;
generator.load_rdf("domain/schema.ttl")?;

let context = HashMap::new();
let generated = generator.generate_workflow_spec("sql-migration.tmpl", context)?;

std::fs::write("migrations/001_create_workflows.sql", generated)?;
```

---

## Use Case 7: Generate OpenAPI Specs from RDF API Definitions

### Scenario

You have API definitions in RDF and want to generate OpenAPI specifications.

### RDF Input (`domain/openapi.ttl`)

```turtle
@prefix ex: <http://example.org/api/> .
@prefix openapi: <http://openapis.org/schema/> .

ex:WorkflowAPI
    a openapi:API ;
    openapi:title "Workflow Engine API" ;
    openapi:version "1.0.0" ;
    openapi:hasPath ex:CreateWorkflowPath ;
    openapi:hasPath ex:GetWorkflowPath .

ex:CreateWorkflowPath
    a openapi:Path ;
    openapi:path "/workflows" ;
    openapi:method "POST" ;
    openapi:requestBody ex:WorkflowRequestBody ;
    openapi:response ex:WorkflowResponse .
```

### Template (`templates/openapi.tmpl`)

```yaml
openapi: 3.0.0
info:
  title: {{ api_title }}
  version: {{ api_version }}
paths:
  {% for path in paths %}
  {{ path.path }}:
    {{ path.method | lower }}:
      summary: {{ path.summary }}
      requestBody:
        content:
          application/json:
            schema:
              $ref: "#/components/schemas/{{ path.request_body }}"
      responses:
        '200':
          description: Success
          content:
            application/json:
              schema:
                $ref: "#/components/schemas/{{ path.response }}"
  {% endfor %}
```

### Usage

```rust
use knhk_workflow_engine::ggen::GgenGenerator;
use std::collections::HashMap;

let mut generator = GgenGenerator::new("templates")?;
generator.load_rdf("domain/openapi.ttl")?;

let context = HashMap::new();
let generated = generator.generate_workflow_spec("openapi.tmpl", context)?;

std::fs::write("openapi.yaml", generated)?;
```

---

## Use Case 8: Generate Fortune 5 Configuration from RDF Enterprise Specs

### Scenario

You have enterprise deployment requirements in RDF and want to generate Fortune 5 configuration files.

### RDF Input (`domain/fortune5.ttl`)

```turtle
@prefix ex: <http://example.org/fortune5/> .
@prefix fortune5: <http://fortune5.org/schema/> .

ex:ProductionConfig
    a fortune5:Fortune5Config ;
    fortune5:hasSLO ex:SLOR1 ;
    fortune5:hasSLO ex:SLOW1 ;
    fortune5:hasSLO ex:SLOC1 ;
    fortune5:hasPromotion ex:PromotionConfig ;
    fortune5:hasKMS ex:KMSConfig .

ex:SLOR1
    a fortune5:SLO ;
    fortune5:runtimeClass "R1" ;
    fortune5:p99MaxNs "2" .

ex:SLOW1
    a fortune5:SLO ;
    fortune5:runtimeClass "W1" ;
    fortune5:p99MaxMs "1" .

ex:SLOC1
    a fortune5:SLO ;
    fortune5:runtimeClass "C1" ;
    fortune5:p99MaxMs "500" .

ex:PromotionConfig
    a fortune5:PromotionConfig ;
    fortune5:environment "Production" ;
    fortune5:sloThreshold "0.99" ;
    fortune5:autoRollbackEnabled true .
```

### Template (`templates/fortune5-config.tmpl`)

```toml
# Generated Fortune 5 Configuration
# Generated from RDF enterprise specs

[fortune5]
{% if has_slo %}
[slo]
{% for slo in slos %}
{{ slo.runtime_class | lower }}_p99_max{% if slo.runtime_class == "R1" %}_ns{% else %}_ms{% endif %} = {{ slo.p99_max }}
{% endfor %}
window_size_seconds = 60
{% endif %}

{% if has_promotion %}
[promotion]
environment = "{{ promotion.environment }}"
slo_threshold = {{ promotion.slo_threshold }}
auto_rollback_enabled = {{ promotion.auto_rollback_enabled | lower }}
rollback_window_seconds = 300
{% endif %}

{% if has_kms %}
[kms]
provider = "{{ kms.provider }}"
rotation_interval_hours = {{ kms.rotation_interval_hours }}
{% endif %}
```

### Usage

```rust
use knhk_workflow_engine::ggen::GgenGenerator;
use std::collections::HashMap;

let mut generator = GgenGenerator::new("templates")?;
generator.load_rdf("domain/fortune5.ttl")?;

let context = HashMap::new();
let generated = generator.generate_workflow_spec("fortune5-config.tmpl", context)?;

std::fs::write("config/fortune5.toml", generated)?;
```

---

## Use Case 9: Generate Workflow Visualization Code from RDF

### Scenario

You want to generate interactive HTML visualizations for workflows from RDF definitions.

### Template (`templates/workflow-viz.tmpl`)

```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ workflow_name }} - Visualization</title>
    <script src="https://d3js.org/d3.v7.min.js"></script>
</head>
<body>
    <h1>{{ workflow_name }}</h1>
    <svg id="workflow-diagram" width="800" height="600"></svg>
    
    <script>
        const workflow = {
            name: "{{ workflow_name }}",
            tasks: [
                {% for task in tasks %}
                {
                    id: "{{ task.id }}",
                    name: "{{ task.name }}",
                    type: "{{ task.task_type }}"
                }{% if not loop.last %},{% endif %}
                {% endfor %}
            ]
        };
        
        // D3.js visualization code
        const svg = d3.select("#workflow-diagram");
        // ... visualization logic ...
    </script>
</body>
</html>
```

### Usage

```rust
use knhk_workflow_engine::ggen::generate_documentation_from_spec;

let spec = /* ... workflow spec ... */;

generate_documentation_from_spec(
    &spec,
    "templates/workflow-viz.tmpl",
    "docs/visualizations/payment_workflow.html",
)?;
```

---

## Use Case 10: Generate CLI Commands from RDF Command Definitions

### Scenario

You have CLI command definitions in RDF and want to generate Rust CLI code using clap-noun-verb.

### RDF Input (`domain/commands.ttl`)

```turtle
@prefix ex: <http://example.org/commands/> .
@prefix cli: <http://cli.org/schema/> .

ex:WorkflowCommand
    a cli:Noun ;
    cli:name "workflow" ;
    cli:hasVerb ex:CreateVerb ;
    cli:hasVerb ex:ListVerb .

ex:CreateVerb
    a cli:Verb ;
    cli:name "create" ;
    cli:hasArg ex:NameArg ;
    cli:hasArg ex:SpecArg .

ex:NameArg
    a cli:Argument ;
    cli:name "name" ;
    cli:type "String" ;
    cli:required true .

ex:SpecArg
    a cli:Argument ;
    cli:name "spec" ;
    cli:type "Path" ;
    cli:required true .
```

### Template (`templates/cli-commands.tmpl`)

```rust
//! Generated CLI Commands
//! Generated from RDF command definitions

use clap_noun_verb_macros::verb;
use clap_noun_verb::Result;

{% for noun in nouns %}
{% for verb in noun.verbs %}
#[verb("{{ verb.name }}", "{{ noun.name }}")]
pub fn {{ noun.name }}_{{ verb.name }}({% for arg in verb.args %}{{ arg.name }}: {{ arg.type }}{% if not loop.last %}, {% endif %}{% endfor %}) -> Result<()> {
    // Generated command implementation
    Ok(())
}
{% endfor %}
{% endfor %}
```

### Usage

```rust
use knhk_workflow_engine::ggen::GgenGenerator;
use std::collections::HashMap;

let mut generator = GgenGenerator::new("templates")?;
generator.load_rdf("domain/commands.ttl")?;

let context = HashMap::new();
let generated = generator.generate_workflow_spec("cli-commands.tmpl", context)?;

std::fs::write("src/commands/generated.rs", generated)?;
```

---

## Summary

These use cases demonstrate the power of ggen integration:

1. **Workflow Spec Generation**: From RDF ontologies to Rust code
2. **Test Generation**: Automatic Chicago TDD test suites
3. **Documentation Generation**: Markdown docs from specs
4. **API Client Generation**: REST clients from service definitions
5. **Configuration Generation**: K8s, Docker, Fortune 5 configs
6. **Database Migration Generation**: SQL from schema definitions
7. **OpenAPI Generation**: API specs from RDF definitions
8. **Fortune 5 Config Generation**: Enterprise configs from RDF
9. **Visualization Generation**: Interactive HTML diagrams
10. **CLI Generation**: Command-line interfaces from RDF

**Key Benefits**:
- ✅ Single source of truth (RDF files)
- ✅ Templates are reusable across projects
- ✅ No hardcoded data in templates
- ✅ Automatic code generation
- ✅ Consistent output format
- ✅ Easy to maintain and update

---

**Last Updated**: 2025-01-XX  
**Status**: ✅ **USE CASES DOCUMENTED**


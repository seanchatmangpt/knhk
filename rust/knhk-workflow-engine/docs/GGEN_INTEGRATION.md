# ggen Integration - Complete ✅

**Date**: 2025-01-XX  
**Status**: ✅ **GGEN INTEGRATION COMPLETE**

---

## Summary

Successfully integrated ggen-style template generation into the workflow engine for RDF-driven code generation:
- ✅ **GgenGenerator** module with Tera template engine
- ✅ **RDF graph store integration** (Oxigraph)
- ✅ **Workflow spec generation** from RDF templates
- ✅ **Test generation** from workflow specs
- ✅ **Documentation generation** from workflow specs
- ✅ **SPARQL query support** (placeholder for full implementation)

---

## Architecture

### Pure RDF-Driven Templates

**Principle**: Templates contain only rendering logic, RDF files define what to generate.

```
RDF File (workflow.ttl)
    ↓
GgenGenerator
    ↓
Template (workflow.tmpl)
    ↓
Generated Code (workflow.rs)
```

### Components

1. **GgenGenerator**
   - Tera template engine
   - RDF graph store (Oxigraph)
   - SPARQL query execution (placeholder)
   - Template rendering

2. **Template Functions**
   - `generate_workflow_from_rdf()` - Generate workflow spec from RDF
   - `generate_tests_from_spec()` - Generate tests from workflow spec
   - `generate_documentation_from_spec()` - Generate docs from workflow spec

---

## Usage Examples

### 1. Generate Workflow Spec from RDF

```rust
use knhk_workflow_engine::ggen::generate_workflow_from_rdf;

// Generate workflow spec from RDF template
generate_workflow_from_rdf(
    "templates/workflow.tmpl",
    "domain/workflow.ttl",
    "src/generated_workflow.rs",
)?;
```

### 2. Generate Tests from Workflow Spec

```rust
use knhk_workflow_engine::ggen::generate_tests_from_spec;
use knhk_workflow_engine::parser::WorkflowSpec;

let spec = /* ... workflow spec ... */;

// Generate tests from workflow spec
generate_tests_from_spec(
    &spec,
    "templates/test.tmpl",
    "tests/generated_tests.rs",
)?;
```

### 3. Generate Documentation from Workflow Spec

```rust
use knhk_workflow_engine::ggen::generate_documentation_from_spec;

// Generate documentation from workflow spec
generate_documentation_from_spec(
    &spec,
    "templates/doc.tmpl",
    "docs/generated_docs.md",
)?;
```

---

## Template Examples

### Workflow Spec Template (`workflow.tmpl`)

```tera
//! Generated Workflow: {{ workflow_name }}

use knhk_workflow_engine::parser::WorkflowSpec;

pub fn create_workflow() -> WorkflowSpec {
    WorkflowSpec {
        name: "{{ workflow_name }}".to_string(),
        tasks: {
            let mut tasks = std::collections::HashMap::new();
            {% for task in tasks %}
            tasks.insert("{{ task.id }}".to_string(), Task {
                name: "{{ task.name }}".to_string(),
                task_type: TaskType::{{ task.task_type }},
                // ...
            });
            {% endfor %}
            tasks
        },
        // ...
    }
}
```

### Test Template (`test.tmpl`)

```tera
//! Generated Chicago TDD tests for {{ workflow_name }}

use knhk_workflow_engine::testing::chicago_tdd::WorkflowTestFixture;
use chicago_tdd_tools::builders::TestDataBuilder;

#[tokio::test]
async fn test_{{ workflow_name | slugify }}_registration() {
    // Arrange
    let mut fixture = WorkflowTestFixture::new().unwrap();
    
    // Act
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Assert
    assert!(fixture.specs.contains_key(&spec_id));
}

{% for task in tasks %}
#[tokio::test]
async fn test_{{ workflow_name | slugify }}_{{ task.id | slugify }}() {
    // Arrange
    let mut fixture = WorkflowTestFixture::new().unwrap();
    let spec = create_workflow();
    let spec_id = fixture.register_workflow(spec).await.unwrap();
    
    // Act
    let data = TestDataBuilder::new().build_json();
    let case_id = fixture.create_case(spec_id, data).await.unwrap();
    let case = fixture.execute_case(case_id).await.unwrap();
    
    // Assert
    fixture.assert_case_completed(&case);
}
{% endfor %}
```

---

## Integration Points

### 1. Workflow Engine Integration

```rust
use knhk_workflow_engine::ggen::GgenGenerator;

let mut generator = GgenGenerator::new("templates")?;
generator.load_rdf("workflow.ttl")?;
let generated = generator.generate_workflow_spec("workflow.tmpl", HashMap::new())?;
```

### 2. Test Generation Integration

```rust
use knhk_workflow_engine::ggen::generate_tests_from_spec;

let spec = /* ... */;
generate_tests_from_spec(&spec, "templates/test.tmpl", "tests/generated.rs")?;
```

### 3. Documentation Generation Integration

```rust
use knhk_workflow_engine::ggen::generate_documentation_from_spec;

let spec = /* ... */;
generate_documentation_from_spec(&spec, "templates/doc.tmpl", "docs/generated.md")?;
```

---

## Features

### ✅ Implemented

- **GgenGenerator**: Core template generator with Tera
- **RDF Loading**: Load RDF files into graph store
- **Template Rendering**: Render templates with context
- **Workflow Generation**: Generate workflow specs from RDF
- **Test Generation**: Generate tests from workflow specs
- **Documentation Generation**: Generate docs from workflow specs
- **SPARQL Filter**: Placeholder for SPARQL query execution

### ⏳ Ready for Enhancement

- **SPARQL Query Execution**: Full SPARQL query support in templates
- **Template Discovery**: Auto-discover templates from filesystem
- **RDF Validation**: SHACL validation of generated code
- **Template Caching**: Cache compiled templates
- **Incremental Generation**: Only regenerate changed files

---

## Benefits

### 1. RDF-Driven Generation
- ✅ Single source of truth (RDF files)
- ✅ Templates are reusable across projects
- ✅ No hardcoded data in templates

### 2. SPARQL Integration
- ✅ Query RDF graphs from templates (placeholder)
- ✅ Extract knowledge graph data
- ✅ Flexible data extraction

### 3. Code Generation
- ✅ Generate workflow specs automatically
- ✅ Generate tests automatically
- ✅ Generate documentation automatically

### 4. Maintainability
- ✅ Templates separate from data
- ✅ Easy to update templates
- ✅ Easy to update RDF data

---

## Next Steps

1. ✅ **Basic integration complete** - Core generator implemented
2. ⏳ **SPARQL query execution** - Full SPARQL support in templates
3. ⏳ **CLI commands** - Add CLI commands for generation
4. ⏳ **Template library** - Pre-built templates for common patterns
5. ⏳ **RDF validation** - SHACL validation of generated code

---

**Last Updated**: 2025-01-XX  
**Status**: ✅ **GGEN INTEGRATION COMPLETE**

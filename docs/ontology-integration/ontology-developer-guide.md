# YAWL Ontology Developer Guide for knhk

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Production Ready
**Target Audience:** Rust developers integrating YAWL workflows into knhk

---

## Executive Summary

This guide teaches Rust developers how to work with the YAWL 4.0 ontology in knhk. You'll learn OWL/RDF concepts, how to read `yawl.ttl`, common patterns, extension strategies, and best practices for ontology-driven workflow engineering.

**What You'll Learn:**
- How to read and understand OWL/RDF ontologies from a Rust perspective
- YAWL ontology structure and patterns
- How to extend the ontology for knhk-specific requirements
- Integration with Rust code using Oxigraph
- Performance optimization for ontology operations
- Maintenance and versioning best practices

**Prerequisites:**
- Rust programming experience
- Basic understanding of workflow concepts
- Familiarity with knhk architecture

---

## Table of Contents

1. [Introduction: Ontologies for Rust Developers](#1-introduction-ontologies-for-rust-developers)
2. [Understanding yawl.ttl Structure](#2-understanding-yawlttl-structure)
3. [OWL/RDF Concepts for Rust Developers](#3-owlrdf-concepts-for-rust-developers)
4. [Common YAWL Ontology Patterns](#4-common-yawl-ontology-patterns)
5. [Reading and Parsing the Ontology](#5-reading-and-parsing-the-ontology)
6. [Extending the Ontology for knhk](#6-extending-the-ontology-for-knhk)
7. [Integration with Rust Code](#7-integration-with-rust-code)
8. [Performance Optimization](#8-performance-optimization)
9. [Best Practices for Ontology Maintenance](#9-best-practices-for-ontology-maintenance)
10. [Troubleshooting Common Issues](#10-troubleshooting-common-issues)

---

## 1. Introduction: Ontologies for Rust Developers

### 1.1 What Is an Ontology?

**For Rust Developers:** Think of an ontology as a **type system + schema + documentation** all in one.

```rust
// Traditional Rust approach:
struct Task {
    id: String,
    name: String,
    join_type: JoinType,
    split_type: SplitType,
}

enum JoinType {
    And,
    Or,
    Xor,
}

// Ontology approach (RDF/OWL):
// - Defines types (classes) with inheritance
// - Defines properties with domains and ranges
// - Self-documenting with rdfs:label and rdfs:comment
// - Machine-readable and human-readable
// - Extensible without breaking existing code
```

**Key Differences:**

| Aspect | Rust Structs/Enums | Ontology (OWL/RDF) |
|--------|-------------------|-------------------|
| **Type System** | Compile-time, static | Runtime, flexible |
| **Inheritance** | Traits, no multiple inheritance | Multiple inheritance via rdfs:subClassOf |
| **Documentation** | Comments, doc strings | Built-in rdfs:comment, rdfs:label |
| **Extensibility** | Requires recompilation | Can extend at runtime |
| **Validation** | Compiler | SHACL, SPARQL, reasoners |
| **Interoperability** | Serde, JSON | RDF (universal format) |

### 1.2 Why YAWL Uses an Ontology

YAWL workflows are **complex, long-lived, and evolving**. An ontology provides:

1. **Formal Semantics:** Unambiguous definition of workflow concepts
2. **Interoperability:** Standard RDF format for tool integration
3. **Extensibility:** Add knhk-specific annotations without modifying YAWL schema
4. **Validation:** SPARQL queries validate workflow correctness
5. **Reasoning:** Infer implicit facts (e.g., task reachability)
6. **Documentation:** Self-documenting schema

### 1.3 How knhk Uses the Ontology

```
┌─────────────────────────────────────────────────────────┐
│                   knhk Architecture                      │
├─────────────────────────────────────────────────────────┤
│ YAWL XML Files (.yawl)                                  │
│         ↓                                                │
│ Parser (knhk-workflow-engine/src/parser/)               │
│         ↓                                                │
│ RDF Graph (Oxigraph in-memory store)                    │
│         ↓                                                │
│ SPARQL Queries (validation, extraction, analysis)       │
│         ↓                                                │
│ Rust Workflow Engine (executes tasks)                   │
└─────────────────────────────────────────────────────────┘
```

**Ontology Role:**
- **Design-time:** Validate workflow structure
- **Runtime:** Query task dependencies, resource allocations
- **Monitoring:** Track execution state via SPARQL
- **Provenance:** Record workflow history in RDF

---

## 2. Understanding yawl.ttl Structure

### 2.1 File Organization

The `yawl.ttl` file is structured as follows:

```turtle
# File: /Users/sac/knhk/ontology/yawl.ttl

# 1. Namespace Declarations (Lines 1-5)
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# 2. Enumeration Classes (Lines 15-192)
#    - ControlType (AND/OR/XOR)
#    - CreationMode (Static/Dynamic)
#    - TimerInterval (YEAR/MONTH/WEEK/DAY/HOUR/MIN/SEC/MSEC)
#    - etc.

# 3. Core Workflow Classes (Lines 197-263)
#    - SpecificationSet, Specification
#    - Decomposition, Net, Task, Condition
#    - Variables, Parameters

# 4. Resource Allocation Classes (Lines 278-317)
#    - Resourcing, ResourcingOffer, ResourcingAllocate
#    - ResourcingSet, ResourcingSelector

# 5. Configuration Classes (Lines 320-408)
#    - Timer, TimerDuration
#    - VarMapping, Expression, Predicate
#    - Configuration, JoinConfig, SplitConfig

# 6. Layout Classes (Lines 415-498)
#    - Layout, LayoutNet, LayoutVertex
#    - Visual representation metadata

# 7. Datatype Properties (Lines 504-1006)
#    - name, id, uri, documentation
#    - Task-specific: minimum, maximum, threshold
#    - Metadata: creator, version, status

# 8. Object Properties (Lines 1012-1557)
#    - hasSpecification, hasDecomposition
#    - hasTask, hasCondition, flowsInto
#    - hasResourcing, hasTimer, hasJoin, hasSplit
```

### 2.2 Anatomy of a Class Definition

Let's break down a typical class definition:

```turtle
# Example: Task class definition
yawl:Task a rdfs:Class ;
    rdfs:label "Task" ;
    rdfs:comment "Workflow task" ;
    rdfs:subClassOf yawl:NetElement .
```

**Rust Mental Model:**

```rust
/// Workflow task
#[derive(Debug, Clone)]
struct Task {
    // Inherits from NetElement
}

impl NetElement for Task {
    // Task is a subtype of NetElement
}
```

**RDF Explanation:**

| RDF Triple | Meaning |
|------------|---------|
| `yawl:Task a rdfs:Class` | Task is a class (type definition) |
| `rdfs:label "Task"` | Human-readable name |
| `rdfs:comment "Workflow task"` | Documentation string |
| `rdfs:subClassOf yawl:NetElement` | Task inherits from NetElement |

### 2.3 Anatomy of a Property Definition

```turtle
# Example: Datatype property
yawl:name a rdf:Property ;
    rdfs:label "name" ;
    rdfs:comment "Name of the element" ;
    rdfs:domain rdfs:Resource ;
    rdfs:range xsd:string .
```

**Rust Mental Model:**

```rust
trait HasName {
    fn name(&self) -> &str;
}

impl<T: Resource> HasName for T {
    // Any Resource can have a name
}
```

**RDF Explanation:**

| RDF Triple | Meaning |
|------------|---------|
| `yawl:name a rdf:Property` | Defines a property |
| `rdfs:domain rdfs:Resource` | Can be applied to any Resource |
| `rdfs:range xsd:string` | Property value must be a string |

### 2.4 Anatomy of an Enumeration

```turtle
# Enumeration: ControlType
yawl:ControlType a rdfs:Class ;
    rdfs:label "ControlType" ;
    rdfs:comment "Control flow type: AND, OR, or XOR" .

yawl:ControlTypeAnd a yawl:ControlType ;
    rdfs:label "AND" ;
    rdfs:comment "AND join/split" .

yawl:ControlTypeOr a yawl:ControlType ;
    rdfs:label "OR" ;
    rdfs:comment "OR join/split" .

yawl:ControlTypeXor a yawl:ControlType ;
    rdfs:label "XOR" ;
    rdfs:comment "XOR join/split" .
```

**Rust Mental Model:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlType {
    And,
    Or,
    Xor,
}

impl ControlType {
    fn label(&self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
        }
    }
}
```

**RDF Explanation:**
- `ControlType` is the parent class (enum type)
- `ControlTypeAnd`, `ControlTypeOr`, `ControlTypeXor` are instances (enum variants)
- Each instance is an individual of the parent class

---

## 3. OWL/RDF Concepts for Rust Developers

### 3.1 RDF Triples: The Building Blocks

**RDF Statement = (Subject, Predicate, Object)**

```turtle
yawl:Task rdfs:subClassOf yawl:NetElement .
```

**Breakdown:**
- **Subject:** `yawl:Task` (what we're talking about)
- **Predicate:** `rdfs:subClassOf` (relationship)
- **Object:** `yawl:NetElement` (value)

**Rust Analogy:**

```rust
// RDF: Task subClassOf NetElement
impl NetElement for Task {}

// RDF: Task hasJoin ControlTypeAnd
struct Task {
    join: ControlType,
}
```

### 3.2 Namespaces and IRIs

**Namespace = Module in Rust**

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
```

**Rust Equivalent:**

```rust
mod yawl {
    pub struct Task { }
    pub enum ControlType { }
}

use yawl::Task;
```

**Full IRI:**

```
yawl:Task → http://www.yawlfoundation.org/yawlschema#Task
```

### 3.3 Classes vs. Instances

```turtle
# Class definition (type)
yawl:Task a rdfs:Class .

# Instance (value)
<http://example.org/workflow#Task1> a yawl:Task .
```

**Rust Analogy:**

```rust
// Class (type)
struct Task;

// Instance (value)
let task1 = Task;
```

### 3.4 Properties: Datatype vs. Object

**Datatype Property:** Links to a literal value

```turtle
?task yawl:name "Approve Request"^^xsd:string .
```

```rust
struct Task {
    name: String,
}
```

**Object Property:** Links to another resource

```turtle
?task yawl:hasJoin yawl:ControlTypeAnd .
```

```rust
struct Task {
    join: ControlType,
}
```

### 3.5 Cardinality and Constraints

**OWL Cardinality (Missing from yawl.ttl):**

```turtle
# NOT in current yawl.ttl, but should be:
yawl:Task a owl:Class ;
    rdfs:subClassOf [
        a owl:Restriction ;
        owl:onProperty yawl:hasJoin ;
        owl:cardinality 1
    ] .
```

**Meaning:** Every Task must have exactly 1 join type.

**Rust Equivalent:**

```rust
struct Task {
    join: ControlType, // Required (not Option<ControlType>)
}
```

### 3.6 Inheritance and Subclassing

```turtle
yawl:NetElement a rdfs:Class .
yawl:Task rdfs:subClassOf yawl:NetElement .
yawl:Condition rdfs:subClassOf yawl:NetElement .
```

**Rust Equivalent:**

```rust
trait NetElement {
    fn id(&self) -> &str;
}

struct Task {
    id: String,
}

impl NetElement for Task {
    fn id(&self) -> &str {
        &self.id
    }
}

struct Condition {
    id: String,
}

impl NetElement for Condition {
    fn id(&self) -> &str {
        &self.id
    }
}
```

---

## 4. Common YAWL Ontology Patterns

### 4.1 Pattern: Enumeration via Instances

**Pattern:**

```turtle
ParentClass a rdfs:Class .
Value1 a ParentClass .
Value2 a ParentClass .
Value3 a ParentClass .
```

**Example:**

```turtle
yawl:ControlType a rdfs:Class .
yawl:ControlTypeAnd a yawl:ControlType .
yawl:ControlTypeOr a yawl:ControlType .
yawl:ControlTypeXor a yawl:ControlType .
```

**When to Use:**
- Closed set of values (enum)
- Values have additional properties (e.g., documentation)

**Rust Mapping:**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ControlType {
    And,
    Or,
    Xor,
}

impl ControlType {
    fn from_iri(iri: &str) -> Option<Self> {
        match iri {
            "http://www.yawlfoundation.org/yawlschema#ControlTypeAnd" => Some(Self::And),
            "http://www.yawlfoundation.org/yawlschema#ControlTypeOr" => Some(Self::Or),
            "http://www.yawlfoundation.org/yawlschema#ControlTypeXor" => Some(Self::Xor),
            _ => None,
        }
    }
}
```

### 4.2 Pattern: Container with Ordered Elements

**Pattern:**

```turtle
?container a ContainerClass .
?container hasElement ?element1 .
?container hasElement ?element2 .
?element1 ordering 1 .
?element2 ordering 2 .
```

**Example:**

```turtle
?predicate a yawl:Predicate ;
    yawl:query "count(//data) > 5" ;
    yawl:ordering 1 .
```

**When to Use:**
- Multiple values with ordering (not just sets)

**Rust Mapping:**

```rust
struct Predicate {
    query: String,
    ordering: i32,
}

struct FlowsInto {
    predicates: Vec<Predicate>, // Sorted by ordering
}
```

### 4.3 Pattern: Decomposition Hierarchy

**Pattern:**

```turtle
ParentClass a rdfs:Class .
ChildClass1 rdfs:subClassOf ParentClass .
ChildClass2 rdfs:subClassOf ParentClass .
```

**Example:**

```turtle
yawl:Decomposition a rdfs:Class .
yawl:Net rdfs:subClassOf yawl:Decomposition .
yawl:WebServiceGateway rdfs:subClassOf yawl:Decomposition .
```

**When to Use:**
- Abstract base class with concrete implementations
- Polymorphism

**Rust Mapping:**

```rust
trait Decomposition {
    fn id(&self) -> &str;
}

struct Net {
    id: String,
    tasks: Vec<Task>,
}

impl Decomposition for Net {
    fn id(&self) -> &str {
        &self.id
    }
}

struct WebServiceGateway {
    id: String,
    wsdl_location: String,
}

impl Decomposition for WebServiceGateway {
    fn id(&self) -> &str {
        &self.id
    }
}

enum DecompositionType {
    Net(Net),
    WebServiceGateway(WebServiceGateway),
}
```

### 4.4 Pattern: Composition with Object Properties

**Pattern:**

```turtle
?parent hasChild ?child1 .
?parent hasChild ?child2 .
```

**Example:**

```turtle
?net yawl:hasTask ?task1 .
?net yawl:hasTask ?task2 .
?net yawl:hasCondition ?condition1 .
```

**When to Use:**
- Parent-child relationships
- Containment

**Rust Mapping:**

```rust
struct Net {
    tasks: Vec<Task>,
    conditions: Vec<Condition>,
}
```

### 4.5 Pattern: Flow/Edge Representation

**Pattern:**

```turtle
?element1 flowsInto ?flow .
?flow nextElementRef ?element2 .
?flow hasPredicate ?predicate .
```

**Example:**

```turtle
?task1 yawl:flowsInto ?flow .
?flow yawl:nextElementRef ?task2 .
?flow yawl:hasPredicate ?pred .
```

**Why Not Direct Edge?**

Instead of:
```turtle
?task1 yawl:flowsDirectlyTo ?task2 . # No way to add predicate
```

Use intermediate node:
```turtle
?task1 yawl:flowsInto ?flow .
?flow yawl:nextElementRef ?task2 .
?flow yawl:hasPredicate ?pred . # Can add attributes to edge
```

**Rust Mapping:**

```rust
struct FlowsInto {
    next_element: String, // Reference to NetElement
    predicate: Option<Predicate>,
    is_default: bool,
}

struct NetElement {
    id: String,
    flows: Vec<FlowsInto>,
}
```

### 4.6 Pattern: Configuration via Optional Properties

**Pattern:**

```turtle
?task a yawl:Task .
OPTIONAL { ?task yawl:hasTimer ?timer }
OPTIONAL { ?task yawl:hasResourcing ?resourcing }
```

**When to Use:**
- Optional features
- Extensibility

**Rust Mapping:**

```rust
struct Task {
    id: String,
    timer: Option<Timer>,
    resourcing: Option<Resourcing>,
}
```

---

## 5. Reading and Parsing the Ontology

### 5.1 Loading the Ontology in Rust

```rust
use oxigraph::store::Store;
use oxigraph::io::GraphFormat;
use std::io::BufReader;
use std::fs::File;

fn load_yawl_ontology() -> Result<Store, Box<dyn std::error::Error>> {
    let store = Store::new()?;

    let file = File::open("/Users/sac/knhk/ontology/yawl.ttl")?;
    let reader = BufReader::new(file);

    store.load_graph(
        reader,
        GraphFormat::Turtle,
        oxigraph::model::GraphNameRef::DefaultGraph,
        None, // base IRI
    )?;

    Ok(store)
}
```

### 5.2 Querying Class Definitions

```rust
use oxigraph::sparql::QueryResults;

fn get_all_classes(store: &Store) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let query = r#"
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?class ?label WHERE {
            ?class a rdfs:Class .
            OPTIONAL { ?class rdfs:label ?label }
        }
    "#;

    let mut classes = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            if let Some(class) = solution.get("class") {
                classes.push(class.to_string());
            }
        }
    }

    Ok(classes)
}
```

### 5.3 Extracting Property Definitions

```rust
fn get_properties_for_class(
    store: &Store,
    class_iri: &str,
) -> Result<Vec<PropertyInfo>, Box<dyn std::error::Error>> {
    let query = format!(r#"
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
        PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>

        SELECT ?property ?label ?range WHERE {{
            ?property a rdf:Property .
            ?property rdfs:domain <{class_iri}> .
            OPTIONAL {{ ?property rdfs:label ?label }}
            OPTIONAL {{ ?property rdfs:range ?range }}
        }}
    "#);

    let mut properties = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(&query)? {
        for solution in solutions {
            let solution = solution?;
            properties.push(PropertyInfo {
                iri: solution.get("property").unwrap().to_string(),
                label: solution.get("label").map(|l| l.to_string()),
                range: solution.get("range").map(|r| r.to_string()),
            });
        }
    }

    Ok(properties)
}

#[derive(Debug, Clone)]
struct PropertyInfo {
    iri: String,
    label: Option<String>,
    range: Option<String>,
}
```

### 5.4 Understanding Class Hierarchies

```rust
fn get_subclasses(
    store: &Store,
    parent_class: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let query = format!(r#"
        PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

        SELECT ?subclass WHERE {{
            ?subclass rdfs:subClassOf <{parent_class}> .
        }}
    "#);

    let mut subclasses = Vec::new();

    if let QueryResults::Solutions(solutions) = store.query(&query)? {
        for solution in solutions {
            let solution = solution?;
            if let Some(subclass) = solution.get("subclass") {
                subclasses.push(subclass.to_string());
            }
        }
    }

    Ok(subclasses)
}
```

### 5.5 Generating Rust Types from Ontology

**Strategy: Code Generation**

```rust
// In build.rs or separate tool
fn generate_rust_types_from_ontology() {
    let store = load_yawl_ontology().unwrap();
    let classes = get_all_classes(&store).unwrap();

    for class in classes {
        let properties = get_properties_for_class(&store, &class).unwrap();
        generate_rust_struct(&class, &properties);
    }
}

fn generate_rust_struct(class: &str, properties: &[PropertyInfo]) {
    let class_name = extract_local_name(class);

    println!("pub struct {} {{", class_name);

    for prop in properties {
        let field_name = extract_local_name(&prop.iri);
        let field_type = map_range_to_rust_type(&prop.range);
        println!("    pub {}: {},", field_name, field_type);
    }

    println!("}}");
}

fn map_range_to_rust_type(range: &Option<String>) -> String {
    match range.as_deref() {
        Some("http://www.w3.org/2001/XMLSchema#string") => "String".to_string(),
        Some("http://www.w3.org/2001/XMLSchema#integer") => "i64".to_string(),
        Some("http://www.w3.org/2001/XMLSchema#boolean") => "bool".to_string(),
        Some(iri) => format!("Box<{}>", extract_local_name(iri)),
        None => "String".to_string(),
    }
}
```

---

## 6. Extending the Ontology for knhk

### 6.1 Extension Strategy: Separate Namespace

**Best Practice:** Don't modify `yawl.ttl`. Create `knhk-extensions.ttl`.

```turtle
# File: /Users/sac/knhk/ontology/knhk-extensions.ttl

@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix knhk: <http://knhk.org/ontology#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .

# ===========================================================================
# knhk Extensions to YAWL Ontology
# ===========================================================================

# Hot Path Annotation
knhk:HotPathTask a rdfs:Class ;
    rdfs:subClassOf yawl:Task ;
    rdfs:label "Hot Path Task" ;
    rdfs:comment "Task on critical path with tight performance constraints (≤8 ticks)" .

knhk:tickBudget a rdf:Property ;
    rdfs:label "tickBudget" ;
    rdfs:comment "Maximum execution time in ticks (Chatman Constant = 8)" ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:integer .

# Lockchain Provenance
knhk:hasProvenanceChain a rdf:Property ;
    rdfs:label "hasProvenanceChain" ;
    rdfs:comment "Git commit hash for workflow version provenance" ;
    rdfs:domain yawl:Specification ;
    rdfs:range xsd:string .

# OTEL Instrumentation
knhk:otelSpan a rdf:Property ;
    rdfs:label "otelSpan" ;
    rdfs:comment "OpenTelemetry span name for this task" ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string .

knhk:otelMetric a rdf:Property ;
    rdfs:label "otelMetric" ;
    rdfs:comment "OpenTelemetry metric name" ;
    rdfs:domain yawl:Task ;
    rdfs:range xsd:string .

# Security Policies
knhk:SecurityPolicy a rdfs:Class ;
    rdfs:label "Security Policy" ;
    rdfs:comment "Security policy for task execution" .

knhk:hasSecurityPolicy a rdf:Property ;
    rdfs:label "hasSecurityPolicy" ;
    rdfs:comment "Security policy reference" ;
    rdfs:domain yawl:Task ;
    rdfs:range knhk:SecurityPolicy .

# Runtime State (for monitoring)
knhk:WorkflowInstance a rdfs:Class ;
    rdfs:label "Workflow Instance" ;
    rdfs:comment "Runtime instance of a workflow specification" .

knhk:hasSpecification a rdf:Property ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range yawl:Specification .

knhk:hasState a rdf:Property ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:string .

knhk:startedAt a rdf:Property ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range xsd:dateTime .

knhk:TaskExecution a rdfs:Class ;
    rdfs:label "Task Execution" ;
    rdfs:comment "Runtime execution of a task" .

knhk:hasTaskExecution a rdf:Property ;
    rdfs:domain knhk:WorkflowInstance ;
    rdfs:range knhk:TaskExecution .

knhk:hasTask a rdf:Property ;
    rdfs:domain knhk:TaskExecution ;
    rdfs:range yawl:Task .
```

### 6.2 Loading Multiple Ontologies

```rust
fn load_knhk_ontologies() -> Result<Store, Box<dyn std::error::Error>> {
    let store = Store::new()?;

    // Load base YAWL ontology
    let yawl_file = File::open("/Users/sac/knhk/ontology/yawl.ttl")?;
    store.load_graph(
        BufReader::new(yawl_file),
        GraphFormat::Turtle,
        oxigraph::model::GraphNameRef::DefaultGraph,
        None,
    )?;

    // Load knhk extensions
    let knhk_file = File::open("/Users/sac/knhk/ontology/knhk-extensions.ttl")?;
    store.load_graph(
        BufReader::new(knhk_file),
        GraphFormat::Turtle,
        oxigraph::model::GraphNameRef::DefaultGraph,
        None,
    )?;

    Ok(store)
}
```

### 6.3 Extension Use Cases

**1. Hot Path Annotation:**

```turtle
<http://example.org/workflow#AuthenticateUser> a knhk:HotPathTask ;
    yawl:name "Authenticate User" ;
    knhk:tickBudget 8 ;
    knhk:otelSpan "workflow.authenticate_user" .
```

```rust
struct HotPathTask {
    base: Task,
    tick_budget: i32,
    otel_span: String,
}
```

**2. Provenance Tracking:**

```turtle
<http://example.org/workflow#OrderProcessing> a yawl:Specification ;
    yawl:name "Order Processing" ;
    knhk:hasProvenanceChain "abc123def456" .
```

**3. Runtime Monitoring:**

```turtle
<http://example.org/instance#Instance123> a knhk:WorkflowInstance ;
    knhk:hasSpecification <http://example.org/workflow#OrderProcessing> ;
    knhk:hasState "running" ;
    knhk:startedAt "2025-11-08T12:00:00Z"^^xsd:dateTime .
```

### 6.4 Validation with SHACL

**Create `knhk-validation.ttl` for constraints:**

```turtle
@prefix sh: <http://www.w3.org/ns/shacl#> .
@prefix knhk: <http://knhk.org/ontology#> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

# Hot path tasks must have tick budget ≤ 8
knhk:HotPathTaskShape a sh:NodeShape ;
    sh:targetClass knhk:HotPathTask ;
    sh:property [
        sh:path knhk:tickBudget ;
        sh:minInclusive 1 ;
        sh:maxInclusive 8 ;
        sh:message "Hot path task must have tick budget ≤ 8" ;
    ] .

# All tasks must have OTEL span
knhk:TaskShape a sh:NodeShape ;
    sh:targetClass yawl:Task ;
    sh:property [
        sh:path knhk:otelSpan ;
        sh:minCount 1 ;
        sh:message "All tasks must have OTEL span annotation" ;
    ] .
```

---

## 7. Integration with Rust Code

### 7.1 Workflow Parsing Pipeline

```rust
use oxigraph::store::Store;
use oxigraph::model::{NamedNode, Triple};

pub struct WorkflowParser {
    store: Store,
}

impl WorkflowParser {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let store = load_knhk_ontologies()?;
        Ok(Self { store })
    }

    pub fn parse_yawl_file(&self, path: &str) -> Result<Workflow, Box<dyn std::error::Error>> {
        // 1. Parse YAWL XML
        let xml_doc = parse_xml(path)?;

        // 2. Convert to RDF triples
        let triples = xml_to_rdf(&xml_doc)?;

        // 3. Insert into store
        for triple in triples {
            self.store.insert(&triple)?;
        }

        // 4. Extract workflow via SPARQL
        self.extract_workflow()
    }

    fn extract_workflow(&self) -> Result<Workflow, Box<dyn std::error::Error>> {
        // Use SPARQL queries to extract workflow structure
        let tasks = self.extract_tasks()?;
        let conditions = self.extract_conditions()?;
        let flows = self.extract_flows()?;

        Ok(Workflow {
            tasks,
            conditions,
            flows,
        })
    }
}
```

### 7.2 Task Extraction

```rust
impl WorkflowParser {
    fn extract_tasks(&self) -> Result<Vec<Task>, Box<dyn std::error::Error>> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
            PREFIX knhk: <http://knhk.org/ontology#>

            SELECT ?task ?name ?join ?split ?tickBudget WHERE {
                ?task a yawl:Task .
                ?task yawl:name ?name .
                ?task yawl:hasJoin ?join .
                ?task yawl:hasSplit ?split .
                OPTIONAL { ?task knhk:tickBudget ?tickBudget }
            }
        "#;

        let mut tasks = Vec::new();

        if let QueryResults::Solutions(solutions) = self.store.query(query)? {
            for solution in solutions {
                let solution = solution?;
                tasks.push(Task {
                    id: solution.get("task").unwrap().to_string(),
                    name: solution.get("name").unwrap().to_string(),
                    join: ControlType::from_iri(solution.get("join").unwrap().as_str())?,
                    split: ControlType::from_iri(solution.get("split").unwrap().as_str())?,
                    tick_budget: solution.get("tickBudget").map(|v| v.as_str().parse().unwrap()),
                });
            }
        }

        Ok(tasks)
    }
}
```

### 7.3 Validation with SPARQL

```rust
impl WorkflowParser {
    pub fn validate(&self) -> Result<Vec<ValidationError>, Box<dyn std::error::Error>> {
        let mut errors = Vec::new();

        // Check: Start condition has no incoming flows
        if self.check_start_condition_incoming_flows()? {
            errors.push(ValidationError::StartHasIncomingFlow);
        }

        // Check: End condition has no outgoing flows
        if self.check_end_condition_outgoing_flows()? {
            errors.push(ValidationError::EndHasOutgoingFlow);
        }

        // Check: All tasks have join and split
        errors.extend(self.check_tasks_have_join_split()?);

        Ok(errors)
    }

    fn check_start_condition_incoming_flows(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let query = r#"
            PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

            ASK {
                ?condition a yawl:InputCondition .
                ?flow yawl:nextElementRef ?condition .
            }
        "#;

        if let QueryResults::Boolean(result) = self.store.query(query)? {
            Ok(result)
        } else {
            Err("Invalid query result".into())
        }
    }
}

#[derive(Debug, Clone)]
enum ValidationError {
    StartHasIncomingFlow,
    EndHasOutgoingFlow,
    TaskMissingJoin(String),
    TaskMissingSplit(String),
}
```

---

## 8. Performance Optimization

### 8.1 Query Optimization

**Bad (Expensive):**

```sparql
# Retrieves all tasks, then filters
SELECT ?task WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin yawl:ControlTypeAnd .
}
```

**Good (Filter Early):**

```sparql
# Filters during traversal
SELECT ?task WHERE {
    ?task yawl:hasJoin yawl:ControlTypeAnd .
    ?task a yawl:Task .
}
```

### 8.2 Caching Strategies

```rust
use std::sync::Arc;
use std::collections::HashMap;

pub struct CachedWorkflowStore {
    store: Store,
    class_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl CachedWorkflowStore {
    pub fn get_instances_of_class(&self, class: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Check cache
        {
            let cache = self.class_cache.read().unwrap();
            if let Some(instances) = cache.get(class) {
                return Ok(instances.clone());
            }
        }

        // Cache miss: query store
        let instances = self.query_instances(class)?;

        // Update cache
        {
            let mut cache = self.class_cache.write().unwrap();
            cache.insert(class.to_string(), instances.clone());
        }

        Ok(instances)
    }
}
```

### 8.3 Indexing

**Oxigraph automatically indexes:**
- SPO (Subject-Predicate-Object)
- POS (Predicate-Object-Subject)
- OSP (Object-Subject-Predicate)

**Optimize Queries for Indices:**

```sparql
# Good: Uses SPO index
SELECT ?o WHERE {
    <http://example.org/task1> yawl:hasJoin ?o .
}

# Bad: Requires full scan
SELECT ?s WHERE {
    ?s ?p ?o .
    FILTER(CONTAINS(STR(?s), "task"))
}
```

### 8.4 Batch Operations

```rust
impl WorkflowParser {
    pub fn insert_triples_batch(&self, triples: Vec<Triple>) -> Result<(), Box<dyn std::error::Error>> {
        // Use transaction for batch insert
        self.store.transaction(|transaction| {
            for triple in &triples {
                transaction.insert(triple)?;
            }
            Ok(())
        })?;

        Ok(())
    }
}
```

---

## 9. Best Practices for Ontology Maintenance

### 9.1 Versioning Strategy

**Approach: Semantic Versioning for Ontology**

```turtle
yawl:Ontology a owl:Ontology ;
    owl:versionInfo "4.0.0" ;
    rdfs:comment "MAJOR.MINOR.PATCH - Breaking.Feature.Fix" .
```

**Version Bump Rules:**
- **MAJOR:** Breaking changes (remove class, change property domain/range)
- **MINOR:** New classes, properties (backward compatible)
- **PATCH:** Documentation, typos, clarifications

### 9.2 Change Log

**File: `ontology/CHANGELOG.md`**

```markdown
# YAWL Ontology Changelog

## [4.0.0] - 2025-11-08

### Added
- knhk extensions for hot path annotation
- Runtime state tracking classes

### Changed
- None

### Deprecated
- None

### Removed
- None
```

### 9.3 Testing Ontology Changes

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ontology_loads() {
        let store = load_knhk_ontologies().unwrap();
        assert!(store.len() > 0);
    }

    #[test]
    fn test_all_classes_have_labels() {
        let store = load_knhk_ontologies().unwrap();
        let query = r#"
            PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

            SELECT ?class WHERE {
                ?class a rdfs:Class .
                FILTER NOT EXISTS { ?class rdfs:label ?label }
            }
        "#;

        if let QueryResults::Solutions(solutions) = store.query(query).unwrap() {
            let results: Vec<_> = solutions.collect();
            assert_eq!(results.len(), 0, "All classes must have labels");
        }
    }
}
```

### 9.4 Documentation Standards

**Every Class Must Have:**
- `rdfs:label` (short name)
- `rdfs:comment` (description)

**Every Property Must Have:**
- `rdfs:label`
- `rdfs:comment`
- `rdfs:domain` (what it applies to)
- `rdfs:range` (what values it takes)

### 9.5 Migration Guide

**When updating ontology, provide migration script:**

```rust
pub fn migrate_v3_to_v4(store: &Store) -> Result<(), Box<dyn std::error::Error>> {
    // Example: Rename property
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

        DELETE {
            ?s yawl:oldProperty ?o .
        }
        INSERT {
            ?s yawl:newProperty ?o .
        }
        WHERE {
            ?s yawl:oldProperty ?o .
        }
    "#;

    store.update(query)?;
    Ok(())
}
```

---

## 10. Troubleshooting Common Issues

### 10.1 Issue: Namespace Resolution Errors

**Symptom:**

```
Error: Unknown prefix 'yawl'
```

**Cause:** Missing `@prefix` declaration

**Fix:**

```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
```

### 10.2 Issue: Type Mismatches

**Symptom:**

```
Error: Expected xsd:integer, got xsd:string
```

**Cause:** Incorrect literal type

**Fix:**

```turtle
# Wrong
?task knhk:tickBudget "8" .

# Correct
?task knhk:tickBudget 8 .
# or
?task knhk:tickBudget "8"^^xsd:integer .
```

### 10.3 Issue: Property Domain/Range Violations

**Symptom:**

```
Warning: Property applied to incompatible domain
```

**Cause:** Using property on wrong class

**Fix:**

```turtle
# Wrong
?condition yawl:hasJoin yawl:ControlTypeAnd . # Condition doesn't have join

# Correct
?task yawl:hasJoin yawl:ControlTypeAnd . # Task has join
```

### 10.4 Issue: Query Returns Empty Results

**Debugging Steps:**

1. **Check if data exists:**

```sparql
SELECT * WHERE { ?s ?p ?o } LIMIT 10
```

2. **Simplify query:**

```sparql
# Start simple
SELECT ?task WHERE {
    ?task a yawl:Task .
}

# Add complexity incrementally
SELECT ?task WHERE {
    ?task a yawl:Task .
    ?task yawl:hasJoin ?join .
}
```

3. **Check namespaces:**

```sparql
# Verify namespace
SELECT ?class WHERE {
    ?class a rdfs:Class .
    FILTER(CONTAINS(STR(?class), "Task"))
}
```

### 10.5 Issue: Performance Degradation

**Symptoms:**
- Queries taking >100ms
- High memory usage

**Solutions:**

1. **Add indices (if using custom store)**
2. **Cache frequent queries**
3. **Use LIMIT in queries**
4. **Avoid property paths (`+`, `*`) in hot path**

**Profiling:**

```rust
use std::time::Instant;

fn profile_query(store: &Store, query: &str) {
    let start = Instant::now();
    let results = store.query(query).unwrap();
    let duration = start.elapsed();

    println!("Query took: {:?}", duration);
}
```

---

## Appendix A: Complete Example

### A.1 End-to-End Workflow

```rust
// File: examples/ontology_example.rs

use oxigraph::store::Store;
use oxigraph::io::GraphFormat;
use oxigraph::sparql::QueryResults;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Load ontologies
    let store = Store::new()?;

    let yawl_file = File::open("ontology/yawl.ttl")?;
    store.load_graph(
        BufReader::new(yawl_file),
        GraphFormat::Turtle,
        oxigraph::model::GraphNameRef::DefaultGraph,
        None,
    )?;

    let knhk_file = File::open("ontology/knhk-extensions.ttl")?;
    store.load_graph(
        BufReader::new(knhk_file),
        GraphFormat::Turtle,
        oxigraph::model::GraphNameRef::DefaultGraph,
        None,
    )?;

    // 2. Load workflow data
    let workflow_file = File::open("examples/sample_workflow.ttl")?;
    store.load_graph(
        BufReader::new(workflow_file),
        GraphFormat::Turtle,
        oxigraph::model::GraphNameRef::DefaultGraph,
        None,
    )?;

    // 3. Query workflow
    let query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
        PREFIX knhk: <http://knhk.org/ontology#>

        SELECT ?task ?name ?tickBudget WHERE {
            ?task a knhk:HotPathTask .
            ?task yawl:name ?name .
            ?task knhk:tickBudget ?tickBudget .
        }
        ORDER BY ?tickBudget
    "#;

    println!("Hot Path Tasks:");
    if let QueryResults::Solutions(solutions) = store.query(query)? {
        for solution in solutions {
            let solution = solution?;
            let name = solution.get("name").unwrap();
            let budget = solution.get("tickBudget").unwrap();
            println!("  {} (budget: {})", name, budget);
        }
    }

    // 4. Validate workflow
    let validation_query = r#"
        PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

        ASK {
            ?condition a yawl:InputCondition .
            ?flow yawl:nextElementRef ?condition .
        }
    "#;

    if let QueryResults::Boolean(has_error) = store.query(validation_query)? {
        if has_error {
            println!("ERROR: Start condition has incoming flows!");
        } else {
            println!("OK: Start condition valid");
        }
    }

    Ok(())
}
```

---

## Appendix B: Quick Reference

### B.1 Common Namespaces

| Prefix | Namespace IRI | Purpose |
|--------|---------------|---------|
| `yawl:` | `http://www.yawlfoundation.org/yawlschema#` | YAWL classes/properties |
| `knhk:` | `http://knhk.org/ontology#` | knhk extensions |
| `rdfs:` | `http://www.w3.org/2000/01/rdf-schema#` | RDF Schema |
| `owl:` | `http://www.w3.org/2002/07/owl#` | OWL ontology language |
| `xsd:` | `http://www.w3.org/2001/XMLSchema#` | XML Schema datatypes |
| `rdf:` | `http://www.w3.org/1999/02/22-rdf-syntax-ns#` | RDF core |

### B.2 Common SPARQL Patterns

```sparql
# Get all instances of a class
SELECT ?x WHERE {
    ?x a yawl:Task .
}

# Get property value
SELECT ?value WHERE {
    <http://example.org/task1> yawl:name ?value .
}

# Check if property exists
SELECT ?task WHERE {
    ?task a yawl:Task .
    FILTER EXISTS { ?task yawl:hasTimer ?timer }
}

# Count instances
SELECT (COUNT(?task) AS ?count) WHERE {
    ?task a yawl:Task .
}
```

### B.3 Oxigraph API Cheat Sheet

```rust
// Load ontology
store.load_graph(reader, GraphFormat::Turtle, GraphNameRef::DefaultGraph, None)?;

// Query (SELECT)
if let QueryResults::Solutions(solutions) = store.query(query)? {
    for solution in solutions {
        let solution = solution?;
        let value = solution.get("var_name");
    }
}

// Query (ASK)
if let QueryResults::Boolean(result) = store.query(query)? {
    // result is true/false
}

// Insert triple
store.insert(&Triple::new(subject, predicate, object))?;

// Remove triple
store.remove(&Triple::new(subject, predicate, object))?;
```

---

## Conclusion

This guide provided a comprehensive introduction to working with the YAWL ontology in knhk. Key takeaways:

1. **Ontologies = Type System + Schema + Documentation**
2. **Read yawl.ttl systematically** (namespaces → classes → properties)
3. **Extend via separate namespace** (knhk-extensions.ttl)
4. **Use SPARQL for validation and extraction**
5. **Optimize queries for performance**
6. **Version and document ontology changes**

**Next Steps:**
- Read `sparql-cookbook.md` for query recipes
- Read `ontology-reference-manual.md` for complete class/property reference
- Explore `/Users/sac/knhk/rust/knhk-workflow-engine/src/parser/` for implementation examples

**Additional Resources:**
- YAWL Foundation: http://www.yawlfoundation.org/
- Oxigraph Documentation: https://docs.rs/oxigraph/
- SPARQL 1.1 Specification: https://www.w3.org/TR/sparql11-query/
- OWL 2 Primer: https://www.w3.org/TR/owl2-primer/

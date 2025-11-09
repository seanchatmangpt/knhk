# YAWL Ontology to Rust Type Mapping

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Work In Progress

## Executive Summary

This document provides detailed mappings from YAWL OWL classes to Rust types in knhk-workflow-engine. Each mapping includes:
- Ontology definition (Turtle)
- Rust type definition
- Integration strategy
- Open questions

## 1. Core Workflow Classes

### 1.1 yawl:Specification

**Ontology Definition:**
```turtle
yawl:Specification a rdfs:Class ;
    rdfs:label "Specification" ;
    rdfs:comment "YAWL workflow specification" ;
    rdfs:subClassOf yawl:SpecificationSet .

# Properties
yawl:hasDecomposition a rdf:Property ;
    rdfs:domain yawl:Specification ;
    rdfs:range yawl:Decomposition .

yawl:hasMetadata a rdf:Property ;
    rdfs:domain yawl:Specification ;
    rdfs:range yawl:Metadata .
```

**Rust Mapping:**
```rust
// File: src/parser/types.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowSpec {
    /// UUID identifier
    pub id: WorkflowSpecId,

    /// Name from rdfs:label
    pub name: String,

    /// Tasks in workflow (from yawl:hasTask via yawl:hasDecomposition)
    pub tasks: HashMap<String, Task>,

    /// Conditions in workflow
    pub conditions: HashMap<String, Condition>,

    /// Start condition (from yawl:hasInputCondition)
    pub start_condition: Option<String>,

    /// End condition (from yawl:hasOutputCondition)
    pub end_condition: Option<String>,
}
```

**Integration Strategy:**
1. **Parse:** Load TTL into Oxigraph store
2. **Extract:** SPARQL query for `?spec rdf:type yawl:Specification`
3. **Map Properties:**
   - `rdfs:label` → `name`
   - `yawl:hasDecomposition` → traverse to extract nets/tasks
4. **Validate:** Check spec has at least one decomposition
5. **Create:** Construct `WorkflowSpec` instance

**SPARQL Extraction:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?spec ?name WHERE {
    ?spec rdf:type yawl:Specification .
    OPTIONAL { ?spec rdfs:label ?name }
} LIMIT 1
```

**Open Questions:**
- How to handle multiple decompositions? (Currently assumes one root net)
- How to version specifications? (Use yawl:version or separate property?)
- How to handle imported nets? (yawl:importedNet property)

---

### 1.2 yawl:Task

**Ontology Definition:**
```turtle
yawl:Task a rdfs:Class ;
    rdfs:label "Task" ;
    rdfs:comment "Workflow task" ;
    rdfs:subClassOf yawl:NetElement .

# Key properties
yawl:hasJoin a rdf:Property ;
    rdfs:domain yawl:Task ;
    rdfs:range yawl:ControlType .

yawl:hasSplit a rdf:Property ;
    rdfs:domain yawl:Task ;
    rdfs:range yawl:ControlType .

yawl:hasResourcing a rdf:Property ;
    rdfs:domain yawl:Task ;
    rdfs:range yawl:Resourcing .

yawl:hasTimer a rdf:Property ;
    rdfs:domain yawl:Task ;
    rdfs:range yawl:Timer .
```

**Rust Mapping:**
```rust
// File: src/parser/types.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    /// Task IRI (from RDF subject)
    pub id: String,

    /// Task name (from rdfs:label)
    pub name: String,

    /// Task type (from rdf:type: Task, MultipleInstanceTask)
    pub task_type: TaskType,

    /// Split type (from yawl:hasSplit)
    pub split_type: SplitType,

    /// Join type (from yawl:hasJoin)
    pub join_type: JoinType,

    /// knhk extension: max execution ticks (from knhk:tickBudget)
    pub max_ticks: Option<u32>,

    /// knhk extension: priority 0-255 (from knhk:priority)
    pub priority: Option<u32>,

    /// knhk extension: use SIMD (from knhk:useSimd)
    pub use_simd: bool,

    /// Input conditions (populated from yawl:flowsInto)
    pub input_conditions: Vec<String>,

    /// Output conditions
    pub output_conditions: Vec<String>,

    /// Outgoing flows (task/condition IRIs)
    pub outgoing_flows: Vec<String>,

    /// Incoming flows
    pub incoming_flows: Vec<String>,

    /// Resource allocation policy (from yawl:hasResourcing)
    pub allocation_policy: Option<crate::resource::AllocationPolicy>,

    /// Required roles (from yawl:hasResourcing → yawl:role)
    pub required_roles: Vec<String>,

    /// Required capabilities (from knhk:requiresCapability)
    pub required_capabilities: Vec<String>,

    /// Exception worklet (from yawl:hasExceptionWorklet)
    pub exception_worklet: Option<crate::worklets::WorkletId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SplitType {
    And,  // yawl:ControlTypeAnd
    Xor,  // yawl:ControlTypeXor
    Or,   // yawl:ControlTypeOr
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JoinType {
    And,  // yawl:ControlTypeAnd
    Xor,  // yawl:ControlTypeXor
    Or,   // yawl:ControlTypeOr
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    Atomic,           // yawl:Task (no decomposition)
    Composite,        // yawl:Task with yawl:hasDecomposesTo
    MultipleInstance, // yawl:MultipleInstanceTask
}
```

**Integration Strategy:**
1. **Extract Tasks:** SPARQL query for all tasks
2. **Map Control Flow:**
   - Extract `yawl:hasJoin` → map to `JoinType`
   - Extract `yawl:hasSplit` → map to `SplitType`
3. **Map Flows:**
   - Query `yawl:flowsInto` relationships
   - Build incoming_flows and outgoing_flows vectors
4. **Map Resources:**
   - Extract `yawl:hasResourcing` → build `AllocationPolicy`
   - Extract roles from `yawl:role` properties
5. **Map knhk Extensions:**
   - Extract `knhk:tickBudget` → `max_ticks`
   - Extract `knhk:priority` → `priority`

**SPARQL Extraction:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>
PREFIX knhk: <http://knhk.org/ontology#>

SELECT ?task ?name ?type ?split ?join ?maxTicks ?priority ?simd WHERE {
    ?task rdf:type ?type .
    FILTER(?type = yawl:Task || ?type = yawl:MultipleInstanceTask)

    OPTIONAL { ?task rdfs:label ?name }
    OPTIONAL { ?task yawl:splitType ?split }
    OPTIONAL { ?task yawl:joinType ?join }
    OPTIONAL { ?task knhk:tickBudget ?maxTicks }
    OPTIONAL { ?task knhk:priority ?priority }
    OPTIONAL { ?task knhk:useSimd ?simd }
}
```

**Open Questions:**
- How to handle missing join/split types? (Default to XOR?)
- How to validate join/split combinations? (Some are invalid)
- How to handle decomposition references? (Nested nets)
- How to map complex resource allocation patterns?

---

### 1.3 yawl:Condition

**Ontology Definition:**
```turtle
yawl:Condition a rdfs:Class ;
    rdfs:label "Condition" ;
    rdfs:comment "Workflow condition (place)" ;
    rdfs:subClassOf yawl:NetElement .

yawl:InputCondition a rdfs:Class ;
    rdfs:label "InputCondition" ;
    rdfs:comment "Input condition of a net" ;
    rdfs:subClassOf yawl:Condition .

yawl:OutputCondition a rdfs:Class ;
    rdfs:label "OutputCondition" ;
    rdfs:comment "Output condition of a net" ;
    rdfs:subClassOf yawl:Condition .
```

**Rust Mapping:**
```rust
// File: src/parser/types.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Condition {
    /// Condition IRI
    pub id: String,

    /// Condition name (from rdfs:label)
    pub name: String,

    /// Outgoing flows (task/condition IRIs)
    pub outgoing_flows: Vec<String>,

    /// Incoming flows
    pub incoming_flows: Vec<String>,
}
```

**Integration Strategy:**
1. **Extract Conditions:** Query for `yawl:Condition`, `yawl:InputCondition`, `yawl:OutputCondition`
2. **Map Flows:** Same as tasks (query `yawl:flowsInto`)
3. **Identify Special Conditions:**
   - Find start: `?condition rdf:type yawl:InputCondition`
   - Find end: `?condition rdf:type yawl:OutputCondition`

**SPARQL Extraction:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>

SELECT ?condition ?name WHERE {
    ?condition rdf:type ?type .
    FILTER(?type = yawl:Condition || ?type = yawl:InputCondition || ?type = yawl:OutputCondition)

    OPTIONAL { ?condition rdfs:label ?name }
}
```

**Open Questions:**
- How to enforce constraints? (InputCondition has no incoming, OutputCondition has no outgoing)
- How to handle implicit conditions? (Some tools auto-generate)

---

## 2. Multiple Instance Classes

### 2.1 yawl:MultipleInstanceTask

**Ontology Definition:**
```turtle
yawl:MultipleInstanceTask a rdfs:Class ;
    rdfs:label "MultipleInstanceTask" ;
    rdfs:comment "Multiple instance task" ;
    rdfs:subClassOf yawl:Task .

# MI properties
yawl:minimum a rdf:Property ;
    rdfs:domain yawl:MultipleInstanceTask ;
    rdfs:range xsd:string .

yawl:maximum a rdf:Property ;
    rdfs:domain yawl:MultipleInstanceTask ;
    rdfs:range xsd:string .

yawl:threshold a rdf:Property ;
    rdfs:domain yawl:MultipleInstanceTask ;
    rdfs:range xsd:string .

yawl:hasSplittingExpression a rdf:Property ;
    rdfs:domain yawl:MultipleInstanceTask ;
    rdfs:range yawl:Expression .

yawl:hasCreationMode a rdf:Property ;
    rdfs:domain yawl:MultipleInstanceTask ;
    rdfs:range yawl:CreationMode .
```

**Rust Mapping:**
```rust
// Extend Task struct with MI fields
pub struct Task {
    // ... standard fields ...

    // MI-specific fields (only populated if task_type == MultipleInstance)
    pub mi_config: Option<MultipleInstanceConfig>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MultipleInstanceConfig {
    /// Minimum instances (XPath expression or integer)
    pub minimum: String,  // From yawl:minimum

    /// Maximum instances
    pub maximum: String,  // From yawl:maximum

    /// Threshold (for dynamic creation)
    pub threshold: String,  // From yawl:threshold

    /// Splitting expression (how to split input data)
    pub splitting_expression: Option<String>,  // From yawl:hasSplittingExpression

    /// Output joining expression (how to merge outputs)
    pub joining_expression: Option<String>,  // From yawl:hasOutputJoiningExpression

    /// Creation mode (static or dynamic)
    pub creation_mode: CreationMode,  // From yawl:hasCreationMode
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CreationMode {
    Static,   // yawl:CreationModeStatic
    Dynamic,  // yawl:CreationModeDynamic
}
```

**Integration Strategy:**
1. **Detect MI Task:** Check if `rdf:type = yawl:MultipleInstanceTask`
2. **Extract MI Properties:**
   - Query `yawl:minimum`, `yawl:maximum`, `yawl:threshold`
   - Query `yawl:hasSplittingExpression`, `yawl:hasCreationMode`
3. **Parse Expressions:**
   - XPath expressions in `minimum`/`maximum` need runtime evaluation
   - Store as strings, evaluate at runtime
4. **Map Creation Mode:**
   - `yawl:CreationModeStatic` → `CreationMode::Static`
   - `yawl:CreationModeDynamic` → `CreationMode::Dynamic`

**SPARQL Extraction:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?min ?max ?threshold ?splitting ?mode WHERE {
    ?task rdf:type yawl:MultipleInstanceTask .
    OPTIONAL { ?task yawl:minimum ?min }
    OPTIONAL { ?task yawl:maximum ?max }
    OPTIONAL { ?task yawl:threshold ?threshold }
    OPTIONAL { ?task yawl:hasSplittingExpression ?splittingExpr .
               ?splittingExpr yawl:query ?splitting }
    OPTIONAL { ?task yawl:hasCreationMode ?mode }
}
```

**Open Questions:**
- How to evaluate XPath expressions at runtime? (Embed XPath engine?)
- How to handle dynamic instance creation? (Runtime instance spawning)
- How to map joining expressions? (Aggregate results)

---

## 3. Resource Allocation Classes

### 3.1 yawl:Resourcing

**Ontology Definition:**
```turtle
yawl:Resourcing a rdfs:Class ;
    rdfs:label "Resourcing" ;
    rdfs:comment "Resource allocation configuration" .

yawl:hasOffer a rdf:Property ;
    rdfs:domain yawl:Resourcing ;
    rdfs:range yawl:ResourcingOffer .

yawl:hasAllocate a rdf:Property ;
    rdfs:domain yawl:Resourcing ;
    rdfs:range yawl:ResourcingAllocate .

yawl:hasStart a rdf:Property ;
    rdfs:domain yawl:Resourcing ;
    rdfs:range yawl:ResourcingInitiator .
```

**Rust Mapping:**
```rust
// File: src/resource/allocation/types.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum AllocationPolicy {
    /// Round-robin allocation
    RoundRobin,

    /// Shortest queue first
    ShortestQueue,

    /// Random allocation
    Random,

    /// Capability-based matching
    CapabilityBased {
        required_capabilities: Vec<String>,
    },

    /// Role-based allocation
    RoleBased {
        required_roles: Vec<String>,
    },

    /// Custom selector (from YAWL ResourcingSelector)
    CustomSelector {
        name: String,
        params: HashMap<String, String>,
    },
}
```

**Integration Strategy:**
1. **Extract Resourcing:** Query `yawl:hasResourcing` from task
2. **Map Offer:**
   - Extract `yawl:hasOffer` → `yawl:hasDistributionSet`
   - Extract roles: `yawl:role`
   - Map to `RoleBased` policy
3. **Map Allocate:**
   - Extract `yawl:hasAllocate` → `yawl:hasAllocator`
   - Extract allocator name from `yawl:ResourcingSelector`
   - Map to `CustomSelector` or predefined policy
4. **Map Initiator:**
   - Extract `yawl:hasStart` → `System` or `User`

**SPARQL Extraction:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?role ?allocator WHERE {
    ?task yawl:hasResourcing ?resourcing .

    OPTIONAL {
        ?resourcing yawl:hasOffer ?offer .
        ?offer yawl:hasDistributionSet ?distSet .
        ?distSet yawl:hasInitialSet ?initSet .
        ?initSet yawl:role ?role .
    }

    OPTIONAL {
        ?resourcing yawl:hasAllocate ?allocate .
        ?allocate yawl:hasAllocator ?allocatorNode .
        ?allocatorNode rdfs:label ?allocator .
    }
}
```

**Open Questions:**
- How to map complex selectors? (Filters, constraints)
- How to handle participant-based allocation? (Individual users)
- How to integrate with external resource services?

---

## 4. Timing Classes

### 4.1 yawl:Timer

**Ontology Definition:**
```turtle
yawl:Timer a rdfs:Class ;
    rdfs:label "Timer" ;
    rdfs:comment "Timer configuration for tasks" .

yawl:hasTrigger a rdf:Property ;
    rdfs:domain yawl:Timer ;
    rdfs:range yawl:TimerTrigger .

yawl:hasDurationParams a rdf:Property ;
    rdfs:domain yawl:Timer ;
    rdfs:range yawl:TimerDuration .

yawl:TimerDuration a rdfs:Class .

yawl:ticks a rdf:Property ;
    rdfs:domain yawl:TimerDuration ;
    rdfs:range xsd:long .

yawl:hasInterval a rdf:Property ;
    rdfs:domain yawl:TimerDuration ;
    rdfs:range yawl:TimerInterval .
```

**Rust Mapping:**
```rust
// File: src/executor/timer.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimerConfig {
    /// When timer triggers (OnEnabled or OnExecuting)
    pub trigger: TimerTrigger,

    /// Timer duration
    pub duration: TimerDuration,

    /// Whether to use workdays only
    pub workdays_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimerTrigger {
    OnEnabled,    // yawl:TimerTriggerOnEnabled
    OnExecuting,  // yawl:TimerTriggerOnExecuting
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimerDuration {
    /// Number of ticks
    pub ticks: u64,

    /// Time interval unit
    pub interval: TimeInterval,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimeInterval {
    Year,   // yawl:TimerIntervalYear
    Month,  // yawl:TimerIntervalMonth
    Week,   // yawl:TimerIntervalWeek
    Day,    // yawl:TimerIntervalDay
    Hour,   // yawl:TimerIntervalHour
    Min,    // yawl:TimerIntervalMin
    Sec,    // yawl:TimerIntervalSec
    Msec,   // yawl:TimerIntervalMsec
}
```

**Integration Strategy:**
1. **Extract Timer:** Query `yawl:hasTimer` from task
2. **Map Trigger:** Extract `yawl:hasTrigger` → map to enum
3. **Map Duration:**
   - Extract `yawl:ticks` → `u64`
   - Extract `yawl:hasInterval` → map to `TimeInterval` enum
4. **Calculate Absolute Time:**
   - Convert (ticks, interval) to Duration
   - Account for workdays if needed

**SPARQL Extraction:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>

SELECT ?task ?trigger ?ticks ?interval ?workdays WHERE {
    ?task yawl:hasTimer ?timer .

    OPTIONAL { ?timer yawl:hasTrigger ?trigger }
    OPTIONAL { ?timer yawl:workdays ?workdays }

    OPTIONAL {
        ?timer yawl:hasDurationParams ?durationParams .
        ?durationParams yawl:ticks ?ticks .
        ?durationParams yawl:hasInterval ?interval .
    }
}
```

**Open Questions:**
- How to handle net parameters in timer duration? (Runtime evaluation)
- How to integrate with calendar systems for workdays?
- How to handle timer cancellation?

---

## 5. Data Classes

### 5.1 yawl:Variable and yawl:Parameter

**Ontology Definition:**
```turtle
yawl:Variable a rdfs:Class ;
    rdfs:label "Variable" ;
    rdfs:comment "Workflow variable" ;
    rdfs:subClassOf yawl:VariableBase .

yawl:InputParameter a rdfs:Class ;
    rdfs:label "InputParameter" ;
    rdfs:comment "Input parameter" ;
    rdfs:subClassOf yawl:VariableBase .

yawl:OutputParameter a rdfs:Class ;
    rdfs:label "OutputParameter" ;
    rdfs:comment "Output parameter" ;
    rdfs:subClassOf yawl:VariableBase .

# Properties
yawl:type a rdf:Property ;
    rdfs:domain yawl:VariableBase ;
    rdfs:range xsd:NCName .

yawl:namespace a rdf:Property ;
    rdfs:domain yawl:VariableBase ;
    rdfs:range xsd:anyURI .

yawl:initialValue a rdf:Property ;
    rdfs:domain yawl:Variable ;
    rdfs:range xsd:string .
```

**Rust Mapping:**
```rust
// File: src/data/variable.rs
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Variable {
    /// Variable name (from rdfs:label or yawl:name)
    pub name: String,

    /// Data type (from yawl:type)
    pub data_type: DataType,

    /// Namespace (from yawl:namespace)
    pub namespace: Option<String>,

    /// Initial value (from yawl:initialValue)
    pub initial_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputParameter {
    pub name: String,
    pub data_type: DataType,
    pub mandatory: bool,  // From yawl:mandatory
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OutputParameter {
    pub name: String,
    pub data_type: DataType,
    pub default_value: Option<serde_json::Value>,  // From yawl:defaultValue
    pub mandatory: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DataType {
    String,
    Integer,
    Double,
    Boolean,
    Date,
    DateTime,
    Custom { type_name: String, namespace: String },
}
```

**Integration Strategy:**
1. **Extract Variables:** Query `yawl:hasLocalVariable` from Net
2. **Extract Parameters:**
   - Query `yawl:hasInputParameter` from Decomposition
   - Query `yawl:hasOutputParameter` from Decomposition
3. **Map Types:**
   - Extract `yawl:type` and `yawl:namespace`
   - Map XSD types to `DataType` enum
4. **Map Values:**
   - Extract `yawl:initialValue` → parse to JSON
   - Extract `yawl:defaultValue` → parse to JSON

**SPARQL Extraction:**
```sparql
PREFIX yawl: <http://www.yawlfoundation.org/yawlschema#>
PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>

SELECT ?var ?name ?type ?ns ?initialValue WHERE {
    ?decomp yawl:hasLocalVariable ?var .

    OPTIONAL { ?var rdfs:label ?name }
    OPTIONAL { ?var yawl:type ?type }
    OPTIONAL { ?var yawl:namespace ?ns }
    OPTIONAL { ?var yawl:initialValue ?initialValue }
}
```

**Open Questions:**
- How to validate data types? (Schema validation)
- How to handle complex types? (Custom XSD types)
- How to serialize/deserialize values? (JSON, XML, binary)

---

## 6. Summary: Complete Mapping Table

| YAWL Class | Rust Type | File | Completeness |
|------------|-----------|------|--------------|
| `yawl:Specification` | `WorkflowSpec` | `parser/types.rs` | 80% |
| `yawl:Net` | Embedded in `WorkflowSpec` | `parser/types.rs` | 60% |
| `yawl:Task` | `Task` | `parser/types.rs` | 90% |
| `yawl:MultipleInstanceTask` | `Task` + `MultipleInstanceConfig` | `parser/types.rs` | 70% |
| `yawl:Condition` | `Condition` | `parser/types.rs` | 95% |
| `yawl:InputCondition` | `Condition` (start_condition) | `parser/types.rs` | 100% |
| `yawl:OutputCondition` | `Condition` (end_condition) | `parser/types.rs` | 100% |
| `yawl:Variable` | `Variable` | `data/variable.rs` | 50% |
| `yawl:InputParameter` | `InputParameter` | `data/variable.rs` | 50% |
| `yawl:OutputParameter` | `OutputParameter` | `data/variable.rs` | 50% |
| `yawl:Resourcing` | `AllocationPolicy` | `resource/allocation/types.rs` | 60% |
| `yawl:Timer` | `TimerConfig` | `executor/timer.rs` | 70% |
| `yawl:FlowsInto` | Vectors in `Task`/`Condition` | `parser/types.rs` | 100% |
| `yawl:ControlType` | `SplitType`, `JoinType` | `parser/types.rs` | 100% |
| `yawl:Metadata` | Not mapped (future) | - | 0% |
| `yawl:Layout` | Not mapped (visual only) | - | 0% |
| `yawl:WebServiceGateway` | Not mapped (future) | - | 0% |

## 7. Future Enhancements

1. **Add Metadata Support:**
   - Map Dublin Core properties
   - Support versioning, contributors, dates

2. **Add Web Service Support:**
   - Map `yawl:WebServiceGateway` to Rust HTTP client
   - Support WSDL integration

3. **Add Layout Support:**
   - Store layout info for visualization
   - Generate layout from workflow structure

4. **Add Expression Evaluation:**
   - Embed XPath/XQuery engine
   - Support runtime expression evaluation

5. **Add Validation:**
   - Type checking for variables
   - Constraint checking for MI tasks
   - Flow soundness validation

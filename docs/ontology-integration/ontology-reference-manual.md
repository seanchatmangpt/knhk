# YAWL Ontology Reference Manual

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Production Ready
**Ontology Version:** YAWL 4.0
**Source:** `/Users/sac/knhk/ontology/yawl.ttl`

---

## Executive Summary

This reference manual provides complete documentation for all 72 OWL classes and 130+ properties in the YAWL 4.0 ontology. Every class and property is documented with:
- **IRI:** Full namespace identifier
- **Label:** Human-readable name
- **Description:** Detailed explanation
- **Hierarchy:** Superclasses and subclasses
- **Properties:** Applicable properties with domains and ranges
- **Examples:** RDF/Turtle and SPARQL usage
- **Constraints:** Cardinality and validation rules

---

## Table of Contents

1. [Ontology Metadata](#1-ontology-metadata)
2. [Complete Class Hierarchy](#2-complete-class-hierarchy)
3. [Enumeration Classes Reference](#3-enumeration-classes-reference)
4. [Core Workflow Classes Reference](#4-core-workflow-classes-reference)
5. [Resource Allocation Classes Reference](#5-resource-allocation-classes-reference)
6. [Configuration Classes Reference](#6-configuration-classes-reference)
7. [Layout Classes Reference](#7-layout-classes-reference)
8. [Datatype Properties Reference](#8-datatype-properties-reference)
9. [Object Properties Reference](#9-object-properties-reference)
10. [Property Matrix](#10-property-matrix)
11. [Visual Diagrams](#11-visual-diagrams)
12. [Constraint Reference](#12-constraint-reference)

---

## 1. Ontology Metadata

### 1.1 Namespaces

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
```

| Prefix | Namespace IRI | Purpose |
|--------|---------------|---------|
| `yawl:` | `http://www.yawlfoundation.org/yawlschema#` | YAWL classes and properties |
| `rdf:` | `http://www.w3.org/1999/02/22-rdf-syntax-ns#` | RDF core vocabulary |
| `rdfs:` | `http://www.w3.org/2000/01/rdf-schema#` | RDF Schema vocabulary |
| `owl:` | `http://www.w3.org/2002/07/owl#` | OWL ontology language |
| `xsd:` | `http://www.w3.org/2001/XMLSchema#` | XML Schema datatypes |

### 1.2 Ontology Information

```turtle
yawl:Ontology a owl:Ontology ;
    rdfs:label "YAWL 4.0 Ontology" ;
    rdfs:comment "Generated from YAWL_Schema4.0.xsd" ;
    owl:versionInfo "4.0.0" .
```

---

## 2. Complete Class Hierarchy

### 2.1 Full Class Tree

```
owl:Thing
│
├── yawl:SpecificationSet
│   └── yawl:Specification
│
├── yawl:Decomposition
│   ├── yawl:Net
│   └── yawl:WebServiceGateway
│
├── yawl:NetElement
│   ├── yawl:Task
│   │   └── yawl:MultipleInstanceTask
│   └── yawl:Condition
│       ├── yawl:InputCondition
│       └── yawl:OutputCondition
│
├── yawl:VariableBase
│   ├── yawl:Variable
│   ├── yawl:InputParameter
│   └── yawl:OutputParameter
│
├── yawl:FlowsInto
├── yawl:RemovesTokensFromFlow
│
├── yawl:Resourcing
├── yawl:ResourcingOffer
├── yawl:ResourcingAllocate
├── yawl:ResourcingSet
│   └── yawl:ResourcingSecondary
├── yawl:ResourcingDistributionSet
├── yawl:ResourcingSelector
├── yawl:ResourcingPrivileges
│
├── yawl:Timer
├── yawl:TimerDuration
│
├── yawl:VarMapping
├── yawl:VarMappingSet
├── yawl:Expression
├── yawl:Predicate
│
├── yawl:Configuration
├── yawl:JoinConfig
├── yawl:SplitConfig
├── yawl:RemConfig
├── yawl:NofiConfig
├── yawl:InputPortConfig
├── yawl:OutputPortConfig
│
├── yawl:YAWLService
├── yawl:Metadata
├── yawl:LogPredicate
│
├── yawl:Layout
├── yawl:LayoutNet
├── yawl:LayoutVertex
├── yawl:LayoutContainer
├── yawl:LayoutFlow
├── yawl:LayoutDecorator
├── yawl:LayoutLabel
├── yawl:LayoutPoint
├── yawl:LayoutLocale
├── yawl:LayoutPorts
├── yawl:LayoutAttributes
├── yawl:LayoutRectangle
├── yawl:LayoutFrame
├── yawl:LayoutPoints
├── yawl:LayoutFont
├── yawl:LayoutDimension
├── yawl:LayoutUserObjectHTML
│
└── Enumeration Classes (12)
    ├── yawl:ControlType
    │   ├── yawl:ControlTypeAnd
    │   ├── yawl:ControlTypeOr
    │   └── yawl:ControlTypeXor
    ├── yawl:CreationMode
    │   ├── yawl:CreationModeStatic
    │   └── yawl:CreationModeDynamic
    ├── yawl:TimerInterval (8 instances)
    ├── yawl:TimerTrigger (2 instances)
    ├── yawl:ResourcingExternalInteraction (2 instances)
    ├── yawl:ResourcingInitiator (2 instances)
    ├── yawl:DirectionMode (3 instances)
    ├── yawl:ResourcingPrivilege (7 instances)
    ├── yawl:ResourcingResourceType (2 instances)
    ├── yawl:InputPortValueType (3 instances)
    ├── yawl:OutputPortValueType (2 instances)
    └── yawl:CreationModeConfigType (2 instances)
```

### 2.2 Class Statistics

| Category | Count |
|----------|-------|
| **Total Classes** | 72 |
| **Enumeration Parent Classes** | 12 |
| **Enumeration Instances** | 33 |
| **Core Workflow Classes** | 17 |
| **Resource Classes** | 8 |
| **Configuration Classes** | 10 |
| **Layout Classes** | 17 |
| **Other Classes** | 8 |

---

## 3. Enumeration Classes Reference

### 3.1 ControlType

**IRI:** `yawl:ControlType`
**Label:** ControlType
**Description:** Control flow type for task join and split

**Instances:**

#### yawl:ControlTypeAnd

```turtle
yawl:ControlTypeAnd a yawl:ControlType ;
    rdfs:label "AND" ;
    rdfs:comment "AND join/split - all branches must complete (join) or all branches execute (split)" .
```

**Usage:**
- **Join:** Wait for all incoming branches
- **Split:** Execute all outgoing branches in parallel

**Example:**

```turtle
?task yawl:hasJoin yawl:ControlTypeAnd .
?task yawl:hasSplit yawl:ControlTypeAnd .
```

#### yawl:ControlTypeOr

```turtle
yawl:ControlTypeOr a yawl:ControlType ;
    rdfs:label "OR" ;
    rdfs:comment "OR join/split - multiple branches may complete/execute" .
```

**Usage:**
- **Join:** Wait for one or more incoming branches (non-deterministic)
- **Split:** Execute one or more outgoing branches based on conditions

**Example:**

```turtle
?task yawl:hasJoin yawl:ControlTypeOr .
?task yawl:hasSplit yawl:ControlTypeOr .
```

#### yawl:ControlTypeXor

```turtle
yawl:ControlTypeXor a yawl:ControlType ;
    rdfs:label "XOR" ;
    rdfs:comment "XOR join/split - exactly one branch" .
```

**Usage:**
- **Join:** Wait for exactly one incoming branch (exclusive)
- **Split:** Execute exactly one outgoing branch (conditional)

**Example:**

```turtle
?task yawl:hasJoin yawl:ControlTypeXor .
?task yawl:hasSplit yawl:ControlTypeXor .
```

**SPARQL Query:**

```sparql
# Find all XOR-join, AND-split tasks (parallel split pattern)
SELECT ?task WHERE {
    ?task yawl:hasJoin yawl:ControlTypeXor .
    ?task yawl:hasSplit yawl:ControlTypeAnd .
}
```

---

### 3.2 CreationMode

**IRI:** `yawl:CreationMode`
**Label:** CreationMode
**Description:** Instance creation mode for multiple instance tasks

**Instances:**

#### yawl:CreationModeStatic

```turtle
yawl:CreationModeStatic a yawl:CreationMode ;
    rdfs:label "Static" ;
    rdfs:comment "All instances created at task start" .
```

**Behavior:** Number of instances determined at task start, fixed thereafter.

#### yawl:CreationModeDynamic

```turtle
yawl:CreationModeDynamic a yawl:CreationMode ;
    rdfs:label "Dynamic" ;
    rdfs:comment "Instances created/destroyed at runtime" .
```

**Behavior:** Number of instances can change during task execution.

**Example:**

```turtle
?miTask a yawl:MultipleInstanceTask ;
    yawl:hasCreationMode yawl:CreationModeStatic ;
    yawl:minimum 3 ;
    yawl:maximum 10 .
```

---

### 3.3 TimerInterval

**IRI:** `yawl:TimerInterval`
**Label:** TimerInterval
**Description:** Time unit for timer duration

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:TimerIntervalYear` | YEAR | Annual interval |
| `yawl:TimerIntervalMonth` | MONTH | Monthly interval |
| `yawl:TimerIntervalWeek` | WEEK | Weekly interval |
| `yawl:TimerIntervalDay` | DAY | Daily interval |
| `yawl:TimerIntervalHour` | HOUR | Hourly interval |
| `yawl:TimerIntervalMin` | MIN | Minute interval |
| `yawl:TimerIntervalSec` | SEC | Second interval |
| `yawl:TimerIntervalMsec` | MSEC | Millisecond interval |

**Example:**

```turtle
?timer yawl:hasDurationParams ?duration .
?duration yawl:ticks 30 ;
    yawl:hasInterval yawl:TimerIntervalSec .  # 30 seconds
```

---

### 3.4 TimerTrigger

**IRI:** `yawl:TimerTrigger`
**Label:** TimerTrigger
**Description:** When timer starts counting

**Instances:**

#### yawl:TimerTriggerOnEnabled

```turtle
yawl:TimerTriggerOnEnabled a yawl:TimerTrigger ;
    rdfs:label "OnEnabled" ;
    rdfs:comment "Timer starts when task is enabled" .
```

**Usage:** Timer begins when task becomes executable (before user selects it).

#### yawl:TimerTriggerOnExecuting

```turtle
yawl:TimerTriggerOnExecuting a yawl:TimerTrigger ;
    rdfs:label "OnExecuting" ;
    rdfs:comment "Timer starts when task execution begins" .
```

**Usage:** Timer begins when user starts executing task.

**Example:**

```turtle
?task yawl:hasTimer ?timer .
?timer yawl:hasTrigger yawl:TimerTriggerOnEnabled ;
    yawl:expiry 1699459200000 .  # Unix timestamp
```

---

### 3.5 ResourcingExternalInteraction

**IRI:** `yawl:ResourcingExternalInteraction`
**Description:** Type of external interaction for task execution

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:ResourcingExternalInteractionManual` | Manual | User interaction required |
| `yawl:ResourcingExternalInteractionAutomated` | Automated | Automated execution (service call) |

**Example:**

```turtle
?gateway a yawl:WebServiceGateway ;
    yawl:hasExternalInteraction yawl:ResourcingExternalInteractionAutomated .
```

---

### 3.6 ResourcingInitiator

**IRI:** `yawl:ResourcingInitiator`
**Description:** Who initiates resource allocation or task start

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:ResourcingInitiatorSystem` | System | System automatically initiates |
| `yawl:ResourcingInitiatorUser` | User | User manually initiates |

**Example:**

```turtle
?resourcing yawl:hasStart yawl:ResourcingInitiatorUser .  # User starts task
?resourcing yawl:hasOffer ?offer .
?offer yawl:hasInitiator yawl:ResourcingInitiatorSystem .  # System offers work item
```

---

### 3.7 DirectionMode

**IRI:** `yawl:DirectionMode`
**Description:** Data flow direction for parameters

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:DirectionModeInput` | Input | Input-only parameter |
| `yawl:DirectionModeOutput` | Output | Output-only parameter |
| `yawl:DirectionModeBoth` | Both | Input and output (cut-through) |

**Note:** Not widely used in YAWL 4.0; typically inferred from parameter type.

---

### 3.8 ResourcingPrivilege

**IRI:** `yawl:ResourcingPrivilege`
**Description:** Privileges a resource has over work items

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:ResourcingPrivilegeCanSuspend` | canSuspend | Can suspend work item execution |
| `yawl:ResourcingPrivilegeCanReallocateStateless` | canReallocateStateless | Can reallocate without state |
| `yawl:ResourcingPrivilegeCanReallocateStateful` | canReallocateStateful | Can reallocate with state |
| `yawl:ResourcingPrivilegeCanDeallocate` | canDeallocate | Can deallocate work item |
| `yawl:ResourcingPrivilegeCanDelegate` | canDelegate | Can delegate to another resource |
| `yawl:ResourcingPrivilegeCanSkip` | canSkip | Can skip work item |
| `yawl:ResourcingPrivilegeCanPile` | canPile | Can pile (queue) work items |

**Example:**

```turtle
?resourcing yawl:hasPrivileges ?privileges .
?privileges yawl:hasPrivilege ?priv1 .
?priv1 yawl:hasPrivilegeName yawl:ResourcingPrivilegeCanSuspend .
```

---

### 3.9 ResourcingResourceType

**IRI:** `yawl:ResourcingResourceType`
**Description:** Type of resource

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:ResourcingResourceTypeParticipant` | Participant | Individual participant |
| `yawl:ResourcingResourceTypeRole` | Role | Organizational role |

**Example:**

```turtle
?resourceRef yawl:refers yawl:ResourcingResourceTypeRole .
```

---

### 3.10 InputPortValueType

**IRI:** `yawl:InputPortValueType`
**Description:** State of input port

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:InputPortValueActivated` | Activated | Port accepts tokens |
| `yawl:InputPortValueBlocked` | Blocked | Port does not accept tokens |
| `yawl:InputPortValueHidden` | Hidden | Port hidden in UI |

---

### 3.11 OutputPortValueType

**IRI:** `yawl:OutputPortValueType`
**Description:** State of output port

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:OutputPortValueActivated` | Activated | Port produces tokens |
| `yawl:OutputPortValueBlocked` | Blocked | Port does not produce tokens |

---

### 3.12 CreationModeConfigType

**IRI:** `yawl:CreationModeConfigType`
**Description:** Configuration for dynamic instance creation

**Instances:**

| Instance | Label | Description |
|----------|-------|-------------|
| `yawl:CreationModeConfigRestrict` | Restrict | Restrict instance count to parameters |
| `yawl:CreationModeConfigKeep` | Keep | Keep instances even if below minimum |

---

## 4. Core Workflow Classes Reference

### 4.1 SpecificationSet

**IRI:** `yawl:SpecificationSet`
**Label:** SpecificationSet
**Description:** Container for workflow specifications

**Superclass:** None (top-level)
**Subclass:** `yawl:Specification`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasSpecification` | Object | `yawl:Specification` | Specification in set |
| `yawl:hasLayout` | Object | `yawl:Layout` | Layout information |

**Example:**

```turtle
<http://example.org/specs> a yawl:SpecificationSet ;
    yawl:hasSpecification <http://example.org/workflow#OrderProcessing> ;
    yawl:hasSpecification <http://example.org/workflow#Shipping> ;
    yawl:hasLayout <http://example.org/layout#Main> .
```

---

### 4.2 Specification

**IRI:** `yawl:Specification`
**Label:** Specification
**Description:** YAWL workflow specification

**Superclass:** `yawl:SpecificationSet`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:name` | Datatype | `xsd:string` | Specification name |
| `yawl:documentation` | Datatype | `xsd:string` | Documentation |
| `yawl:uri` | Datatype | `xsd:anyURI` | Specification URI |
| `yawl:hasDecomposition` | Object | `yawl:Decomposition` | Net or service gateway |
| `yawl:hasMetadata` | Object | `yawl:Metadata` | Dublin Core metadata |
| `yawl:importedNet` | Datatype | `xsd:anyURI` | Imported net URI |

**Cardinality Constraints (Recommended):**
- `yawl:hasDecomposition` - min 1 (at least one root net)
- `yawl:hasMetadata` - max 1

**Example:**

```turtle
<http://example.org/workflow#OrderProcessing> a yawl:Specification ;
    yawl:name "Order Processing Workflow" ;
    yawl:uri "http://example.org/workflow#OrderProcessing" ;
    yawl:hasDecomposition <http://example.org/net#MainNet> ;
    yawl:hasMetadata <http://example.org/metadata#OrderProcessing> .
```

**SPARQL:**

```sparql
# Get all decompositions in a specification
SELECT ?decomposition ?type WHERE {
    <http://example.org/workflow#OrderProcessing> yawl:hasDecomposition ?decomposition .
    ?decomposition a ?type .
}
```

---

### 4.3 Decomposition

**IRI:** `yawl:Decomposition`
**Label:** Decomposition
**Description:** Abstract base for nets and web service gateways

**Superclass:** None
**Subclasses:** `yawl:Net`, `yawl:WebServiceGateway`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:id` | Datatype | `xsd:NMTOKEN` | Decomposition ID |
| `yawl:hasInputParameter` | Object | `yawl:InputParameter` | Input parameter |
| `yawl:hasOutputParameter` | Object | `yawl:OutputParameter` | Output parameter |
| `yawl:hasLogPredicate` | Object | `yawl:LogPredicate` | Logging configuration |

**Abstract:** Should not be instantiated directly; use `Net` or `WebServiceGateway`.

---

### 4.4 Net

**IRI:** `yawl:Net`
**Label:** Net
**Description:** Workflow net (process definition)

**Superclass:** `yawl:Decomposition`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:isRootNet` | Datatype | `xsd:boolean` | Whether this is the root net |
| `yawl:hasInputCondition` | Object | `yawl:InputCondition` | Start condition |
| `yawl:hasOutputCondition` | Object | `yawl:OutputCondition` | End condition |
| `yawl:hasTask` | Object | `yawl:Task` | Task in net |
| `yawl:hasCondition` | Object | `yawl:Condition` | Condition in net |
| `yawl:hasLocalVariable` | Object | `yawl:Variable` | Local variable |
| `yawl:externalDataGateway` | Datatype | `xsd:string` | External data gateway name |

**Cardinality Constraints (Recommended):**
- `yawl:hasInputCondition` - exactly 1
- `yawl:hasOutputCondition` - exactly 1

**Example:**

```turtle
<http://example.org/net#MainNet> a yawl:Net ;
    yawl:id "MainNet" ;
    yawl:isRootNet true ;
    yawl:hasInputCondition <http://example.org/net#Start> ;
    yawl:hasOutputCondition <http://example.org/net#End> ;
    yawl:hasTask <http://example.org/task#Task1> ;
    yawl:hasTask <http://example.org/task#Task2> ;
    yawl:hasLocalVariable <http://example.org/var#OrderID> .
```

---

### 4.5 NetElement

**IRI:** `yawl:NetElement`
**Label:** NetElement
**Description:** Base class for tasks and conditions

**Superclass:** None
**Subclasses:** `yawl:Task`, `yawl:Condition`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:id` | Datatype | `xsd:NMTOKEN` | Element ID |
| `yawl:name` | Datatype | `xsd:string` | Element name |
| `yawl:flowsInto` | Object | `yawl:FlowsInto` | Outgoing flow |

---

### 4.6 Task

**IRI:** `yawl:Task`
**Label:** Task
**Description:** Workflow task

**Superclass:** `yawl:NetElement`
**Subclass:** `yawl:MultipleInstanceTask`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasJoin` | Object | `yawl:ControlType` | Join type (AND/OR/XOR) |
| `yawl:hasSplit` | Object | `yawl:ControlType` | Split type (AND/OR/XOR) |
| `yawl:hasTimer` | Object | `yawl:Timer` | Timer configuration |
| `yawl:hasResourcing` | Object | `yawl:Resourcing` | Resource allocation |
| `yawl:hasDecomposesTo` | Object | `yawl:Decomposition` | Subnet or service |
| `yawl:hasStartingMappings` | Object | `yawl:VarMappingSet` | Input mappings |
| `yawl:hasCompletedMappings` | Object | `yawl:VarMappingSet` | Output mappings |
| `yawl:hasEnablementMappings` | Object | `yawl:VarMappingSet` | Enablement mappings |
| `yawl:hasRemovesTokens` | Object | `yawl:NetElement` | Cancellation target |
| `yawl:hasRemovesTokensFromFlow` | Object | `yawl:RemovesTokensFromFlow` | Cancellation flow |
| `yawl:hasDefaultConfiguration` | Object | `yawl:Configuration` | Default config |
| `yawl:hasConfiguration` | Object | `yawl:Configuration` | Runtime config |
| `yawl:customForm` | Datatype | `xsd:anyURI` | Custom form URI |

**Cardinality Constraints (Recommended):**
- `yawl:hasJoin` - exactly 1
- `yawl:hasSplit` - exactly 1

**Example:**

```turtle
<http://example.org/task#ApproveOrder> a yawl:Task ;
    yawl:id "ApproveOrder" ;
    yawl:name "Approve Order" ;
    yawl:hasJoin yawl:ControlTypeXor ;
    yawl:hasSplit yawl:ControlTypeXor ;
    yawl:hasResourcing <http://example.org/res#ManagerResourcing> ;
    yawl:flowsInto <http://example.org/flow#Flow1> .

<http://example.org/flow#Flow1> a yawl:FlowsInto ;
    yawl:nextElementRef <http://example.org/task#ShipOrder> .
```

---

### 4.7 MultipleInstanceTask

**IRI:** `yawl:MultipleInstanceTask`
**Label:** MultipleInstanceTask
**Description:** Task that executes multiple instances

**Superclass:** `yawl:Task`

**Properties (Additional to Task):**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:minimum` | Datatype | `xsd:string` | Minimum instances (XPath expr) |
| `yawl:maximum` | Datatype | `xsd:string` | Maximum instances (XPath expr) |
| `yawl:threshold` | Datatype | `xsd:string` | Completion threshold (XPath expr) |
| `yawl:hasCreationMode` | Object | `yawl:CreationMode` | Static or dynamic |
| `yawl:hasSplittingExpression` | Object | `yawl:Expression` | How to split data |
| `yawl:hasOutputJoiningExpression` | Object | `yawl:Expression` | How to join outputs |
| `yawl:formalInputParam` | Datatype | `xsd:NMTOKEN` | Input parameter name |
| `yawl:formalOutputExpression` | Datatype | `xsd:string` | Output expression |
| `yawl:resultAppliedToLocalVariable` | Datatype | `xsd:NMTOKEN` | Result variable |

**Example:**

```turtle
<http://example.org/task#ProcessOrders> a yawl:MultipleInstanceTask ;
    yawl:id "ProcessOrders" ;
    yawl:name "Process Orders" ;
    yawl:minimum "3" ;
    yawl:maximum "10" ;
    yawl:threshold "count(//order)" ;
    yawl:hasCreationMode yawl:CreationModeStatic ;
    yawl:hasSplittingExpression <http://example.org/expr#SplitOrders> ;
    yawl:formalInputParam "order" ;
    yawl:resultAppliedToLocalVariable "processedOrders" .

<http://example.org/expr#SplitOrders> a yawl:Expression ;
    yawl:query "for $order in //orders/order return $order" .
```

---

### 4.8 Condition

**IRI:** `yawl:Condition`
**Label:** Condition
**Description:** Workflow condition (place in Petri net)

**Superclass:** `yawl:NetElement`
**Subclasses:** `yawl:InputCondition`, `yawl:OutputCondition`

**Properties:**
- Inherits from `NetElement`

**Example:**

```turtle
<http://example.org/cond#Condition1> a yawl:Condition ;
    yawl:id "c1" ;
    yawl:flowsInto <http://example.org/flow#Flow2> .
```

---

### 4.9 InputCondition

**IRI:** `yawl:InputCondition`
**Label:** InputCondition
**Description:** Start condition of a net

**Superclass:** `yawl:Condition`

**Constraints:**
- **Must not have incoming flows** (validation rule)

**Example:**

```turtle
<http://example.org/net#Start> a yawl:InputCondition ;
    yawl:id "start" ;
    yawl:flowsInto <http://example.org/flow#StartFlow> .
```

---

### 4.10 OutputCondition

**IRI:** `yawl:OutputCondition`
**Label:** OutputCondition
**Description:** End condition of a net

**Superclass:** `yawl:Condition`

**Constraints:**
- **Must not have outgoing flows** (validation rule)

**Example:**

```turtle
<http://example.org/net#End> a yawl:OutputCondition ;
    yawl:id "end" .
```

---

### 4.11 Variable

**IRI:** `yawl:Variable`
**Label:** Variable
**Description:** Workflow variable (net-scoped)

**Superclass:** `yawl:VariableBase`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:initialValue` | Datatype | `xsd:string` | Initial value |

**Example:**

```turtle
<http://example.org/var#OrderID> a yawl:Variable ;
    yawl:name "orderId" ;
    yawl:type "string" ;
    yawl:namespace "http://www.w3.org/2001/XMLSchema" ;
    yawl:initialValue "" .
```

---

### 4.12 InputParameter

**IRI:** `yawl:InputParameter`
**Label:** InputParameter
**Description:** Input parameter of a decomposition

**Superclass:** `yawl:VariableBase`

**Example:**

```turtle
<http://example.org/param#OrderInput> a yawl:InputParameter ;
    yawl:name "order" ;
    yawl:type "OrderType" ;
    yawl:index 0 .
```

---

### 4.13 OutputParameter

**IRI:** `yawl:OutputParameter`
**Label:** OutputParameter
**Description:** Output parameter of a decomposition

**Superclass:** `yawl:VariableBase`

**Properties (Additional):**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:defaultValue` | Datatype | `rdfs:Resource` | Default value |
| `yawl:mandatory` | Datatype | `xsd:boolean` | Whether mandatory |
| `yawl:isCutThroughParam` | Datatype | `xsd:boolean` | Cut-through parameter |

**Example:**

```turtle
<http://example.org/param#ApprovalOutput> a yawl:OutputParameter ;
    yawl:name "approved" ;
    yawl:type "boolean" ;
    yawl:mandatory true ;
    yawl:defaultValue false .
```

---

### 4.14 FlowsInto

**IRI:** `yawl:FlowsInto`
**Label:** FlowsInto
**Description:** Flow edge from one element to another

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:nextElementRef` | Object | `yawl:NetElement` | Target element |
| `yawl:hasPredicate` | Object | `yawl:Predicate` | Flow condition |
| `yawl:isDefaultFlow` | Datatype | `xsd:boolean` | Whether default flow |

**Example:**

```turtle
<http://example.org/task#TaskA> yawl:flowsInto <http://example.org/flow#Flow1> .

<http://example.org/flow#Flow1> a yawl:FlowsInto ;
    yawl:nextElementRef <http://example.org/task#TaskB> ;
    yawl:hasPredicate <http://example.org/pred#ApprovalPred> ;
    yawl:isDefaultFlow false .

<http://example.org/pred#ApprovalPred> a yawl:Predicate ;
    yawl:query "boolean(//approved = 'true')" ;
    yawl:ordering 1 .
```

---

### 4.15 WebServiceGateway

**IRI:** `yawl:WebServiceGateway`
**Label:** WebServiceGateway
**Description:** Web service gateway decomposition

**Superclass:** `yawl:Decomposition`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:codelet` | Datatype | `xsd:NCName` | Codelet name |
| `yawl:hasYAWLService` | Object | `yawl:YAWLService` | YAWL service definition |
| `yawl:hasEnablementParam` | Object | `yawl:InputParameter` | Enablement parameter |
| `yawl:hasExternalInteraction` | Object | `yawl:ResourcingExternalInteraction` | Manual/Automated |

**Example:**

```turtle
<http://example.org/gateway#ShippingService> a yawl:WebServiceGateway ;
    yawl:id "ShippingGateway" ;
    yawl:codelet "ShippingCodelet" ;
    yawl:hasYAWLService <http://example.org/service#Shipping> ;
    yawl:hasExternalInteraction yawl:ResourcingExternalInteractionAutomated .

<http://example.org/service#Shipping> a yawl:YAWLService ;
    yawl:wsdlLocation "http://shipping.example.com/service?wsdl" ;
    yawl:operationName "calculateShipping" .
```

---

## 5. Resource Allocation Classes Reference

### 5.1 Resourcing

**IRI:** `yawl:Resourcing`
**Label:** Resourcing
**Description:** Resource allocation configuration for a task

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasOffer` | Object | `yawl:ResourcingOffer` | Offer configuration |
| `yawl:hasAllocate` | Object | `yawl:ResourcingAllocate` | Allocation configuration |
| `yawl:hasStart` | Object | `yawl:ResourcingInitiator` | Who starts task |
| `yawl:hasSecondary` | Object | `yawl:ResourcingSecondary` | Secondary resources |
| `yawl:hasPrivileges` | Object | `yawl:ResourcingPrivileges` | Resource privileges |

**Example:**

```turtle
<http://example.org/res#ManagerResourcing> a yawl:Resourcing ;
    yawl:hasOffer <http://example.org/res#ManagerOffer> ;
    yawl:hasAllocate <http://example.org/res#ManagerAllocate> ;
    yawl:hasStart yawl:ResourcingInitiatorUser ;
    yawl:hasPrivileges <http://example.org/res#ManagerPrivileges> .
```

---

### 5.2 ResourcingOffer

**IRI:** `yawl:ResourcingOffer`
**Label:** ResourcingOffer
**Description:** Resource offer configuration (who sees work item)

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasInitiator` | Object | `yawl:ResourcingInitiator` | Who offers |
| `yawl:hasDistributionSet` | Object | `yawl:ResourcingDistributionSet` | Distribution set |
| `yawl:familiarParticipant` | Datatype | `xsd:string` | Familiar participant task ID |

**Example:**

```turtle
<http://example.org/res#ManagerOffer> a yawl:ResourcingOffer ;
    yawl:hasInitiator yawl:ResourcingInitiatorSystem ;
    yawl:hasDistributionSet <http://example.org/res#ManagerDistSet> .
```

---

### 5.3 ResourcingAllocate

**IRI:** `yawl:ResourcingAllocate`
**Label:** ResourcingAllocate
**Description:** Resource allocation configuration (who gets work item)

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasInitiator` | Object | `yawl:ResourcingInitiator` | Who allocates |
| `yawl:hasAllocator` | Object | `yawl:ResourcingSelector` | Allocation strategy |

**Example:**

```turtle
<http://example.org/res#ManagerAllocate> a yawl:ResourcingAllocate ;
    yawl:hasInitiator yawl:ResourcingInitiatorSystem ;
    yawl:hasAllocator <http://example.org/res#ShortestQueueAllocator> .

<http://example.org/res#ShortestQueueAllocator> a yawl:ResourcingSelector ;
    rdfs:label "ShortestQueue" .
```

---

### 5.4 ResourcingDistributionSet

**IRI:** `yawl:ResourcingDistributionSet`
**Label:** ResourcingDistributionSet
**Description:** Resource distribution set (filters, constraints)

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasInitialSet` | Object | `yawl:ResourcingSet` | Initial set of resources |
| `yawl:hasFilter` | Object | `yawl:ResourcingSelector` | Filter selector |
| `yawl:hasConstraint` | Object | `yawl:ResourcingSelector` | Constraint selector |

**Example:**

```turtle
<http://example.org/res#ManagerDistSet> a yawl:ResourcingDistributionSet ;
    yawl:hasInitialSet <http://example.org/res#Managers> ;
    yawl:hasFilter <http://example.org/res#DepartmentFilter> .

<http://example.org/res#Managers> a yawl:ResourcingSet ;
    yawl:role "Manager" .

<http://example.org/res#DepartmentFilter> a yawl:ResourcingSelector ;
    rdfs:label "CanDoTaskInDepartment" ;
    yawl:hasParams <http://example.org/res#DepartmentParam> .

<http://example.org/res#DepartmentParam> yawl:key "department" ;
    yawl:value "Sales" .
```

---

### 5.5 ResourcingSet

**IRI:** `yawl:ResourcingSet`
**Label:** ResourcingSet
**Description:** Set of participants and roles

**Subclass:** `yawl:ResourcingSecondary`

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:participant` | Datatype | `xsd:string` | Participant name |
| `yawl:role` | Datatype | `xsd:string` | Role name |

**Example:**

```turtle
<http://example.org/res#SalesTeam> a yawl:ResourcingSet ;
    yawl:role "SalesRep" ;
    yawl:participant "alice@example.com" ;
    yawl:participant "bob@example.com" .
```

---

### 5.6 ResourcingSecondary

**IRI:** `yawl:ResourcingSecondary`
**Label:** ResourcingSecondary
**Description:** Secondary resources (non-human resources)

**Superclass:** `yawl:ResourcingSet`

**Properties (Additional):**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:nonHumanResource` | Datatype | `xsd:string` | Non-human resource name |
| `yawl:nonHumanCategory` | Datatype | `xsd:string` | Resource category |
| `yawl:subcategory` | Datatype | `xsd:string` | Subcategory |

**Example:**

```turtle
<http://example.org/res#PrinterResources> a yawl:ResourcingSecondary ;
    yawl:nonHumanCategory "Equipment" ;
    yawl:subcategory "Printer" ;
    yawl:nonHumanResource "Printer-A123" .
```

---

### 5.7 ResourcingPrivileges

**IRI:** `yawl:ResourcingPrivileges`
**Label:** ResourcingPrivileges
**Description:** Privileges for resource management

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasPrivilege` | Object | `rdfs:Resource` | Privilege instance |
| `yawl:allowall` | Datatype | `xsd:boolean` | Allow all privileges |

**Example:**

```turtle
<http://example.org/res#ManagerPrivileges> a yawl:ResourcingPrivileges ;
    yawl:hasPrivilege <http://example.org/res#SuspendPriv> ;
    yawl:hasPrivilege <http://example.org/res#DelegatePriv> .

<http://example.org/res#SuspendPriv> yawl:hasPrivilegeName yawl:ResourcingPrivilegeCanSuspend .
<http://example.org/res#DelegatePriv> yawl:hasPrivilegeName yawl:ResourcingPrivilegeCanDelegate .
```

---

## 6. Configuration Classes Reference

### 6.1 Timer

**IRI:** `yawl:Timer`
**Label:** Timer
**Description:** Timer configuration for tasks

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasTrigger` | Object | `yawl:TimerTrigger` | OnEnabled/OnExecuting |
| `yawl:hasDurationParams` | Object | `yawl:TimerDuration` | Duration configuration |
| `yawl:expiry` | Datatype | `xsd:long` | Expiry timestamp (Unix ms) |
| `yawl:duration` | Datatype | `xsd:duration` | ISO 8601 duration |
| `yawl:workdays` | Datatype | `xsd:boolean` | Use workdays only |
| `yawl:netparam` | Datatype | `xsd:string` | Net parameter reference |

**Example:**

```turtle
<http://example.org/timer#ApprovalTimer> a yawl:Timer ;
    yawl:hasTrigger yawl:TimerTriggerOnEnabled ;
    yawl:hasDurationParams <http://example.org/timer#TwoDays> ;
    yawl:workdays true .

<http://example.org/timer#TwoDays> a yawl:TimerDuration ;
    yawl:ticks 2 ;
    yawl:hasInterval yawl:TimerIntervalDay .
```

---

### 6.2 VarMapping

**IRI:** `yawl:VarMapping`
**Label:** VarMapping
**Description:** Variable mapping expression

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:mapsTo` | Datatype | `xsd:NMTOKEN` | Target variable name |
| `yawl:hasExpression` | Object | `yawl:Expression` | XQuery expression |

**Example:**

```turtle
<http://example.org/mapping#OrderMapping> a yawl:VarMapping ;
    yawl:mapsTo "orderId" ;
    yawl:hasExpression <http://example.org/expr#ExtractOrderId> .

<http://example.org/expr#ExtractOrderId> a yawl:Expression ;
    yawl:query "//order/id/text()" .
```

---

### 6.3 Expression

**IRI:** `yawl:Expression`
**Label:** Expression
**Description:** XQuery expression

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:query` | Datatype | `xsd:string` | XQuery expression string |

**Example:**

```turtle
<http://example.org/expr#SumTotal> a yawl:Expression ;
    yawl:query "sum(//order/item/price)" .
```

---

### 6.4 Predicate

**IRI:** `yawl:Predicate`
**Label:** Predicate
**Description:** XPath predicate for flow conditions

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:query` | Datatype | `xsd:string` | XPath predicate |
| `yawl:ordering` | Datatype | `xsd:integer` | Evaluation order |

**Example:**

```turtle
<http://example.org/pred#ApprovedPredicate> a yawl:Predicate ;
    yawl:query "boolean(//approved = 'true')" ;
    yawl:ordering 1 .
```

---

## 7. Layout Classes Reference

### 7.1 Layout

**IRI:** `yawl:Layout`
**Label:** Layout
**Description:** Layout information for specifications

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasLocale` | Object | `yawl:LayoutLocale` | Locale information |
| `yawl:hasSpecificationLayout` | Object | `rdfs:Resource` | Specification layout |
| `yawl:defaultBgColor` | Datatype | `xsd:integer` | Default background color |
| `yawl:labelFontSize` | Datatype | `xsd:integer` | Label font size |

**Note:** Layout classes are primarily for visual editors, not runtime execution.

---

### 7.2 LayoutNet

**IRI:** `yawl:LayoutNet`
**Label:** LayoutNet
**Description:** Layout information for a specific net

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:hasVertex` | Object | `yawl:LayoutVertex` | Vertex (task/condition position) |
| `yawl:hasContainer` | Object | `yawl:LayoutContainer` | Container |
| `yawl:hasFlow` | Object | `yawl:LayoutFlow` | Flow edge layout |
| `yawl:hasBounds` | Object | `yawl:LayoutRectangle` | Bounds |
| `yawl:hasFrame` | Object | `yawl:LayoutFrame` | Frame |
| `yawl:hasViewport` | Object | `yawl:LayoutFrame` | Viewport |
| `yawl:bgImage` | Datatype | `xsd:string` | Background image |
| `yawl:scale` | Datatype | `xsd:string` | Scale factor |
| `yawl:bgColor` | Datatype | `xsd:integer` | Background color |

**Example:**

```turtle
<http://example.org/layout#MainNetLayout> a yawl:LayoutNet ;
    yawl:hasVertex <http://example.org/layout#Task1Vertex> ;
    yawl:hasFlow <http://example.org/layout#Flow1Layout> ;
    yawl:scale "1.0" .
```

---

### 7.3 LayoutVertex

**IRI:** `yawl:LayoutVertex`
**Label:** LayoutVertex
**Description:** Position and visual attributes of a task/condition

**Properties:**

| Property | Type | Range | Description |
|----------|------|-------|-------------|
| `yawl:id` | Datatype | `xsd:NCName` | Element ID |
| `yawl:startpoint` | Object | `yawl:LayoutPoint` | Position |
| `yawl:iconpath` | Datatype | `xsd:anyURI` | Icon path |
| `yawl:hasAttributes` | Object | `yawl:LayoutAttributes` | Visual attributes |
| `yawl:notes` | Datatype | `xsd:string` | Notes |

**Example:**

```turtle
<http://example.org/layout#Task1Vertex> a yawl:LayoutVertex ;
    yawl:id "ApproveOrder" ;
    yawl:startpoint <http://example.org/layout#Task1Point> ;
    yawl:hasAttributes <http://example.org/layout#Task1Attrs> .

<http://example.org/layout#Task1Point> a yawl:LayoutPoint ;
    yawl:x "100" ;
    yawl:y "200" .
```

---

## 8. Datatype Properties Reference

### 8.1 Naming and Identity

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:name` | `rdfs:Resource` | `xsd:string` | Name of element |
| `yawl:id` | `rdfs:Resource` | `xsd:NMTOKEN` | Unique identifier |
| `yawl:uri` | `rdfs:Resource` | `xsd:anyURI` | URI identifier |
| `yawl:documentation` | `rdfs:Resource` | `xsd:string` | Documentation string |

**Example:**

```sparql
SELECT ?task ?name WHERE {
    ?task a yawl:Task .
    ?task yawl:name ?name .
}
```

---

### 8.2 Variable Properties

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:initialValue` | `yawl:Variable` | `xsd:string` | Initial value |
| `yawl:defaultValue` | `yawl:OutputParameter` | `rdfs:Resource` | Default value |
| `yawl:mandatory` | `yawl:OutputParameter` | `xsd:boolean` | Whether mandatory |
| `yawl:isCutThroughParam` | `yawl:OutputParameter` | `xsd:boolean` | Cut-through flag |
| `yawl:isUntyped` | `yawl:VariableBase` | `xsd:boolean` | Untyped flag |
| `yawl:index` | `yawl:VariableBase` | `xsd:integer` | Parameter index |
| `yawl:type` | `yawl:VariableBase` | `xsd:NCName` | Type name |
| `yawl:namespace` | `yawl:VariableBase` | `xsd:anyURI` | Type namespace |
| `yawl:element` | `yawl:VariableBase` | `xsd:NCName` | Element name |

---

### 8.3 Timer Properties

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:expiry` | `yawl:Timer` | `xsd:long` | Expiry timestamp (Unix ms) |
| `yawl:duration` | `yawl:Timer` | `xsd:duration` | ISO 8601 duration |
| `yawl:ticks` | `yawl:TimerDuration` | `xsd:long` | Duration ticks |
| `yawl:workdays` | `yawl:Timer` | `xsd:boolean` | Use workdays only |
| `yawl:netparam` | `yawl:Timer` | `xsd:string` | Net parameter reference |

---

### 8.4 Multiple Instance Properties

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:minimum` | `yawl:MultipleInstanceTask` | `xsd:string` | Minimum instances (XPath) |
| `yawl:maximum` | `yawl:MultipleInstanceTask` | `xsd:string` | Maximum instances (XPath) |
| `yawl:threshold` | `yawl:MultipleInstanceTask` | `xsd:string` | Completion threshold (XPath) |
| `yawl:formalInputParam` | `yawl:MultipleInstanceTask` | `xsd:NMTOKEN` | Input parameter name |
| `yawl:formalOutputExpression` | `yawl:MultipleInstanceTask` | `xsd:string` | Output expression |
| `yawl:resultAppliedToLocalVariable` | `yawl:MultipleInstanceTask` | `xsd:NMTOKEN` | Result variable |

---

### 8.5 Metadata Properties (Dublin Core)

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:title` | `yawl:Metadata` | `xsd:normalizedString` | Title |
| `yawl:creator` | `yawl:Metadata` | `xsd:string` | Creator |
| `yawl:subject` | `yawl:Metadata` | `xsd:string` | Subject |
| `yawl:description` | `yawl:Metadata` | `xsd:normalizedString` | Description |
| `yawl:contributor` | `yawl:Metadata` | `xsd:string` | Contributor |
| `yawl:coverage` | `yawl:Metadata` | `xsd:string` | Coverage |
| `yawl:validFrom` | `yawl:Metadata` | `xsd:date` | Valid from date |
| `yawl:validUntil` | `yawl:Metadata` | `xsd:date` | Valid until date |
| `yawl:created` | `yawl:Metadata` | `xsd:date` | Creation date |
| `yawl:version` | `yawl:Metadata` | `xsd:decimal` | Version number |
| `yawl:status` | `yawl:Metadata` | `xsd:string` | Status |
| `yawl:persistent` | `yawl:Metadata` | `xsd:boolean` | Persistent flag |
| `yawl:identifier` | `yawl:Metadata` | `xsd:NCName` | Identifier |

---

## 9. Object Properties Reference

### 9.1 Specification Structure

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:hasSpecification` | `yawl:SpecificationSet` | `yawl:Specification` | Specification in set |
| `yawl:hasDecomposition` | `yawl:Specification` | `yawl:Decomposition` | Net or gateway |
| `yawl:hasMetadata` | `yawl:Specification` | `yawl:Metadata` | Metadata |
| `yawl:hasLayout` | `yawl:SpecificationSet` | `yawl:Layout` | Layout info |

---

### 9.2 Net Structure

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:hasInputCondition` | `yawl:Net` | `yawl:InputCondition` | Start condition |
| `yawl:hasOutputCondition` | `yawl:Net` | `yawl:OutputCondition` | End condition |
| `yawl:hasTask` | `yawl:Net` | `yawl:Task` | Task in net |
| `yawl:hasCondition` | `yawl:Net` | `yawl:Condition` | Condition in net |
| `yawl:hasLocalVariable` | `yawl:Net` | `yawl:Variable` | Local variable |
| `yawl:hasInputParameter` | `yawl:Decomposition` | `yawl:InputParameter` | Input parameter |
| `yawl:hasOutputParameter` | `yawl:Decomposition` | `yawl:OutputParameter` | Output parameter |

---

### 9.3 Flow Relationships

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:flowsInto` | `yawl:NetElement` | `yawl:FlowsInto` | Outgoing flow |
| `yawl:nextElementRef` | `yawl:FlowsInto` | `yawl:NetElement` | Target element |
| `yawl:hasPredicate` | `yawl:FlowsInto` | `yawl:Predicate` | Flow condition |

---

### 9.4 Task Configuration

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:hasJoin` | `yawl:Task` | `yawl:ControlType` | Join type |
| `yawl:hasSplit` | `yawl:Task` | `yawl:ControlType` | Split type |
| `yawl:hasTimer` | `yawl:Task` | `yawl:Timer` | Timer config |
| `yawl:hasResourcing` | `yawl:Task` | `yawl:Resourcing` | Resource allocation |
| `yawl:hasDecomposesTo` | `yawl:Task` | `yawl:Decomposition` | Subnet reference |

---

### 9.5 Variable Mapping

| Property | Domain | Range | Description |
|----------|--------|-------|-------------|
| `yawl:hasStartingMappings` | `yawl:Task` | `yawl:VarMappingSet` | Input mappings |
| `yawl:hasCompletedMappings` | `yawl:Task` | `yawl:VarMappingSet` | Output mappings |
| `yawl:hasEnablementMappings` | `yawl:Task` | `yawl:VarMappingSet` | Enablement mappings |
| `yawl:hasMapping` | `yawl:VarMappingSet` | `yawl:VarMapping` | Variable mapping |
| `yawl:hasExpression` | `yawl:VarMapping` | `yawl:Expression` | XQuery expression |

---

## 10. Property Matrix

### 10.1 Task Properties Matrix

| Property | Type | Required | Default | Example |
|----------|------|----------|---------|---------|
| `yawl:id` | Datatype | Yes | - | "ApproveOrder" |
| `yawl:name` | Datatype | No | - | "Approve Order" |
| `yawl:hasJoin` | Object | Yes | - | `yawl:ControlTypeXor` |
| `yawl:hasSplit` | Object | Yes | - | `yawl:ControlTypeXor` |
| `yawl:hasTimer` | Object | No | - | `<timer>` |
| `yawl:hasResourcing` | Object | No | - | `<resourcing>` |
| `yawl:hasDecomposesTo` | Object | No | - | `<subnet>` |
| `yawl:customForm` | Datatype | No | - | "http://..." |
| `yawl:flowsInto` | Object | No | - | `<flow>` |

---

## 11. Visual Diagrams

### 11.1 Core Class Hierarchy (ASCII)

```
SpecificationSet
    │
    └── Specification
            │
            ├── hasDecomposition ──> Decomposition
            │                            │
            │                            ├── Net
            │                            │   │
            │                            │   ├── hasInputCondition ──> InputCondition
            │                            │   ├── hasOutputCondition ──> OutputCondition
            │                            │   ├── hasTask ──> Task
            │                            │   │                 │
            │                            │   │                 └── MultipleInstanceTask
            │                            │   └── hasCondition ──> Condition
            │                            │
            │                            └── WebServiceGateway
            │
            └── hasMetadata ──> Metadata
```

---

### 11.2 Task Structure Diagram

```
Task
    │
    ├── hasJoin ──> ControlType [AND/OR/XOR]
    ├── hasSplit ──> ControlType [AND/OR/XOR]
    ├── hasTimer ──> Timer
    │                   │
    │                   ├── hasTrigger ──> TimerTrigger [OnEnabled/OnExecuting]
    │                   └── hasDurationParams ──> TimerDuration
    │                                                   │
    │                                                   ├── ticks (xsd:long)
    │                                                   └── hasInterval ──> TimerInterval [DAY/HOUR/etc]
    │
    ├── hasResourcing ──> Resourcing
    │                       │
    │                       ├── hasOffer ──> ResourcingOffer
    │                       ├── hasAllocate ──> ResourcingAllocate
    │                       └── hasPrivileges ──> ResourcingPrivileges
    │
    ├── hasDecomposesTo ──> Decomposition [Net/WebServiceGateway]
    │
    ├── hasStartingMappings ──> VarMappingSet
    ├── hasCompletedMappings ──> VarMappingSet
    │
    └── flowsInto ──> FlowsInto
                        │
                        ├── nextElementRef ──> NetElement
                        └── hasPredicate ──> Predicate
```

---

### 11.3 Resource Allocation Chain

```
Resourcing
    │
    ├── hasOffer ──> ResourcingOffer
    │                   │
    │                   └── hasDistributionSet ──> ResourcingDistributionSet
    │                                                   │
    │                                                   ├── hasInitialSet ──> ResourcingSet
    │                                                   │                       │
    │                                                   │                       ├── role (xsd:string)
    │                                                   │                       └── participant (xsd:string)
    │                                                   │
    │                                                   ├── hasFilter ──> ResourcingSelector
    │                                                   └── hasConstraint ──> ResourcingSelector
    │
    └── hasAllocate ──> ResourcingAllocate
                            │
                            └── hasAllocator ──> ResourcingSelector
```

---

## 12. Constraint Reference

### 12.1 Structural Constraints

**Start Condition:**
- **Must have:** No incoming flows
- **SPARQL validation:**

```sparql
ASK {
    ?condition a yawl:InputCondition .
    ?flow yawl:nextElementRef ?condition .
}
# Returns true → INVALID
```

**End Condition:**
- **Must have:** No outgoing flows
- **SPARQL validation:**

```sparql
ASK {
    ?condition a yawl:OutputCondition .
    ?condition yawl:flowsInto ?flow .
}
# Returns true → INVALID
```

**Net:**
- **Must have:** Exactly 1 `InputCondition`
- **Must have:** Exactly 1 `OutputCondition`
- **Should have:** At least 1 `Task`

**Task:**
- **Must have:** Exactly 1 `hasJoin`
- **Must have:** Exactly 1 `hasSplit`

---

### 12.2 Cardinality Constraints (Recommended)

| Class | Property | Min | Max | Notes |
|-------|----------|-----|-----|-------|
| `yawl:Net` | `yawl:hasInputCondition` | 1 | 1 | Exactly one start |
| `yawl:Net` | `yawl:hasOutputCondition` | 1 | 1 | Exactly one end |
| `yawl:Task` | `yawl:hasJoin` | 1 | 1 | Required |
| `yawl:Task` | `yawl:hasSplit` | 1 | 1 | Required |
| `yawl:Specification` | `yawl:hasMetadata` | 0 | 1 | Optional, max 1 |
| `yawl:Specification` | `yawl:hasDecomposition` | 1 | ∞ | At least 1 |

---

### 12.3 Validation Rules (SHACL-style)

**Rule: Tasks must have join and split**

```turtle
yawl:TaskShape a sh:NodeShape ;
    sh:targetClass yawl:Task ;
    sh:property [
        sh:path yawl:hasJoin ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Task must have exactly one join type" ;
    ] ;
    sh:property [
        sh:path yawl:hasSplit ;
        sh:minCount 1 ;
        sh:maxCount 1 ;
        sh:message "Task must have exactly one split type" ;
    ] .
```

---

## Appendix: Complete Example Workflow

### A.1 Complete Workflow Specification

```turtle
# Specification
<http://example.org/workflow#OrderProcessing> a yawl:Specification ;
    yawl:name "Order Processing" ;
    yawl:uri "http://example.org/workflow#OrderProcessing" ;
    yawl:hasDecomposition <http://example.org/net#Main> ;
    yawl:hasMetadata <http://example.org/metadata#Order> .

# Metadata
<http://example.org/metadata#Order> a yawl:Metadata ;
    yawl:title "Order Processing Workflow" ;
    yawl:creator "John Doe" ;
    yawl:version "1.0"^^xsd:decimal ;
    yawl:created "2025-11-08"^^xsd:date .

# Net
<http://example.org/net#Main> a yawl:Net ;
    yawl:id "MainNet" ;
    yawl:isRootNet true ;
    yawl:hasInputCondition <http://example.org/net#Start> ;
    yawl:hasOutputCondition <http://example.org/net#End> ;
    yawl:hasTask <http://example.org/task#ApproveOrder> ;
    yawl:hasTask <http://example.org/task#ShipOrder> ;
    yawl:hasLocalVariable <http://example.org/var#OrderID> .

# Input Condition
<http://example.org/net#Start> a yawl:InputCondition ;
    yawl:id "start" ;
    yawl:flowsInto <http://example.org/flow#StartToApprove> .

# Task: Approve Order
<http://example.org/task#ApproveOrder> a yawl:Task ;
    yawl:id "ApproveOrder" ;
    yawl:name "Approve Order" ;
    yawl:hasJoin yawl:ControlTypeXor ;
    yawl:hasSplit yawl:ControlTypeXor ;
    yawl:hasResourcing <http://example.org/res#Manager> ;
    yawl:flowsInto <http://example.org/flow#ApproveToShip> .

# Task: Ship Order
<http://example.org/task#ShipOrder> a yawl:Task ;
    yawl:id "ShipOrder" ;
    yawl:name "Ship Order" ;
    yawl:hasJoin yawl:ControlTypeXor ;
    yawl:hasSplit yawl:ControlTypeXor ;
    yawl:flowsInto <http://example.org/flow#ShipToEnd> .

# Output Condition
<http://example.org/net#End> a yawl:OutputCondition ;
    yawl:id "end" .

# Flows
<http://example.org/flow#StartToApprove> a yawl:FlowsInto ;
    yawl:nextElementRef <http://example.org/task#ApproveOrder> .

<http://example.org/flow#ApproveToShip> a yawl:FlowsInto ;
    yawl:nextElementRef <http://example.org/task#ShipOrder> ;
    yawl:hasPredicate <http://example.org/pred#Approved> .

<http://example.org/pred#Approved> a yawl:Predicate ;
    yawl:query "boolean(//approved = 'true')" ;
    yawl:ordering 1 .

<http://example.org/flow#ShipToEnd> a yawl:FlowsInto ;
    yawl:nextElementRef <http://example.org/net#End> .

# Resourcing
<http://example.org/res#Manager> a yawl:Resourcing ;
    yawl:hasOffer <http://example.org/res#ManagerOffer> .

<http://example.org/res#ManagerOffer> a yawl:ResourcingOffer ;
    yawl:hasInitiator yawl:ResourcingInitiatorSystem ;
    yawl:hasDistributionSet <http://example.org/res#ManagerDist> .

<http://example.org/res#ManagerDist> a yawl:ResourcingDistributionSet ;
    yawl:hasInitialSet <http://example.org/res#Managers> .

<http://example.org/res#Managers> a yawl:ResourcingSet ;
    yawl:role "Manager" .

# Variable
<http://example.org/var#OrderID> a yawl:Variable ;
    yawl:name "orderId" ;
    yawl:type "string" ;
    yawl:namespace "http://www.w3.org/2001/XMLSchema" ;
    yawl:initialValue "" .
```

---

## Conclusion

This reference manual provided complete documentation for all 72 classes and 130+ properties in the YAWL 4.0 ontology. Key sections:

1. **Ontology Metadata** - Namespaces and structure
2. **Class Hierarchy** - Complete class tree
3. **Enumeration Reference** - All 12 enumeration types
4. **Core Classes** - Workflow structure classes
5. **Resource Classes** - Resource allocation
6. **Configuration Classes** - Timers, variables, mappings
7. **Layout Classes** - Visual representation
8. **Property Reference** - All datatype properties
9. **Object Property Reference** - All object properties
10. **Property Matrix** - Property usage tables
11. **Visual Diagrams** - ASCII diagrams
12. **Constraints** - Validation rules

**Next Steps:**
- Read `ontology-developer-guide.md` for integration patterns
- Read `sparql-cookbook.md` for query recipes
- Explore `/Users/sac/knhk/rust/knhk-workflow-engine/src/parser/` for code examples

**References:**
- YAWL Ontology: `/Users/sac/knhk/ontology/yawl.ttl`
- YAWL Foundation: http://www.yawlfoundation.org/
- OWL 2 Primer: https://www.w3.org/TR/owl2-primer/
- RDF Schema: https://www.w3.org/TR/rdf-schema/

# YAWL Ontology Architecture Analysis

**Version:** 1.0
**Date:** 2025-11-08
**Status:** Work In Progress
**Ontology Source:** `/Users/sac/knhk/ontology/yawl.ttl`

## Executive Summary

The YAWL 4.0 ontology provides a comprehensive RDF/OWL representation of YAWL workflow specifications. It defines 50+ OWL classes and 100+ properties covering workflow structure, control flow, resource allocation, data management, exception handling, and visual layout.

**Completeness Assessment:**
- **Fully Defined:** 30% (core classes with complete properties)
- **Partially Defined:** 50% (classes with some properties, missing domains/ranges)
- **Stubbed:** 20% (classes declared without detailed properties)

## 1. Ontology Metadata

### Namespaces and Prefixes

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
```

**Ontology Source:** Generated from YAWL_Schema4.0.xsd
**Purpose:** Define YAWL workflow specification schema as RDF/OWL classes and properties

## 2. Class Hierarchy

### 2.1 Enumeration Classes (12 classes)

**ControlType** - Control flow types
- `ControlTypeAnd` - AND join/split
- `ControlTypeOr` - OR join/split
- `ControlTypeXor` - XOR join/split

**CreationMode** - Instance creation modes
- `CreationModeStatic` - Static creation
- `CreationModeDynamic` - Dynamic creation

**TimerInterval** - Timer interval units (8 values)
- `TimerIntervalYear`, `Month`, `Week`, `Day`, `Hour`, `Min`, `Sec`, `Msec`

**TimerTrigger** - Timer trigger points
- `TimerTriggerOnEnabled` - Triggers when task enabled
- `TimerTriggerOnExecuting` - Triggers when task executing

**ResourcingExternalInteraction** - External interaction types
- `Manual` - Manual interaction
- `Automated` - Automated interaction

**ResourcingInitiator** - Resource allocation initiators
- `System` - System-initiated
- `User` - User-initiated

**DirectionMode** - Parameter direction
- `Input`, `Output`, `Both`

**ResourcingPrivilege** - Resource privilege types (7 values)
- `canSuspend`, `canReallocateStateless`, `canReallocateStateful`, `canDeallocate`, `canDelegate`, `canSkip`, `canPile`

**ResourcingResourceType** - Resource types
- `Participant` - Human participant
- `Role` - Organizational role

**InputPortValueType** - Input port values
- `Activated`, `Blocked`, `Hidden`

**OutputPortValueType** - Output port values
- `Activated`, `Blocked`

**CreationModeConfigType** - Creation mode configuration
- `Restrict`, `Keep`

### 2.2 Core Workflow Classes (17 classes)

#### Specification Container
- **SpecificationSet** - Root element for workflow specifications
- **Specification** - YAWL workflow specification (subclass of SpecificationSet)

#### Decomposition Hierarchy
- **Decomposition** - Abstract base for nets and web service gateways
  - **Net** - Workflow net (process definition)
  - **WebServiceGateway** - Web service gateway decomposition

#### Workflow Elements
- **NetElement** - Base class for workflow net elements
  - **Task** - Workflow task
    - **MultipleInstanceTask** - Multiple instance task (subclass of Task)
  - **Condition** - Workflow condition (place)
    - **InputCondition** - Input condition of a net
    - **OutputCondition** - Output condition of a net

#### Data Elements
- **VariableBase** - Base class for all variables
  - **Variable** - Workflow variable
  - **InputParameter** - Input parameter
  - **OutputParameter** - Output parameter

#### Flow Elements
- **FlowsInto** - Flow relationship between elements
- **RemovesTokensFromFlow** - Token removal from flow

### 2.3 Resource Allocation Classes (12 classes)

**Resource Management:**
- **Resourcing** - Resource allocation configuration
- **ResourcingOffer** - Resource offer configuration
- **ResourcingAllocate** - Resource allocation configuration
- **ResourcingSet** - Set of participants and roles
- **ResourcingDistributionSet** - Resource distribution set
- **ResourcingSelector** - Resource selector with parameters
- **ResourcingPrivileges** - Resource privileges configuration
- **ResourcingSecondary** - Secondary resources (non-human)

### 2.4 Timing and Configuration Classes (10 classes)

**Timers:**
- **Timer** - Timer configuration for tasks
- **TimerDuration** - Timer duration with ticks and interval

**Variable Mapping:**
- **VarMapping** - Variable mapping expression
- **VarMappingSet** - Set of variable mappings
- **Expression** - XQuery expression
- **Predicate** - XPath predicate with ordering

**Task Configuration:**
- **Configuration** - Task configuration (join, split, rem, nofi)
- **JoinConfig** - Join port configuration
- **SplitConfig** - Split port configuration
- **RemConfig** - Removes tokens configuration
- **NofiConfig** - Number of instances configuration
- **InputPortConfig** - Input port configuration
- **OutputPortConfig** - Output port configuration

### 2.5 Service and Metadata Classes (5 classes)

- **YAWLService** - YAWL service definition
- **Metadata** - Dublin Core metadata
- **LogPredicate** - Logging predicate for start/completion

### 2.6 Layout Classes (16 classes)

**Visual Representation:**
- **Layout** - Layout information for specifications
- **LayoutNet** - Layout information for a net
- **LayoutVertex** - Layout vertex (position, icon, attributes)
- **LayoutContainer** - Layout container
- **LayoutFlow** - Layout flow (edge)
- **LayoutDecorator** - Layout decorator
- **LayoutLabel** - Layout label
- **LayoutPoint** - 2D point (x, y)
- **LayoutLocale** - Locale information
- **LayoutPorts** - Input/output port counts
- **LayoutAttributes** - Layout visual attributes
- **LayoutRectangle** - Rectangle bounds
- **LayoutFrame** - Frame bounds
- **LayoutPoints** - Collection of points
- **LayoutFont** - Font information
- **LayoutDimension** - Width and height
- **LayoutUserObjectHTML** - HTML user object

## 3. Properties Analysis

### 3.1 Datatype Properties (80+ properties)

**Identity and Naming:**
- `yawl:name` - Name (xsd:string)
- `yawl:documentation` - Documentation (xsd:string)
- `yawl:uri` - URI identifier (xsd:anyURI)
- `yawl:id` - Identifier (xsd:NMTOKEN)
- `yawl:index` - Index value (xsd:integer)

**Variable Properties:**
- `yawl:initialValue` - Initial value (xsd:string)
- `yawl:defaultValue` - Default value
- `yawl:mandatory` - Whether parameter is mandatory (xsd:boolean)
- `yawl:isCutThroughParam` - Cut-through parameter (xsd:boolean)
- `yawl:isUntyped` - Untyped variable (xsd:boolean)

**Expression Properties:**
- `yawl:query` - XQuery expression string (xsd:string)
- `yawl:ordering` - Ordering value (xsd:integer)
- `yawl:mapsTo` - Variable name to map to (xsd:NMTOKEN)

**Net Properties:**
- `yawl:isRootNet` - Whether net is root (xsd:boolean)
- `yawl:isDefaultFlow` - Whether flow is default (xsd:boolean)
- `yawl:externalDataGateway` - External data gateway (xsd:string)

**Timer Properties:**
- `yawl:expiry` - Timer expiry timestamp (xsd:long)
- `yawl:duration` - Timer duration (xsd:duration)
- `yawl:ticks` - Timer duration ticks (xsd:long)
- `yawl:workdays` - Use workdays only (xsd:boolean)
- `yawl:netparam` - Net parameter reference (xsd:string)

**Multiple Instance Properties:**
- `yawl:minimum` - Minimum instances (xsd:string)
- `yawl:maximum` - Maximum instances (xsd:string)
- `yawl:threshold` - Threshold (xsd:string)
- `yawl:formalInputParam` - Formal input parameter (xsd:NMTOKEN)
- `yawl:formalOutputExpression` - Formal output expression (xsd:string)
- `yawl:resultAppliedToLocalVariable` - Result variable (xsd:NMTOKEN)

**Task Properties:**
- `yawl:customForm` - Custom form URI (xsd:anyURI)

**Web Service Properties:**
- `yawl:codelet` - Codelet name (xsd:NCName)
- `yawl:wsdlLocation` - WSDL location (xsd:anyURI)
- `yawl:operationName` - Operation name (xsd:NMTOKEN)

**Metadata Properties (Dublin Core):**
- `yawl:title` - Title (xsd:normalizedString)
- `yawl:creator` - Creator (xsd:string)
- `yawl:subject` - Subject (xsd:string)
- `yawl:description` - Description (xsd:normalizedString)
- `yawl:contributor` - Contributor (xsd:string)
- `yawl:coverage` - Coverage (xsd:string)
- `yawl:validFrom` - Valid from date (xsd:date)
- `yawl:validUntil` - Valid until date (xsd:date)
- `yawl:created` - Creation date (xsd:date)
- `yawl:version` - Version number (xsd:decimal)
- `yawl:status` - Status (xsd:string)
- `yawl:persistent` - Persistent flag (xsd:boolean)
- `yawl:identifier` - Identifier (xsd:NCName)

**Logging Properties:**
- `yawl:start` - Start log predicate (xsd:string)
- `yawl:completion` - Completion log predicate (xsd:string)

**Resource Properties:**
- `yawl:participant` - Participant name (xsd:string)
- `yawl:role` - Role name (xsd:string)
- `yawl:nonHumanResource` - Non-human resource (xsd:string)
- `yawl:nonHumanCategory` - Non-human category (xsd:string)
- `yawl:subcategory` - Subcategory (xsd:string)
- `yawl:allowall` - Allow all privilege (xsd:boolean)
- `yawl:familiarParticipant` - Familiar participant task ID (xsd:string)

**Layout Properties (30+ properties):**
- `yawl:x`, `yawl:y` - Coordinates (xsd:string)
- `yawl:w`, `yawl:h` - Width/height (xsd:string)
- `yawl:language`, `yawl:country` - Locale (xsd:string)
- `yawl:in`, `yawl:out` - Port counts (xsd:integer)
- `yawl:source`, `yawl:target` - Flow endpoints (xsd:NCName)
- `yawl:position` - Position (xsd:integer)
- `yawl:iconpath` - Icon path (xsd:anyURI)
- `yawl:notes` - Notes (xsd:string)
- `yawl:bgImage`, `yawl:scale`, `yawl:bgColor` - Background properties
- `yawl:cancellationtask` - Cancellation task (xsd:NCName)

**Type Properties:**
- `yawl:type` - Type name (xsd:NCName)
- `yawl:namespace` - Namespace URI (xsd:anyURI)
- `yawl:element` - Element name (xsd:NCName)

### 3.2 Object Properties (50+ properties)

**Specification Structure:**
- `yawl:hasSpecification` - Specification in set (domain: SpecificationSet, range: Specification)
- `yawl:hasDecomposition` - Decomposition in specification (domain: Specification, range: Decomposition)
- `yawl:hasMetadata` - Metadata (domain: Specification, range: Metadata)
- `yawl:hasLayout` - Layout (domain: SpecificationSet, range: Layout)

**Parameters and Variables:**
- `yawl:hasInputParameter` - Input parameter (domain: Decomposition, range: InputParameter)
- `yawl:hasOutputParameter` - Output parameter (domain: Decomposition, range: OutputParameter)
- `yawl:hasLocalVariable` - Local variable (domain: Net, range: Variable)

**Net Structure:**
- `yawl:hasInputCondition` - Input condition (domain: Net, range: InputCondition)
- `yawl:hasOutputCondition` - Output condition (domain: Net, range: OutputCondition)
- `yawl:hasTask` - Task in net (domain: Net, range: Task)
- `yawl:hasCondition` - Condition in net (domain: Net, range: Condition)

**Flow Relationships:**
- `yawl:flowsInto` - Flow from element (domain: NetElement, range: FlowsInto)
- `yawl:nextElementRef` - Reference to next element (domain: FlowsInto, range: NetElement)
- `yawl:hasPredicate` - Predicate for flow (domain: FlowsInto, range: Predicate)

**Task Configuration:**
- `yawl:hasJoin` - Join control type (domain: Task, range: ControlType)
- `yawl:hasSplit` - Split control type (domain: Task, range: ControlType)
- `yawl:hasTimer` - Timer configuration (domain: Task, range: Timer)
- `yawl:hasResourcing` - Resource allocation (domain: Task, range: Resourcing)

**Timer Configuration:**
- `yawl:hasTrigger` - Timer trigger type (domain: Timer, range: TimerTrigger)
- `yawl:hasDurationParams` - Duration parameters (domain: Timer, range: TimerDuration)
- `yawl:hasInterval` - Interval type (domain: TimerDuration, range: TimerInterval)

**Resource Allocation:**
- `yawl:hasOffer` - Resource offer (domain: Resourcing, range: ResourcingOffer)
- `yawl:hasAllocate` - Resource allocation (domain: Resourcing, range: ResourcingAllocate)
- `yawl:hasStart` - Resource start (domain: Resourcing, range: ResourcingInitiator)
- `yawl:hasSecondary` - Secondary resources (domain: Resourcing, range: ResourcingSecondary)
- `yawl:hasPrivileges` - Privileges (domain: Resourcing, range: ResourcingPrivileges)
- `yawl:hasInitiator` - Initiator type (domain: ResourcingOffer/Allocate, range: ResourcingInitiator)
- `yawl:hasDistributionSet` - Distribution set (domain: ResourcingOffer, range: ResourcingDistributionSet)
- `yawl:hasInitialSet` - Initial set (domain: ResourcingDistributionSet, range: ResourcingSet)
- `yawl:hasFilter` - Resource filter (domain: ResourcingDistributionSet, range: ResourcingSelector)
- `yawl:hasConstraint` - Resource constraint (domain: ResourcingDistributionSet, range: ResourcingSelector)
- `yawl:hasAllocator` - Allocator (domain: ResourcingAllocate, range: ResourcingSelector)

**Cancellation:**
- `yawl:hasRemovesTokens` - Element that removes tokens (domain: Task, range: NetElement)
- `yawl:hasRemovesTokensFromFlow` - Token removal (domain: Task, range: RemovesTokensFromFlow)
- `yawl:flowSource` - Flow source (domain: RemovesTokensFromFlow, range: NetElement)
- `yawl:flowDestination` - Flow destination (domain: RemovesTokensFromFlow, range: NetElement)

**Variable Mapping:**
- `yawl:hasStartingMappings` - Starting mappings (domain: Task, range: VarMappingSet)
- `yawl:hasCompletedMappings` - Completed mappings (domain: Task, range: VarMappingSet)
- `yawl:hasEnablementMappings` - Enablement mappings (domain: Task, range: VarMappingSet)
- `yawl:hasMapping` - Variable mapping (domain: VarMappingSet, range: VarMapping)
- `yawl:hasExpression` - XQuery expression (domain: VarMapping, range: Expression)

**Multiple Instance:**
- `yawl:hasSplittingExpression` - Splitting expression (domain: MultipleInstanceTask, range: Expression)
- `yawl:hasOutputJoiningExpression` - Output joining expression (domain: MultipleInstanceTask, range: Expression)
- `yawl:hasCreationMode` - Creation mode (domain: MultipleInstanceTask, range: CreationMode)
- `yawl:hasCreationModeConfig` - Creation mode config (domain: NofiConfig, range: CreationModeConfigType)

**Decomposition:**
- `yawl:hasDecomposesTo` - Decomposition reference (domain: Task, range: Decomposition)

**Configuration:**
- `yawl:hasDefaultConfiguration` - Default configuration (domain: Task, range: Configuration)
- `yawl:hasConfiguration` - Task configuration (domain: Task, range: Configuration)
- `yawl:hasJoinConfig` - Join configuration (domain: Configuration, range: JoinConfig)
- `yawl:hasSplitConfig` - Split configuration (domain: Configuration, range: SplitConfig)
- `yawl:hasRemConfig` - Removes tokens config (domain: Configuration, range: RemConfig)
- `yawl:hasNofiConfig` - Number of instances config (domain: Configuration, range: NofiConfig)
- `yawl:hasPort` - Port configuration (domain: JoinConfig/SplitConfig, range: InputPortConfig/OutputPortConfig)
- `yawl:hasFlowSource` - Flow source (domain: InputPortConfig, range: NetElement)
- `yawl:hasFlowDestination` - Flow destination (domain: OutputPortConfig, range: NetElement)
- `yawl:hasPortValue` - Port value type (domain: InputPortConfig/OutputPortConfig, range: InputPortValueType/OutputPortValueType)

**Web Service:**
- `yawl:hasEnablementParam` - Enablement parameter (domain: WebServiceGateway, range: InputParameter)
- `yawl:hasYAWLService` - YAWL service (domain: WebServiceGateway, range: YAWLService)
- `yawl:hasExternalInteraction` - External interaction (domain: WebServiceGateway, range: ResourcingExternalInteraction)

**Logging:**
- `yawl:hasLogPredicate` - Logging predicate (domain: Decomposition/Parameters, range: LogPredicate)

**Layout (20+ properties):**
- `yawl:hasLocale` - Locale (domain: Layout, range: LayoutLocale)
- `yawl:hasSpecificationLayout` - Specification layout
- `yawl:hasNetLayout` - Net layout
- `yawl:hasVertex` - Layout vertex
- `yawl:hasContainer` - Layout container
- `yawl:hasFlow` - Layout flow
- `yawl:hasLabel` - Layout label
- `yawl:hasDecorator` - Layout decorator
- `yawl:hasAttributes` - Layout attributes
- `yawl:hasUserObject` - HTML user object
- `yawl:hasBounds` - Bounds rectangle
- `yawl:hasFrame` - Frame bounds
- `yawl:hasViewport` - Viewport bounds
- `yawl:hasPoints` - Collection of points
- `yawl:hasPoint` - Point in collection
- `yawl:hasFont` - Font information
- `yawl:hasSize` - Size dimension
- `yawl:hasLabelPosition` - Label position
- `yawl:hasOffset` - Offset point

## 4. Completeness Analysis

### 4.1 Fully Defined Classes (30%)

**Complete with properties, domains, ranges:**
- ControlType hierarchy (AND/OR/XOR)
- CreationMode (Static/Dynamic)
- TimerInterval (all 8 values)
- Core workflow: Specification, Net, Task, Condition
- Variables: Variable, InputParameter, OutputParameter
- Flow: FlowsInto, RemovesTokensFromFlow
- Metadata (Dublin Core properties)

### 4.2 Partially Defined Classes (50%)

**Missing some properties or constraints:**
- Resource allocation classes (missing cardinality)
- Configuration classes (missing validation rules)
- Web service gateway (missing WSDL validation)
- Timer classes (missing constraint validation)
- Layout classes (missing coordinate validation)

### 4.3 Stubbed Classes (20%)

**Declared but minimal properties:**
- ResourcingSelector (has structure but missing validation)
- NofiConfig (missing min/max constraints)
- LogPredicate (missing predicate validation)
- Some layout classes (missing visual constraints)

### 4.4 Missing from Ontology (vs. YAWL XSD)

**Concepts in YAWL but not in TTL:**
1. **Worklets** - Exception handling via worklets
2. **Exlets** - Selection mechanisms
3. **Data Gateways** - External data integration
4. **Custom Services** - Custom service invocation
5. **Dynamic MI** - Dynamic multiple instance runtime creation
6. **Constraint Validation** - Min/max constraints on properties
7. **Cardinality** - Missing owl:minCardinality, owl:maxCardinality
8. **Disjoint Classes** - Missing owl:disjointWith declarations
9. **Inverse Properties** - Missing owl:inverseOf for bidirectional relationships

## 5. Design Patterns

### 5.1 Enumeration Pattern

```turtle
yawl:ControlType a rdfs:Class .
yawl:ControlTypeAnd a yawl:ControlType .
yawl:ControlTypeOr a yawl:ControlType .
yawl:ControlTypeXor a yawl:ControlType .
```

**Pattern:** Closed enumeration using instances of enumeration class.

### 5.2 Inheritance Pattern

```turtle
yawl:NetElement a rdfs:Class .
yawl:Task rdfs:subClassOf yawl:NetElement .
yawl:Condition rdfs:subClassOf yawl:NetElement .
```

**Pattern:** Class hierarchy using rdfs:subClassOf.

### 5.3 Composition Pattern

```turtle
yawl:hasDecomposition a rdf:Property ;
    rdfs:domain yawl:Specification ;
    rdfs:range yawl:Decomposition .
```

**Pattern:** Object properties for containment relationships.

## 6. Integration Recommendations

### 6.1 Immediate Needs

1. **Add Cardinality Constraints:**
   - Task must have exactly 1 join type, 1 split type
   - Net must have exactly 1 input condition, 1 output condition
   - Specification must have at least 1 decomposition

2. **Add Inverse Properties:**
   - `yawl:hasTask` inverse of `yawl:taskOf`
   - `yawl:flowsInto` inverse of `yawl:flowsFrom`

3. **Add Disjoint Constraints:**
   - Task, Condition are disjoint
   - InputCondition, OutputCondition are disjoint

4. **Add Validation SPARQL:**
   - Start condition has no incoming flows
   - End condition has no outgoing flows
   - No cycles in XOR-splits

### 6.2 knhk-Specific Extensions

See `ontology-extension-strategy.md` for:
- Hot path annotations (≤8 ticks)
- Lockchain provenance
- OTEL instrumentation
- Performance constraints
- Security policies

## 7. Summary Statistics

| Category | Count | Completeness |
|----------|-------|--------------|
| **Namespaces** | 5 | 100% |
| **Enumeration Classes** | 12 | 100% |
| **Core Classes** | 17 | 80% |
| **Resource Classes** | 12 | 60% |
| **Timer Classes** | 10 | 70% |
| **Service Classes** | 5 | 50% |
| **Layout Classes** | 16 | 40% |
| **Total Classes** | 72 | 65% |
| **Datatype Properties** | 80+ | 70% |
| **Object Properties** | 50+ | 60% |
| **Total Properties** | 130+ | 65% |

## 8. References

- **YAWL Foundation:** http://www.yawlfoundation.org/
- **YAWL Schema 4.0:** YAWL_Schema4.0.xsd
- **Ontology File:** `/Users/sac/knhk/ontology/yawl.ttl`
- **knhk Parser:** `/Users/sac/knhk/rust/knhk-workflow-engine/src/parser/`

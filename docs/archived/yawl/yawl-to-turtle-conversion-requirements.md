# YAWL XML to Turtle/RDF Conversion Requirements

**Document Type**: Technical Specification
**Target**: knhk Parser Implementation
**Date**: 2025-11-08

---

## 1. Overview

This document specifies the conversion logic required to transform YAWL XML specifications into Turtle/RDF format compatible with knhk's workflow engine.

---

## 2. Namespace Mappings

### 2.1 Standard Prefixes

```turtle
@prefix knhk: <http://knhk.org/ontology#> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix wf: <http://www.w3.org/2005/01/wf/flow#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix dc: <http://purl.org/dc/terms/> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
```

### 2.2 Legacy Namespace Handling

**Legacy YAWL**: `http://www.citi.qut.edu.au/yawl`
**Modern YAWL**: `http://www.yawlfoundation.org/yawlschema`

**Strategy**: Normalize to modern namespace during parsing.

---

## 3. Element-to-Class Mappings

### 3.1 Decomposition Types

| YAWL XML Type | Turtle Class | RDF Type |
|---------------|--------------|----------|
| `NetFactsType` | `knhk:WorkflowNet` | Process definition |
| `WebServiceGatewayFactsType` | `knhk:ServiceTask` | External service invocation |
| `MultipleInstanceExternalTaskFactsType` | `knhk:MultiInstanceTask` | Parallel task instances |

### 3.2 Process Control Elements

| YAWL Element | Turtle Class | RDF Type |
|--------------|--------------|----------|
| `<inputCondition>` | `knhk:StartEvent` | Process start point |
| `<outputCondition>` | `knhk:EndEvent` | Process end point |
| `<task>` | `knhk:Task` | Atomic task |
| `<condition>` | `knhk:Condition` | Intermediate condition |

### 3.3 Control Flow Operators

| YAWL Join/Split Code | Turtle Individual | Semantics |
|---------------------|-------------------|-----------|
| `and` | `knhk:AND` | Parallel gateway |
| `or` | `knhk:OR` | Inclusive gateway |
| `xor` | `knhk:XOR` | Exclusive gateway |

---

## 4. Property Mappings

### 4.1 Task Properties

| YAWL Element/Attribute | Turtle Property | Range |
|----------------------|-----------------|-------|
| `<task id="...">` | `knhk:hasID` | `xsd:NCName` |
| `<name>` | `rdfs:label` | `xsd:string` |
| `<documentation>` | `rdfs:comment` | `xsd:string` |
| `<join code="...">` | `knhk:hasJoinOperator` | `knhk:ControlOperator` |
| `<split code="...">` | `knhk:hasSplitOperator` | `knhk:ControlOperator` |
| `<decomposesTo id="...">` | `knhk:decomposesTo` | `knhk:Decomposition` |

### 4.2 Flow Properties

| YAWL Element | Turtle Property | Range |
|--------------|-----------------|-------|
| `<flowsInto><nextElementRef id="...">` | `knhk:flowsInto` | `knhk:FlowElement` |
| `<predicate>` | `knhk:hasPredicate` | `xsd:string` |
| `<predicate ordering="N">` | `knhk:predicateOrdering` | `xsd:integer` |
| `<isDefaultFlow>` | `knhk:isDefaultFlow` | `xsd:boolean` |

### 4.3 Data Properties

| YAWL Element | Turtle Property | Range |
|--------------|-----------------|-------|
| `<inputParam>` | `knhk:hasInputParameter` | `knhk:Parameter` |
| `<outputParam>` | `knhk:hasOutputParameter` | `knhk:Parameter` |
| `<localVariable>` | `knhk:hasLocalVariable` | `knhk:Variable` |
| `<initialValue>` | `knhk:hasInitialValue` | `xsd:string` |

### 4.4 Mapping Properties

| YAWL Element | Turtle Property | Range |
|--------------|-----------------|-------|
| `<startingMappings>` | `knhk:hasStartingMapping` | `knhk:DataMapping` |
| `<completedMappings>` | `knhk:hasCompletedMapping` | `knhk:DataMapping` |
| `<enablementMappings>` | `knhk:hasEnablementMapping` | `knhk:DataMapping` |
| `<expression query="...">` | `knhk:hasExpression` | `xsd:string` |
| `<mapsTo>` | `knhk:mapsToParameter` | `xsd:NCName` |

---

## 5. Conversion Patterns

### 5.1 Basic Task Conversion

**YAWL XML**:
```xml
<task id="ApproveOrder">
  <name>Approve Customer Order</name>
  <documentation>Manager approves order over $1000</documentation>
  <flowsInto>
    <nextElementRef id="ProcessPayment"/>
  </flowsInto>
  <join code="xor"/>
  <split code="and"/>
  <decomposesTo id="ApproveOrderService"/>
</task>
```

**Turtle RDF**:
```turtle
:ApproveOrder a knhk:Task ;
  rdfs:label "Approve Customer Order" ;
  rdfs:comment "Manager approves order over $1000" ;
  knhk:hasJoinOperator knhk:XOR ;
  knhk:hasSplitOperator knhk:AND ;
  knhk:flowsInto :ProcessPayment ;
  knhk:decomposesTo :ApproveOrderService .
```

### 5.2 Conditional Flow Conversion

**YAWL XML**:
```xml
<flowsInto>
  <nextElementRef id="StandardShipping"/>
  <predicate ordering="1">/data/orderAmount &lt; 100</predicate>
</flowsInto>
<flowsInto>
  <nextElementRef id="ExpressShipping"/>
  <predicate ordering="2">/data/urgent = true()</predicate>
</flowsInto>
<flowsInto>
  <nextElementRef id="DefaultShipping"/>
  <isDefaultFlow/>
</flowsInto>
```

**Turtle RDF**:
```turtle
:TaskX knhk:flowsInto [
  a knhk:ConditionalFlow ;
  knhk:targetTask :StandardShipping ;
  knhk:hasPredicate "/data/orderAmount < 100"^^xsd:string ;
  knhk:predicateOrdering 1
] , [
  a knhk:ConditionalFlow ;
  knhk:targetTask :ExpressShipping ;
  knhk:hasPredicate "/data/urgent = true()"^^xsd:string ;
  knhk:predicateOrdering 2
] , [
  a knhk:ConditionalFlow ;
  knhk:targetTask :DefaultShipping ;
  knhk:isDefaultFlow true
] .
```

### 5.3 Data Mapping Conversion

**YAWL XML**:
```xml
<startingMappings>
  <mapping>
    <expression query="&lt;orderID&gt;{/data/order/id}&lt;/orderID&gt;"/>
    <mapsTo>orderId</mapsTo>
  </mapping>
  <mapping>
    <expression query="&lt;total&gt;{sum(/data/order/items/item/price)}&lt;/total&gt;"/>
    <mapsTo>orderTotal</mapsTo>
  </mapping>
</startingMappings>
```

**Turtle RDF**:
```turtle
:TaskX knhk:hasStartingMapping [
  a knhk:DataMapping ;
  knhk:hasExpression "<orderID>{/data/order/id}</orderID>"^^xsd:string ;
  knhk:mapsToParameter "orderId"^^xsd:NCName
] , [
  a knhk:DataMapping ;
  knhk:hasExpression "<total>{sum(/data/order/items/item/price)}</total>"^^xsd:string ;
  knhk:mapsToParameter "orderTotal"^^xsd:NCName
] .
```

### 5.4 Parameter Conversion

**YAWL XML**:
```xml
<inputParam>
  <index>0</index>
  <name>customerID</name>
  <type>string</type>
  <namespace>http://www.w3.org/2001/XMLSchema</namespace>
  <mandatory/>
</inputParam>
```

**Turtle RDF**:
```turtle
:TaskX knhk:hasInputParameter [
  a knhk:Parameter ;
  knhk:parameterIndex 0 ;
  knhk:parameterName "customerID"^^xsd:NCName ;
  knhk:parameterType xsd:string ;
  knhk:isMandatory true
] .
```

### 5.5 Resourcing Conversion

**YAWL XML**:
```xml
<resourcing>
  <offer initiator="system">
    <distributionSet>
      <initialSet>
        <role>Manager</role>
        <role>Supervisor</role>
      </initialSet>
    </distributionSet>
  </offer>
  <allocate initiator="user">
    <allocator>
      <name>ShortestQueue</name>
    </allocator>
  </allocate>
  <start initiator="user"/>
</resourcing>
```

**Turtle RDF**:
```turtle
:TaskX knhk:hasResourcing [
  a knhk:ResourcingSpecification ;

  knhk:hasOfferPhase [
    knhk:initiatedBy knhk:System ;
    knhk:offeredToRole "Manager"^^xsd:string , "Supervisor"^^xsd:string
  ] ;

  knhk:hasAllocatePhase [
    knhk:initiatedBy knhk:User ;
    knhk:allocator knhk:ShortestQueue
  ] ;

  knhk:hasStartPhase [
    knhk:initiatedBy knhk:User
  ]
] .
```

### 5.6 Timer Conversion

**YAWL XML**:
```xml
<timer>
  <trigger>OnEnabled</trigger>
  <durationparams>
    <ticks>2</ticks>
    <interval>HOUR</interval>
  </durationparams>
  <workdays>true</workdays>
</timer>
```

**Turtle RDF**:
```turtle
:TaskX knhk:hasTimer [
  a knhk:Timer ;
  knhk:timerTrigger knhk:OnEnabled ;
  knhk:timerDuration [
    knhk:durationTicks 2 ;
    knhk:durationInterval knhk:HOUR
  ] ;
  knhk:workdaysOnly true
] .
```

### 5.7 Multiple Instance Task Conversion

**YAWL XML**:
```xml
<task id="ProcessItems" xsi:type="MultipleInstanceExternalTaskFactsType">
  <minimum>1</minimum>
  <maximum>10</maximum>
  <threshold>5</threshold>
  <creationMode code="dynamic"/>

  <miDataInput>
    <expression query="/data/items"/>
    <splittingExpression query="/items/item"/>
    <formalInputParam>item</formalInputParam>
  </miDataInput>

  <miDataOutput>
    <formalOutputExpression query="/processItem/result"/>
    <outputJoiningExpression query="&lt;results&gt;{$results}&lt;/results&gt;"/>
    <resultAppliedToLocalVariable>allResults</resultAppliedToLocalVariable>
  </miDataOutput>
</task>
```

**Turtle RDF**:
```turtle
:ProcessItems a knhk:MultiInstanceTask ;
  knhk:minimumInstances 1 ;
  knhk:maximumInstances 10 ;
  knhk:completionThreshold 5 ;
  knhk:creationMode knhk:Dynamic ;

  knhk:hasInputSplitting [
    knhk:inputExpression "/data/items"^^xsd:string ;
    knhk:splittingExpression "/items/item"^^xsd:string ;
    knhk:formalParameter "item"^^xsd:NCName
  ] ;

  knhk:hasOutputJoining [
    knhk:outputExpression "/processItem/result"^^xsd:string ;
    knhk:joiningExpression "<results>{$results}</results>"^^xsd:string ;
    knhk:assignToVariable "allResults"^^xsd:NCName
  ] .
```

### 5.8 Cancellation Region Conversion

**YAWL XML**:
```xml
<task id="CancelOrder">
  <removesTokens id="ProcessPayment"/>
  <removesTokens id="ArrangeShipment"/>
  <removesTokensFromFlow>
    <flowSource id="ValidateOrder"/>
    <flowDestination id="ApproveOrder"/>
  </removesTokensFromFlow>
</task>
```

**Turtle RDF**:
```turtle
:CancelOrder a knhk:Task ;
  knhk:cancelsTask :ProcessPayment , :ArrangeShipment ;
  knhk:cancelsFlow [
    knhk:flowSource :ValidateOrder ;
    knhk:flowDestination :ApproveOrder
  ] .
```

---

## 6. XML Entity Handling

### 6.1 Common XML Entities

| XML Entity | Character | Conversion |
|-----------|-----------|------------|
| `&lt;` | `<` | Direct replacement |
| `&gt;` | `>` | Direct replacement |
| `&amp;` | `&` | Direct replacement |
| `&apos;` | `'` | Direct replacement |
| `&quot;` | `"` | Direct replacement |
| `$apos;` | `'` | XQuery-specific replacement |

**Example**:
```xml
<!-- YAWL XML -->
<expression query="&lt;result&gt;{/data/value &gt; 10}&lt;/result&gt;"/>
```

```turtle
# Turtle RDF
knhk:hasExpression "<result>{/data/value > 10}</result>"^^xsd:string
```

---

## 7. Metadata Conversion (Dublin Core)

**YAWL XML**:
```xml
<metaData>
  <title>Order Fulfillment</title>
  <creator>John Doe</creator>
  <creator>Jane Smith</creator>
  <description>Complete order processing workflow</description>
  <version>1.2</version>
  <created>2024-01-01</created>
  <identifier>UID_ae0b797c-2ac8-4d5e-9421-ece89d8043d0</identifier>
</metaData>
```

**Turtle RDF**:
```turtle
:OrderFulfillment a knhk:WorkflowSpecification ;
  dc:title "Order Fulfillment" ;
  dc:creator "John Doe" , "Jane Smith" ;
  dc:description "Complete order processing workflow" ;
  dc:hasVersion "1.2"^^xsd:decimal ;
  dc:created "2024-01-01"^^xsd:date ;
  dc:identifier "UID_ae0b797c-2ac8-4d5e-9421-ece89d8043d0"^^xsd:string .
```

---

## 8. Complex Example: Complete Specification

### 8.1 YAWL XML Source

```xml
<specificationSet xmlns="http://www.yawlfoundation.org/yawlschema" version="4.0">
  <specification uri="SimpleApproval">
    <name>Simple Approval Workflow</name>
    <metaData>
      <title>Simple Approval</title>
      <version>1.0</version>
    </metaData>

    <decomposition id="MainProcess" isRootNet="true" xsi:type="NetFactsType">
      <inputParam>
        <index>0</index>
        <name>requestID</name>
        <type>string</type>
        <namespace>http://www.w3.org/2001/XMLSchema</namespace>
      </inputParam>

      <outputParam>
        <index>0</index>
        <name>approved</name>
        <type>boolean</type>
        <namespace>http://www.w3.org/2001/XMLSchema</namespace>
      </outputParam>

      <processControlElements>
        <inputCondition id="Start">
          <flowsInto>
            <nextElementRef id="Review"/>
          </flowsInto>
        </inputCondition>

        <task id="Review">
          <name>Review Request</name>
          <flowsInto>
            <nextElementRef id="Approve"/>
            <predicate>/data/amount &lt; 1000</predicate>
          </flowsInto>
          <flowsInto>
            <nextElementRef id="ManagerApprove"/>
            <isDefaultFlow/>
          </flowsInto>
          <join code="xor"/>
          <split code="xor"/>
          <decomposesTo id="ReviewService"/>
        </task>

        <task id="Approve">
          <name>Approve (Standard)</name>
          <flowsInto>
            <nextElementRef id="End"/>
          </flowsInto>
          <join code="xor"/>
          <split code="and"/>
          <completedMappings>
            <mapping>
              <expression query="&lt;approved&gt;true&lt;/approved&gt;"/>
              <mapsTo>approved</mapsTo>
            </mapping>
          </completedMappings>
          <decomposesTo id="ApproveService"/>
        </task>

        <task id="ManagerApprove">
          <name>Manager Approval</name>
          <flowsInto>
            <nextElementRef id="End"/>
          </flowsInto>
          <join code="xor"/>
          <split code="and"/>
          <resourcing>
            <offer initiator="system">
              <distributionSet>
                <initialSet>
                  <role>Manager</role>
                </initialSet>
              </distributionSet>
            </offer>
            <allocate initiator="user"/>
            <start initiator="user"/>
          </resourcing>
          <decomposesTo id="ManagerApproveService"/>
        </task>

        <outputCondition id="End"/>
      </processControlElements>
    </decomposition>

    <decomposition id="ReviewService" xsi:type="WebServiceGatewayFactsType">
      <inputParam>
        <index>0</index>
        <name>requestID</name>
        <type>string</type>
        <namespace>http://www.w3.org/2001/XMLSchema</namespace>
      </inputParam>
      <outputParam>
        <index>0</index>
        <name>amount</name>
        <type>double</type>
        <namespace>http://www.w3.org/2001/XMLSchema</namespace>
      </outputParam>
      <externalInteraction>automated</externalInteraction>
    </decomposition>
  </specification>
</specificationSet>
```

### 8.2 Turtle RDF Output

```turtle
@prefix knhk: <http://knhk.org/ontology#> .
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix dc: <http://purl.org/dc/terms/> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Specification
:SimpleApproval a knhk:WorkflowSpecification ;
  rdfs:label "Simple Approval Workflow" ;
  dc:title "Simple Approval" ;
  dc:hasVersion "1.0"^^xsd:decimal ;
  knhk:hasDecomposition :MainProcess , :ReviewService .

# Main Process Net
:MainProcess a knhk:WorkflowNet ;
  knhk:isRootNet true ;

  knhk:hasInputParameter [
    a knhk:Parameter ;
    knhk:parameterIndex 0 ;
    knhk:parameterName "requestID"^^xsd:NCName ;
    knhk:parameterType xsd:string
  ] ;

  knhk:hasOutputParameter [
    a knhk:Parameter ;
    knhk:parameterIndex 0 ;
    knhk:parameterName "approved"^^xsd:NCName ;
    knhk:parameterType xsd:boolean
  ] ;

  knhk:hasProcessElements (
    :Start
    :Review
    :Approve
    :ManagerApprove
    :End
  ) .

# Start Event
:Start a knhk:StartEvent ;
  knhk:flowsInto :Review .

# Review Task
:Review a knhk:Task ;
  rdfs:label "Review Request" ;
  knhk:hasJoinOperator knhk:XOR ;
  knhk:hasSplitOperator knhk:XOR ;
  knhk:decomposesTo :ReviewService ;

  knhk:flowsInto [
    a knhk:ConditionalFlow ;
    knhk:targetTask :Approve ;
    knhk:hasPredicate "/data/amount < 1000"^^xsd:string
  ] , [
    a knhk:ConditionalFlow ;
    knhk:targetTask :ManagerApprove ;
    knhk:isDefaultFlow true
  ] .

# Approve Task
:Approve a knhk:Task ;
  rdfs:label "Approve (Standard)" ;
  knhk:hasJoinOperator knhk:XOR ;
  knhk:hasSplitOperator knhk:AND ;
  knhk:decomposesTo :ApproveService ;
  knhk:flowsInto :End ;

  knhk:hasCompletedMapping [
    a knhk:DataMapping ;
    knhk:hasExpression "<approved>true</approved>"^^xsd:string ;
    knhk:mapsToParameter "approved"^^xsd:NCName
  ] .

# Manager Approve Task
:ManagerApprove a knhk:Task ;
  rdfs:label "Manager Approval" ;
  knhk:hasJoinOperator knhk:XOR ;
  knhk:hasSplitOperator knhk:AND ;
  knhk:decomposesTo :ManagerApproveService ;
  knhk:flowsInto :End ;

  knhk:hasResourcing [
    a knhk:ResourcingSpecification ;

    knhk:hasOfferPhase [
      knhk:initiatedBy knhk:System ;
      knhk:offeredToRole "Manager"^^xsd:string
    ] ;

    knhk:hasAllocatePhase [
      knhk:initiatedBy knhk:User
    ] ;

    knhk:hasStartPhase [
      knhk:initiatedBy knhk:User
    ]
  ] .

# End Event
:End a knhk:EndEvent .

# Service Decomposition
:ReviewService a knhk:ServiceTask ;
  knhk:externalInteraction knhk:Automated ;

  knhk:hasInputParameter [
    a knhk:Parameter ;
    knhk:parameterIndex 0 ;
    knhk:parameterName "requestID"^^xsd:NCName ;
    knhk:parameterType xsd:string
  ] ;

  knhk:hasOutputParameter [
    a knhk:Parameter ;
    knhk:parameterIndex 0 ;
    knhk:parameterName "amount"^^xsd:NCName ;
    knhk:parameterType xsd:double
  ] .
```

---

## 9. Validation Rules

### 9.1 Reference Integrity

1. **Task Decomposition**: Every `<decomposesTo id="X">` must reference valid decomposition
2. **Flow References**: Every `<nextElementRef id="X">` must reference element in same net
3. **Cancellation References**: Every `<removesTokens id="X">` must reference valid task
4. **Parameter Mapping**: Every `<mapsTo>` must reference valid parameter name

### 9.2 Control Flow Soundness

1. **Single Start**: Exactly one `<inputCondition>` per net
2. **Single End**: Exactly one `<outputCondition>` per net
3. **Reachability**: All tasks reachable from input condition
4. **Completion**: All tasks can reach output condition
5. **No Deadlocks**: Join/split combinations avoid deadlock patterns

### 9.3 Data Flow Consistency

1. **Type Matching**: Mapped parameters have compatible types
2. **Expression Validity**: XQuery expressions are syntactically valid
3. **Variable Scope**: Variables referenced in expressions exist in scope

---

## 10. Parser Implementation Checklist

### Phase 1: XML Parsing
- [ ] Detect schema version (2.0-4.0, Beta)
- [ ] Handle legacy vs. modern namespaces
- [ ] Parse embedded XML Schema
- [ ] Build element ID registry
- [ ] Unescape XML entities

### Phase 2: Structure Extraction
- [ ] Parse `specificationSet` → `specification`
- [ ] Extract metadata (Dublin Core)
- [ ] Parse decomposition elements
- [ ] Build decomposition registry

### Phase 3: Control Flow
- [ ] Parse `processControlElements`
- [ ] Extract tasks, conditions
- [ ] Parse join/split operators
- [ ] Parse flow predicates
- [ ] Parse cancellation regions

### Phase 4: Data Flow
- [ ] Parse input/output parameters
- [ ] Parse local variables
- [ ] Parse data mappings (starting, completed, enablement)
- [ ] Preserve XQuery expressions

### Phase 5: Resourcing
- [ ] Parse offer/allocate/start phases
- [ ] Parse distribution sets
- [ ] Parse filters and constraints
- [ ] Parse secondary resources
- [ ] Parse privileges

### Phase 6: Advanced Features
- [ ] Parse timer specifications
- [ ] Parse configuration elements
- [ ] Parse log predicates
- [ ] Parse custom form URIs

### Phase 7: Validation
- [ ] Validate reference integrity
- [ ] Check control flow soundness
- [ ] Validate data type consistency
- [ ] Verify XQuery syntax

### Phase 8: Turtle Generation
- [ ] Generate namespace declarations
- [ ] Convert specifications to Turtle
- [ ] Convert decompositions to classes
- [ ] Generate control flow triples
- [ ] Generate data flow triples
- [ ] Generate resourcing triples
- [ ] Output complete Turtle document

---

## 11. Error Handling

### 11.1 Parse Errors

| Error Type | Handling Strategy |
|------------|-------------------|
| Invalid XML | Report line/column, abort |
| Unknown schema version | Warn, attempt best-effort parse |
| Invalid namespace | Normalize to modern namespace |
| Invalid XSD type | Report error, skip element |
| Malformed XQuery | Report error, preserve as string |

### 11.2 Validation Errors

| Error Type | Handling Strategy |
|------------|-------------------|
| Broken reference | Report error, skip reference |
| Missing decomposition | Report error, create stub |
| Duplicate ID | Report error, rename with suffix |
| Type mismatch | Report warning, preserve mapping |
| Deadlock pattern | Report warning, continue |

---

## 12. Testing Strategy

### 12.1 Unit Tests

- Parse individual YAWL elements
- Convert single task to Turtle
- Validate reference resolution
- Test XQuery entity unescaping

### 12.2 Integration Tests

- Parse complete specifications
- Validate reference integrity
- Test control flow soundness
- Verify Turtle generation

### 12.3 Regression Tests

- All `/exampleSpecs/` workflows
- Order Fulfillment workflow
- Worklet examples
- Legacy namespace handling

### 12.4 Roundtrip Tests

- YAWL XML → Turtle → YAWL XML
- Semantic equivalence verification

---

## 13. Performance Considerations

### 13.1 Parsing Optimization

- **Streaming XML Parser**: Avoid loading entire document
- **ID Registry**: O(1) reference resolution
- **Lazy Evaluation**: Parse decompositions on demand
- **Schema Caching**: Cache parsed XML Schema types

### 13.2 Memory Management

- **Reference Counting**: Track element references for cleanup
- **String Interning**: Deduplicate repeated strings (IDs, namespaces)
- **Expression Pooling**: Reuse parsed XQuery ASTs

---

## 14. Extension Points

### 14.1 Custom Decomposition Types

Support future YAWL extensions with plugin architecture:

```turtle
:CustomTaskType rdfs:subClassOf knhk:Task ;
  knhk:hasCustomProperty "value" .
```

### 14.2 Custom Resourcing Strategies

Allow domain-specific resource allocation:

```turtle
knhk:CustomAllocator a knhk:AllocationStrategy ;
  knhk:implementedBy "com.example.CustomAllocator"^^xsd:anyURI .
```

---

## 15. Summary

This document specifies the complete conversion logic from YAWL XML to Turtle/RDF format. Key requirements:

1. **Namespace Normalization**: Legacy → Modern YAWL namespace
2. **Element Mapping**: YAWL types → knhk classes
3. **Property Mapping**: XML elements/attributes → RDF properties
4. **Entity Handling**: Unescape XML entities in expressions
5. **Validation**: Reference integrity, control flow soundness, type consistency
6. **Error Handling**: Graceful degradation with informative errors

**Implementation Priority**:
1. Basic control flow (tasks, flows, join/split)
2. Data flow (parameters, mappings)
3. Resourcing (offer, allocate, start)
4. Advanced features (timers, configuration, MI tasks)

---

**Document Version**: 1.0
**Last Updated**: 2025-11-08
**Status**: Complete

# YAWL Specification Analysis for KNHK Parser Implementation

**Research Date**: 2025-11-08
**Purpose**: Understand YAWL XML specification format for knhk Turtle/RDF conversion
**Schema Versions Analyzed**: 2.0, 2.1, 2.2, 3.0, 4.0, Beta3-7.1

---

## Executive Summary

YAWL (Yet Another Workflow Language) uses XML-based specifications with a sophisticated schema hierarchy. This analysis identifies the key structures, data types, and patterns knhk's parser must support for full YAWL compatibility.

**Key Findings**:
- **Root Element**: `<specificationSet>` containing one or more `<specification>` elements
- **Schema Evolution**: 10 schema versions with progressive feature additions (Beta3 → 4.0)
- **Namespace**: `http://www.yawlfoundation.org/yawlschema` (modern) vs `http://www.citi.qut.edu.au/yawl` (legacy)
- **Core Patterns**: 43+ workflow control-flow patterns supported
- **Type System**: XML Schema embedded data types with custom complex types

---

## 1. YAWL Schema Structure Overview

### 1.1 Root Element: `specificationSet`

```xml
<specificationSet
  xmlns="http://www.yawlfoundation.org/yawlschema"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  version="4.0"
  xsi:schemaLocation="...">

  <specification uri="unique-identifier">
    <!-- Workflow definition -->
  </specification>

  <layout>
    <!-- Visual layout metadata -->
  </layout>
</specificationSet>
```

**Attributes**:
- `version`: Schema version (2.0, 2.1, 2.2, 3.0, 4.0, Beta variants)
- Namespace URIs for schema validation

### 1.2 Specification Element Structure

```xml
<specification uri="orderfulfillment">
  <name>Human-readable name</name>
  <documentation>Description text</documentation>

  <metaData>
    <!-- Dublin Core metadata -->
  </metaData>

  <xs:schema>
    <!-- Embedded XML Schema for data types -->
  </xs:schema>

  <decomposition id="..." xsi:type="NetFactsType|WebServiceGatewayFactsType">
    <!-- Workflow net or service task -->
  </decomposition>

  <importedNet>uri-to-external-net</importedNet>
</specification>
```

---

## 2. Key Data Types and Enumerations

### 2.1 Control Flow Types

```xsd
<xs:simpleType name="ControlTypeCodeType">
  <xs:restriction base="xs:string">
    <xs:enumeration value="and"/>  <!-- Parallel split/join -->
    <xs:enumeration value="or"/>   <!-- Multi-choice -->
    <xs:enumeration value="xor"/>  <!-- Exclusive choice -->
  </xs:restriction>
</xs:simpleType>
```

**Usage**: Join and split operators for task gateways

### 2.2 Creation Modes (Multiple Instance Tasks)

```xsd
<xs:simpleType name="CreationModeCodeType">
  <xs:restriction base="xs:string">
    <xs:enumeration value="static"/>   <!-- Fixed number of instances -->
    <xs:enumeration value="dynamic"/>  <!-- Runtime-determined instances -->
  </xs:restriction>
</xs:simpleType>
```

### 2.3 Parameter Directions

```xsd
<xs:simpleType name="DirectionModeType">
  <xs:restriction base="xs:string">
    <xs:enumeration value="input"/>   <!-- Input parameter -->
    <xs:enumeration value="output"/>  <!-- Output parameter -->
    <xs:enumeration value="both"/>    <!-- Bidirectional -->
  </xs:restriction>
</xs:simpleType>
```

### 2.4 Resourcing Types

#### Initiator Types
```xsd
<xs:simpleType name="ResourcingInitiatorType">
  <xs:enumeration value="system"/>  <!-- System-initiated -->
  <xs:enumeration value="user"/>    <!-- User-initiated -->
</xs:simpleType>
```

#### Resource Types
```xsd
<xs:simpleType name="ResourcingResourceType">
  <xs:enumeration value="participant"/>  <!-- Human resource -->
  <xs:enumeration value="role"/>         <!-- Role-based -->
</xs:simpleType>

<xs:simpleType name="ResourcingNonHumanType">
  <xs:enumeration value="nonHumanResource"/>  <!-- Equipment -->
  <xs:enumeration value="nonHumanCategory"/>  <!-- Resource category -->
</xs:simpleType>
```

#### Privileges
```xsd
<xs:simpleType name="ResourcingPrivilegeType">
  <xs:enumeration value="canSuspend"/>
  <xs:enumeration value="canReallocateStateless"/>
  <xs:enumeration value="canReallocateStateful"/>
  <xs:enumeration value="canDeallocate"/>
  <xs:enumeration value="canDelegate"/>
  <xs:enumeration value="canSkip"/>
  <xs:enumeration value="canPile"/>
</xs:simpleType>
```

### 2.5 Timer Configuration

```xsd
<xs:simpleType name="TimerTriggerType">
  <xs:enumeration value="OnEnabled"/>    <!-- Timer starts when task enabled -->
  <xs:enumeration value="OnExecuting"/>  <!-- Timer starts when task executing -->
</xs:simpleType>

<xs:simpleType name="TimerIntervalType">
  <xs:enumeration value="YEAR"/>
  <xs:enumeration value="MONTH"/>
  <xs:enumeration value="WEEK"/>
  <xs:enumeration value="DAY"/>
  <xs:enumeration value="HOUR"/>
  <xs:enumeration value="MIN"/>
  <xs:enumeration value="SEC"/>
  <xs:enumeration value="MSEC"/>
</xs:simpleType>
```

---

## 3. Decomposition Types

### 3.1 NetFactsType (Workflow Nets)

The primary decomposition type for workflow processes:

```xml
<decomposition id="Root" isRootNet="true" xsi:type="NetFactsType">
  <name>Process Name</name>
  <documentation>Description</documentation>

  <!-- Data interface -->
  <inputParam>
    <index>0</index>
    <name>PatientID</name>
    <type>string</type>
    <namespace>http://www.w3.org/2001/XMLSchema</namespace>
  </inputParam>

  <outputParam>
    <index>0</index>
    <name>Result</name>
    <type>boolean</type>
    <namespace>http://www.w3.org/2001/XMLSchema</namespace>
  </outputParam>

  <!-- Local variables -->
  <localVariable>
    <index>0</index>
    <name>counter</name>
    <type>integer</type>
    <namespace>http://www.w3.org/2001/XMLSchema</namespace>
    <initialValue>0</initialValue>
  </localVariable>

  <!-- Workflow structure -->
  <processControlElements>
    <inputCondition id="InputCondition">
      <flowsInto>
        <nextElementRef id="Task_1"/>
      </flowsInto>
    </inputCondition>

    <task id="Task_1">
      <!-- Task definition -->
    </task>

    <outputCondition id="OutputCondition"/>
  </processControlElements>

  <!-- Optional external data gateway -->
  <externalDataGateway>GatewayServiceName</externalDataGateway>
</decomposition>
```

**Key Elements**:
- `isRootNet`: Boolean indicating top-level process
- `processControlElements`: Control flow graph
- `inputParam/outputParam`: Data interface
- `localVariable`: Process-level variables

### 3.2 WebServiceGatewayFactsType (Service Tasks)

For tasks implemented by external services:

```xml
<decomposition id="GetTime" xsi:type="WebServiceGatewayFactsType">
  <inputParam>
    <index>0</index>
    <name>time</name>
    <type>string</type>
    <namespace>http://www.w3.org/2001/XMLSchema</namespace>
  </inputParam>

  <outputParam>
    <index>0</index>
    <name>result</name>
    <type>string</type>
    <namespace>http://www.w3.org/2001/XMLSchema</namespace>
  </outputParam>

  <!-- YAWL Service specification -->
  <yawlService id="http://localhost:8080/timeService/ib">
    <documentation>Time service invocation</documentation>
    <wsdlLocation>http://example.com/TimeService.wsdl</wsdlLocation>
    <operationName>getTime</operationName>
  </yawlService>

  <!-- Alternative: Codelet implementation -->
  <codelet>com.example.TimeCodelet</codelet>

  <!-- External interaction type -->
  <externalInteraction>manual|automated</externalInteraction>
</decomposition>
```

**Service Integration Options**:
1. **YAWL Service**: WSDL-based web service
2. **Codelet**: Java class implementing service logic
3. **External Interaction**: Manual (human) or automated

---

## 4. Task Definition Structure

### 4.1 Basic Task Elements

```xml
<task id="Task_1">
  <name>Human-readable task name</name>
  <documentation>Task description</documentation>

  <!-- Control flow -->
  <flowsInto>
    <nextElementRef id="Task_2"/>
    <predicate ordering="1">
      /data/approved = true()
    </predicate>
    <isDefaultFlow/>  <!-- Default path if no predicate matches -->
  </flowsInto>

  <!-- Join/Split operators -->
  <join code="xor"/>   <!-- and|or|xor -->
  <split code="and"/>  <!-- and|or|xor -->

  <!-- Cancellation regions -->
  <removesTokens id="Task_X"/>
  <removesTokensFromFlow>
    <flowSource id="Task_A"/>
    <flowDestination id="Task_B"/>
  </removesTokensFromFlow>

  <!-- Task decomposition -->
  <decomposesTo id="SubProcess"/>
</task>
```

### 4.2 Data Mappings

#### Starting Mappings (Input Data Transfer)

```xml
<startingMappings>
  <mapping>
    <expression query="&lt;Fever&gt;{/TreatFever/Fever/text()}&lt;/Fever&gt;"/>
    <mapsTo>Fever</mapsTo>
  </mapping>
  <mapping>
    <expression query="&lt;Notes&gt;{/TreatFever/Notes/text()}&lt;/Notes&gt;"/>
    <mapsTo>Notes</mapsTo>
  </mapping>
</startingMappings>
```

**Syntax**: XQuery expressions for data transformation

#### Completed Mappings (Output Data Transfer)

```xml
<completedMappings>
  <mapping>
    <expression query="&lt;Notes&gt;{/Test_Fever/Notes/text()}&lt;/Notes&gt;"/>
    <mapsTo>Notes</mapsTo>
  </mapping>
</completedMappings>
```

#### Enablement Mappings (Runtime Parameters)

```xml
<enablementMappings>
  <mapping>
    <expression query="&lt;YawlResourceAllocationQuery&gt;{'select hresid from hresperformsrole where rolename = $apos;manager$apos;'}&lt;/YawlResourceAllocationQuery&gt;"/>
    <mapsTo>YawlResourceAllocationQuery</mapsTo>
  </mapping>
</enablementMappings>
```

### 4.3 Multiple Instance Tasks

```xml
<task id="ProcessOrders" xsi:type="MultipleInstanceExternalTaskFactsType">
  <name>Process Each Order</name>

  <!-- Instance control -->
  <minimum>1</minimum>           <!-- XQuery expression -->
  <maximum>10</maximum>          <!-- XQuery expression -->
  <threshold>5</threshold>       <!-- Completion threshold -->
  <creationMode code="dynamic"/> <!-- static|dynamic -->

  <!-- Input data splitting -->
  <miDataInput>
    <expression query="/data/orders"/>
    <splittingExpression query="/orders/order"/>
    <formalInputParam>order</formalInputParam>
  </miDataInput>

  <!-- Output data aggregation -->
  <miDataOutput>
    <formalOutputExpression query="/processOrder/result"/>
    <outputJoiningExpression query="&lt;results&gt;{$results}&lt;/results&gt;"/>
    <resultAppliedToLocalVariable>allResults</resultAppliedToLocalVariable>
  </miDataOutput>
</task>
```

**Key Concepts**:
- **Minimum**: Minimum instances to create
- **Maximum**: Maximum instances allowed
- **Threshold**: Instances required to complete before continuation
- **Creation Mode**: Static (predetermined) or dynamic (runtime)
- **Splitting Expression**: How to divide input data
- **Joining Expression**: How to aggregate output data

---

## 5. Resourcing Specification

### 5.1 Complete Resourcing Example

```xml
<resourcing>
  <!-- Offer phase: Who can see the task? -->
  <offer initiator="system">
    <distributionSet>
      <initialSet>
        <participant>JohnDoe</participant>
        <role>Manager</role>
      </initialSet>

      <filters>
        <filter>
          <name>CapabilityFilter</name>
          <params>
            <param>
              <key>capability</key>
              <value>approval</value>
            </param>
          </params>
        </filter>
      </filters>

      <constraints>
        <constraint>
          <name>SeparationOfDuties</name>
          <params>
            <param>
              <key>excludeTask</key>
              <value>PrepareDocument</value>
            </param>
          </params>
        </constraint>
      </constraints>
    </distributionSet>

    <!-- Familiar participant from previous task -->
    <familiarParticipant taskID="PreviousTask"/>
  </offer>

  <!-- Allocate phase: Who executes the task? -->
  <allocate initiator="user">
    <allocator>
      <name>ShortestQueue</name>
    </allocator>
  </allocate>

  <!-- Start phase: When does execution begin? -->
  <start initiator="user"/>

  <!-- Secondary resources (equipment, etc.) -->
  <secondary>
    <participant>TechnicalAssistant</participant>
    <nonHumanResource>LaserPrinter</nonHumanResource>
    <nonHumanCategory subcategory="Color">Printer</nonHumanCategory>
  </secondary>

  <!-- Privileges during execution -->
  <privileges>
    <privilege>
      <name>canSuspend</name>
      <allowall>true</allowall>
    </privilege>
    <privilege>
      <name>canDelegate</name>
      <set>
        <role>Manager</role>
      </set>
    </privilege>
  </privileges>
</resourcing>
```

### 5.2 Resourcing Phases

1. **Offer Phase**: Determines which resources (participants/roles) can see/claim the task
2. **Allocate Phase**: Assigns task to specific resource
3. **Start Phase**: Triggers task execution
4. **Secondary Resources**: Non-primary resources (equipment, assistants)
5. **Privileges**: Runtime control permissions

**Common Allocators**:
- `Random`
- `RoundRobin`
- `ShortestQueue`
- `LeastUtilized`
- `RandomChoice`

**Common Filters**:
- `CapabilityFilter`
- `OrganizationalFilter`
- `HistoryFilter`

**Common Constraints**:
- `SeparationOfDuties`
- `RetainFamiliar`
- `CaseBinding`

---

## 6. Timer Specifications

### 6.1 Timer Elements

```xml
<timer>
  <!-- Option 1: Net parameter reference -->
  <netparam>timerDurationParam</netparam>

  <!-- Option 2: Explicit configuration -->
  <trigger>OnEnabled|OnExecuting</trigger>

  <!-- Expiry: absolute timestamp -->
  <expiry>1699459200000</expiry>  <!-- milliseconds since epoch -->

  <!-- OR Duration: relative time -->
  <duration>PT2H30M</duration>  <!-- ISO 8601 duration -->

  <!-- OR Duration with parameters -->
  <durationparams>
    <ticks>5</ticks>
    <interval>HOUR</interval>
  </durationparams>

  <!-- Work days only? -->
  <workdays>true</workdays>
</timer>
```

**Timer Triggers**:
- `OnEnabled`: Timer starts when task becomes enabled
- `OnExecuting`: Timer starts when task execution begins

**Duration Intervals**: YEAR, MONTH, WEEK, DAY, HOUR, MIN, SEC, MSEC

---

## 7. Configuration (Dynamic Process Configuration)

### 7.1 Configurable Regions

```xml
<configuration>
  <!-- Join configuration -->
  <join>
    <port value="activated|blocked|hidden">
      <flowSource id="Task_A"/>
      <flowSource id="Task_B"/>
    </port>
  </join>

  <!-- Number of instances (NOFI) configuration -->
  <nofi>
    <minIncrease>1</minIncrease>
    <maxDecrease>2</maxDecrease>
    <thresIncrease>1</thresIncrease>
    <creationMode>restrict|keep</creationMode>
  </nofi>

  <!-- Removal (cancellation) configuration -->
  <rem value="activated|blocked"/>

  <!-- Split configuration -->
  <split>
    <port value="activated|blocked">
      <flowDestination id="Task_C"/>
      <flowDestination id="Task_D"/>
    </port>
  </split>
</configuration>
```

**Port Values**:
- `activated`: Port is enabled
- `blocked`: Port is disabled
- `hidden`: Port is hidden (join only)

**Use Case**: Allow process variants to be configured at runtime without changing the core model.

---

## 8. Metadata (Dublin Core)

```xml
<metaData>
  <title>Order Fulfillment Process</title>
  <creator>John Doe</creator>
  <creator>Jane Smith</creator>
  <subject>Order Processing</subject>
  <subject>Logistics</subject>
  <description>Complete order fulfillment workflow</description>
  <contributor>Technical Team</contributor>
  <coverage>Global Operations</coverage>
  <validFrom>2024-01-01</validFrom>
  <validUntil>2025-12-31</validUntil>
  <created>2024-01-01</created>
  <version>1.2</version>
  <status>Production</status>
  <persistent>false</persistent>
  <identifier>UID_ae0b797c-2ac8-4d5e-9421-ece89d8043d0</identifier>
</metaData>
```

**Fields** (Dublin Core compliant):
- Standard metadata: title, creator, description, version
- Validity period: validFrom, validUntil
- Status tracking: status, created, identifier
- Persistence: whether specification persists across engine restarts

---

## 9. Custom Forms

```xml
<task id="UserInputTask">
  <!-- ... task definition ... -->

  <!-- Reference to custom HTML/JavaScript form -->
  <customForm>http://example.com/forms/orderEntry.html</customForm>

  <decomposesTo id="OrderEntry"/>
</task>
```

**Purpose**: Override default generated forms with custom UI.

---

## 10. Log Predicates (Selective Logging)

### 10.1 Decomposition-Level Logging

```xml
<decomposition id="Process" xsi:type="NetFactsType">
  <!-- ... -->

  <logPredicate>
    <start>LogPredicate1</start>
    <completion>LogPredicate2</completion>
  </logPredicate>
</decomposition>
```

### 10.2 Parameter-Level Logging

```xml
<inputParam>
  <name>sensitiveData</name>
  <type>string</type>
  <namespace>http://www.w3.org/2001/XMLSchema</namespace>

  <logPredicate>
    <start>never</start>         <!-- Don't log on start -->
    <completion>always</completion>  <!-- Log on completion -->
  </logPredicate>
</inputParam>
```

**Use Case**: Control which data is logged for audit/compliance.

---

## 11. Layout Information (Visual Editor Support)

```xml
<layout>
  <locale language="en" country="AU"/>

  <specification id="OrderFulfillment" defaultBgColor="-526351">
    <size w="1200" h="800"/>

    <net id="MainProcess" bgColor="-526351">
      <bounds x="0" y="0" w="1200" h="800"/>
      <frame x="0" y="0" w="1200" h="800"/>
      <viewport x="0" y="0" w="1200" h="800"/>
      <scale>1.0</scale>

      <!-- Task visual representation -->
      <container id="Task_1">
        <vertex>
          <attributes>
            <bounds x="100" y="100" w="32" h="32"/>
            <backgroundColor>-197913</backgroundColor>
          </attributes>
        </vertex>

        <label>
          <attributes>
            <bounds x="68" y="132" w="96" h="18"/>
            <font>
              <name>Arial</name>
              <style>0</style>
              <size>12</size>
            </font>
          </attributes>
        </label>
      </container>

      <!-- Flow visual representation -->
      <flow source="Task_1" target="Task_2">
        <ports in="13" out="12"/>
        <attributes>
          <lineStyle>11</lineStyle>
          <points>
            <value x="132" y="116"/>
            <value x="200" y="116"/>
          </points>
        </attributes>
      </flow>
    </net>
  </specification>
</layout>
```

**Purpose**: Store graphical editor layout separately from logical workflow definition.

---

## 12. Embedded XML Schema for Data Types

### 12.1 Example: Order Fulfillment Types

```xml
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema" elementFormDefault="qualified">
  <xs:complexType name="PurchaseOrderType">
    <xs:sequence>
      <xs:element name="Company" type="CompanyType"/>
      <xs:element name="Order" type="OrderType"/>
      <xs:element name="FreightCost" type="xs:double"/>
      <xs:element name="DeliveryLocation" type="xs:string"/>
      <xs:element name="InvoiceRequired" type="xs:boolean"/>
      <xs:element name="PrePaid" type="xs:boolean"/>
    </xs:sequence>
  </xs:complexType>

  <xs:simpleType name="CurrencyType">
    <xs:restriction base="xs:string">
      <xs:enumeration value="AUD"/>
      <xs:enumeration value="USD"/>
      <xs:enumeration value="EUR"/>
    </xs:restriction>
  </xs:simpleType>
</xs:schema>
```

**Usage**: Define domain-specific data types used in variable declarations and mappings.

---

## 13. YAWL Workflow Patterns Catalog

### 13.1 Control Flow Patterns

Based on example workflows, YAWL supports:

#### Basic Patterns
1. **Sequence**: Tasks execute in order
2. **Parallel Split (AND-split)**: Fork execution into parallel branches
3. **Synchronization (AND-join)**: Wait for all parallel branches
4. **Exclusive Choice (XOR-split)**: Choose one path based on condition
5. **Simple Merge (XOR-join)**: Merge alternative paths

#### Advanced Patterns
6. **Multi-Choice (OR-split)**: Choose multiple paths based on conditions
7. **Synchronizing Merge (OR-join)**: Wait for all active branches
8. **Multi-Merge**: Accept tokens from multiple incoming branches
9. **Discriminator**: Continue after first incoming branch completes
10. **Arbitrary Cycles**: Loops with dynamic exit conditions

#### Multiple Instance Patterns
11. **Multiple Instances without Synchronization**: Create N instances, no waiting
12. **Multiple Instances with a Priori Design-Time Knowledge**: Fixed N instances
13. **Multiple Instances with a Priori Runtime Knowledge**: N determined at runtime
14. **Multiple Instances without a Priori Runtime Knowledge**: Dynamic instance creation

#### State-Based Patterns
15. **Deferred Choice**: External event determines path
16. **Interleaved Parallel Routing**: Parallel tasks with order constraints
17. **Milestone**: Tasks enabled based on process state
18. **Critical Section**: Mutual exclusion for shared resources

#### Cancellation Patterns
19. **Cancel Task**: Explicit task cancellation
20. **Cancel Case**: Terminate entire process instance
21. **Cancel Region**: Cancel specific process region
22. **Cancel Multiple Instance Activity**: Cancel MI task instances

### 13.2 Data Flow Patterns

1. **Task Data**: Local task variables
2. **Block Data**: Subprocess-scoped variables
3. **Scope Data**: Net-level local variables
4. **Multiple Instance Data**: Instance-specific data
5. **Case Data**: Process instance variables
6. **Environment Data**: External data access
7. **Task-to-Task Data**: Direct data flow between tasks
8. **Task-to-Environment Data**: Output to external systems
9. **Environment-to-Task Data**: Input from external systems

### 13.3 Resource Patterns

From resourcing schema analysis:

1. **Direct Distribution**: Specific participant assignment
2. **Role-Based Distribution**: Assignment by role
3. **Deferred Distribution**: Runtime participant selection
4. **Authorization**: User-initiated allocation
5. **Separation of Duties**: Constraint-based exclusion
6. **Case Handling**: Retain same participant
7. **Retain Familiar**: Prefer previous task performer
8. **Capability-Based Distribution**: Skill matching
9. **History-Based Distribution**: Past performance
10. **Organizational Distribution**: Hierarchical assignment

---

## 14. XQuery/XPath Expression Usage

YAWL extensively uses XQuery for data transformation and predicates:

### 14.1 Predicate Expressions

```xml
<predicate ordering="1">
  /data/orderAmount &gt; 1000
</predicate>

<predicate ordering="2">
  /data/approved = true() and /data/creditCheck = 'passed'
</predicate>
```

### 14.2 Mapping Expressions

```xml
<expression query="&lt;customer&gt;{/data/order/customer}&lt;/customer&gt;"/>

<expression query="&lt;total&gt;{
  sum(/data/order/items/item/price * /data/order/items/item/quantity)
}&lt;/total&gt;"/>

<expression query="&lt;time&gt; {
  if ( /data/time/text() )
  then /data/time/text()
  else /data/time-fallback/text()
}&lt;/time&gt;"/>
```

**Syntax Notes**:
- XML entities: `&lt;` for `<`, `&gt;` for `>`
- Variable references: `/data/variableName`
- XPath functions: `sum()`, `if-then-else`, `text()`
- Apostrophe escaping: `$apos;` for `'`

---

## 15. Schema Evolution Summary

### Version Comparison

| Feature | Beta3-4 | Beta6-7 | 2.0-2.2 | 3.0 | 4.0 |
|---------|---------|---------|---------|-----|-----|
| Basic control flow | ✓ | ✓ | ✓ | ✓ | ✓ |
| Resourcing | Basic | ✓ | ✓ | ✓ | ✓ |
| Multiple instances | ✓ | ✓ | ✓ | ✓ | ✓ |
| Timers | - | ✓ | ✓ | ✓ | ✓ |
| Configuration | - | - | ✓ | ✓ | ✓ |
| Custom forms | - | - | ✓ | ✓ | ✓ |
| Log predicates | - | - | - | ✓ | ✓ |
| Secondary resources | - | - | - | ✓ | ✓ |
| Layout specification | - | ✓ | ✓ | ✓ | ✓ |

### Namespace Changes

- **Legacy**: `http://www.citi.qut.edu.au/yawl` (Beta versions)
- **Modern**: `http://www.yawlfoundation.org/yawlschema` (2.0+)

---

## 16. KNHK Parser Requirements

### 16.1 Must-Have Features

To support YAWL specification import, knhk's parser must handle:

#### Core Structure
- [x] Parse `<specificationSet>` root element
- [x] Extract `<specification>` elements with URI identifiers
- [x] Support multiple schema versions (2.0-4.0)
- [x] Parse embedded XML Schema definitions

#### Decompositions
- [x] `NetFactsType`: Workflow nets with control flow
- [x] `WebServiceGatewayFactsType`: Service tasks
- [x] `MultipleInstanceExternalTaskFactsType`: MI tasks
- [x] Input/output parameters with XML Schema types
- [x] Local variables with initial values

#### Control Flow
- [x] `inputCondition`, `outputCondition`
- [x] `task` elements with join/split operators (and, or, xor)
- [x] `flowsInto` with predicates and default flows
- [x] `removesTokens` (cancellation regions)
- [x] `removesTokensFromFlow` (flow cancellation)

#### Data Flow
- [x] `startingMappings`: Input data transfer
- [x] `completedMappings`: Output data transfer
- [x] `enablementMappings`: Runtime parameters
- [x] XQuery/XPath expression parsing

#### Resourcing
- [x] Offer/Allocate/Start phases
- [x] Distribution sets (participants, roles)
- [x] Filters and constraints
- [x] Secondary resources (human and non-human)
- [x] Privileges configuration

#### Advanced Features
- [x] Timer specifications (triggers, durations, intervals)
- [x] Configuration elements (join, split, nofi, rem)
- [x] Custom forms (URI references)
- [x] Log predicates (selective logging)
- [x] Metadata (Dublin Core)

#### Layout
- [ ] Optional: Visual layout information (for editor support)

### 16.2 Conversion to Turtle/RDF

#### Namespace Mappings

```turtle
@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
@prefix knhk: <http://knhk.org/ontology#> .
@prefix wf: <http://www.w3.org/2005/01/wf/flow#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix dc: <http://purl.org/dc/terms/> .
```

#### Example Conversion: Task to Turtle

**YAWL XML**:
```xml
<task id="ProcessOrder">
  <name>Process Customer Order</name>
  <join code="xor"/>
  <split code="and"/>
  <flowsInto>
    <nextElementRef id="CheckInventory"/>
    <predicate>/data/amount &gt; 1000</predicate>
  </flowsInto>
  <decomposesTo id="ProcessOrderService"/>
</task>
```

**KNHK Turtle**:
```turtle
:ProcessOrder a knhk:Task ;
  rdfs:label "Process Customer Order" ;
  knhk:hasJoinOperator knhk:XOR ;
  knhk:hasSplitOperator knhk:AND ;
  knhk:flowsInto [
    knhk:targetTask :CheckInventory ;
    knhk:predicate "/data/amount > 1000"^^xsd:string
  ] ;
  knhk:decomposesTo :ProcessOrderService .
```

#### Type Mappings

| YAWL Type | Turtle Class |
|-----------|--------------|
| `NetFactsType` | `knhk:WorkflowNet` |
| `WebServiceGatewayFactsType` | `knhk:ServiceTask` |
| `MultipleInstanceExternalTaskFactsType` | `knhk:MultiInstanceTask` |
| `inputCondition` | `knhk:StartEvent` |
| `outputCondition` | `knhk:EndEvent` |
| `task` | `knhk:Task` |
| `condition` | `knhk:Condition` |

### 16.3 Validation Requirements

Parser must validate:

1. **Schema Version Compatibility**: Support all versions 2.0-4.0
2. **Unique IDs**: All elements have unique identifiers within specification
3. **Reference Integrity**:
   - `nextElementRef` IDs exist in same net
   - `decomposesTo` IDs reference valid decompositions
   - `flowSource`/`flowDestination` reference valid tasks
4. **Data Type Consistency**: Variable types match XML Schema definitions
5. **XQuery Syntax**: Valid XPath/XQuery expressions
6. **Control Flow Soundness**:
   - Single input condition
   - Single output condition
   - All tasks reachable from input
   - All tasks can reach output

---

## 17. Example Workflow: Order Fulfillment

### High-Level Structure

```
OrderFulfillment (specificationSet)
├── Specification: orderfulfillment
│   ├── MetaData
│   ├── Schema (PurchaseOrderType, CompanyType, OrderType, etc.)
│   └── Decompositions
│       ├── MainProcess (NetFactsType, isRootNet=true)
│       │   ├── InputCondition
│       │   ├── Task: ReceiveOrder → ApproveOrder → ...
│       │   └── OutputCondition
│       ├── ReceiveOrderService (WebServiceGatewayFactsType)
│       ├── ApproveOrderService (WebServiceGatewayFactsType)
│       └── ... (other services)
└── Layout
    └── Visual positioning data
```

### Pattern Usage in Order Fulfillment

1. **Sequence**: ReceiveOrder → ValidateOrder → ApproveOrder
2. **Exclusive Choice**: ApproveOrder → (Approved: ProcessOrder) XOR (Rejected: NotifyCustomer)
3. **Parallel Split**: ProcessOrder → (CheckInventory AND ArrangeShipment AND GenerateInvoice)
4. **Synchronization**: Wait for all parallel tasks before Shipment
5. **Multiple Instances**: Process each order line item separately
6. **Cancellation**: Cancel entire order process on customer request

---

## 18. Example Workflow: Medical Treatment (Worklet)

### Structure

```
TreatFever (worklet specification)
├── Input Parameters: PatientID, Notes, Fever, Pharmacy, Name, Treatment
├── Output Parameters: Notes, Pharmacy, Treatment
├── MainProcess (NetFactsType)
│   ├── InputCondition
│   ├── Test_Fever (manual task)
│   ├── Treat_Fever (manual task)
│   └── OutputCondition
├── Test_Fever (WebServiceGatewayFactsType - manual interaction)
└── Treat_Fever (WebServiceGatewayFactsType - manual interaction)
```

### Key Features

1. **Data Flow**: XQuery mappings transfer data between tasks
2. **Manual Tasks**: `<externalInteraction>manual</externalInteraction>`
3. **Resourcing**: User-initiated offer, allocate, start
4. **Worklet Pattern**: Small reusable workflow component

---

## 19. Challenges for Parser Implementation

### 19.1 XML Complexity

- **Mixed Content**: Elements, attributes, text all convey meaning
- **Namespace Handling**: Legacy vs. modern namespaces
- **XML Entities**: Escaped characters in XQuery expressions
- **Schema Validation**: Must validate against XSD while parsing

### 19.2 XQuery/XPath Processing

- **Expression Parsing**: Complex XQuery with embedded XML
- **Variable References**: `/data/varName` path resolution
- **Function Calls**: XPath functions (sum, if-then-else, etc.)
- **Entity Escaping**: `&lt;`, `&gt;`, `$apos;`, etc.

### 19.3 Type System Integration

- **XML Schema Types**: Map to Rust/C type system
- **Custom Complex Types**: Preserve domain-specific structures
- **Namespace Resolution**: Handle multiple schema namespaces

### 19.4 Semantic Preservation

- **Control Flow Semantics**: Preserve YAWL execution semantics
- **Pattern Recognition**: Identify workflow patterns for optimization
- **Resourcing Semantics**: Translate resource allocation logic
- **Timer Semantics**: Convert timer specifications to executable form

---

## 20. Recommended Parsing Strategy

### Phase 1: Schema Analysis
1. Detect schema version (2.0-4.0, Beta variants)
2. Extract namespace declarations
3. Parse embedded XML Schema definitions
4. Build type registry

### Phase 2: Structural Parsing
1. Parse `specificationSet` → `specification` hierarchy
2. Extract metadata (Dublin Core)
3. Parse decomposition elements (Net, WebService, MI)
4. Build element ID registry for reference resolution

### Phase 3: Control Flow Extraction
1. Parse `processControlElements` graph structure
2. Extract tasks, conditions, flows
3. Parse join/split operators
4. Extract flow predicates and default flows
5. Parse cancellation regions

### Phase 4: Data Flow Extraction
1. Parse input/output parameters
2. Parse local variables
3. Extract data mappings (starting, completed, enablement)
4. Parse XQuery/XPath expressions

### Phase 5: Advanced Features
1. Parse resourcing specifications
2. Parse timer configurations
3. Parse configuration elements
4. Parse log predicates
5. Parse custom form references

### Phase 6: Semantic Validation
1. Validate reference integrity
2. Check control flow soundness
3. Validate data type consistency
4. Verify XQuery expression syntax

### Phase 7: Turtle/RDF Generation
1. Generate namespace declarations
2. Convert decompositions to Turtle classes
3. Generate control flow triples
4. Generate data flow triples
5. Generate resourcing triples
6. Output complete Turtle document

---

## 21. Example Parser Test Cases

### Test Case 1: Simple Sequence
```xml
<task id="A">
  <flowsInto><nextElementRef id="B"/></flowsInto>
  <join code="xor"/>
  <split code="and"/>
</task>
```

**Expected Turtle**:
```turtle
:A a knhk:Task ;
  knhk:flowsInto :B ;
  knhk:hasJoinOperator knhk:XOR ;
  knhk:hasSplitOperator knhk:AND .
```

### Test Case 2: Conditional Flow
```xml
<flowsInto>
  <nextElementRef id="Approve"/>
  <predicate>/data/amount &lt; 1000</predicate>
</flowsInto>
<flowsInto>
  <nextElementRef id="ManagerApproval"/>
  <isDefaultFlow/>
</flowsInto>
```

**Expected Turtle**:
```turtle
:TaskX knhk:flowsInto [
  knhk:targetTask :Approve ;
  knhk:predicate "/data/amount < 1000"^^xsd:string
] , [
  knhk:targetTask :ManagerApproval ;
  knhk:isDefaultFlow true
] .
```

### Test Case 3: Data Mapping
```xml
<startingMappings>
  <mapping>
    <expression query="&lt;total&gt;{sum(/order/items/item/price)}&lt;/total&gt;"/>
    <mapsTo>orderTotal</mapsTo>
  </mapping>
</startingMappings>
```

**Expected Turtle**:
```turtle
:TaskX knhk:hasStartingMapping [
  knhk:expression "<total>{sum(/order/items/item/price)}</total>"^^xsd:string ;
  knhk:mapsTo "orderTotal"^^xsd:string
] .
```

### Test Case 4: Resourcing
```xml
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
```

**Expected Turtle**:
```turtle
:TaskX knhk:hasResourcing [
  knhk:offerInitiator knhk:System ;
  knhk:offerToRole "Manager"^^xsd:string ;
  knhk:allocateInitiator knhk:User ;
  knhk:startInitiator knhk:User
] .
```

---

## 22. Memory Storage for Coordination

Storing research findings for swarm coordination:

```bash
npx claude-flow@alpha hooks post-edit \
  --file "docs/yawl-specification-analysis.md" \
  --memory-key "hive/research/yawl-specs"
```

**Key Findings to Store**:

1. **Schema Versions**: 2.0, 2.1, 2.2, 3.0, 4.0, Beta3-7.1
2. **Decomposition Types**: NetFactsType, WebServiceGatewayFactsType, MultipleInstanceExternalTaskFactsType
3. **Control Flow Operators**: and, or, xor (join/split)
4. **Data Flow Mappings**: starting, completed, enablement
5. **Resourcing Phases**: offer, allocate, start, secondary, privileges
6. **Timer Types**: OnEnabled, OnExecuting with duration/expiry
7. **Configuration Elements**: join, split, nofi, rem
8. **Pattern Support**: 43+ workflow patterns
9. **XQuery Usage**: Predicates, mappings, expressions
10. **Namespace Evolution**: Legacy (citi.qut.edu.au) → Modern (yawlfoundation.org)

---

## 23. References and Resources

### Primary Sources
- **YAWL Schema 4.0**: `/Users/sac/knhk/vendors/yawl/schema/YAWL_Schema4.0.xsd`
- **Example Workflows**: `/Users/sac/knhk/vendors/yawl/exampleSpecs/`
- **Worklet Examples**: `/Users/sac/knhk/vendors/yawl/build/workletService/samples/worklets/`

### Key Specifications
- Order Fulfillment: Complex multi-task process with parallel flows
- Treat Fever (Worklet): Medical treatment subprocess
- Resource Example: Resourcing specification demo
- Timer Example: Time service invocation demo

### Schema Versions Analyzed
1. YAWL_Schema.xsd (original)
2. YAWL_Schema2.0.xsd
3. YAWL_Schema2.1.xsd
4. YAWL_Schema2.2.xsd
5. YAWL_Schema3.0.xsd
6. YAWL_Schema4.0.xsd
7. YAWL_SchemaBeta3.xsd
8. YAWL_SchemaBeta4.xsd
9. YAWL_SchemaBeta6.xsd
10. YAWL_SchemaBeta7.1.xsd

---

## 24. Next Steps for Implementation

### Immediate Actions

1. **Parser Architecture**:
   - XML parser with namespace support
   - XSD validator for schema compliance
   - XQuery/XPath expression parser
   - Type system mapper (XML Schema → Rust/C)

2. **Core Parsers**:
   - Specification parser (metadata, decompositions)
   - Net parser (control flow graph)
   - Task parser (join, split, mappings)
   - Resourcing parser (phases, distribution sets)
   - Timer parser (triggers, durations)

3. **Data Structure Design**:
   - Rust/C structs for YAWL elements
   - Graph representation for control flow
   - Type registry for XML Schema types
   - Expression AST for XQuery

4. **Conversion Logic**:
   - YAWL XML → Internal representation
   - Internal representation → Turtle/RDF
   - Namespace mapping
   - Semantic preservation

5. **Validation Logic**:
   - Schema version detection
   - Reference integrity checking
   - Control flow soundness verification
   - Data type consistency validation

### Testing Strategy

1. **Unit Tests**: Parse individual YAWL elements
2. **Integration Tests**: Parse complete specifications
3. **Regression Tests**: All example workflows from `/exampleSpecs/`
4. **Roundtrip Tests**: XML → Turtle → XML consistency
5. **Pattern Tests**: Verify all 43+ workflow patterns

---

## Conclusion

YAWL provides a comprehensive XML-based workflow specification language with rich support for control flow patterns, data flow, resource allocation, and advanced features like timers and configuration. The parser must handle:

- **10 schema versions** with progressive feature additions
- **3 decomposition types** (Net, WebService, MI)
- **43+ workflow patterns**
- **XQuery/XPath expressions** for data transformation
- **Complex resourcing semantics**
- **Timer and configuration specifications**

The conversion to Turtle/RDF requires careful mapping of XML structures to RDF triples while preserving workflow semantics. This research provides the foundation for implementing a complete YAWL-to-Turtle converter for the knhk workflow engine.

---

**Document Version**: 1.0
**Last Updated**: 2025-11-08
**Prepared By**: Research Agent (Hive Mind Swarm)
**Status**: Complete

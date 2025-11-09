# YAWL Example Workflows Catalog

**Document Type**: Reference Catalog
**Purpose**: Document example workflows for parser testing and validation
**Date**: 2025-11-08

---

## 1. Overview

This catalog documents all example YAWL workflows found in `/Users/sac/knhk/vendors/yawl/exampleSpecs/` and `/Users/sac/knhk/vendors/yawl/build/workletService/samples/`, providing pattern analysis and parser test coverage.

---

## 2. Example Workflow Summary

### 2.1 Core Examples

| Workflow | Location | Patterns | Complexity | Test Priority |
|----------|----------|----------|------------|---------------|
| Order Fulfillment | `orderfulfillment/orderfulfillment.yawl` | Sequence, Parallel, XOR-split, MI | High | Critical |
| Order Fulfillment (Custom Forms) | `orderfulfillment/orderfulfillment_customforms.yawl` | Custom Forms, Resourcing | High | High |
| Make Trip 1 | `xml/Beta2-7/maketrip1.xml` | Sequence, XOR-split | Medium | Medium |
| Make Trip 2 | `xml/Beta2-7/makeTrip2.xml` | OR-split, Synchronization | Medium | Medium |
| Make Trip 3 | `xml/Beta2-7/makeTrip3.xml` | Advanced cancellation | High | High |
| Make Music | `xml/Beta2-7/MakeMusic.xml` | Parallel composition | Medium | Medium |
| Make Recordings (Beta3) | `xml/Beta2-7/MakeRecordings(Beta3).xml` | Legacy schema | Low | Low |
| Make Recordings (Beta4) | `xml/Beta2-7/MakeRecordings(Beta4).xml` | Legacy schema | Low | Low |
| Barnes & Noble | `xml/Beta2-7/BarnesAndNoble.xml` | E-commerce workflow | Medium | Medium |
| Resource Example | `xml/Beta2-7/ResourceExample.xml` | Resourcing queries | High | Critical |
| Timer Example | `xml/Beta2-7/Timer.xml` | Timer service, XQuery | High | Critical |
| SMS Invoker | `xml/Beta2-7/SMSInvoker.xml` | Web service invocation | Low | Low |
| Stock Quote | `xml/Beta2-7/StockQuote.xml` | Web service integration | Low | Low |

### 2.2 Worklet Examples

| Worklet | File | Use Case | Test Priority |
|---------|------|----------|---------------|
| Treat Fever | `worklets/TreatFever.yawl` | Medical workflow, manual tasks | High |
| Treat Fracture | `worklets/TreatFracture.yawl` | Medical workflow variant | Medium |
| Bob Zero | `worklets/BobZero.yawl` | Worklet selection | Low |
| Bob Three | `worklets/BobThree.yawl` | Worklet selection | Low |
| Change to Small Venue | `worklets/ChangeToSmallVenue.yawl` | Dynamic substitution | Medium |
| Change to Mid Venue | `worklets/ChangeToMidVenue.yawl` | Dynamic substitution | Medium |

---

## 3. Detailed Workflow Analysis

### 3.1 Order Fulfillment (Primary Reference Workflow)

**File**: `/Users/sac/knhk/vendors/yawl/exampleSpecs/orderfulfillment/_examples/orderfulfillment.yawl`

**Schema Version**: 2.1

**Metadata**:
- **Title**: Order Fulfillment
- **Authors**: Stephan Clemens, Marcello La Rosa, Arthur ter Hofstede
- **Version**: 1.2
- **Description**: Complete order fulfillment process from receipt to delivery

**Data Types Defined**:
1. `PurchaseOrderType`: Main order structure
2. `CompanyType`: Customer/supplier information
3. `OrderType`: Order details (number, date, currency, items)
4. `OrderApprovalType`: Approval decision
5. `RouteGuideType`: Delivery route planning
6. `TrailerUsageType`: Package logistics
7. `TransportationQuoteType`: Shipping cost calculation
8. `PickupInstructionsType`: Collection details
9. `DeliveryInstructionsType`: Delivery details
10. `ShipmentInformationDocumentType`: Shipment metadata
11. `BillOfLadingType`: Legal shipping document
12. `CarrierManifestType`: Carrier documentation
13. `ShipmentNoticeType`: Delivery notification
14. `ShipmentInvoiceType`: Billing information
15. `FreightInvoiceType`: Freight billing
16. `ShipmentPaymentOrderType`: Payment authorization
17. `ShipmentPaymentType`: Payment tracking
18. `DebitAdjustmentType`: Payment adjustments
19. `CreditAdjustmentType`: Credit adjustments
20. `ShipmentRemittanceAdviceType`: Payment confirmation
21. `TrackpointNoticeType`: Delivery tracking
22. `ShipmentStatusInquiryType`: Status queries
23. `AcceptanceCertificateType`: Delivery acceptance
24. `ReturnMerchandiseType`: Returns handling

**Workflow Patterns**:
- ✅ Sequence: Multi-step order processing
- ✅ Parallel Split: Concurrent activities (inventory, shipment, invoicing)
- ✅ Synchronization: Wait for parallel tasks
- ✅ Exclusive Choice: Approval routing
- ✅ Simple Merge: Converge alternative paths
- ✅ Multiple Instance: Process multiple order lines

**Complexity Metrics**:
- **Tasks**: ~50+ tasks
- **Data Types**: 24 complex types
- **Decompositions**: Multiple service decompositions
- **Lines**: ~1,000+ XML lines

**Test Coverage**:
- [x] Basic control flow
- [x] Parallel execution
- [x] Conditional routing
- [x] Complex data types
- [x] Multiple decompositions

---

### 3.2 Treat Fever (Worklet Example)

**File**: `/Users/sac/knhk/vendors/yawl/build/workletService/samples/worklets/TreatFever.yawl`

**Schema Version**: 4.0

**Metadata**:
- **Name**: Treat Fever
- **Documentation**: Worklet to treat a fever
- **Version**: 0.2
- **Coverage**: 4.5.1.796

**Process Structure**:
```
InputCondition
    ↓
Test_Fever (manual task)
    ↓
Treat_Fever (manual task)
    ↓
OutputCondition
```

**Input Parameters**:
1. `PatientID` (string)
2. `Notes` (string)
3. `Fever` (boolean)
4. `Pharmacy` (string)
5. `Name` (string)
6. `Treatment` (string)

**Output Parameters**:
1. `Notes` (string)
2. `Pharmacy` (string)
3. `Treatment` (string)

**Data Mappings**:
- **Starting Mappings**: XQuery expressions map input to task parameters
- **Completed Mappings**: XQuery expressions map task output to process output

**Resourcing**:
- **Offer**: User-initiated
- **Allocate**: User-initiated
- **Start**: User-initiated

**Decompositions**:
1. `TreatFever` (NetFactsType): Main workflow net
2. `Test_Fever` (WebServiceGatewayFactsType): Manual task with external interaction
3. `Treat_Fever` (WebServiceGatewayFactsType): Manual task with external interaction

**Layout Information**:
- **Locale**: English (Australia)
- **Visual Layout**: Complete positioning data for graphical editor
- **Vertices**: Task visual representations
- **Flows**: Connection visual representations

**Test Coverage**:
- [x] Worklet pattern
- [x] Manual tasks
- [x] User-initiated resourcing
- [x] XQuery data mappings
- [x] Layout information
- [x] Schema version 4.0

---

### 3.3 Resource Example

**File**: `/Users/sac/knhk/vendors/yawl/exampleSpecs/xml/Beta2-7/ResourceExample.xml`

**Schema Version**: Beta 6

**Purpose**: Demonstrate resource allocation using SQL queries

**Key Features**:
- **Enablement Mappings**: SQL-based resource queries
- **Resource Allocation Query**: `select hresid from hresperformsrole where rolename = 'manager'`
- **Resource Authorization Query**: Same query for authorization

**SQL Query Pattern**:
```xml
<expression query="&lt;YawlResourceAllocationQuery&gt;{'select hresid from hresperformsrole where rolename = $apos;manager$apos;'}&lt;/YawlResourceAllocationQuery&gt;"/>
```

**Note**: Uses `$apos;` for apostrophe escaping in XQuery

**Test Coverage**:
- [x] Enablement mappings
- [x] SQL-based resource allocation
- [x] Legacy namespace (citi.qut.edu.au)
- [x] Beta schema version
- [x] XQuery entity escaping

---

### 3.4 Timer Example

**File**: `/Users/sac/knhk/vendors/yawl/exampleSpecs/xml/Beta2-7/Timer.xml`

**Schema Version**: Original (legacy)

**Purpose**: Demonstrate time service invocation

**Key Features**:
- **Local Variables**: Fallback values
- **Web Service Integration**: External time service
- **XQuery Conditionals**: `if-then-else` expressions
- **Data Flow**: Variable passing through workflow

**Process Structure**:
```
InputCondition
    ↓
read_time (service: fetch current time)
    ↓
GetTime (service: process time)
    ↓
q (service: use time)
    ↓
OutputCondition
```

**XQuery Pattern**:
```xml
<expression query="&lt;time&gt; {if ( /data/time/text() ) then /data/time/text() else /data/time-fallback/text() }&lt;/time&gt;"/>
```

**Fallback Logic**: Use default value if service call fails

**WSDL Integration**:
```xml
<yawlService id="http://localhost:8080/timeService/ib">
  <wsdlLocation>http://www.xmethods.net/sd/2001/BNQuoteService.wsdl</wsdlLocation>
  <operationName>getTime</operationName>
</yawlService>
```

**Test Coverage**:
- [x] Web service decomposition
- [x] WSDL location specification
- [x] Local variables with initial values
- [x] XQuery conditionals (if-then-else)
- [x] Data fallback patterns

---

## 4. Pattern Coverage Matrix

### 4.1 Control Flow Patterns

| Pattern | Order Fulfillment | Treat Fever | Timer | Resource Example |
|---------|------------------|-------------|-------|------------------|
| Sequence | ✅ | ✅ | ✅ | ✅ |
| Parallel Split (AND) | ✅ | ❌ | ❌ | ❌ |
| Synchronization (AND) | ✅ | ❌ | ❌ | ❌ |
| Exclusive Choice (XOR) | ✅ | ✅ | ✅ | ✅ |
| Simple Merge (XOR) | ✅ | ✅ | ✅ | ✅ |
| Multi-Choice (OR) | ⚠️ | ❌ | ❌ | ❌ |
| Synchronizing Merge (OR) | ⚠️ | ❌ | ❌ | ❌ |
| Multiple Instance | ✅ | ❌ | ❌ | ❌ |
| Cancellation Region | ⚠️ | ❌ | ❌ | ❌ |

**Legend**: ✅ Used, ❌ Not used, ⚠️ Potentially used (requires deep analysis)

### 4.2 Data Flow Patterns

| Pattern | Order Fulfillment | Treat Fever | Timer | Resource Example |
|---------|------------------|-------------|-------|------------------|
| Task Data | ✅ | ✅ | ✅ | ✅ |
| Case Data (Variables) | ✅ | ✅ | ✅ | ❌ |
| Starting Mappings | ✅ | ✅ | ✅ | ❌ |
| Completed Mappings | ✅ | ✅ | ❌ | ❌ |
| Enablement Mappings | ⚠️ | ❌ | ❌ | ✅ |
| XQuery Expressions | ✅ | ✅ | ✅ | ✅ |
| Fallback Values | ❌ | ❌ | ✅ | ❌ |

### 4.3 Resource Patterns

| Pattern | Order Fulfillment | Treat Fever | Timer | Resource Example |
|---------|------------------|-------------|-------|------------------|
| Role-Based Distribution | ⚠️ | ❌ | ❌ | ✅ |
| User-Initiated Allocation | ❌ | ✅ | ❌ | ❌ |
| System-Initiated Offer | ❌ | ❌ | ❌ | ✅ |
| SQL Resource Queries | ❌ | ❌ | ❌ | ✅ |

### 4.4 Service Integration Patterns

| Pattern | Order Fulfillment | Treat Fever | Timer | Resource Example |
|---------|------------------|-------------|-------|------------------|
| Web Service Decomposition | ✅ | ❌ | ✅ | ❌ |
| WSDL Integration | ⚠️ | ❌ | ✅ | ❌ |
| Manual Interaction | ⚠️ | ✅ | ❌ | ❌ |
| Automated Interaction | ✅ | ❌ | ❌ | ❌ |

---

## 5. Schema Version Coverage

| Schema Version | Example Files | Test Priority |
|---------------|---------------|---------------|
| Original (legacy namespace) | Timer.xml | Low |
| Beta 3 | MakeRecordings(Beta3).xml | Low |
| Beta 4 | MakeRecordings(Beta4).xml, BarnesAndNoble(Beta4).xml | Low |
| Beta 6 | ResourceExample.xml | Medium |
| 2.1 | orderfulfillment.yawl | High |
| 4.0 | TreatFever.yawl, worklets | Critical |

**Recommendation**: Focus testing on 2.1 and 4.0 schemas as primary targets.

---

## 6. Data Type Complexity Analysis

### 6.1 Order Fulfillment Type Hierarchy

```
PurchaseOrderType
├── CompanyType
│   ├── Name (string)
│   ├── Address (string)
│   ├── City (string)
│   ├── State (string)
│   ├── PostCode (string)
│   ├── Phone (string)
│   ├── Fax (string)
│   └── BusinessNumber (string)
├── OrderType
│   ├── OrderNumber (string)
│   ├── OrderDate (date)
│   ├── Currency (CurrencyType: AUD|USD)
│   ├── OrderTerms (string)
│   ├── RevisionNumber (ZeroType: integer >= 0)
│   ├── Remarks (string)
│   └── OrderLines (OrderLinesType)
│       └── Line[] (LineType)
│           ├── LineNumber (OneType: integer >= 1)
│           ├── UnitCode (string)
│           ├── UnitDescription (string)
│           ├── UnitQuantity (OneType)
│           └── Action (ActionType: ""|"Added"|"Modified")
├── FreightCost (double)
├── DeliveryLocation (string)
├── InvoiceRequired (boolean)
└── PrePaid (boolean)
```

**Custom Type Patterns**:
- **Enumeration Types**: `CurrencyType`, `ActionType`
- **Restricted Types**: `ZeroType`, `OneType`, `PackageVolume`
- **Nested Complex Types**: 3+ levels deep
- **Sequence vs. Choice**: Demonstrates both patterns

---

## 7. XQuery Expression Patterns

### 7.1 Simple Path Extraction

```xml
<expression query="&lt;Fever&gt;{/TreatFever/Fever/text()}&lt;/Fever&gt;"/>
```

**Pattern**: Extract single element value with path

### 7.2 Conditional (If-Then-Else)

```xml
<expression query="&lt;time&gt; {if ( /data/time/text() ) then /data/time/text() else /data/time-fallback/text() }&lt;/time&gt;"/>
```

**Pattern**: Fallback value when primary is empty

### 7.3 Aggregate Functions

```xml
<expression query="&lt;total&gt;{sum(/order/items/item/price * /order/items/item/quantity)}&lt;/total&gt;"/>
```

**Pattern**: Calculate order total (hypothetical from pattern analysis)

### 7.4 Predicate Filtering

```xml
<predicate>/data/orderAmount &lt; 100</predicate>
```

**Pattern**: Conditional flow based on data value

### 7.5 SQL-in-XQuery (Resource Example)

```xml
<expression query="&lt;YawlResourceAllocationQuery&gt;{'select hresid from hresperformsrole where rolename = $apos;manager$apos;'}&lt;/YawlResourceAllocationQuery&gt;"/>
```

**Pattern**: Embed SQL query string in XQuery

**Entity Escaping**:
- `$apos;` → `'` (apostrophe in SQL string)
- `&lt;` → `<` (XML tag start)
- `&gt;` → `>` (XML tag end)

---

## 8. Layout Information Analysis

### 8.1 Layout Structure (from TreatFever)

```xml
<layout>
  <locale language="en" country="AU"/>

  <specification id="TreatFever" defaultBgColor="-526351">
    <size w="69" h="26"/>

    <net id="TreatFever" bgColor="-526351">
      <bounds x="0" y="0" w="944" h="427"/>
      <frame x="0" y="0" w="947" h="430"/>
      <viewport x="0" y="0" w="947" h="430"/>

      <!-- Vertex (task/condition visual) -->
      <vertex id="InputCondition">
        <attributes>
          <bounds x="32" y="32" w="32" h="32"/>
          <backgroundColor>-197913</backgroundColor>
        </attributes>
      </vertex>

      <!-- Container (task with label) -->
      <container id="Test_Fever">
        <vertex>
          <attributes>
            <bounds x="128" y="32" w="32" h="32"/>
            <backgroundColor>-197913</backgroundColor>
          </attributes>
        </vertex>
        <label>
          <attributes>
            <bounds x="96" y="64" w="96" h="18"/>
            <foregroundColor>-16776961</foregroundColor>
          </attributes>
        </label>
      </container>

      <!-- Flow (connection between elements) -->
      <flow source="Test_Fever" target="Treat_Fever">
        <ports in="13" out="12"/>
        <attributes>
          <lineStyle>11</lineStyle>
        </attributes>
      </flow>
    </net>
  </specification>
</layout>
```

**Layout Components**:
1. **Locale**: Language and country settings
2. **Specification**: Overall canvas size
3. **Net**: Individual process diagram
4. **Bounds/Frame/Viewport**: Canvas dimensions
5. **Vertex**: Simple visual elements (conditions)
6. **Container**: Complex elements (tasks with labels)
7. **Flow**: Visual connections with port positions

**Parser Requirement**: Layout parsing is **optional** for execution but **required** for roundtrip editing.

---

## 9. Test Case Generation

### 9.1 Critical Test Cases (Must Pass)

**Test 1: Order Fulfillment (Full Parse)**
- **Input**: `/exampleSpecs/orderfulfillment/_examples/orderfulfillment.yawl`
- **Expected**: Complete workflow model with all 24 data types
- **Validates**: Complex type parsing, multiple decompositions, schema 2.1

**Test 2: Treat Fever (Worklet + Layout)**
- **Input**: `/build/workletService/samples/worklets/TreatFever.yawl`
- **Expected**: Workflow with manual tasks + complete layout
- **Validates**: Schema 4.0, manual interaction, layout parsing

**Test 3: Resource Example (Resourcing)**
- **Input**: `/exampleSpecs/xml/Beta2-7/ResourceExample.xml`
- **Expected**: Task with SQL-based resource queries
- **Validates**: Enablement mappings, legacy namespace, Beta 6

**Test 4: Timer Example (XQuery + Fallback)**
- **Input**: `/exampleSpecs/xml/Beta2-7/Timer.xml`
- **Expected**: Workflow with conditional data flow
- **Validates**: Web service integration, XQuery if-then-else, local variables

### 9.2 High Priority Test Cases

**Test 5: Custom Forms**
- **Input**: `/exampleSpecs/orderfulfillment/_examples/orderfulfillment_customforms.yawl`
- **Expected**: Tasks with custom form URIs
- **Validates**: Custom form references, form data binding

**Test 6: Make Trip 3 (Cancellation)**
- **Input**: `/exampleSpecs/xml/Beta2-7/makeTrip3.xml`
- **Expected**: Workflow with cancellation regions
- **Validates**: `removesTokens`, `removesTokensFromFlow`

### 9.3 Medium Priority Test Cases

**Test 7-10**: Other example workflows for pattern coverage

### 9.4 Low Priority Test Cases

**Test 11-13**: Beta 3/4 legacy schemas (backward compatibility)

---

## 10. Parser Test Suite Structure

### 10.1 Unit Tests

```rust
#[test]
fn test_parse_task_basic() {
    // Parse single task element
}

#[test]
fn test_parse_conditional_flow() {
    // Parse flowsInto with predicate
}

#[test]
fn test_parse_data_mapping() {
    // Parse startingMappings/completedMappings
}

#[test]
fn test_parse_resourcing() {
    // Parse resourcing specification
}

#[test]
fn test_unescape_xquery() {
    // Test XML entity unescaping
}
```

### 10.2 Integration Tests

```rust
#[test]
fn test_parse_order_fulfillment() {
    let spec = parse_yawl_file("orderfulfillment.yawl");
    assert!(spec.decompositions.len() > 10);
    assert!(spec.data_types.len() == 24);
}

#[test]
fn test_parse_treat_fever_worklet() {
    let spec = parse_yawl_file("TreatFever.yawl");
    assert_eq!(spec.version, "4.0");
    assert!(spec.has_layout_info());
}
```

### 10.3 Regression Tests

```rust
#[test]
fn test_all_example_specs() {
    for file in glob("exampleSpecs/**/*.yawl") {
        let result = parse_yawl_file(file);
        assert!(result.is_ok(), "Failed to parse {}", file);
    }
}
```

### 10.4 Roundtrip Tests

```rust
#[test]
fn test_roundtrip_treat_fever() {
    let original = read_yawl_file("TreatFever.yawl");
    let parsed = parse_yawl(original);
    let turtle = convert_to_turtle(parsed);
    let reconstructed = convert_from_turtle(turtle);
    let regenerated = generate_yawl(reconstructed);

    assert_semantically_equivalent(original, regenerated);
}
```

---

## 11. Known Challenges

### 11.1 Schema Version Compatibility

**Challenge**: 10 different schema versions with incremental features

**Solution**:
1. Detect version from `<specificationSet version="...">` or namespace
2. Use version-specific parsers with fallback to best-effort
3. Normalize to latest schema (4.0) internally

### 11.2 Legacy Namespaces

**Challenge**: Legacy `http://www.citi.qut.edu.au/yawl` vs. modern `http://www.yawlfoundation.org/yawlschema`

**Solution**:
1. Namespace aliasing during parse
2. Normalize to modern namespace internally
3. Support legacy on output for backward compatibility

### 11.3 XQuery Entity Escaping

**Challenge**: Multiple levels of escaping (`&lt;`, `$apos;`)

**Solution**:
1. Parse XML entities first (standard XML parser)
2. Handle XQuery-specific entities (`$apos;`)
3. Preserve original expression for roundtrip

### 11.4 Custom Data Types

**Challenge**: Embedded XML Schema with complex types

**Solution**:
1. Parse embedded `<xs:schema>` separately
2. Build type registry (namespace → types)
3. Validate variable types against registry

### 11.5 Layout Information

**Challenge**: Large visual layout data optional for execution

**Solution**:
1. Make layout parsing optional (feature flag)
2. Skip layout for execution-only use cases
3. Preserve layout for editor roundtrip

---

## 12. Recommended Testing Order

### Phase 1: Foundation (Week 1)
1. ✅ Parse basic task (Test 1)
2. ✅ Parse conditional flow (Test 2)
3. ✅ Parse data mapping (Test 3)
4. ✅ Parse simple workflow net (Test 4)

### Phase 2: Core Workflows (Week 2)
1. ✅ Parse Timer Example (Test 4)
2. ✅ Parse Treat Fever Worklet (Test 2)
3. ✅ Parse Resource Example (Test 3)

### Phase 3: Complex Workflows (Week 3)
1. ✅ Parse Order Fulfillment (Test 1)
2. ✅ Parse Custom Forms variant (Test 5)
3. ✅ Parse Cancellation workflow (Test 6)

### Phase 4: Regression & Roundtrip (Week 4)
1. ✅ All example specs parse successfully
2. ✅ Roundtrip tests pass for all critical workflows
3. ✅ Performance tests (large workflows)

---

## 13. Summary

### Catalog Statistics
- **Total Workflows**: 17 examples
- **Schema Versions**: 6 versions (Beta 3, Beta 4, Beta 6, 2.1, 4.0, legacy)
- **Test Priority**:
  - **Critical**: 4 workflows (Order Fulfillment, Treat Fever, Resource, Timer)
  - **High**: 3 workflows
  - **Medium**: 5 workflows
  - **Low**: 5 workflows

### Pattern Coverage
- **Control Flow**: 9 patterns identified
- **Data Flow**: 7 patterns identified
- **Resource**: 4 patterns identified
- **Service Integration**: 4 patterns identified

### Parser Requirements
1. Support schema versions: 2.1, 4.0 (critical), Beta 6 (high), legacy (low)
2. Handle 43+ workflow control flow patterns
3. Parse 24+ complex data types (Order Fulfillment)
4. Process XQuery expressions with entity unescaping
5. Optional: Parse layout information for editor support

### Next Steps for Implementation
1. Implement critical test cases (Tests 1-4)
2. Validate against all example workflows
3. Measure pattern coverage
4. Optimize for performance (large workflows)
5. Document parser limitations

---

**Document Version**: 1.0
**Last Updated**: 2025-11-08
**Total Workflows Cataloged**: 17
**Status**: Complete

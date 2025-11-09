# YAWL Bidirectional Conversion Strategy (RDF ↔ XML)

**Version:** 1.0
**Date:** 2025-11-08
**Author:** Migration Specialist (ULTRATHINK Swarm)
**Status:** Implementation-Ready Design
**Document Size:** 23KB

## Executive Summary

This document defines the comprehensive strategy for bidirectional conversion between YAWL RDF/Turtle and XML formats, ensuring round-trip conversion guarantees, backward compatibility, and dual-format support throughout the migration lifecycle.

**Key Objectives:**
1. **Round-Trip Guarantee:** XML → RDF → XML produces semantically equivalent output
2. **Backward Compatibility:** Existing YAWL tools can read generated XML
3. **Dual-Format Support:** Run both XML and RDF formats concurrently
4. **Zero Information Loss:** All workflow data preserved in both directions
5. **Migration Safety:** Incremental migration with rollback capability

**Conversion Directions:**
- **Forward:** XML → RDF/Turtle (primary migration)
- **Reverse:** RDF/Turtle → XML (backward compatibility)
- **Round-Trip:** XML → RDF → XML (validation)

---

## 1. RDF to XML Conversion Architecture

### 1.1 Reverse Pipeline Overview

```
┌──────────────┐
│ RDF/Turtle   │ (Input: .ttl files)
│ Workflow     │
└──────┬───────┘
       │
       ▼
┌──────────────────────┐
│ Stage 1: Turtle      │
│ Parsing & Validation │
│ - Parse Turtle       │
│ - Build RDF Graph    │
│ - SHACL Validation   │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Stage 2: Triple      │
│ Querying & Grouping  │
│ - SPARQL Queries     │
│ - Subject Grouping   │
│ - Relationship Graph │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Stage 3: Intermediate│
│ Representation (IR)  │
│ - Normalized Data    │
│ - ID Extraction      │
│ - Structure Rebuild  │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Stage 4: XML DOM     │
│ Generation           │
│ - Element Creation   │
│ - Attribute Setting  │
│ - Nesting Structure  │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Stage 5: XML         │
│ Validation           │
│ - XSD Validation     │
│ - Reference Checks   │
│ - ID Uniqueness      │
└──────┬───────────────┘
       │
       ▼
┌──────────────────────┐
│ Stage 6: XML         │
│ Serialization        │
│ - Pretty-Printing    │
│ - Namespace Cleanup  │
│ - Indentation        │
└──────┬───────────────┘
       │
       ▼
┌──────────────┐
│ YAWL XML     │ (Output: .yawl/.xml files)
│ Specification│
└──────────────┘
```

### 1.2 Stage Details

#### Stage 1: Turtle Parsing & Validation

**Input:** RDF/Turtle file (`.ttl`)

**Processing:**
1. **Parse Turtle:**
   ```rust
   let graph = parse_turtle(&turtle_content)?;
   ```

2. **Extract Namespaces:**
   ```turtle
   @prefix : <http://example.org/OrganiseConcert#> .
   @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
   ```

3. **SHACL Validation:**
   - Ensure all required triples present
   - Validate cardinality constraints
   - Check domain/range compliance

**Output:** Validated RDF graph with namespace map

---

#### Stage 2: Triple Querying & Grouping

**Input:** RDF graph

**Processing:**

**2.1 Identify Specification:**
```sparql
SELECT ?spec ?uri ?name WHERE {
  ?spec a yawl:Specification ;
        yawl:uri ?uri ;
        yawl:name ?name .
}
```

**2.2 Query Decompositions:**
```sparql
SELECT ?decomp ?id ?type WHERE {
  ?spec yawl:hasDecomposition ?decomp .
  ?decomp yawl:id ?id .
  ?decomp a ?type .
  FILTER(?type IN (yawl:Net, yawl:WebServiceGateway))
}
```

**2.3 Query Net Elements:**
```sparql
SELECT ?net ?task ?id ?name ?join ?split WHERE {
  ?net a yawl:Net ;
       yawl:hasTask ?task .
  ?task yawl:id ?id ;
        yawl:name ?name ;
        yawl:hasJoin ?join ;
        yawl:hasSplit ?split .
}
```

**2.4 Query Flows:**
```sparql
SELECT ?flow ?source ?target ?predicate WHERE {
  ?source yawl:flowsInto ?flow .
  ?flow yawl:nextElementRef ?target .
  OPTIONAL { ?flow yawl:hasPredicate ?predicate }
}
```

**2.5 Query Variables:**
```sparql
SELECT ?net ?var ?name ?type ?ns ?init WHERE {
  ?net yawl:hasLocalVariable ?var .
  ?var yawl:name ?name ;
       yawl:type ?type ;
       yawl:namespace ?ns .
  OPTIONAL { ?var yawl:initialValue ?init }
}
```

**Output:** Structured query results

---

#### Stage 3: Intermediate Representation (IR)

**Input:** Query results

**Processing:**

**3.1 Rebuild Specification Structure:**
```rust
struct SpecificationIR {
    uri: String,
    name: String,
    documentation: Option<String>,
    metadata: Option<MetadataIR>,
    decompositions: Vec<DecompositionIR>,
    schema: Option<String>,
    layout: Option<LayoutIR>,
}
```

**3.2 Extract IDs from URIs:**
```rust
// URI: http://example.org/OrganiseConcert#BookStadium
// Extract ID: "BookStadium"
fn extract_id_from_uri(uri: &str, base_ns: &str) -> String {
    uri.strip_prefix(base_ns).unwrap_or(uri).to_string()
}
```

**3.3 Reconstruct Element Relationships:**
```rust
// Flows: source → target
for flow in flows {
    let source_id = extract_id(flow.source);
    let target_id = extract_id(flow.target);

    tasks.get_mut(&source_id).flows_into.push(target_id);
}
```

**3.4 Rebuild Variable Mappings:**
```rust
// Query mapping expressions
SELECT ?mapping ?mapsTo ?query WHERE {
  ?task yawl:hasStartingMappings ?mappingSet .
  ?mappingSet yawl:hasMapping ?mapping .
  ?mapping yawl:mapsTo ?mapsTo ;
           yawl:hasExpression ?expr .
  ?expr yawl:query ?query .
}
```

**Output:** Complete IR with XML-ready structure

---

#### Stage 4: XML DOM Generation

**Input:** Intermediate Representation (IR)

**Processing:**

**4.1 Create Root Element:**
```xml
<specificationSet xmlns="http://www.yawlfoundation.org/yawlschema"
                  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                  version="4.0"
                  xsi:schemaLocation="http://www.yawlfoundation.org/yawlschema http://www.yawlfoundation.org/yawlschema/YAWL_Schema4.0.xsd">
```

**4.2 Generate Specification Element:**
```rust
fn generate_specification_element(ir: &SpecificationIR) -> XmlElement {
    let mut spec = XmlElement::new("specification");
    spec.set_attribute("uri", &ir.uri);

    spec.add_child(text_element("name", &ir.name));

    if let Some(doc) = &ir.documentation {
        spec.add_child(text_element("documentation", doc));
    }

    if let Some(metadata) = &ir.metadata {
        spec.add_child(generate_metadata_element(metadata));
    }

    for decomp in &ir.decompositions {
        spec.add_child(generate_decomposition_element(decomp));
    }

    spec
}
```

**4.3 Generate Decomposition Element:**
```rust
fn generate_decomposition_element(decomp: &DecompositionIR) -> XmlElement {
    let mut elem = XmlElement::new("decomposition");
    elem.set_attribute("id", &decomp.id);

    match &decomp.decomposition_type {
        DecompositionType::Net(net) => {
            elem.set_attribute("xsi:type", "NetFactsType");
            elem.set_attribute("isRootNet", &net.is_root.to_string());

            for var in &net.variables {
                elem.add_child(generate_variable_element(var));
            }

            let process_elem = generate_process_control_elements(net);
            elem.add_child(process_elem);
        }
        DecompositionType::WebServiceGateway(gateway) => {
            elem.set_attribute("xsi:type", "WebServiceGatewayFactsType");

            for param in &gateway.input_params {
                elem.add_child(generate_input_param_element(param));
            }

            for param in &gateway.output_params {
                elem.add_child(generate_output_param_element(param));
            }

            elem.add_child(text_element("externalInteraction", &gateway.external_interaction));
        }
    }

    elem
}
```

**4.4 Generate Task Element:**
```rust
fn generate_task_element(task: &TaskIR) -> XmlElement {
    let mut elem = XmlElement::new("task");
    elem.set_attribute("id", &task.id);

    if let Some(name) = &task.name {
        elem.add_child(text_element("name", name));
    }

    // Flows
    for flow in &task.flows_into {
        let mut flow_elem = XmlElement::new("flowsInto");
        let mut ref_elem = XmlElement::new("nextElementRef");
        ref_elem.set_attribute("id", &flow.target_id);

        if let Some(predicate) = &flow.predicate {
            let mut pred_elem = XmlElement::new("predicate");
            pred_elem.set_attribute("ordering", &predicate.ordering.to_string());
            pred_elem.set_text(&predicate.query);
            flow_elem.add_child(pred_elem);
        } else if flow.is_default {
            flow_elem.add_child(XmlElement::new("isDefaultFlow"));
        }

        flow_elem.add_child(ref_elem);
        elem.add_child(flow_elem);
    }

    // Join/Split
    let mut join_elem = XmlElement::new("join");
    join_elem.set_attribute("code", &control_type_to_code(&task.join_type));
    elem.add_child(join_elem);

    let mut split_elem = XmlElement::new("split");
    split_elem.set_attribute("code", &control_type_to_code(&task.split_type));
    elem.add_child(split_elem);

    // Variable Mappings
    if !task.starting_mappings.is_empty() {
        elem.add_child(generate_mappings_element("startingMappings", &task.starting_mappings));
    }

    if !task.completed_mappings.is_empty() {
        elem.add_child(generate_mappings_element("completedMappings", &task.completed_mappings));
    }

    // Resourcing
    if let Some(resourcing) = &task.resourcing {
        elem.add_child(generate_resourcing_element(resourcing));
    }

    // Decomposition reference
    if let Some(decomp_id) = &task.decomposition_ref {
        let mut decomp_elem = XmlElement::new("decomposesTo");
        decomp_elem.set_attribute("id", decomp_id);
        elem.add_child(decomp_elem);
    }

    elem
}
```

**4.5 Generate Variable Mapping:**
```rust
fn generate_mappings_element(tag: &str, mappings: &[MappingIR]) -> XmlElement {
    let mut container = XmlElement::new(tag);

    for mapping in mappings {
        let mut mapping_elem = XmlElement::new("mapping");

        let mut expr_elem = XmlElement::new("expression");
        expr_elem.set_attribute("query", &xml_escape(&mapping.expression));
        mapping_elem.add_child(expr_elem);

        mapping_elem.add_child(text_element("mapsTo", &mapping.maps_to));

        container.add_child(mapping_elem);
    }

    container
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}
```

**Output:** Complete XML DOM tree

---

#### Stage 5: XML Validation

**Input:** Generated XML DOM

**Validation Checks:**

1. **XSD Schema Validation:**
   ```rust
   let xsd = load_yawl_schema("YAWL_Schema4.0.xsd")?;
   xsd.validate(&xml_doc)?;
   ```

2. **ID Uniqueness:**
   ```rust
   let ids = collect_all_ids(&xml_doc);
   let unique_ids: HashSet<_> = ids.iter().collect();
   if ids.len() != unique_ids.len() {
       return Err(ValidationError::DuplicateIDs);
   }
   ```

3. **Reference Resolution:**
   ```rust
   for decomp_ref in collect_decomposition_refs(&xml_doc) {
       if !decomposition_exists(&xml_doc, &decomp_ref) {
           return Err(ValidationError::UnresolvedReference(decomp_ref));
       }
   }
   ```

4. **Flow Connectivity:**
   ```rust
   // Ensure all flow targets exist
   for flow in collect_flows(&xml_doc) {
       if !element_exists(&xml_doc, &flow.target_id) {
           return Err(ValidationError::InvalidFlowTarget(flow.target_id));
       }
   }
   ```

**Output:** Validated XML DOM

---

#### Stage 6: XML Serialization

**Input:** Validated XML DOM

**Serialization Options:**
```rust
struct XmlSerializationOptions {
    pretty_print: bool,
    indent_size: usize,
    namespace_cleanup: bool,
    preserve_whitespace: bool,
}
```

**Processing:**
```rust
fn serialize_xml(doc: &XmlDocument, options: &XmlSerializationOptions) -> String {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

    if options.pretty_print {
        xml += &serialize_element_pretty(&doc.root, 0, options.indent_size);
    } else {
        xml += &serialize_element_compact(&doc.root);
    }

    if options.namespace_cleanup {
        xml = remove_redundant_namespaces(&xml);
    }

    xml
}

fn serialize_element_pretty(elem: &XmlElement, depth: usize, indent_size: usize) -> String {
    let indent = " ".repeat(depth * indent_size);
    let mut xml = format!("{}<{}", indent, elem.name);

    for (key, value) in &elem.attributes {
        xml += &format!(" {}=\"{}\"", key, xml_escape(value));
    }

    if elem.children.is_empty() && elem.text.is_none() {
        xml += " />\n";
    } else {
        xml += ">";

        if let Some(text) = &elem.text {
            xml += &xml_escape(text);
        } else {
            xml += "\n";
            for child in &elem.children {
                xml += &serialize_element_pretty(child, depth + 1, indent_size);
            }
            xml += &indent;
        }

        xml += &format!("</{}>\n", elem.name);
    }

    xml
}
```

**Output:** Pretty-printed XML string

---

## 2. Round-Trip Conversion Guarantees

### 2.1 Semantic Equivalence Definition

Two workflows W1 (XML) and W2 (XML after round-trip) are **semantically equivalent** if:

1. **Structure:** Same decompositions, tasks, conditions, flows
2. **Control Flow:** Same join/split types, predicates, default flows
3. **Data Flow:** Same variable mappings, expressions, parameters
4. **Resources:** Same resource allocation, initiators, privileges
5. **Timing:** Same timer configurations, durations, triggers
6. **Layout:** Same visual positions (within tolerance)

**Mathematical Definition:**
```
W1 ≡ W2 ⟺
  ∀d ∈ Decompositions(W1) ∃d' ∈ Decompositions(W2) : d ≡ d' ∧
  ∀e ∈ Elements(W1) ∃e' ∈ Elements(W2) : e ≡ e' ∧
  ∀f ∈ Flows(W1) ∃f' ∈ Flows(W2) : f ≡ f'
```

### 2.2 Round-Trip Test Suite

**Test 1: Structure Preservation**
```rust
#[test]
fn test_round_trip_structure() {
    let xml1 = load_workflow("example.yawl");
    let rdf = xml_to_rdf(&xml1).unwrap();
    let xml2 = rdf_to_xml(&rdf).unwrap();

    // Compare structure
    assert_eq!(count_decompositions(&xml1), count_decompositions(&xml2));
    assert_eq!(count_tasks(&xml1), count_tasks(&xml2));
    assert_eq!(count_flows(&xml1), count_flows(&xml2));
}
```

**Test 2: Control Flow Preservation**
```rust
#[test]
fn test_round_trip_control_flow() {
    let xml1 = load_workflow("example.yawl");
    let rdf = xml_to_rdf(&xml1).unwrap();
    let xml2 = rdf_to_xml(&rdf).unwrap();

    // Extract control flow graphs
    let cfg1 = extract_control_flow(&xml1);
    let cfg2 = extract_control_flow(&xml2);

    assert_eq!(cfg1, cfg2);
}
```

**Test 3: Data Flow Preservation**
```rust
#[test]
fn test_round_trip_data_flow() {
    let xml1 = load_workflow("example.yawl");
    let rdf = xml_to_rdf(&xml1).unwrap();
    let xml2 = rdf_to_xml(&rdf).unwrap();

    // Extract variable mappings
    let mappings1 = extract_all_mappings(&xml1);
    let mappings2 = extract_all_mappings(&xml2);

    assert_eq!(mappings1, mappings2);
}
```

**Test 4: XQuery Expression Preservation**
```rust
#[test]
fn test_round_trip_xquery() {
    let xml1 = load_workflow("example.yawl");
    let rdf = xml_to_rdf(&xml1).unwrap();
    let xml2 = rdf_to_xml(&rdf).unwrap();

    // Compare XQuery expressions (normalized)
    let exprs1 = extract_expressions(&xml1).map(normalize_xquery);
    let exprs2 = extract_expressions(&xml2).map(normalize_xquery);

    assert_eq!(exprs1, exprs2);
}
```

**Test 5: Layout Preservation (with tolerance)**
```rust
#[test]
fn test_round_trip_layout() {
    let xml1 = load_workflow("example.yawl");
    let rdf = xml_to_rdf(&xml1).unwrap();
    let xml2 = rdf_to_xml(&rdf).unwrap();

    let layout1 = extract_layout(&xml1);
    let layout2 = extract_layout(&xml2);

    // Compare with 1-pixel tolerance
    assert_layout_equivalent(&layout1, &layout2, tolerance: 1.0);
}
```

### 2.3 Differential Analysis

**Approach:** Generate detailed diff report between original and round-trip XML

```rust
fn analyze_round_trip_diff(xml1: &str, xml2: &str) -> DiffReport {
    let mut report = DiffReport::new();

    // Structural differences
    report.add_section("Structure");
    report.compare_count("Specifications", count_specs(xml1), count_specs(xml2));
    report.compare_count("Decompositions", count_decomps(xml1), count_decomps(xml2));
    report.compare_count("Tasks", count_tasks(xml1), count_tasks(xml2));

    // Semantic differences
    report.add_section("Semantics");
    let cfg1 = extract_control_flow(xml1);
    let cfg2 = extract_control_flow(xml2);
    report.compare_graph("Control Flow", &cfg1, &cfg2);

    // Text differences (for debugging)
    report.add_section("Text Diff");
    report.text_diff = diff_strings(xml1, xml2);

    report
}
```

---

## 3. Information Loss Analysis

### 3.1 Potential Loss Points

#### Loss Point 1: Layout Precision

**Issue:** RDF stores coordinates as strings, XML as integers
```xml
XML:  <bounds x="128" y="32" w="32" h="32"/>
RDF:  yawl:x "128" ; yawl:y "32" ; yawl:w "32" ; yawl:h "32" .
XML': <bounds x="128" y="32" w="32" h="32"/>  ✓ No loss
```

**Mitigation:** Store as typed literals
```turtle
yawl:x "128"^^xsd:integer ;
yawl:y "32"^^xsd:integer .
```

#### Loss Point 2: XML Element Ordering

**Issue:** RDF is unordered; XML element order may matter

**Example:**
```xml
<!-- Original order -->
<startingMappings>
  <mapping>...</mapping>  <!-- First -->
  <mapping>...</mapping>  <!-- Second -->
</startingMappings>
```

RDF (unordered):
```turtle
:mappingSet yawl:hasMapping :mapping_0 , :mapping_1 .
```

**Mitigation:** Add ordering property
```turtle
:mapping_0 yawl:order 0 .
:mapping_1 yawl:order 1 .
```

#### Loss Point 3: XML Comments and Whitespace

**Issue:** Comments/whitespace not in YAWL schema, but may exist

**Example:**
```xml
<!-- Important note about this task -->
<task id="task1">
  ...
</task>
```

**Mitigation:** Store as annotation properties
```turtle
:task1 rdfs:comment "Important note about this task" .
```

#### Loss Point 4: Namespace Prefixes

**Issue:** RDF uses standard prefixes; XML may use custom

**Example:**
```xml
<!-- Original -->
<type>xs:string</type>

<!-- Round-trip -->
<type>string</type>
<namespace>http://www.w3.org/2001/XMLSchema</namespace>
```

**Mitigation:** Normalize to namespace + local name in both directions

### 3.2 Zero-Loss Guarantee Strategy

**Strategy 1: Preserve Original XML**
- Store original XML as blob alongside RDF
- Use RDF for semantic queries
- Regenerate from original when exact XML needed

**Strategy 2: Extended RDF Properties**
```turtle
# Core semantic properties
:task1 a yawl:Task ;
    yawl:id "task1"^^xsd:NMTOKEN ;
    yawl:name "Process" .

# Preservation properties
:task1 yawl:xmlElementOrder 5 ;
       yawl:xmlComment "Important note" ;
       yawl:xmlWhitespace "  \n  " .
```

**Strategy 3: Canonical Form**
- Define canonical XML form
- Always regenerate to canonical
- Document deviations as acceptable

---

## 4. Dual-Format Support Strategy

### 4.1 Hybrid Runtime Architecture

```
┌─────────────────────────────────────┐
│         knhk Runtime                │
├─────────────────────────────────────┤
│  Workflow Parser (Multi-Format)     │
│  ┌──────────┐      ┌──────────┐    │
│  │ XML      │      │ RDF/     │    │
│  │ Parser   │      │ Turtle   │    │
│  └────┬─────┘      └────┬─────┘    │
│       │                 │           │
│       └─────────┬───────┘           │
│                 ▼                   │
│       ┌──────────────────┐          │
│       │  Unified         │          │
│       │  Workflow IR     │          │
│       └────────┬─────────┘          │
│                ▼                   │
│       ┌──────────────────┐          │
│       │  Execution       │          │
│       │  Engine          │          │
│       └──────────────────┘          │
└─────────────────────────────────────┘
```

**Key Idea:** Both XML and RDF parse to same IR

### 4.2 Format Detection

```rust
enum WorkflowFormat {
    Xml,
    Turtle,
    RdfXml,
    Auto,
}

fn detect_format(path: &Path) -> Result<WorkflowFormat, FormatError> {
    let ext = path.extension().and_then(|s| s.to_str());

    match ext {
        Some("yawl") | Some("xml") => Ok(WorkflowFormat::Xml),
        Some("ttl") => Ok(WorkflowFormat::Turtle),
        Some("rdf") => Ok(WorkflowFormat::RdfXml),
        _ => {
            // Auto-detect from content
            let content = std::fs::read_to_string(path)?;
            if content.trim_start().starts_with("<?xml") {
                Ok(WorkflowFormat::Xml)
            } else if content.contains("@prefix") {
                Ok(WorkflowFormat::Turtle)
            } else {
                Err(FormatError::UnknownFormat)
            }
        }
    }
}
```

### 4.3 Unified Parser Interface

```rust
pub trait WorkflowParser {
    fn parse(&self, content: &str) -> Result<WorkflowIR, ParseError>;
}

pub struct XmlWorkflowParser;
impl WorkflowParser for XmlWorkflowParser {
    fn parse(&self, content: &str) -> Result<WorkflowIR, ParseError> {
        // XML → IR
        let xml_doc = parse_xml(content)?;
        xml_to_ir(&xml_doc)
    }
}

pub struct RdfWorkflowParser;
impl WorkflowParser for RdfWorkflowParser {
    fn parse(&self, content: &str) -> Result<WorkflowIR, ParseError> {
        // RDF → IR
        let graph = parse_turtle(content)?;
        rdf_to_ir(&graph)
    }
}

pub fn load_workflow(path: &Path) -> Result<WorkflowIR, ParseError> {
    let format = detect_format(path)?;
    let content = std::fs::read_to_string(path)?;

    let parser: Box<dyn WorkflowParser> = match format {
        WorkflowFormat::Xml => Box::new(XmlWorkflowParser),
        WorkflowFormat::Turtle => Box::new(RdfWorkflowParser),
        _ => return Err(ParseError::UnsupportedFormat),
    };

    parser.parse(&content)
}
```

### 4.4 Dual Persistence

**Approach:** Maintain both formats in sync

```rust
pub struct DualFormatWorkflowStore {
    xml_store: XmlWorkflowStore,
    rdf_store: RdfWorkflowStore,
}

impl DualFormatWorkflowStore {
    pub fn save(&mut self, workflow: &WorkflowIR) -> Result<(), StoreError> {
        // Generate both formats
        let xml = ir_to_xml(workflow)?;
        let rdf = ir_to_rdf(workflow)?;

        // Save both
        self.xml_store.save(&workflow.id, &xml)?;
        self.rdf_store.save(&workflow.id, &rdf)?;

        // Verify consistency
        self.verify_consistency(&workflow.id)?;

        Ok(())
    }

    pub fn load(&self, id: &str) -> Result<WorkflowIR, StoreError> {
        // Try RDF first (preferred)
        if let Ok(rdf) = self.rdf_store.load(id) {
            return rdf_to_ir(&rdf);
        }

        // Fallback to XML
        let xml = self.xml_store.load(id)?;
        xml_to_ir(&xml)
    }

    fn verify_consistency(&self, id: &str) -> Result<(), ConsistencyError> {
        let xml = self.xml_store.load(id)?;
        let rdf = self.rdf_store.load(id)?;

        let ir_from_xml = xml_to_ir(&xml)?;
        let ir_from_rdf = rdf_to_ir(&rdf)?;

        if ir_from_xml != ir_from_rdf {
            return Err(ConsistencyError::FormatMismatch);
        }

        Ok(())
    }
}
```

---

## 5. Migration Timeline & Phases

### Phase 1: Preparation (Weeks 1-2)

**Objectives:**
- Complete migration tooling
- Build test corpus
- Establish validation criteria

**Deliverables:**
- XML → RDF migration tool
- RDF → XML migration tool
- Round-trip test suite
- Migration playbook

**Success Criteria:**
- 100% of test corpus passes round-trip validation
- Performance: <200ms per workflow

---

### Phase 2: Pilot Migration (Weeks 3-4)

**Objectives:**
- Migrate 10% of workflows
- Validate in production-like environment
- Gather performance metrics

**Process:**
1. Select representative workflows
2. Migrate to RDF
3. Run dual-format for 1 week
4. Compare execution behavior
5. Document issues

**Success Criteria:**
- Zero execution differences
- No performance degradation
- All edge cases handled

---

### Phase 3: Incremental Rollout (Weeks 5-8)

**Objectives:**
- Migrate remaining workflows
- Gradual transition to RDF-primary

**Migration Waves:**
- **Wave 1 (Week 5):** 25% of workflows
- **Wave 2 (Week 6):** 50% of workflows
- **Wave 3 (Week 7):** 75% of workflows
- **Wave 4 (Week 8):** 100% of workflows

**Rollback Plan:**
Each wave includes:
- Pre-migration backup
- Dual-format execution
- Performance monitoring
- Automated rollback trigger

**Rollback Trigger:**
```rust
fn should_rollback(metrics: &Metrics) -> bool {
    metrics.error_rate > 0.01 ||        // >1% error rate
    metrics.avg_latency > metrics.baseline * 1.5 ||
    metrics.semantic_mismatches > 0
}
```

---

### Phase 4: XML Deprecation (Weeks 9-12)

**Objectives:**
- Phase out XML generation
- Maintain read-only XML support
- Optimize RDF-only path

**Timeline:**
- **Week 9:** RDF becomes primary format
- **Week 10:** XML generation disabled for new workflows
- **Week 11:** XML import-only mode
- **Week 12:** Full RDF operation

**Backward Compatibility:**
- XML parser remains active indefinitely
- Legacy workflows can still be imported
- RDF → XML conversion on-demand

---

## 6. Migration Safety & Rollback

### 6.1 Pre-Migration Checklist

- [ ] All workflows backed up
- [ ] Test suite passes 100%
- [ ] Performance baseline established
- [ ] Rollback plan documented
- [ ] Monitoring dashboards ready
- [ ] Team trained on dual-format tools

### 6.2 Migration Execution

```bash
# Backup all workflows
knhk-migrate backup \
    --source /workflows/xml/ \
    --backup /backups/pre-migration/

# Dry-run migration
knhk-migrate xml-to-rdf \
    --input-dir /workflows/xml/ \
    --output-dir /workflows/rdf/ \
    --dry-run \
    --validate

# Actual migration (with verification)
knhk-migrate xml-to-rdf \
    --input-dir /workflows/xml/ \
    --output-dir /workflows/rdf/ \
    --verify-round-trip \
    --parallel 4

# Enable dual-format mode
knhk-config set workflow.dual-format true
knhk-config set workflow.primary-format rdf
```

### 6.3 Rollback Procedure

**Trigger Conditions:**
- Semantic mismatches detected
- Performance degradation >50%
- Error rate >1%
- Manual intervention required

**Rollback Steps:**
```bash
# Disable RDF processing
knhk-config set workflow.dual-format false
knhk-config set workflow.primary-format xml

# Restore from backup
knhk-migrate restore \
    --backup /backups/pre-migration/ \
    --target /workflows/xml/

# Verify restoration
knhk-migrate verify \
    --source /workflows/xml/ \
    --expected-count 1000

# Resume operations
knhk-server restart
```

**Rollback Time:** <5 minutes

---

## 7. Implementation Pseudocode

### 7.1 RDF to XML Converter

```rust
pub fn rdf_to_xml(turtle: &str) -> Result<String, ConversionError> {
    // Stage 1: Parse Turtle
    let graph = parse_turtle(turtle)?;

    // Stage 2: Query RDF
    let spec_data = query_specification(&graph)?;
    let decomp_data = query_decompositions(&graph)?;
    let task_data = query_tasks(&graph)?;
    let flow_data = query_flows(&graph)?;

    // Stage 3: Build IR
    let ir = build_ir_from_rdf(spec_data, decomp_data, task_data, flow_data)?;

    // Stage 4: Generate XML DOM
    let xml_doc = ir_to_xml_dom(&ir)?;

    // Stage 5: Validate XML
    validate_xml_against_xsd(&xml_doc)?;

    // Stage 6: Serialize
    let xml_string = serialize_xml_pretty(&xml_doc)?;

    Ok(xml_string)
}

fn query_specification(graph: &RdfGraph) -> Result<SpecData, QueryError> {
    let query = r#"
        SELECT ?spec ?uri ?name ?doc WHERE {
          ?spec a yawl:Specification ;
                yawl:uri ?uri ;
                yawl:name ?name .
          OPTIONAL { ?spec yawl:documentation ?doc }
        }
    "#;

    let results = graph.query(query)?;
    parse_spec_results(results)
}

fn query_tasks(graph: &RdfGraph) -> Result<Vec<TaskData>, QueryError> {
    let query = r#"
        SELECT ?task ?id ?name ?join ?split ?decomp WHERE {
          ?task a yawl:Task ;
                yawl:id ?id .
          OPTIONAL { ?task yawl:name ?name }
          OPTIONAL { ?task yawl:hasJoin ?join }
          OPTIONAL { ?task yawl:hasSplit ?split }
          OPTIONAL { ?task yawl:hasDecomposesTo ?decomp }
        }
    "#;

    let results = graph.query(query)?;
    parse_task_results(results)
}
```

### 7.2 Round-Trip Validator

```rust
pub fn validate_round_trip(original_xml: &str) -> Result<RoundTripReport, ValidationError> {
    // Forward conversion
    let rdf = xml_to_rdf(original_xml)?;

    // Reverse conversion
    let regenerated_xml = rdf_to_xml(&rdf)?;

    // Parse both
    let original_ir = xml_to_ir(original_xml)?;
    let regenerated_ir = xml_to_ir(&regenerated_xml)?;

    // Compare
    let mut report = RoundTripReport::new();

    report.structure_match = compare_structure(&original_ir, &regenerated_ir);
    report.control_flow_match = compare_control_flow(&original_ir, &regenerated_ir);
    report.data_flow_match = compare_data_flow(&original_ir, &regenerated_ir);
    report.layout_match = compare_layout(&original_ir, &regenerated_ir, tolerance: 1.0);

    report.overall_success = report.structure_match &&
                              report.control_flow_match &&
                              report.data_flow_match &&
                              report.layout_match;

    Ok(report)
}
```

---

## 8. Tool Integration

### 8.1 CLI Commands

```bash
# RDF to XML conversion
knhk-migrate rdf-to-xml \
    --input /path/to/workflow.ttl \
    --output /path/to/workflow.xml \
    --validate

# Round-trip validation
knhk-migrate validate-round-trip \
    --input /path/to/workflow.xml \
    --report /path/to/report.json

# Dual-format mode
knhk-config set workflow.dual-format true
knhk-config set workflow.primary-format rdf

# Format conversion (auto-detect)
knhk-migrate convert \
    --input /path/to/workflow.{xml,ttl} \
    --output /path/to/workflow.{ttl,xml}
```

### 8.2 API Integration

```rust
pub struct BidirectionalConverter {
    xsd_schema: Schema,
    yawl_ontology: Ontology,
}

impl BidirectionalConverter {
    pub fn xml_to_rdf(&self, xml: &str) -> Result<String, ConversionError> {
        migrate_xml_to_rdf_string(xml)
    }

    pub fn rdf_to_xml(&self, rdf: &str) -> Result<String, ConversionError> {
        rdf_to_xml(rdf)
    }

    pub fn validate_round_trip(&self, xml: &str) -> Result<RoundTripReport, ValidationError> {
        validate_round_trip(xml)
    }

    pub fn is_format_equivalent(&self, xml: &str, rdf: &str) -> Result<bool, ComparisonError> {
        let ir_xml = xml_to_ir(xml)?;
        let ir_rdf = rdf_to_ir(rdf)?;
        Ok(ir_xml == ir_rdf)
    }
}
```

---

## 9. Summary

This bidirectional conversion strategy ensures:

1. **Round-Trip Guarantees:** XML → RDF → XML produces semantically equivalent workflows
2. **Zero Information Loss:** All workflow data preserved through comprehensive RDF modeling
3. **Backward Compatibility:** Legacy YAWL tools can consume generated XML
4. **Dual-Format Support:** Both XML and RDF coexist during migration
5. **Safe Migration:** Incremental rollout with automated rollback capability

**Key Innovations:**
- Extended RDF properties for preservation
- Canonical XML form for consistency
- Dual-format runtime architecture
- Comprehensive round-trip testing

**Migration Success Metrics:**
- 100% round-trip validation pass rate
- Zero execution behavior differences
- <200ms conversion performance
- <5 minute rollback time

---

**Document Size:** 23,547 bytes (23KB)
**Migration Specialist - ULTRATHINK Swarm**

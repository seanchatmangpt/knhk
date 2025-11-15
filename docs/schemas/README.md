# Schema & Ontology Documentation

This document provides a comprehensive guide to all schema definitions, ontology files, and semantic structures in KNHK.

## 📚 Overview

KNHK uses multiple semantic definition languages to ensure consistent, validated behavior:

1. **OpenTelemetry Weaver YAML** - Runtime telemetry schema (source of truth validation)
2. **RDF/OWL Ontologies** - Semantic relationships and domain modeling
3. **SHACL Shapes** - Data shape validation
4. **Turtle (TTL) Files** - RDF graph definitions

## 🔍 Quick Navigation

- **New to schemas?** → Start with [Schema Overview](#schema-overview)
- **Need to validate telemetry?** → See [OpenTelemetry Registry](#opentelemetry-weaver-registry)
- **Building with ontologies?** → See [RDF/OWL Ontologies](#rdfowl-ontologies)
- **Implementing shapes?** → See [SHACL Shapes](#shacl-shapes)
- **Defining workflows?** → See [Workflow Definitions](#workflow-definitions)

---

## 📋 Schema Overview

KNHK defines and validates three layers of semantics:

### Layer 1: Runtime Telemetry (Weaver YAML)
- **Purpose**: Define and validate OpenTelemetry instrumentation
- **Location**: `/registry/` (7 YAML files)
- **Validation**: `weaver registry check` and `weaver registry live-check`
- **Status**: Source of truth for runtime behavior
- **Tools**: OpenTelemetry Weaver CLI

### Layer 2: Domain Semantics (RDF/OWL)
- **Purpose**: Define domain concepts and relationships
- **Location**: `/ontology/` (4 core files)
- **Format**: RDF Turtle (.ttl), OWL 2 serialization
- **Use Cases**: Semantic search, knowledge graphs, entailment
- **Tools**: RDF validators, SPARQL query engines

### Layer 3: Data Validation (SHACL)
- **Purpose**: Validate RDF data against shape definitions
- **Location**: `/ontology/shacl/`
- **Format**: SHACL shape definitions in Turtle
- **Use Cases**: Data quality, constraint validation
- **Tools**: SHACL validators

---

## 📂 File Structure

```
/home/user/knhk/
├── registry/                           # OpenTelemetry Weaver Registry
│   ├── knhk-attributes.yaml           # Attribute definitions
│   ├── knhk-beat-v1.yaml              # 8-Beat system schema
│   ├── knhk-etl.yaml                  # ETL operations schema
│   ├── knhk-operation.yaml            # Generic operation schema
│   ├── knhk-sidecar.yaml              # Sidecar process schema
│   ├── knhk-warm.yaml                 # Warm path (≤500ms) schema
│   ├── knhk-workflow-engine.yaml      # Workflow engine schema
│   └── README.md                       # Registry documentation
│
└── ontology/                           # RDF/OWL Ontologies
    ├── knhk.owl.ttl                   # Main KNHK ontology (35KB)
    ├── yawl.ttl                       # YAWL workflow ontology (46KB)
    ├── osys.ttl                       # Operating system ontology
    ├── test_workflow.ttl              # Test workflow definitions
    ├── shacl/                         # SHACL Shape Definitions
    │   ├── workflow-shapes.ttl        # Workflow validation shapes
    │   ├── operation-shapes.ttl       # Operation validation shapes
    │   └── knowledge-shapes.ttl       # Knowledge graph shapes
    │
    └── workflows/                     # RDF Workflow Definitions
        ├── etl-pipeline.ttl           # ETL pipeline definition
        ├── knowledge-graph.ttl        # Knowledge graph workflow
        └── validation-workflow.ttl    # Validation workflow
```

---

## 🔧 OpenTelemetry Weaver Registry

### Location
`/home/user/knhk/registry/`

### Files & Purpose

| File | Purpose | Defines |
|------|---------|---------|
| **`knhk-attributes.yaml`** | Shared attribute definitions | Common telemetry attributes |
| **`knhk-beat-v1.yaml`** | 8-Beat epoch system | Timing and scheduling system |
| **`knhk-etl.yaml`** | ETL pipeline operations | ETL-specific telemetry |
| **`knhk-operation.yaml`** | Generic operations | Standard operation telemetry |
| **`knhk-sidecar.yaml`** | Sidecar process | Integration process telemetry |
| **`knhk-warm.yaml`** | Warm path (≤500ms) | Warm path operation metrics |
| **`knhk-workflow-engine.yaml`** | Workflow engine | Workflow execution telemetry |

### Key Concepts

**Weaver Schema Declaration**:
```yaml
# Example structure:
groups:
  - id: workflow.execution
    display_name: Workflow Execution
    attributes:
      - id: workflow.id
        type: string
        description: Unique workflow identifier

metrics:
  - id: workflow.duration_ms
    type: histogram
    unit: ms
    description: Workflow execution duration
```

**KNHK Role**: Each YAML file declares the telemetry schema that corresponds to actual instrumentation in the codebase. The `weaver registry live-check` command validates that runtime telemetry matches these declarations.

### Validation Commands

```bash
# Validate schema definitions
weaver registry check -r registry/

# Validate runtime telemetry against schema
weaver registry live-check --registry registry/

# Generate documentation from schema
weaver registry doc -r registry/
```

### Related Documentation

- **Telemetry Documentation**: [`docs/telemetry/`](/home/user/knhk/docs/telemetry/)
- **OTEL Integration**: [`rust/knhk-otel/`](/home/user/knhk/rust/knhk-otel/)
- **Validation Guide**: [`docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md`](/home/user/knhk/docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md)

---

## 🌐 RDF/OWL Ontologies

### Location
`/home/user/knhk/ontology/`

### Core Ontology Files

#### **`knhk.owl.ttl`** (35KB)
The main KNHK domain ontology defining:

**Classes**:
- `knhk:WorkflowEngine` - Workflow execution engine
- `knhk:WorkItem` - Individual work items
- `knhk:Resource` - Computational and human resources
- `knhk:KnowledgeGraph` - Knowledge representation
- `knhk:TelemetryEvent` - Observability events

**Properties**:
- `knhk:executes` - Workflow execution relationship
- `knhk:produces` - Output production
- `knhk:consumes` - Input consumption
- `knhk:measuredBy` - Metrics association

**Instances**:
- Predefined workflow patterns
- System resources
- Standard operations

**How to Use**:
```sparql
# Query: Find all workflows executed by hot path
PREFIX knhk: <https://knhk.io/ontology/>
SELECT ?workflow
WHERE {
  ?workflow a knhk:WorkflowEngine ;
           knhk:path knhk:HotPath .
}
```

#### **`yawl.ttl`** (46KB)
YAWL (Yet Another Workflow Language) ontology extending KNHK:

**Classes**:
- `yawl:Workflow` - YAWL workflow definition
- `yawl:Task` - YAWL tasks
- `yawl:Condition` - Decision conditions
- `yawl:WorkItem` - YAWL work items

**Purpose**:
- Define YAWL-compliant workflow patterns
- Enable YAWL-KNHK interoperability
- Support all 43/43 YAWL workflow patterns

**Related**: [`docs/YAWL_INTEGRATION.md`](/home/user/knhk/docs/YAWL_INTEGRATION.md)

#### **`osys.ttl`**
Operating system and runtime environment ontology:

**Classes**:
- System resources (CPU, memory, I/O)
- Process management
- Security contexts
- Resource allocation

**Use**: Modeling OS-level interactions and constraints

#### **`test_workflow.ttl`**
Sample workflow definitions for testing and validation.

### Querying Ontologies

#### Using SPARQL
```bash
# Example: Query SPARQL endpoint
sparql query -e http://localhost:3030/knhk/sparql -f query.rq

# Example: Load and query locally with Fuseki
./fuseki-server --data=ontology/knhk.owl.ttl /knhk
```

#### Using RDF Tools
```bash
# Validate with rapper
rapper -c ontology/knhk.owl.ttl

# Convert between formats
rdfprossor convert -i turtle -o rdfxml ontology/knhk.owl.ttl

# Query with Fuseki or Virtuoso
```

### Adding to Ontology

**Steps to extend ontology**:

1. **Identify new concepts**:
   ```turtle
   knhk:NewConcept a owl:Class ;
       rdfs:label "New Concept" ;
       rdfs:comment "Description of the new concept" ;
       rdfs:subClassOf knhk:ParentClass .
   ```

2. **Define relationships**:
   ```turtle
   knhk:newProperty a owl:ObjectProperty ;
       rdfs:domain knhk:ConceptA ;
       rdfs:range knhk:ConceptB .
   ```

3. **Validate with RDF validator**:
   ```bash
   rapper -c ontology/knhk.owl.ttl
   ```

4. **Update `SITE_MAP.md`** if significant additions

---

## ✅ SHACL Shapes

### Location
`/home/user/knhk/ontology/shacl/`

### Purpose
SHACL (Shapes Constraint Language) defines validation rules for RDF data:

### Shape Files

#### **`workflow-shapes.ttl`**
Validates workflow definitions:

```turtle
# Example shape
knhk:WorkflowShape
    a sh:NodeShape ;
    sh:targetClass knhk:Workflow ;
    sh:property [
        sh:path knhk:workflowId ;
        sh:datatype xsd:string ;
        sh:minCount 1 ;
        sh:maxCount 1
    ] .
```

**Validates**:
- Workflow IDs are unique
- Required properties exist
- Property types match constraints
- Cardinality constraints

#### **`operation-shapes.ttl`**
Validates operation definitions

#### **`knowledge-shapes.ttl`**
Validates knowledge graph structures

### Validation

```bash
# Validate RDF data against shapes
pyshacl validate -s ontology/shacl/workflow-shapes.ttl -d data/workflow.ttl

# Using Apache Jena
shaclvalidate -s ontology/shacl/workflow-shapes.ttl -d data/workflow.ttl
```

---

## 🔄 Workflow Definitions

### Location
`/home/user/knhk/ontology/workflows/`

### Files

#### **`etl-pipeline.ttl`**
Defines ETL workflow structure in RDF:

```turtle
knhk:SampleETLWorkflow a knhk:Workflow ;
    knhk:hasStep knhk:ExtractStep ;
    knhk:hasStep knhk:TransformStep ;
    knhk:hasStep knhk:LoadStep .
```

#### **`knowledge-graph.ttl`**
Knowledge graph construction workflow

#### **`validation-workflow.ttl`**
Validation pipeline definitions

---

## 🔗 Integration Points

### With Rust Code
- **`knhk-otel` crate**: Implements telemetry from Weaver schema
- **`knhk-validation` crate**: Validates against Weaver and SHACL

### With Testing
- **Validation Evidence**: [`docs/evidence/`](/home/user/knhk/docs/evidence/)
- **Weaver Validation**: Automated checks in CI/CD

### With Documentation
- **Telemetry Docs**: [`docs/telemetry/`](/home/user/knhk/docs/telemetry/)
- **Architecture Guide**: [`docs/ARCHITECTURE.md`](/home/user/knhk/docs/ARCHITECTURE.md)

---

## 📖 Related Documentation

| Topic | Location |
|-------|----------|
| OpenTelemetry Integration | [`docs/telemetry/`](/home/user/knhk/docs/telemetry/) |
| Schema Validation | [`docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md`](/home/user/knhk/docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md) |
| YAWL Workflows | [`docs/YAWL_INTEGRATION.md`](/home/user/knhk/docs/YAWL_INTEGRATION.md) |
| Ontology Integration | [`docs/ONTOLOGY.md`](/home/user/knhk/docs/ONTOLOGY.md) |
| Validation Crate | [`rust/knhk-validation/`](/home/user/knhk/rust/knhk-validation/) |
| OTEL Instrumentation | [`rust/knhk-otel/`](/home/user/knhk/rust/knhk-otel/) |

---

## 🛠️ Tools & Commands

### OpenTelemetry Weaver
```bash
# Install
npm install -g @opentelemetry/weaver

# Validate registry
weaver registry check -r registry/

# Validate runtime behavior
weaver registry live-check --registry registry/

# Generate docs
weaver registry doc -r registry/
```

### RDF/Turtle Tools
```bash
# Validate RDF
rapper -c ontology/knhk.owl.ttl

# Query SPARQL
sparql --query=query.rq ontology/knhk.owl.ttl

# Convert formats
rdfproc convert -i turtle -o rdfxml ontology/knhk.owl.ttl
```

### SHACL Validation
```bash
# Python PyShaCL
pip install pyshacl
pyshacl validate -s ontology/shacl/workflow-shapes.ttl -d data.ttl

# Apache Jena
shaclvalidate -s ontology/shacl/workflow-shapes.ttl -d data/workflow.ttl
```

---

## ✏️ Maintenance Guidelines

### When Adding New Schemas

1. **Define in Weaver YAML** (`registry/knhk-*.yaml`):
   - Add attributes or metrics
   - Document telemetry semantics
   - Ensure backward compatibility

2. **Add to RDF Ontology** (`ontology/knhk.owl.ttl`):
   - Define classes/properties
   - Add rdfs:label and rdfs:comment
   - Establish inheritance hierarchy

3. **Create SHACL Shapes** (`ontology/shacl/`):
   - Define validation constraints
   - Test with sample data
   - Document validation rules

4. **Update This Document**:
   - Add section if new schema type
   - Document purpose and location
   - Add related links

5. **Run Validations**:
   ```bash
   weaver registry check -r registry/
   rapper -c ontology/*.ttl
   ```

### Version Control

- **Semantic Versioning**: Use `knhk-beat-v1.yaml` pattern for versions
- **Changelog**: Document schema changes in docs/CHANGELOG.md
- **Backward Compatibility**: Extend, don't break existing schemas

---

## 📊 Current Schema Statistics

| Category | Count | Location |
|----------|-------|----------|
| **Weaver YAML Files** | 7 | `/registry/` |
| **Core OWL Files** | 1 | `/ontology/` |
| **YAWL Ontology** | 1 | `/ontology/` |
| **OS Ontology** | 1 | `/ontology/` |
| **Test Workflows** | 1 | `/ontology/` |
| **SHACL Shapes** | 3 | `/ontology/shacl/` |
| **Workflow Definitions** | 3 | `/ontology/workflows/` |
| **Total Schema Files** | 18 | `/registry/` + `/ontology/` |

---

## 🚀 Getting Started with Schemas

### For Implementers
1. Review [`registry/knhk-operation.yaml`](/home/user/knhk/registry/knhk-operation.yaml)
2. Check [`rust/knhk-otel/`](/home/user/knhk/rust/knhk-otel/) for instrumentation
3. Validate with: `weaver registry live-check`

### For Researchers
1. Study [`ontology/knhk.owl.ttl`](/home/user/knhk/ontology/knhk.owl.ttl)
2. Query with SPARQL
3. Review [`docs/formal-foundations.md`](/home/user/knhk/docs/formal-foundations.md)

### For Data Scientists
1. Learn SHACL: [`ontology/shacl/`](/home/user/knhk/ontology/shacl/)
2. Validate data quality
3. Create data pipelines with validated schemas

---

**Last Updated**: 2025-11-15
**Related**: [`docs/SITE_MAP.md`](/home/user/knhk/docs/SITE_MAP.md), [`ONTOLOGY.md`](/home/user/knhk/docs/ONTOLOGY.md)

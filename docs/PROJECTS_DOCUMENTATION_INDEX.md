# Projects Documentation Index

Comprehensive guide to documentation for all KNHK projects: 20 Rust crates, C library, Erlang implementation, and 5 reference examples.

**Quick Navigation:**
- [Rust Crates (20 projects)](#-rust-crates)
- [C Library](#-c-library)
- [Erlang Implementation](#-erlang-implementation)
- [Example Projects (5)](#-example-projects)

---

## 🦀 Rust Crates

All Rust projects are located in `/home/user/knhk/rust/` with Cargo.toml files and workspace configuration.

### Core System Crates

#### **`knhk-cli`** - Command-Line Interface
**Location**: `/rust/knhk-cli/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Main command-line interface for KNHK |
| **Primary Docs** | [`docs/CLI.md`](/docs/CLI.md) |
| **Project README** | Check `/rust/knhk-cli/README.md` |
| **API Reference** | [`docs/API.md`](/docs/API.md) |
| **Examples** | [`/examples/cli-usage/`](/examples/cli-usage/) |
| **Key Commands** | `knhk --help`, see [`VAN_DER_AALST_CLI_COMMANDS.md`](/VAN_DER_AALST_CLI_COMMANDS.md) |

**What It Does**:
- Provides command-line interface to all KNHK operations
- Handles workflow execution, validation, and management
- Integrates with all subsystems

**Documentation to Read**:
1. **Getting started**: [`docs/QUICK_START.md`](/docs/QUICK_START.md)
2. **Full reference**: [`docs/CLI.md`](/docs/CLI.md)
3. **Van der Aalst**: [`VAN_DER_AALST_CLI_COMMANDS.md`](/VAN_DER_AALST_CLI_COMMANDS.md)
4. **Advanced**: [`docs/INTEGRATION.md`](/docs/INTEGRATION.md)

---

#### **`knhk-workflow-engine`** - Workflow Engine
**Location**: `/rust/knhk-workflow-engine/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | YAWL-compatible workflow execution engine (43/43 patterns) |
| **Primary Docs** | [`docs/WORKFLOW_ENGINE.md`](/docs/WORKFLOW_ENGINE.md) |
| **Project README** | Check `/rust/knhk-workflow-engine/README.md` |
| **API Reference** | [`docs/API.md`](/docs/API.md) - Workflow section |
| **YAWL Patterns** | [`docs/YAWL_INTEGRATION.md`](/docs/YAWL_INTEGRATION.md) |
| **Architecture** | [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md) |

**What It Does**:
- Executes YAWL (Yet Another Workflow Language) workflows
- Implements all 43 YAWL control flow patterns
- Handles work items, resource allocation, and task execution
- Integrates with warm and hot paths

**Documentation to Read**:
1. **Overview**: [`docs/WORKFLOW_ENGINE.md`](/docs/WORKFLOW_ENGINE.md)
2. **YAWL Patterns**: [`docs/YAWL_INTEGRATION.md`](/docs/YAWL_INTEGRATION.md)
3. **Architecture**: [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md)
4. **Interface B**: [`docs/architecture/ADR-001-interface-b-work-item-lifecycle.md`](/docs/architecture/ADR-001-interface-b-work-item-lifecycle.md)

---

#### **`knhk-otel`** - OpenTelemetry Integration
**Location**: `/rust/knhk-otel/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | OpenTelemetry instrumentation and metrics |
| **Primary Docs** | [`docs/telemetry/`](/docs/telemetry/) |
| **Schema Registry** | [`registry/`](/registry/) |
| **Validation** | [`docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md`](/docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md) |
| **Evidence** | [`docs/evidence/`](/docs/evidence/) |

**What It Does**:
- Instruments all KNHK operations with OpenTelemetry
- Emits spans, metrics, and logs
- Validates against Weaver schema definitions
- Provides observability for performance monitoring

**Documentation to Read**:
1. **Telemetry basics**: [`docs/telemetry/`](/docs/telemetry/)
2. **Schema definition**: [`docs/schemas/README.md`](/docs/schemas/README.md)
3. **Validation**: [`docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md`](/docs/AUTOMATED_VAN_DER_AALST_VALIDATION.md)
4. **Registry files**: [`registry/*.yaml`](/registry/)

---

#### **`knhk-validation`** - Schema Validation
**Location**: `/rust/knhk-validation/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Weaver schema validation and conformance checking |
| **Primary Docs** | [`docs/schemas/README.md`](/docs/schemas/README.md) |
| **Registry** | [`registry/`](/registry/) |
| **Architecture** | [`docs/architecture/`](/docs/architecture/) |

**What It Does**:
- Validates telemetry against Weaver schemas
- Ensures conformance to declared telemetry structure
- Performs static and runtime validation

**Documentation to Read**:
1. **Schema overview**: [`docs/schemas/README.md`](/docs/schemas/README.md)
2. **Validation process**: See validation evidence in [`docs/evidence/`](/docs/evidence/)

---

### Performance & Optimization Crates

#### **`knhk-hot`** - Hot Path Operations (≤8 ticks)
**Location**: `/rust/knhk-hot/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Ultra-high-performance operations (≤8 CPU ticks) |
| **Primary Docs** | [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md) - Hot Path section |
| **Performance Docs** | [`docs/PERFORMANCE.md`](/docs/PERFORMANCE.md) |
| **Benchmarks** | [`docs/performance-benchmarks.md`](/docs/performance-benchmarks.md) |
| **C Engine** | [`docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md`](/docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md) |

**What It Does**:
- Executes critical operations within 8 CPU tick constraint
- Uses branchless C implementation for performance
- Handles highest-frequency operations

**Documentation to Read**:
1. **Performance guide**: [`docs/PERFORMANCE.md`](/docs/PERFORMANCE.md)
2. **Architecture**: [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md)
3. **Implementation**: [`docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md`](/docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md)
4. **Benchmarks**: [`docs/performance-benchmarks.md`](/docs/performance-benchmarks.md)

---

#### **`knhk-warm`** - Warm Path Operations (≤500ms, CONSTRUCT8)
**Location**: `/rust/knhk-warm/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Medium-performance operations (≤500ms, CONSTRUCT8 pattern) |
| **Primary Docs** | [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md) - Warm Path section |
| **Performance** | [`docs/PERFORMANCE.md`](/docs/PERFORMANCE.md) |

**What It Does**:
- Executes operations with ≤500ms constraint
- Uses CONSTRUCT8 pattern for optimization
- Balances performance and functionality

---

#### **`knhk-test-cache`** - Test Performance Optimization
**Location**: `/rust/knhk-test-cache/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Test result caching and performance optimization |
| **Primary Docs** | [`docs/TEST_OPTIMIZATION_MAX_INNOVATION.md`](/docs/TEST_OPTIMIZATION_MAX_INNOVATION.md) |
| **Performance** | [`docs/PERFORMANCE.md`](/docs/PERFORMANCE.md) |

**What It Does**:
- Caches test results for faster CI/CD
- Optimizes test execution time
- Maintains cache validity and coherence

---

### Workflow & Integration Crates

#### **`knhk-workflow-engine`** - (See above)

#### **`knhk-etl`** - ETL Pipeline Operations
**Location**: `/rust/knhk-etl/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Extract-Transform-Load pipeline execution |
| **Primary Docs** | [`docs/INTEGRATION.md`](/docs/INTEGRATION.md) |
| **Examples** | [`/examples/etl-pipeline/`](/examples/etl-pipeline/) |
| **Ontology** | [`ontology/workflows/etl-pipeline.ttl`](/ontology/workflows/etl-pipeline.ttl) |

**What It Does**:
- Executes ETL operations (Extract, Transform, Load)
- Manages data pipelines
- Integrates with external data sources

**Documentation to Read**:
1. **Integration guide**: [`docs/INTEGRATION.md`](/docs/INTEGRATION.md)
2. **Example**: [`/examples/etl-pipeline/README.md`](/examples/etl-pipeline/)

---

#### **`knhk-connectors`** - Data Connectors
**Location**: `/rust/knhk-connectors/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | External system connectors (Kafka, databases, APIs) |
| **Primary Docs** | [`docs/INTEGRATION.md`](/docs/INTEGRATION.md) |
| **Kafka Example** | [`/examples/kafka-connector/`](/examples/kafka-connector/) |

**What It Does**:
- Provides connectors to external systems
- Kafka integration for event streaming
- Database and API connectivity

**Documentation to Read**:
1. **Integration patterns**: [`docs/INTEGRATION.md`](/docs/INTEGRATION.md)
2. **Kafka example**: [`/examples/kafka-connector/README.md`](/examples/kafka-connector/)

---

#### **`knhk-config`** - Configuration Management
**Location**: `/rust/knhk-config/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Configuration loading and management |
| **Primary Docs** | [`docs/configuration.md`](/docs/configuration.md) |

**What It Does**:
- Loads and manages configuration
- Environment variable support
- Config file parsing

---

### Specialized Crates

#### **`knhk-lockchain`** - Cryptographic Audit Trails
**Location**: `/rust/knhk-lockchain/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Cryptographic provenance and audit trail tracking |
| **Primary Docs** | [`docs/LOCKCHAIN_INTEGRATION_COMPLETE.md`](/docs/LOCKCHAIN_INTEGRATION_COMPLETE.md) |
| **Example** | [`/examples/receipt-verification/`](/examples/receipt-verification/) |

**What It Does**:
- Creates immutable audit trails
- Cryptographic proof of execution
- Receipt generation and verification

**Documentation to Read**:
1. **Integration**: [`docs/LOCKCHAIN_INTEGRATION_COMPLETE.md`](/docs/LOCKCHAIN_INTEGRATION_COMPLETE.md)
2. **Example**: [`/examples/receipt-verification/README.md`](/examples/receipt-verification/)

---

#### **`knhk-dflss`** - DMAIC Lean Six Sigma
**Location**: `/rust/knhk-dflss/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Design for Lean Six Sigma methodology |
| **Primary Docs** | [`aa-dmedi-define.md`](/aa-dmedi-define.md) |
| **Define Phase** | [`aa-dmedi-define.md`](/aa-dmedi-define.md) |
| **Explore Phase** | [`aa-dmedi-explore.md`](/aa-dmedi-explore.md) |
| **Measure Phase** | [`aa-dmedi-measure.md`](/aa-dmedi-measure.md) |

**What It Does**:
- Implements DMAIC (Define, Measure, Analyze, Improve, Control)
- Lean Six Sigma process optimization
- DMEDI framework integration

---

#### **`knhk-patterns`** - Design Patterns Library
**Location**: `/rust/knhk-patterns/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Reusable design patterns |
| **Primary Docs** | [`docs/architecture/`](/docs/architecture/) |

**What It Does**:
- Provides standard design patterns
- Pattern reuse library
- Best practices implementation

---

#### **`knhk-sidecar`** - Sidecar Process
**Location**: `/rust/knhk-sidecar/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | Sidecar process for system integration |
| **Primary Docs** | [`docs/INTEGRATION.md`](/docs/INTEGRATION.md) |
| **Schema** | [`registry/knhk-sidecar.yaml`](/registry/knhk-sidecar.yaml) |

**What It Does**:
- Runs as sidecar process
- Provides system integration points
- Handles cross-cutting concerns

---

#### **`knhk-json-bench`** - JSON Parsing Benchmarks
**Location**: `/rust/knhk-json-bench/`

**What It Does**:
- Benchmarks JSON parsing performance
- Evaluates different JSON libraries
- Provides performance metrics

---

#### **`knhk-latex` & `knhk-latex-compiler`** - LaTeX Support
**Location**: `/rust/knhk-latex/`, `/rust/knhk-latex-compiler/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | LaTeX document generation and compilation |
| **Primary Docs** | [`docs/papers/`](/docs/papers/) |

**What It Does**:
- Generates LaTeX documents
- Compiles LaTeX to PDF
- Supports research paper generation

**Documentation to Read**:
1. **Papers**: [`docs/papers/chatman-equation/`](/docs/papers/chatman-equation/)

---

#### **`knhk-admission`** - Admission Control
**Location**: `/rust/knhk-admission/`

**What It Does**:
- Controls workflow admission
- Resource management
- Queue management

---

#### **`knhk-integration-tests`** - Integration Tests
**Location**: `/rust/knhk-integration-tests/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | End-to-end integration test suite |
| **Primary Docs** | [`docs/TESTING.md`](/docs/TESTING.md) |
| **Test Results** | [`docs/evidence/`](/docs/evidence/) |

**What It Does**:
- Runs integration tests
- Tests full system workflows
- Validates system behavior

---

#### **`knhk-ontology`** - Ontology Integration
**Location**: `/rust/ontology/`

| Type | Reference |
|------|-----------|
| **Crate Purpose** | RDF ontology integration |
| **Primary Docs** | [`docs/schemas/README.md`](/docs/schemas/README.md) |
| **Ontology Files** | [`ontology/*.ttl`](/ontology/) |

**What It Does**:
- Integrates with RDF ontologies
- Provides semantic querying
- Knowledge graph operations

---

## 🔧 C Library

**Location**: `/home/user/knhk/c/`

| Type | Reference |
|------|-----------|
| **Purpose** | Hot path engine implementation (branchless C) |
| **Primary Docs** | [`docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md`](/docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md) |
| **Architecture** | [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md) - Hot Path section |
| **Performance** | [`docs/performance-benchmarks.md`](/docs/performance-benchmarks.md) |
| **Testing** | `make test` (Chicago TDD format) |

### Structure
```
c/
├── src/           # C source files
├── include/       # Header files
├── tests/         # Unit tests (Chicago TDD)
├── tools/         # Utility tools
├── docs/          # C-specific documentation
└── Makefile       # Build configuration
```

### Key Commands
```bash
make build                  # Build C library
make test                   # Run tests
make test-chicago-v04       # Run Chicago TDD tests
make test-performance-v04   # Run performance tests (verify ≤8 ticks)
cargo clippy --workspace    # Lint
```

### Documentation to Read
1. **C Implementation**: [`docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md`](/docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md)
2. **Performance**: [`docs/PERFORMANCE.md`](/docs/PERFORMANCE.md)
3. **Architecture**: [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md)

---

## 🔴 Erlang Implementation

**Location**: `/home/user/knhk/erlang/`

| Type | Reference |
|------|-----------|
| **Purpose** | Erlang/OTP reference implementation |
| **Primary Docs** | Check `/erlang/README.md` |
| **Project** | `knhk_rc` (Reference Compiler) |

### Structure
```
erlang/
├── knhk_rc/       # Reference implementation
├── docs/          # Erlang-specific docs
└── src/           # Erlang source files
```

### Documentation to Read
1. **Project README**: `/erlang/README.md`
2. **Architecture**: [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md)

---

## 📚 Example Projects (5 reference implementations)

**Location**: `/home/user/knhk/examples/`

### 1. **`basic-hook`** - Hooks Integration Example
**Documentation**: [`/examples/basic-hook/README.md`](/examples/basic-hook/)

**Purpose**: Demonstrates basic hooks integration
**References**: [`docs/INTEGRATION.md`](/docs/INTEGRATION.md)

---

### 2. **`cli-usage`** - CLI Command Examples
**Documentation**: [`/examples/cli-usage/README.md`](/examples/cli-usage/)

**Purpose**: Shows how to use KNHK CLI commands
**References**: [`docs/CLI.md`](/docs/CLI.md), [`VAN_DER_AALST_CLI_COMMANDS.md`](/VAN_DER_AALST_CLI_COMMANDS.md)

---

### 3. **`etl-pipeline`** - ETL Workflow Example
**Documentation**: [`/examples/etl-pipeline/README.md`](/examples/etl-pipeline/)

**Purpose**: Demonstrates ETL pipeline implementation
**References**: [`docs/INTEGRATION.md`](/docs/INTEGRATION.md), [`rust/knhk-etl/`](/rust/knhk-etl/)

---

### 4. **`kafka-connector`** - Kafka Integration Example
**Documentation**: [`/examples/kafka-connector/README.md`](/examples/kafka-connector/)

**Purpose**: Shows Kafka connector usage
**References**: [`docs/INTEGRATION.md`](/docs/INTEGRATION.md), [`rust/knhk-connectors/`](/rust/knhk-connectors/)

---

### 5. **`receipt-verification`** - Cryptographic Verification
**Documentation**: [`/examples/receipt-verification/README.md`](/examples/receipt-verification/)

**Purpose**: Demonstrates lockchain receipt generation and verification
**References**: [`docs/LOCKCHAIN_INTEGRATION_COMPLETE.md`](/docs/LOCKCHAIN_INTEGRATION_COMPLETE.md)

---

## 📖 How to Navigate This Index

### By Role

**Developer** implementing a feature:
1. Review architecture: [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md)
2. Check relevant crate documentation above
3. Read example: `/examples/`
4. Review API: [`docs/API.md`](/docs/API.md)

**Architect** designing a system:
1. Review full project list above
2. Check architecture docs: [`docs/architecture/`](/docs/architecture/)
3. Review ADRs: [`docs/architecture/ADR/`](/docs/architecture/ADR/)
4. Check schemas: [`docs/schemas/README.md`](/docs/schemas/README.md)

**Researcher** analyzing the system:
1. Study papers: [`docs/papers/`](/docs/papers/)
2. Review ontologies: [`docs/schemas/README.md`](/docs/schemas/README.md)
3. Check validation: [`docs/evidence/`](/docs/evidence/)

**Operator** deploying to production:
1. Production readiness: [`docs/PRODUCTION.md`](/docs/PRODUCTION.md)
2. Deployment guide: [`docs/deployment.md`](/docs/deployment.md)
3. Configuration: [`docs/configuration.md`](/docs/configuration.md)

### By Project Type

**Infrastructure Projects**:
- `knhk-cli` - Main entry point
- `knhk-config` - Configuration
- `knhk-otel` - Observability
- `knhk-sidecar` - Integration

**Performance Projects**:
- `knhk-hot` - Ultra-fast operations
- `knhk-warm` - Medium performance
- `knhk-test-cache` - Test optimization
- C library - Hot path implementation

**Core Functionality**:
- `knhk-workflow-engine` - Workflow execution
- `knhk-etl` - Data pipelines
- `knhk-connectors` - External integration

**Quality & Validation**:
- `knhk-validation` - Schema validation
- `knhk-integration-tests` - Integration tests
- `knhk-ontology` - Semantic validation

---

## 📊 Projects Summary

| Type | Count | Location | Docs |
|------|-------|----------|------|
| **Rust Crates** | 20 | `/rust/` | This index + crate READMEs |
| **C Library** | 1 | `/c/` | [`docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md`](/docs/BRANCHLESS_C_ENGINE_IMPLEMENTATION.md) |
| **Erlang Implementation** | 1 | `/erlang/` | `/erlang/README.md` |
| **Examples** | 5 | `/examples/` | Each example README |
| **Total Projects** | 27 | Multiple | All documented here |

---

**Last Updated**: 2025-11-15
**Related**: [`docs/SITE_MAP.md`](/docs/SITE_MAP.md), [`docs/ARCHITECTURE.md`](/docs/ARCHITECTURE.md)

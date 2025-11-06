# Weaver Live-Check Implementation with Diagrams

## Overview

Complete implementation of Weaver live-check for telemetry validation with comprehensive diagrams showing the workflow, architecture, and integration points.

## Architecture Diagram

```mermaid
graph TB
    subgraph "KNHK CLI"
        CLI[knhk metrics weaver-*]
        Metrics[Metrics Commands]
    end
    
    subgraph "KNHK OTEL"
        Tracer[OTEL Tracer]
        OTELExporter[OTLP Exporter]
        WeaverCheck[WeaverLiveCheck]
    end
    
    subgraph "Weaver Process"
        WeaverProc[Weaver Binary]
        OTLPEndpoint[OTLP gRPC :4317]
        AdminEndpoint[Admin HTTP :8080]
    end
    
    subgraph "Semantic Conventions"
        Registry[Semantic Convention Registry]
        Validator[Validator Engine]
    end
    
    subgraph "Output"
        Reports[JSON/ANSI Reports]
        Violations[Violation Details]
    end
    
    CLI --> Metrics
    Metrics --> WeaverCheck
    WeaverCheck --> WeaverProc
    WeaverProc --> OTLPEndpoint
    WeaverProc --> AdminEndpoint
    WeaverProc --> Registry
    Registry --> Validator
    Tracer --> OTELExporter
    OTELExporter --> OTLPEndpoint
    Validator --> Reports
    Reports --> Violations
```

## Live-Check Workflow Diagram

```mermaid
sequenceDiagram
    participant User
    participant CLI as knhk CLI
    participant Metrics as Metrics Module
    participant OTEL as knhk-otel
    participant Weaver as Weaver Process
    participant Registry as Semantic Registry
    
    User->>CLI: knhk metrics weaver-start
    CLI->>Metrics: weaver_start()
    Metrics->>OTEL: WeaverLiveCheck::new()
    OTEL->>Weaver: spawn weaver process
    Weaver->>Registry: load semantic conventions
    Weaver-->>OTEL: process started
    OTEL-->>Metrics: (endpoint, admin_port, pid)
    Metrics-->>CLI: WeaverStartResult
    CLI-->>User: Weaver started
    
    Note over User,Weaver: Telemetry Generation Phase
    
    User->>CLI: knhk boot init
    CLI->>OTEL: Tracer::start_span()
    OTEL->>OTEL: record spans/metrics
    CLI->>OTEL: Tracer::export_to_weaver()
    OTEL->>Weaver: OTLP gRPC export
    Weaver->>Registry: validate against conventions
    Registry-->>Weaver: validation results
    
    User->>CLI: knhk metrics weaver-validate
    CLI->>Metrics: weaver_validate()
    Metrics->>OTEL: Tracer::export_to_weaver()
    OTEL->>Weaver: export remaining telemetry
    Metrics->>Metrics: wait for validation
    Metrics->>Metrics: parse_weaver_report()
    Metrics->>CLI: ValidationResult
    CLI-->>User: compliant, violations, message
    
    User->>CLI: knhk metrics weaver-stop
    CLI->>Metrics: weaver_stop()
    Metrics->>Weaver: HTTP POST /stop
    Weaver-->>Metrics: stopped
    Metrics-->>CLI: success
    CLI-->>User: Weaver stopped
```

## Component Interaction Diagram

```mermaid
graph LR
    subgraph "Command Layer"
        A[weaver_start]
        B[weaver_stop]
        C[weaver_validate]
    end
    
    subgraph "WeaverLiveCheck API"
        D[new]
        E[with_registry]
        F[with_otlp_port]
        G[with_admin_port]
        H[start]
        I[stop]
        J[otlp_endpoint]
    end
    
    subgraph "Telemetry Export"
        K[Tracer]
        L[export_to_weaver]
        M[OtlpExporter]
    end
    
    subgraph "Validation"
        N[parse_weaver_report]
        O[ValidationResult]
    end
    
    A --> D
    A --> E
    A --> F
    A --> G
    A --> H
    
    B --> I
    
    C --> A
    C --> L
    C --> N
    C --> B
    
    L --> K
    L --> M
    
    N --> O
```

## Data Flow Diagram

```mermaid
flowchart TD
    Start([Start Validation]) --> StartWeaver[Start Weaver Process]
    StartWeaver --> ConfigWeaver[Configure Weaver]
    ConfigWeaver --> Listen[Weaver Listens on OTLP Port]
    
    Listen --> GenerateTelemetry[Generate Telemetry]
    GenerateTelemetry --> CreateSpan[Create Spans with Attributes]
    CreateSpan --> RecordMetrics[Record Metrics]
    
    RecordMetrics --> Export[Export via OTLP]
    Export --> Receive[Weaver Receives Telemetry]
    
    Receive --> Validate[Validate Against Registry]
    Validate --> CheckConventions{Check Semantic<br/>Conventions}
    
    CheckConventions -->|Valid| Valid[Valid Telemetry]
    CheckConventions -->|Invalid| Invalid[Invalid Telemetry]
    
    Valid --> GenerateReport[Generate JSON Report]
    Invalid --> GenerateReport
    GenerateReport --> ParseReport[Parse Report]
    
    ParseReport --> ExtractViolations[Extract Violations]
    ExtractViolations --> ReturnResult[Return Validation Result]
    
    ReturnResult --> StopWeaver[Stop Weaver Process]
    StopWeaver --> End([End])
    
    style Valid fill:#90EE90
    style Invalid fill:#FFB6C1
    style CheckConventions fill:#FFE4B5
```

## State Machine Diagram

```mermaid
stateDiagram-v2
    [*] --> Idle: Initial State
    
    Idle --> Starting: weaver_start()
    Starting --> Running: Process Started
    Running --> Validating: Export Telemetry
    Validating --> Validating: Wait for Validation
    Validating --> Stopping: Validation Complete
    Stopping --> Stopped: Process Stopped
    Stopped --> [*]
    
    Running --> Stopping: weaver_stop()
    Running --> Error: Process Failure
    Error --> [*]
    
    note right of Running
        Weaver listening on:
        - OTLP gRPC: :4317
        - Admin HTTP: :8080
    end note
    
    note right of Validating
        Parsing JSON report:
        - violations count
        - compliance status
        - details
    end note
```

## Integration Points Diagram

```mermaid
graph TB
    subgraph "KNHK CLI Commands"
        C1[metrics weaver-start]
        C2[metrics weaver-validate]
        C3[metrics weaver-stop]
    end
    
    subgraph "knhk-otel Library"
        L1[WeaverLiveCheck]
        L2[Tracer]
        L3[OtlpExporter]
    end
    
    subgraph "Weaver Binary"
        B1[weaver registry live-check]
        B2[OTLP gRPC Server]
        B3[Admin HTTP Server]
        B4[Report Generator]
    end
    
    subgraph "External"
        E1[Semantic Convention Registry]
        E2[OTEL Collector]
    end
    
    C1 --> L1
    C2 --> L2
    C2 --> L1
    C3 --> L1
    
    L1 --> B1
    L2 --> L3
    L3 --> B2
    
    B1 --> E1
    B1 --> B2
    B1 --> B3
    B2 --> B4
    
    B4 --> E2
```

## Validation Process Diagram

```mermaid
flowchart TD
    A[Telemetry Export] --> B[OTLP gRPC Connection]
    B --> C[Weaver Receives Spans]
    C --> D[Parse Span Attributes]
    
    D --> E{Attribute Name<br/>Validation}
    E -->|Valid Format| F[Check Semantic Convention]
    E -->|Invalid Format| G[Record Violation]
    
    F --> H{Convention<br/>Exists?}
    H -->|Yes| I[Validate Value Type]
    H -->|No| J[Check Custom Registry]
    
    I --> K{Type<br/>Matches?}
    K -->|Yes| L[Valid]
    K -->|No| G
    
    J --> M{Found in<br/>Registry?}
    M -->|Yes| I
    M -->|No| G
    
    G --> N[Add to Violations List]
    L --> O[Add to Valid List]
    
    N --> P[Generate Report]
    O --> P
    
    P --> Q[Return Result]
    
    style G fill:#FFB6C1
    style L fill:#90EE90
    style E fill:#FFE4B5
    style H fill:#FFE4B5
    style K fill:#FFE4B5
```

## Error Handling Flow

```mermaid
graph TD
    Start([Start Operation]) --> Try{Try Operation}
    
    Try -->|Success| Success([Success])
    Try -->|Failure| ErrorType{Error Type}
    
    ErrorType -->|Weaver Not Found| Err1[Error: Weaver binary not found<br/>Solution: Run install-weaver.sh]
    ErrorType -->|Port In Use| Err2[Error: Port already in use<br/>Solution: Use different port]
    ErrorType -->|Export Failed| Err3[Error: OTLP export failed<br/>Solution: Check network/endpoint]
    ErrorType -->|Report Parse Failed| Err4[Error: Cannot parse report<br/>Solution: Check Weaver output format]
    ErrorType -->|Process Start Failed| Err5[Error: Cannot start Weaver<br/>Solution: Check permissions/binaries]
    
    Err1 --> Fail([Return Error])
    Err2 --> Fail
    Err3 --> Fail
    Err4 --> Fail
    Err5 --> Fail
    
    Success --> End([End])
    Fail --> End
    
    style Err1 fill:#FFB6C1
    style Err2 fill:#FFB6C1
    style Err3 fill:#FFB6C1
    style Err4 fill:#FFB6C1
    style Err5 fill:#FFB6C1
    style Success fill:#90EE90
```

## CI/CD Integration Diagram

```mermaid
graph LR
    A[Git Commit] --> B[CI Pipeline]
    B --> C[Build KNHK]
    C --> D[Run Tests]
    D --> E[Generate Telemetry]
    E --> F[Start Weaver]
    F --> G[Export Telemetry]
    G --> H[Validate]
    H --> I{Violations?}
    
    I -->|Yes| J[Fail Build]
    I -->|No| K[Pass Build]
    
    J --> L[Report Violations]
    K --> M[Deploy]
    
    L --> N[Fix Issues]
    N --> A
    
    style J fill:#FFB6C1
    style K fill:#90EE90
    style I fill:#FFE4B5
```

## File Structure

```
rust/knhk-cli/src/
├── commands/
│   └── metrics.rs          # weaver_start, weaver_stop, weaver_validate
└── metrics.rs              # CLI command wrappers

rust/knhk-otel/src/
└── lib.rs                  # WeaverLiveCheck, Tracer, export_to_weaver

docs/
└── weaver-live-check-diagrams.md  # This file
```

## Implementation Details

### 1. Weaver Start Process

```rust
pub fn weaver_start(
    registry: Option<String>,
    otlp_port: Option<u16>,
    admin_port: Option<u16>,
    format: Option<String>,
    output: Option<String>,
) -> Result<(String, u16, Option<u32>), String>
```

**Flow:**
1. Create `WeaverLiveCheck` instance
2. Configure with provided parameters
3. Spawn Weaver process via `start()`
4. Return endpoint, admin port, and process ID

### 2. Telemetry Export

```rust
pub fn export_to_weaver(&mut self, weaver_endpoint: &str) -> Result<(), String>
```

**Flow:**
1. Create `OtlpExporter` with Weaver endpoint
2. Export all spans via OTLP JSON
3. Export all metrics via OTLP JSON
4. Return success/failure

### 3. Validation Process

```rust
pub fn weaver_validate(
    registry: Option<String>,
    otlp_port: Option<u16>,
    admin_port: Option<u16>,
    timeout: Option<u64>,
) -> Result<(bool, u32, String), String>
```

**Flow:**
1. Start Weaver with output directory
2. Export current telemetry
3. Wait for validation timeout
4. Stop Weaver process
5. Parse validation report
6. Extract violations count
7. Return compliance status

### 4. Report Parsing

```rust
fn parse_weaver_report(output_dir: &std::path::Path) -> Result<ValidationResult, String>
```

**Flow:**
1. Read JSON report files from output directory
2. Find most recent report
3. Parse JSON structure
4. Extract violations array
5. Count violations
6. Generate compliance message

## Usage Examples

### Basic Validation

```bash
# Start Weaver
knhk metrics weaver-start --otlp-port 4317 --admin-port 8080

# Generate telemetry (other commands)
knhk boot init schema.ttl invariants.sparql

# Validate
knhk metrics weaver-validate --timeout 10

# Stop Weaver
knhk metrics weaver-stop --admin-port 8080
```

### With Custom Registry

```bash
knhk metrics weaver-start \
    --registry ./schemas/my-registry \
    --otlp-port 4317 \
    --admin-port 8080 \
    --format json \
    --output ./weaver-reports
```

### CI/CD Integration

```yaml
# .github/workflows/telemetry-validation.yml
- name: Install Weaver
  run: ./scripts/install-weaver.sh

- name: Start Weaver
  run: knhk metrics weaver-start --otlp-port 4317

- name: Run Tests (generate telemetry)
  run: cargo test

- name: Validate Telemetry
  run: |
    RESULT=$(knhk metrics weaver-validate --timeout 30)
    if echo "$RESULT" | grep -q '"compliant":false'; then
      echo "Telemetry validation failed"
      exit 1
    fi
```

## Testing

See `docs/chicago-tdd-weaver-tests.md` for comprehensive test coverage.

## References

- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Live-Check](https://github.com/open-telemetry/opentelemetry-rust/tree/main/vendors/weaver)
- [OTLP Protocol](https://opentelemetry.io/docs/specs/otlp/)


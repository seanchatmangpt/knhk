# Weaver Live-Check Integration for Workflow Engine

## Overview

The KNHK Workflow Engine integrates with Weaver.ai live-check for semantic convention validation of OpenTelemetry telemetry. This ensures that all telemetry emitted by the workflow engine conforms to OpenTelemetry semantic conventions defined in the Weaver registry.

## Features

- **Automatic Weaver live-check validation** - Validates all OTEL spans and metrics against semantic conventions
- **Comprehensive span coverage** - All workflow operations emit OTEL spans matching Weaver schema
- **CLI integration** - Built-in Weaver live-check command in the workflow engine CLI
- **Automated validation script** - Script to run Weaver validation for the entire workflow engine
- **Schema compliance** - All spans match the Weaver schema defined in `registry/knhk-workflow-engine.yaml`

## Weaver Schema

The workflow engine telemetry schema is defined in `registry/knhk-workflow-engine.yaml` and includes:

### Spans

- `knhk.workflow_engine.register_workflow` - Register a workflow specification
- `knhk.workflow_engine.create_case` - Create a new workflow case
- `knhk.workflow_engine.execute_case` - Execute a workflow case
- `knhk.workflow_engine.execute_task` - Execute a workflow task
- `knhk.workflow_engine.execute_pattern` - Execute a Van der Aalst workflow pattern
- `knhk.workflow_engine.get_case_history` - Get case history (audit trail)
- `knhk.workflow_engine.execute_mi_task` - Execute multiple instance task

### Attributes

- `knhk.workflow_engine.spec_id` - Workflow specification identifier (UUID)
- `knhk.workflow_engine.case_id` - Case identifier (UUID)
- `knhk.workflow_engine.task_id` - Task identifier
- `knhk.workflow_engine.pattern_id` - Van der Aalst pattern ID (1-43)
- `knhk.workflow_engine.case_state` - Case state (Created, Running, Completed, Cancelled)
- `knhk.workflow_engine.operation` - Operation name
- `knhk.workflow_engine.success` - Whether the operation succeeded
- `knhk.workflow_engine.latency_ms` - Operation latency in milliseconds

## Usage

### 1. Using the CLI

Start Weaver live-check validation:

```bash
cargo run --bin knhk-workflow -- \
    weaver-check \
    --registry ./registry \
    --otlp-port 4317 \
    --admin-port 8080 \
    --enable
```

This will:
1. Check if Weaver binary is available
2. Start Weaver live-check process
3. Wait for telemetry export
4. Generate validation report on exit

### 2. Using the Script

Run the automated validation script:

```bash
./scripts/workflow-engine-weaver-check.sh
```

The script will:
1. Check Weaver installation
2. Build the workflow engine binary
3. Start Weaver live-check
4. Run workflow operations to generate telemetry
5. Stop Weaver and generate validation report

### 3. Programmatic Usage

```rust
use knhk_workflow_engine::integration::WeaverIntegration;
use std::path::PathBuf;

// Create Weaver integration
let mut weaver = WeaverIntegration::with_config(
    PathBuf::from("./registry"),
    4317,  // OTLP gRPC port
    8080,  // Admin port
);

// Enable Weaver
weaver.enable();

// Start Weaver live-check
weaver.start().await?;

// Get OTLP endpoint for telemetry export
let otlp_endpoint = weaver.otlp_endpoint();
// Set environment variable: OTEL_EXPORTER_OTLP_ENDPOINT=$otlp_endpoint

// ... run workflow operations ...

// Stop Weaver
weaver.stop().await?;
```

### 4. Using OTEL Integration

The workflow engine's OTEL integration automatically emits spans matching the Weaver schema:

```rust
use knhk_workflow_engine::integration::OtelIntegration;

// Initialize OTEL integration
let otel = OtelIntegration::new(Some("http://127.0.0.1:4317".to_string()));
otel.initialize().await?;

// Start workflow registration span (matches Weaver schema)
let span = otel.start_register_workflow_span(&spec_id).await?;

// ... perform workflow registration ...

// End span
otel.end_span(span, SpanStatus::Ok).await?;
```

## Environment Variables

### OpenTelemetry Configuration

- `OTEL_EXPORTER_OTLP_ENDPOINT` - OTLP endpoint URL (default: `http://127.0.0.1:4317`)
- `OTEL_SERVICE_NAME` - Service name (default: `knhk-workflow-engine`)
- `OTEL_SERVICE_VERSION` - Service version (default: package version)

### Weaver Configuration

- `REGISTRY_PATH` - Path to Weaver registry (default: `./registry`)
- `OTLP_PORT` - OTLP gRPC port (default: `4317`)
- `ADMIN_PORT` - Weaver admin port (default: `8080`)

## Validation Report

After running Weaver live-check, validation reports are generated in `./weaver-reports/` directory:

- **JSON format** - Machine-readable validation results
- **Validation status** - Pass/fail for each span and attribute
- **Schema compliance** - Detailed compliance information
- **Missing attributes** - List of missing required attributes
- **Invalid attributes** - List of invalid attribute values

## Example Workflow

1. **Start Weaver live-check**:
   ```bash
   cargo run --bin knhk-workflow -- weaver-check --enable
   ```

2. **In another terminal, run workflow operations**:
   ```bash
   export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
   
   # Register workflow
   cargo run --bin knhk-workflow -- register --file workflow.ttl
   
   # Create case
   cargo run --bin knhk-workflow -- create-case --spec-id <spec-id>
   
   # Execute case
   cargo run --bin knhk-workflow -- execute-case --case-id <case-id>
   ```

3. **Stop Weaver** (Ctrl+C in the first terminal)

4. **Check validation report**:
   ```bash
   cat ./weaver-reports/*.json
   ```

## Integration with Workflow Engine

The workflow engine automatically emits OTEL spans for all operations when OTEL integration is enabled:

- **Workflow registration** - `knhk.workflow_engine.register_workflow`
- **Case creation** - `knhk.workflow_engine.create_case`
- **Case execution** - `knhk.workflow_engine.execute_case`
- **Task execution** - `knhk.workflow_engine.execute_task`
- **Pattern execution** - `knhk.workflow_engine.execute_pattern`
- **Case history** - `knhk.workflow_engine.get_case_history`

All spans include the required attributes as defined in the Weaver schema.

## Troubleshooting

### Weaver Binary Not Found

```bash
# Install Weaver
cargo install weaver

# Or use the install script
./scripts/install-weaver.sh
```

### Registry Path Not Found

Ensure the registry directory exists:

```bash
ls -la ./registry/knhk-workflow-engine.yaml
```

### Port Conflicts

If port 4317 is already in use, use a different port:

```bash
cargo run --bin knhk-workflow -- \
    weaver-check \
    --otlp-port 4318 \
    --enable
```

### No Telemetry Received

Ensure the OTLP endpoint is correctly configured:

```bash
export OTEL_EXPORTER_OTLP_ENDPOINT=http://127.0.0.1:4317
```

## See Also

- [Weaver Registry Schema](../../../registry/knhk-workflow-engine.yaml)
- [OTEL Integration](./OTEL_INTEGRATION.md)
- [Workflow Engine Documentation](../README.md)


# Weaver Validation Guide for KNHK

**Purpose**: This guide explains how to use OpenTelemetry Weaver to validate that KNHK's runtime behavior matches its declared schemas.

**Principle**: "Only Weaver validation is truth" - Weaver is the ONLY tool that can prove features actually work.

---

## Table of Contents

1. [Why Weaver Validation](#why-weaver-validation)
2. [Installation](#installation)
3. [Schema Validation](#schema-validation)
4. [Runtime Validation](#runtime-validation)
5. [Continuous Validation](#continuous-validation)
6. [Troubleshooting](#troubleshooting)

---

## Why Weaver Validation

### The False Positive Problem

KNHK exists to detect false positives in testing. Ironically, traditional test suites can have false positives:

```
❌ Tests can pass even when features don't work
❌ Tests can validate test logic, not production behavior
❌ --help text exists ≠ command works
❌ Compilation succeeds ≠ runtime works
```

### The Weaver Solution

Weaver validates that **actual runtime telemetry** matches **declared schemas**:

```
✅ Schema defines what telemetry SHOULD be emitted
✅ Runtime validation proves telemetry WAS emitted
✅ No circular dependency: External tool validates our framework
✅ Industry standard: OTel's official validation approach
```

**Key Insight**: Weaver validation is the ONLY way to prove features work without re-executing them.

---

## Installation

### Install Weaver

```bash
# Method 1: Using installer script (recommended)
curl --proto '=https' --tlsv1.2 -sSfL \
  https://github.com/open-telemetry/weaver/releases/latest/download/install.sh | sh

# Method 2: Using KNHK script
./scripts/install-weaver.sh

# Method 3: Manual download
# Visit: https://github.com/open-telemetry/weaver/releases
# Download binary for your platform

# Verify installation
weaver --version
```

**Expected Output**:
```
weaver 0.8.0 (or later)
```

### Verify KNHK Registry

```bash
# Check that registry files exist
ls -la registry/

# Expected files:
# - registry_manifest.yaml (main manifest)
# - knhk-workflow-engine.yaml (workflow telemetry)
# - knhk-operation.yaml (operation telemetry)
# - knhk-mape-k.yaml (MAPE-K telemetry)
# - knhk-guards.yaml (guard telemetry)
# - knhk-receipts.yaml (receipt telemetry)
```

---

## Schema Validation

**Purpose**: Validates that schemas are well-formed and conform to OTel semantic conventions.

### Basic Schema Check

```bash
# Validate all schemas in registry
weaver registry check -r registry/

# Expected output:
# ✅ Registry validation passed
# ✅ 7 schema files validated
# ✅ 0 policy violations
```

### Detailed Schema Check

```bash
# Check specific schema file
weaver registry check -r registry/ --schema knhk-workflow-engine.yaml

# Check with verbose output
weaver registry check -r registry/ --verbose

# Check with strict mode (treat warnings as errors)
weaver registry check -r registry/ --strict
```

### Common Schema Errors

#### Error: Invalid Attribute Type

```yaml
# ❌ Wrong
attributes:
  - id: knhk.operation.count
    type: number  # Invalid type

# ✅ Correct
attributes:
  - id: knhk.operation.count
    type: int  # Valid types: string, int, double, boolean, string[], int[], double[], boolean[]
```

#### Error: Missing Required Fields

```yaml
# ❌ Wrong
- id: knhk.operation.execute
  type: span
  # Missing: stability, brief

# ✅ Correct
- id: knhk.operation.execute
  type: span
  stability: experimental  # Required
  brief: "Execute operation"  # Required
```

#### Error: Invalid Reference

```yaml
# ❌ Wrong
attributes:
  - ref: knhk.unknown.attribute  # Undefined attribute

# ✅ Correct
attributes:
  - ref: knhk.operation.name  # Defined in attribute_group
```

### Schema Validation Best Practices

1. **Run before every commit**: Catch schema errors early
2. **Use strict mode**: Treat warnings as errors
3. **Validate incrementally**: Check new schemas as you add them
4. **Document attributes**: Add clear `brief` and `note` fields

---

## Runtime Validation

**Purpose**: Validates that **actual runtime telemetry** matches **declared schemas**.

**This is the ONLY source of truth for production readiness.**

### Prerequisites

1. **OTEL Collector running**: Collecting traces/metrics
2. **Application running**: Emitting telemetry to collector
3. **Schemas deployed**: Registry files accessible

### Basic Live Check

```bash
# Start OTEL collector (port 4318)
docker run -d \
  --name otel-collector \
  -p 4317:4317 \
  -p 4318:4318 \
  otel/opentelemetry-collector:latest

# Start KNHK application with telemetry
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
./target/release/knhk server start

# Run live validation
weaver registry live-check --registry registry/
```

**Expected Output (SUCCESS)**:
```
✅ Registry validation passed
✅ Live telemetry validation passed
   - All declared spans found in runtime traces
   - All declared metrics found in runtime metrics
   - No undeclared telemetry detected
   - 0 policy violations
```

**Expected Output (FAILURE)**:
```
❌ Live telemetry validation failed

Violations:
1. Declared span not found: knhk.workflow_engine.execute_case
   - Declared in: registry/knhk-workflow-engine.yaml:143
   - Not found in runtime traces
   - Possible causes:
     - Code doesn't emit this span
     - Span name doesn't match schema
     - Feature not executed during validation

2. Undeclared span found: knhk.unknown.operation
   - Found in runtime traces
   - Not declared in any schema
   - Possible causes:
     - Missing schema definition
     - Typo in span name
     - Dead code emitting telemetry
```

### Validate Specific Components

```bash
# Validate only workflow engine telemetry
weaver registry live-check \
  --registry registry/ \
  --filter "knhk.workflow_engine"

# Validate only MAPE-K telemetry
weaver registry live-check \
  --registry registry/ \
  --filter "knhk.mape_k"

# Validate only guard telemetry
weaver registry live-check \
  --registry registry/ \
  --filter "knhk.guard"
```

### Advanced Live Check Options

```bash
# Check with custom OTLP endpoint
weaver registry live-check \
  --registry registry/ \
  --otlp-endpoint http://custom-collector:4318

# Check with duration (collect telemetry for 60 seconds)
weaver registry live-check \
  --registry registry/ \
  --duration 60s

# Check with verbose output
weaver registry live-check \
  --registry registry/ \
  --verbose

# Export violations report
weaver registry live-check \
  --registry registry/ \
  --output violations.json
```

### Interpreting Live Check Results

#### Result 1: Schema Valid, Runtime Valid ✅

```
✅ Registry validation passed
✅ Live telemetry validation passed
```

**Meaning**: Features are working correctly!
- Schemas are well-formed
- Code emits telemetry as declared
- Runtime behavior matches specification

**Action**: Deploy to production with confidence.

#### Result 2: Schema Valid, Runtime Failed ❌

```
✅ Registry validation passed
❌ Live telemetry validation failed
```

**Meaning**: Schemas are correct, but code doesn't match.
- Declared spans/metrics not emitted
- Undeclared spans/metrics found
- Code behavior doesn't match schema

**Action**: Fix code or update schema to match reality.

#### Result 3: Schema Failed ❌

```
❌ Registry validation failed
```

**Meaning**: Schemas are malformed.

**Action**: Fix schema errors before running live-check.

---

## Continuous Validation

### CI/CD Integration

Add Weaver validation to your CI/CD pipeline:

```yaml
# .github/workflows/validation.yml
name: Weaver Validation

on: [push, pull_request]

jobs:
  schema-validation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Weaver
        run: |
          curl -sSfL https://github.com/open-telemetry/weaver/releases/latest/download/install.sh | sh

      - name: Validate Schemas
        run: weaver registry check -r registry/

  runtime-validation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Start OTEL Collector
        run: |
          docker run -d \
            --name otel-collector \
            -p 4318:4318 \
            otel/opentelemetry-collector:latest

      - name: Build and Run KNHK
        run: |
          cargo build --release
          export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4318"
          ./target/release/knhk server start &
          sleep 10

      - name: Run Live Validation
        run: weaver registry live-check --registry registry/
```

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running Weaver schema validation..."
weaver registry check -r registry/

if [ $? -ne 0 ]; then
    echo "❌ Weaver schema validation failed"
    echo "   Fix schema errors before committing"
    exit 1
fi

echo "✅ Weaver schema validation passed"
```

### Production Monitoring

```bash
#!/bin/bash
# continuous-weaver-monitor.sh

while true; do
    echo "$(date): Running Weaver live-check..."

    weaver registry live-check --registry /etc/knhk/registry/ \
      --output /var/log/knhk/weaver-validation.json

    if [ $? -ne 0 ]; then
        echo "❌ WEAVER VALIDATION FAILED"

        # Alert on-call engineer
        curl -X POST https://alert-system/api/alert \
          -H "Content-Type: application/json" \
          -d '{
            "severity": "critical",
            "service": "knhk",
            "message": "Weaver validation failed - runtime telemetry does not match schema",
            "details": "See /var/log/knhk/weaver-validation.json"
          }'
    fi

    sleep 300  # Check every 5 minutes
done
```

---

## Troubleshooting

### Issue 1: "No telemetry received"

**Symptom**:
```
❌ Live telemetry validation failed
   - No telemetry received from collector
```

**Causes**:
1. Application not running
2. Application not emitting telemetry
3. Collector not receiving telemetry
4. Wrong OTLP endpoint

**Solutions**:
```bash
# 1. Verify application is running
ps aux | grep knhk

# 2. Check OTEL environment variables
env | grep OTEL

# 3. Test collector endpoint
curl http://localhost:4318/v1/traces -I

# 4. Check collector logs
docker logs otel-collector

# 5. Emit test telemetry
cargo run --example emit_telemetry
```

### Issue 2: "Declared span not found"

**Symptom**:
```
❌ Declared span not found: knhk.workflow_engine.execute_case
```

**Causes**:
1. Code doesn't emit this span
2. Span name typo (code vs schema)
3. Feature not executed during validation

**Solutions**:
```bash
# 1. Search code for span emission
grep -r "execute_case" rust/*/src

# 2. Verify span name matches schema
# In code:
span!("knhk.workflow_engine.execute_case")  # ✅
span!("knhk.workflow_engine.execute-case")  # ❌ (hyphen vs underscore)

# 3. Execute feature to trigger span
./target/release/knhk workflow execute test.ttl
```

### Issue 3: "Undeclared span found"

**Symptom**:
```
❌ Undeclared span found: knhk.unknown.operation
```

**Causes**:
1. Missing schema definition
2. Dead code emitting telemetry
3. Typo in span name

**Solutions**:
```bash
# 1. Add to schema (if intentional)
# registry/knhk-*.yaml:
# - id: knhk.unknown.operation
#   type: span
#   brief: "..."

# 2. Remove from code (if unintentional)
# Find and remove dead code

# 3. Fix typo in code
# Ensure span name matches schema exactly
```

### Issue 4: "Port 4318 already in use"

**Symptom**:
```
Error: Cannot bind to port 4318
```

**Cause**: Docker Desktop OTLP analytics using port 4318

**Solution**:
```bash
# Option 1: Use alternative port
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4319"
docker run -d -p 4319:4318 otel/opentelemetry-collector:latest

# Option 2: Disable Docker Desktop OTLP
# Docker Desktop > Settings > Features > Disable "Send usage statistics"

# Option 3: Use gRPC instead (port 4317)
export OTEL_EXPORTER_OTLP_PROTOCOL="grpc"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
```

---

## Appendix: Validation Hierarchy

KNHK uses a strict validation hierarchy:

```
LEVEL 1: Weaver Validation (Source of Truth)
├─ Schema check: Proves schema is well-formed
└─ Live-check: Proves runtime matches schema
   ↓
LEVEL 2: Compilation & Code Quality (Baseline)
├─ cargo build: Proves code compiles
├─ cargo clippy: Proves code quality
└─ Pattern checks: Proves no unsafe patterns
   ↓
LEVEL 3: Traditional Tests (Supporting Evidence)
├─ cargo test: Proves test logic works
├─ Integration tests: Proves components integrate
└─ Performance tests: Proves performance targets met
```

**Key Principle**: If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.

---

**Last Updated**: 2025-11-16
**Weaver Version**: 0.8.0+
**Status**: Production Guide

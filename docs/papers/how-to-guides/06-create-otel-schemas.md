# How-to Guide: Create OpenTelemetry Schemas

**Goal**: Design and implement OTel schemas for your components
**Time**: 20 minutes
**Difficulty**: Intermediate

## Overview

An OTel schema documents:
- What telemetry your component emits
- Spans, metrics, and logs
- Attributes and their meanings
- Expected values and types

## Step 1: Create Schema File

Create `registry/my-component.yaml`:

```yaml
instrumentation:
  name: my_component
  version: 1.0.0
  description: "My component's telemetry specification"

spans:
  - name: process_data
    description: "Processes input data"

metrics:
  - name: processing_duration
    type: histogram
    description: "Duration of processing"

logs:
  - name: process_start
    level: INFO
    description: "Processing started"
```

## Step 2: Define Spans

```yaml
spans:
  - name: validate_input
    description: "Validates user input"

  - name: process
    description: "Main processing operation"

  - name: save_result
    description: "Saves result to storage"
```

Each span becomes a function with `#[tracing::instrument]`.

## Step 3: Add Span Attributes

Document what data each span captures:

```yaml
spans:
  - name: process_data
    description: "Processes input data"
    attributes:
      - input_size:
          description: "Size of input in bytes"
          type: int
          examples: [1024, 2048]

      - operation_type:
          description: "Type of operation"
          type: string
          examples: ["compress", "encrypt"]

      - success:
          description: "Whether operation succeeded"
          type: bool
```

## Step 4: Add Span Events

```yaml
spans:
  - name: process_data
    description: "Processes input data"
    events:
      - name: validation_started
        description: "Input validation started"

      - name: validation_failed
        description: "Input validation failed"
        attributes:
          - error_reason:
              description: "Why validation failed"
              type: string
```

## Step 5: Define Metrics

```yaml
metrics:
  - name: processing_duration
    type: histogram
    unit: "ms"
    description: "Duration of processing in milliseconds"
    attributes:
      - operation_type:
          description: "Type of operation"
          type: string

  - name: items_processed
    type: counter
    unit: "1"
    description: "Total items processed"

  - name: error_rate
    type: gauge
    unit: "%"
    description: "Current error rate percentage"
```

## Step 6: Define Logs

```yaml
logs:
  - name: process_start
    level: INFO
    description: "Processing started"
    attributes:
      - item_id:
          description: "ID of item being processed"
          type: string

  - name: process_error
    level: ERROR
    description: "Processing failed"
    attributes:
      - error_message:
          description: "Error message"
          type: string
      - stack_trace:
          description: "Stack trace"
          type: string
```

## Step 7: Complete Schema Example

```yaml
instrumentation:
  name: data_processor
  version: 2.1.0
  description: "Data processing pipeline with full telemetry"

spans:
  - name: validate_input
    description: "Validates input data format and content"
    attributes:
      - data_size:
          description: "Size of input data in bytes"
          type: int
      - format:
          description: "Input format (json, csv, etc)"
          type: string
    events:
      - name: validation_passed
        description: "Input validation passed"
      - name: validation_failed
        description: "Input validation failed"
        attributes:
          - error_code:
              description: "Validation error code"
              type: string

  - name: process_data
    description: "Main data processing operation"
    attributes:
      - record_count:
          description: "Number of records processed"
          type: int
      - processing_strategy:
          description: "Algorithm used for processing"
          type: string
    events:
      - name: processing_started
        description: "Data processing started"
      - name: batch_processed
        description: "A batch of records processed"
        attributes:
          - batch_id:
              description: "ID of the batch"
              type: string
          - batch_size:
              description: "Number of items in batch"
              type: int
      - name: processing_completed
        description: "Data processing completed"
        attributes:
          - total_processed:
              description: "Total items processed"
              type: int
          - total_errors:
              description: "Total errors encountered"
              type: int

  - name: save_results
    description: "Saves processed results"
    attributes:
      - destination:
          description: "Where results are saved"
          type: string
      - format:
          description: "Output format"
          type: string

metrics:
  - name: processing_duration
    type: histogram
    unit: "ms"
    description: "Duration of data processing"
    attributes:
      - processing_strategy:
          description: "Algorithm used"
          type: string

  - name: items_processed
    type: counter
    unit: "1"
    description: "Total items successfully processed"
    attributes:
      - data_type:
          description: "Type of data processed"
          type: string

  - name: processing_errors
    type: counter
    unit: "1"
    description: "Total processing errors"
    attributes:
      - error_type:
          description: "Type of error"
          type: string

  - name: queue_depth
    type: gauge
    unit: "1"
    description: "Current number of items in queue"

logs:
  - name: processing_started
    level: INFO
    description: "Data processing job started"
    attributes:
      - job_id:
          description: "Unique job identifier"
          type: string
      - input_file:
          description: "Input file path"
          type: string

  - name: processing_error
    level: ERROR
    description: "Error during processing"
    attributes:
      - error_message:
          description: "Error message"
          type: string
      - item_id:
          description: "ID of item causing error"
          type: string

  - name: processing_completed
    level: INFO
    description: "Data processing job completed"
    attributes:
      - job_id:
          description: "Job identifier"
          type: string
      - items_processed:
          description: "Total items processed"
          type: int
      - duration_seconds:
          description: "Total duration in seconds"
          type: int
```

## Step 8: Attribute Types

Common attribute types:

```yaml
attributes:
  # String
  - name:
      type: string
      description: "A text value"

  # Integer
  - count:
      type: int
      description: "A whole number"

  # Float
  - percentage:
      type: float
      description: "A decimal number"

  # Boolean
  - success:
      type: bool
      description: "True or false"

  # Array
  - tags:
      type: array
      description: "A list of values"

  # String enum
  - status:
      type: string
      enum: ["pending", "processing", "completed", "failed"]
      description: "Job status"
```

## Step 9: Metric Types

Available metric types:

| Type | Purpose | Example |
|------|---------|---------|
| **counter** | Always increases | Items processed, errors |
| **gauge** | Can increase/decrease | Queue depth, memory usage |
| **histogram** | Distribution of values | Processing duration, request size |

```yaml
metrics:
  - name: total_items
    type: counter      # Only goes up

  - name: active_connections
    type: gauge        # Can go up or down

  - name: request_duration
    type: histogram    # Distribution
```

## Step 10: Validate Schema

```bash
# Check schema syntax
weaver registry check -r registry/

# Expected output:
# ✓ Schema valid
# ✓ my_component schema is well-formed
```

## Best Practices

✅ **DO:**
- Use consistent naming
- Document all attributes
- Include examples
- Use appropriate types
- Group related telemetry
- Version your schema

❌ **DON'T:**
- Leave attributes undocumented
- Use vague names
- Mix concerns
- Have breaking changes
- Forget to validate

## Naming Conventions

```yaml
# Spans: verb_noun (what it does)
spans:
  - name: validate_input       # ✅ Good
  - name: process_data         # ✅ Good
  - name: save_result          # ✅ Good
  - name: thing                # ❌ Bad - vague

# Metrics: noun_descriptor (what it measures)
metrics:
  - name: items_processed      # ✅ Good
  - name: processing_duration  # ✅ Good
  - name: m1                   # ❌ Bad - unclear

# Attributes: snake_case
attributes:
  - input_size                 # ✅ Good
  - operation_type             # ✅ Good
  - inputSize                  # ❌ Bad - camelCase
```

## Versioning

Update version when schema changes:

```yaml
instrumentation:
  name: my_component
  version: 1.0.0    # Major.Minor.Patch
```

- **Major**: Breaking changes (1.0 → 2.0)
- **Minor**: New features (1.0 → 1.1)
- **Patch**: Bug fixes (1.0 → 1.0.1)

## Multiple Schemas

Organize by component:

```
registry/
├── auth_service.yaml
├── data_processor.yaml
├── api_gateway.yaml
└── storage_service.yaml
```

## Next Steps

- **Emit telemetry**: [How to Emit Proper Telemetry](05-emit-telemetry.md)
- **Fix validation**: [How to Fix Weaver Validation Errors](03-fix-weaver-validation-errors.md)
- **Add features**: [How to Add New Features](04-add-new-features.md)

## Key Commands

```bash
# Validate all schemas
weaver registry check -r registry/

# Validate specific component
weaver registry check -r registry/ --component my_component

# Live validation (if running)
weaver registry live-check --registry registry/
```

---

**Category**: How-to Guides (Task-oriented)
**Framework**: Diátaxis
**Difficulty**: Intermediate
**Related**: Emitting Telemetry, Fixing Validation

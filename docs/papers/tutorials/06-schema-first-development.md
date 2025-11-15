# Tutorial: Schema-First Development with OpenTelemetry

**Level**: Advanced
**Time**: 40-50 minutes
**Learning Objectives**: Design and implement using schema-first approach

## What You'll Learn

By the end of this tutorial, you'll understand:
- Schema-first development philosophy
- Designing behavior before code
- Test generation from schemas
- Iterative refinement process
- Schema validation in CI/CD
- Documentation through schemas

## Prerequisites

- Completed: [Chicago TDD Basics](03-chicago-tdd-basics.md)
- Completed: [How to Create OTel Schemas](../how-to-guides/06-create-otel-schemas.md)
- ~50 minutes

## The Schema-First Philosophy

### Traditional Approach
```
Code → Schema (if at all)
❌ Schema documents code after-the-fact
❌ Code evolves, schema becomes outdated
❌ Behavior is implicit, not explicit
```

### Schema-First Approach
```
Schema → Code → Validation
✅ Schema defines contracts first
✅ Code implements the schema
✅ Schema validation proves compliance
```

## Step 1: Define the Schema First

Before writing any code, write the complete schema:

```yaml
instrumentation:
  name: data_pipeline
  version: 1.0.0
  description: "Data pipeline with complete telemetry specification"

# Define the CONTRACT before implementation
spans:
  - name: load_data
    description: "Load data from source"
    attributes:
      - source_type:
          type: string
          enum: ["file", "api", "database"]
      - record_count:
          type: int
      - duration_ms:
          type: int
    events:
      - name: "Load started"
      - name: "Load completed"
      - name: "Load failed"
        attributes:
          - error_reason:
              type: string

  - name: transform_data
    description: "Transform and enrich data"
    attributes:
      - input_count:
          type: int
      - output_count:
          type: int
      - transformation_type:
          type: string
    events:
      - name: "Transformation started"
      - name: "Transformation completed"
      - name: "Validation error"
        attributes:
          - validation_message:
              type: string

  - name: validate_data
    description: "Validate data quality"
    attributes:
      - record_count:
          type: int
      - valid_count:
          type: int
      - invalid_count:
          type: int
      - validation_rules:
          type: int

metrics:
  - name: data_loaded
    type: counter
    unit: "records"
    description: "Records loaded from source"
    attributes:
      - source_type:
          type: string

  - name: data_transformed
    type: counter
    unit: "records"
    description: "Records successfully transformed"

  - name: data_validation_failures
    type: counter
    unit: "records"
    description: "Records failing validation"
    attributes:
      - failure_reason:
          type: string

  - name: pipeline_duration
    type: histogram
    unit: "ms"
    description: "Total pipeline duration"
```

## Step 2: Design Code Structure from Schema

The schema tells us what the code must do:

```rust
// Schema tells us we need these functions

/// Loads data from specified source (SCHEMA CONTRACTS)
/// MUST emit span: "load_data"
/// MUST emit attributes: source_type, record_count, duration_ms
/// MUST emit events: "Load started", "Load completed" or "Load failed"
#[tracing::instrument(skip(data))]
pub async fn load_data(source: DataSource) -> Result<Vec<Record>> {
    // Implementation must match schema contract
}

/// Transforms data (SCHEMA CONTRACTS)
/// MUST emit span: "transform_data"
/// MUST emit attributes: input_count, output_count, transformation_type
/// MUST emit events: "Transformation started", "Transformation completed"
#[tracing::instrument]
pub fn transform_data(records: Vec<Record>, transform: TransformType) -> Result<Vec<TransformedRecord>> {
    // Implementation must match schema contract
}

/// Validates data quality (SCHEMA CONTRACTS)
/// MUST emit span: "validate_data"
/// MUST emit attributes: record_count, valid_count, invalid_count, validation_rules
/// MUST emit events: validation results
#[tracing::instrument]
pub fn validate_data(records: &[Record]) -> ValidationResult {
    // Implementation must match schema contract
}
```

## Step 3: Write Tests from Schema

Generate test cases from schema attributes:

```rust
#[cfg(test)]
mod schema_driven_tests {
    use super::*;

    // Test based on schema attributes
    #[tokio::test]
    async fn test_load_data_file_source() {
        let source = DataSource::File("data.csv".into());
        let result = load_data(source).await;

        // Schema says we must have record_count attribute
        assert!(result.is_ok());
        let records = result.unwrap();
        assert!(!records.is_empty());
    }

    #[tokio::test]
    async fn test_load_data_api_source() {
        let source = DataSource::Api("https://api.example.com".into());
        let result = load_data(source).await;

        // Schema says source_type must be in {file, api, database}
        assert!(result.is_ok());
    }

    #[test]
    fn test_transform_data_output_count() {
        let records = create_test_records(10);
        let result = transform_data(records, TransformType::Uppercase);

        assert!(result.is_ok());
        let transformed = result.unwrap();
        // Schema says we must emit output_count attribute
        assert_eq!(transformed.len(), 10);
    }

    #[test]
    fn test_validate_data_counts() {
        let records = create_test_records(100);
        let result = validate_data(&records);

        // Schema says we must have valid_count and invalid_count
        assert_eq!(result.valid_count + result.invalid_count, 100);
    }

    // Validation error test from schema events
    #[test]
    fn test_transform_validation_error() {
        let bad_records = create_invalid_records();
        let result = transform_data(bad_records, TransformType::Validation);

        // Schema says "Validation error" event must include validation_message
        match result {
            Err(e) => {
                assert!(!e.message.is_empty());
            }
            Ok(_) => panic!("Expected validation error"),
        }
    }
}
```

## Step 4: Implement Code to Match Schema

```rust
use tracing::{info, warn, Span};
use std::time::Instant;

#[tracing::instrument(skip(data))]
pub async fn load_data(source: DataSource) -> Result<Vec<Record>> {
    let start = Instant::now();

    // Schema requires "Load started" event
    info!(
        source_type = ?source,
        "Load started"
    );

    match source {
        DataSource::File(path) => {
            let records = load_from_file(&path).await?;

            // Schema requires record_count and duration_ms attributes
            let duration = start.elapsed();
            info!(
                record_count = records.len(),
                duration_ms = duration.as_millis(),
                source_type = "file",
                "Load completed"
            );

            Ok(records)
        }
        DataSource::Api(url) => {
            let records = load_from_api(&url).await?;

            let duration = start.elapsed();
            info!(
                record_count = records.len(),
                duration_ms = duration.as_millis(),
                source_type = "api",
                "Load completed"
            );

            Ok(records)
        }
    }
}

#[tracing::instrument]
pub fn transform_data(records: Vec<Record>, transform: TransformType) -> Result<Vec<TransformedRecord>> {
    let start = Instant::now();
    let input_count = records.len();

    // Schema requires "Transformation started" event
    info!(
        input_count = input_count,
        transformation_type = ?transform,
        "Transformation started"
    );

    let mut output_records = Vec::with_capacity(input_count);

    for record in records {
        match apply_transformation(&record, &transform) {
            Ok(transformed) => {
                output_records.push(transformed);
            }
            Err(e) => {
                // Schema requires validation_message in event
                warn!(
                    validation_message = %e,
                    record_id = ?record.id,
                    "Validation error"
                );
                return Err(TransformError::ValidationFailed(e.to_string()));
            }
        }
    }

    let output_count = output_records.len();
    let duration = start.elapsed();

    // Schema requires these attributes
    info!(
        input_count = input_count,
        output_count = output_count,
        transformation_type = ?transform,
        duration_ms = duration.as_millis(),
        "Transformation completed"
    );

    Ok(output_records)
}

#[tracing::instrument]
pub fn validate_data(records: &[Record]) -> ValidationResult {
    let start = Instant::now();
    let record_count = records.len();
    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut validation_rules_applied = 0;

    for record in records {
        if validate_record(record) {
            valid_count += 1;
        } else {
            invalid_count += 1;
        }
        validation_rules_applied += 5; // Example: 5 rules per record
    }

    let duration = start.elapsed();

    // Schema requires these attributes
    info!(
        record_count = record_count,
        valid_count = valid_count,
        invalid_count = invalid_count,
        validation_rules = validation_rules_applied,
        duration_ms = duration.as_millis(),
        "Data validation completed"
    );

    ValidationResult {
        record_count,
        valid_count,
        invalid_count,
        validation_rules: validation_rules_applied,
    }
}
```

## Step 5: Validate Implementation Against Schema

```bash
# Run Weaver validation
weaver registry check -r registry/

# Expected output:
# ✓ data_pipeline schema is valid
# ✓ All spans documented
# ✓ All attributes present
# ✓ All events defined
```

## Step 6: Iterative Refinement

When requirements change, update schema first:

```yaml
# ITERATION 1: New requirement - track batch processing
spans:
  - name: transform_data
    # ... existing attributes ...

    # NEW: Batch processing tracking
    events:
      - name: "Batch started"
        attributes:
          - batch_id:
              type: string
          - batch_size:
              type: int

      - name: "Batch completed"
        attributes:
          - batch_id:
              type: string
          - successful:
              type: int
          - failed:
              type: int
```

Update tests:
```rust
#[test]
fn test_batch_processing() {
    let records = create_test_records(1000);
    let result = transform_data_with_batches(records);

    assert!(result.is_ok());
    // Schema now requires batch events
}
```

Update code:
```rust
#[tracing::instrument]
pub fn transform_data_with_batches(records: Vec<Record>) -> Result<Vec<TransformedRecord>> {
    const BATCH_SIZE: usize = 100;
    let mut output = Vec::new();

    for (idx, batch) in records.chunks(BATCH_SIZE).enumerate() {
        let batch_id = format!("batch_{}", idx);

        // Schema requires "Batch started" event
        info!(batch_id = %batch_id, batch_size = batch.len(), "Batch started");

        let mut successful = 0;
        let mut failed = 0;

        for record in batch {
            match apply_transformation(record) {
                Ok(t) => {
                    output.push(t);
                    successful += 1;
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        // Schema requires "Batch completed" event with stats
        info!(
            batch_id = %batch_id,
            successful = successful,
            failed = failed,
            "Batch completed"
        );
    }

    Ok(output)
}
```

## Step 7: Schema as Documentation

The schema IS the documentation:

```rust
// This code is now self-documenting
// Users can read the schema to understand:
// - What telemetry to expect
// - What metrics to monitor
// - What spans to trace
// - How to correlate events
```

Developers can use the schema to:
- Write monitoring/alerting rules
- Create dashboards
- Understand behavior
- Debug issues

## Step 8: Schema in CI/CD

Enforce schema compliance:

```yaml
# .github/workflows/ci.yml
- name: Validate schema compliance
  run: |
    weaver registry check -r registry/
    weaver registry live-check --registry registry/

- name: Run tests
  run: |
    cargo test --workspace

# IMPORTANT: If schema validation fails, CI fails
# This prevents undocumented behavior reaching production
```

## Benefits of Schema-First Development

✅ **Clear Contracts** - Schema defines behavior explicitly
✅ **Better Design** - Thinking about telemetry improves design
✅ **Automatic Documentation** - Schema IS the documentation
✅ **Early Error Detection** - Schema violations caught in CI
✅ **Better Observability** - All behavior documented and validated
✅ **Reduced Bugs** - Schema mismatches prevent deployment

## What You've Learned

Congratulations! You now understand:

1. **Schema-First Philosophy** - Design behavior in schema first
2. **Schema-Driven Testing** - Generate tests from schema
3. **Implementation Contracts** - Code must match schema
4. **Iterative Refinement** - Update schema → tests → code
5. **Documentation Through Schema** - Schema IS documentation
6. **CI/CD Integration** - Enforce schema compliance

## Next Steps

- **Apply schema-first**: Use for next feature
- **Automate testing**: Generate tests from schemas
- **Monitor compliance**: Catch schema violations
- **Share schemas**: Use for API contracts

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~50 minutes
**Difficulty**: Advanced
**Prerequisites**: Chicago TDD, OTel Schemas

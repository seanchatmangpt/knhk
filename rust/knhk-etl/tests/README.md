# knhk-etl OpenTelemetry E2E Tests

## Test Coverage

### End-to-End Validation (`chicago_tdd_otel_e2e.rs`)
- **Purpose**: Verify ETL pipeline emits telemetry correctly
- **Status**: ⏳ Requires infrastructure
- **Requires**: 
  - ETL pipeline execution
  - OTLP collector running (Docker or local)
  - Weaver binary (optional)

**Tests**:
- `test_etl_pipeline_emits_telemetry` - Verify ETL pipeline emits telemetry
- `test_etl_operations_record_metrics` - Verify ETL operations record metrics
- `test_weaver_validates_etl_telemetry` - Verify Weaver validates ETL telemetry

**Run**: `cargo test --test chicago_tdd_otel_e2e --features std -- --ignored`

## Running Tests

### Unit Tests (No Infrastructure)
```bash
cd rust/knhk-etl
cargo test --features std
```

### E2E Tests (Requires Infrastructure)
```bash
# Start OTLP collector
docker compose -f tests/integration/docker-compose.yml up -d otel-collector

# Run E2E tests
cargo test --test chicago_tdd_otel_e2e --features std -- --ignored
```








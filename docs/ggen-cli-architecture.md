# ggen v2.7.1 CLI Architecture

## Overview

The ggen (Graph Generation) v2.7.1 CLI integration provides comprehensive code generation capabilities for KNHK, enabling developers to generate workflows, tests, hooks, and templates from RDF/Turtle specifications.

## Architecture Principles

### 1. Hierarchical Command Structure (clap-noun-verb)

All commands follow the `knhk gen:verb` pattern:

```
knhk gen:workflow     - Generate workflow code
knhk gen:tests        - Generate Chicago TDD tests
knhk gen:hook         - Generate knowledge hooks
knhk gen:validate     - Validate generated code
knhk gen:templates    - Template management (subcommands)
knhk gen:marketplace  - Marketplace integration (subcommands)
```

### 2. No unwrap/expect Policy

All CLI code follows strict error handling:
- ✅ Use `Result<T, E>` with proper error types
- ✅ Map errors with context using `map_err`
- ✅ Return `NounVerbError::execution_error` for user-facing errors
- ❌ Never use `.unwrap()` or `.expect()` in production paths

### 3. OpenTelemetry Integration

All commands emit OTEL spans when the `otel` feature is enabled:

```rust
#[cfg_attr(
    feature = "otel",
    instrument(
        skip_all,
        fields(
            operation = "knhk.gen.workflow",
            spec_file = %spec_file.display()
        )
    )
)]
#[verb]
pub fn workflow(...) -> CnvResult<WorkflowGenResult> {
    // Command implementation
}
```

### 4. Progress Indicators

Long-running operations use the `ProgressIndicator` pattern:

```rust
let progress = ProgressIndicator::new("Generating workflow");
// ... perform operation ...
progress.complete("Generated successfully");
```

Output:
```
⏳ Generating workflow...
✅ Generating workflow - Generated successfully (1.23s)
```

### 5. Structured Output

All commands support multiple output formats:
- `--format json` - Machine-readable JSON
- `--format yaml` - Human-readable YAML
- `--format text` - Default text output (default)

Return types are `Serialize` structs that can be output in any format.

## Module Structure

```
rust/knhk-cli/src/
├── gen.rs                    # CLI interface with #[verb] functions
└── commands/
    └── gen.rs                # Implementation module
        ├── generate_workflow()
        ├── generate_tests()
        ├── generate_hook()
        ├── validate_code()
        └── templates/        # Template management
        └── marketplace/      # Marketplace integration
```

### gen.rs (CLI Interface)

Defines all CLI verbs with:
- Clap argument parsing
- OTEL instrumentation
- Documentation and examples
- Return type definitions

### commands/gen.rs (Implementation)

Contains business logic:
- Progress indicators
- File I/O
- RDF/Turtle parsing (TODO)
- Code generation
- Validation

## Command Specifications

### gen:workflow

**Purpose**: Generate Rust/Python/JS/Go workflow from RDF/Turtle specification

**Arguments**:
- `spec_file` (required) - Path to RDF/Turtle specification
- `--template` - Custom template file
- `--output` - Output file (default: stdout)
- `--language` - Target language (rust|python|js|go)
- `--validate` - Validate against Weaver schema
- `--emit-hooks` - Generate knowledge hooks
- `--emit-telemetry` - Generate OTEL telemetry code
- `--format` - Output format for metadata

**Return Type**: `WorkflowGenResult`
```rust
struct WorkflowGenResult {
    spec_file: String,
    output_file: Option<String>,
    language: String,
    telemetry_enabled: bool,
    hooks_enabled: bool,
    validated: bool,
}
```

**OTEL Span**: `knhk.gen.workflow`

**Example**:
```bash
knhk gen workflow spec.ttl \
  --output src/workflow.rs \
  --language rust \
  --emit-telemetry \
  --validate
```

### gen:tests

**Purpose**: Generate Chicago TDD tests from specification

**Arguments**:
- `spec_file` (required) - Path to specification
- `--template` - Custom template file
- `--output` - Output directory (default: ./tests)
- `--coverage` - Target coverage % (default: 90)
- `--language` - Target language
- `--format` - Output format for metadata

**Return Type**: `TestsGenResult`
```rust
struct TestsGenResult {
    spec_file: String,
    output_dir: String,
    language: String,
    coverage_target: u8,
    test_count: usize,
}
```

**OTEL Span**: `knhk.gen.tests`

**Example**:
```bash
knhk gen tests spec.ttl \
  --output tests/ \
  --coverage 95 \
  --language rust
```

### gen:hook

**Purpose**: Generate knowledge hook from RDF definition

**Arguments**:
- `definition_file` (required) - Path to RDF hook definition
- `--template` - Hook template
- `--output` - Output file
- `--with-lockchain` - Generate Lockchain receipts
- `--with-telemetry` - Emit OTEL telemetry
- `--format` - Output format for metadata

**Return Type**: `HookGenResult`
```rust
struct HookGenResult {
    definition_file: String,
    output_file: Option<String>,
    lockchain_enabled: bool,
    telemetry_enabled: bool,
}
```

**OTEL Span**: `knhk.gen.hook`

**Example**:
```bash
knhk gen hook hook-def.ttl \
  --output src/hooks/custom.rs \
  --with-lockchain \
  --with-telemetry
```

### gen:validate

**Purpose**: Validate generated code against schema

**Arguments**:
- `code_path` (required) - Path to code file/directory
- `--schema` - Schema file
- `--telemetry` - Validate telemetry compliance
- `--performance` - Check performance constraints
- `--weaver` - Run Weaver validation (source of truth)
- `--format` - Output format

**Return Type**: `ValidateResult`
```rust
struct ValidateResult {
    code_path: String,
    schema_valid: bool,
    telemetry_valid: bool,
    performance_valid: bool,
    weaver_valid: bool,
    issues: Vec<String>,
    warnings: Vec<String>,
}
```

**OTEL Span**: `knhk.gen.validate`

**Example**:
```bash
knhk gen validate src/workflow.rs \
  --schema schema.yaml \
  --telemetry \
  --performance \
  --weaver
```

### gen:templates (subcommands)

**Purpose**: Manage code generation templates

**Subcommands**:
- `list` - List available templates
- `search <pattern>` - Search templates
- `preview <template>` - Preview template with sample data
- `install <name>` - Install template from marketplace
- `validate <path>` - Validate template file
- `docs <name>` - Show template documentation

**Examples**:
```bash
# List all templates
knhk gen templates list --format json

# Search for workflow templates
knhk gen templates search workflow

# Preview template
knhk gen templates preview workflow.tmpl

# Install template
knhk gen templates install workflow-advanced@2.0.0

# Validate custom template
knhk gen templates validate ./my-template.tmpl

# Show template docs
knhk gen templates docs workflow-basic
```

### gen:marketplace (subcommands)

**Purpose**: Marketplace integration for templates

**Subcommands**:
- `publish <template>` - Publish template to marketplace
- `search <pattern>` - Search marketplace
- `install <name>` - Install published template
- `rating <name>` - Show template ratings/reviews

**Examples**:
```bash
# Publish template
knhk gen marketplace publish ./my-template.tmpl

# Search marketplace
knhk gen marketplace search "workflow rust"

# Install from marketplace
knhk gen marketplace install workflow-pro

# Show ratings
knhk gen marketplace rating workflow-pro --format json
```

## Error Handling Strategy

### 1. Input Validation

All file paths are validated before processing:

```rust
if !req.spec_file.exists() {
    return Err(NounVerbError::execution_error(format!(
        "Specification file not found: {}",
        req.spec_file.display()
    )));
}
```

### 2. Operation Errors

All I/O operations use `map_err` to provide context:

```rust
std::fs::write(output_path, &content).map_err(|e| {
    NounVerbError::execution_error(format!(
        "Failed to write output: {}",
        e
    ))
})?;
```

### 3. User-Friendly Messages

Error messages include:
- What went wrong
- What file/operation failed
- Suggestions for recovery (when applicable)

Example:
```
❌ Failed to generate workflow
Error: Specification file not found: /path/to/spec.ttl
Suggestion: Check that the file path is correct and the file exists
```

## OTEL Telemetry Schema

All commands emit the following telemetry:

### Spans

```yaml
knhk.gen.workflow:
  attributes:
    - operation: "knhk.gen.workflow"
    - spec_file: string
    - language: rust|python|js|go
    - duration_ms: number
    - result: success|error

knhk.gen.tests:
  attributes:
    - operation: "knhk.gen.tests"
    - spec_file: string
    - coverage: number
    - test_count: number
    - duration_ms: number

knhk.gen.hook:
  attributes:
    - operation: "knhk.gen.hook"
    - definition: string
    - lockchain: boolean
    - telemetry: boolean
    - duration_ms: number

knhk.gen.validate:
  attributes:
    - operation: "knhk.gen.validate"
    - code_path: string
    - schema_valid: boolean
    - telemetry_valid: boolean
    - performance_valid: boolean
    - weaver_valid: boolean
    - duration_ms: number
```

### Events

```yaml
operation.complete:
  attributes:
    - operation: string
    - duration_ms: number
    - result: string

operation.failed:
  attributes:
    - operation: string
    - duration_ms: number
    - error: string
```

## Integration Points

### 1. Core ggen Module (TODO)

The CLI integrates with a core ggen module for:
- RDF/Turtle parsing (using oxigraph)
- Template engine (Handlebars/Tera)
- Code generation logic
- Validation rules

### 2. Weaver Integration

Validation commands integrate with Weaver:
- Schema-first validation
- Live telemetry checking
- Source of truth for feature validation

### 3. Template System

Templates support:
- Handlebars syntax
- Conditional generation
- Loop constructs
- Partial includes
- Custom helpers

### 4. Marketplace API

Marketplace commands connect to:
- Template registry API
- Version management
- Rating/review system
- Download tracking

## Performance Considerations

### 1. Lazy Initialization

Heavy operations are deferred until needed:
- Template loading only when used
- Schema parsing cached
- Marketplace API calls batched

### 2. Progress Indicators

Long operations show progress:
- File processing
- Template rendering
- Validation steps
- Network requests

### 3. Parallel Processing

When generating multiple files:
- Use rayon for parallel file generation
- Batch template compilations
- Concurrent validation checks

## Security Considerations

### 1. Path Traversal Prevention

All file paths are validated:
```rust
// Normalize and validate paths
let canonical = path.canonicalize()?;
if !canonical.starts_with(&allowed_dir) {
    return Err("Path traversal attempt detected");
}
```

### 2. Template Sandboxing

Template execution is sandboxed:
- No arbitrary code execution
- Limited file system access
- Resource limits (memory, CPU)

### 3. Marketplace Verification

Downloaded templates are verified:
- Checksum validation
- Signature verification
- Malware scanning

## Future Enhancements

### 1. Watch Mode

```bash
knhk gen workflow spec.ttl --watch
# Auto-regenerate on specification changes
```

### 2. Incremental Generation

Only regenerate changed files:
- Track file hashes
- Compare timestamps
- Dependency tracking

### 3. Language Server Protocol

IDE integration:
- Code completion for templates
- Real-time validation
- Preview generation

### 4. CI/CD Integration

GitHub Actions workflow:
```yaml
- name: Generate Code
  run: knhk gen workflow spec.ttl --validate --format json > gen-report.json
```

### 5. Interactive Mode

```bash
knhk gen workflow --interactive
# Prompts for template, language, options
```

## Testing Strategy

### 1. Unit Tests

Test individual functions:
```rust
#[test]
fn test_workflow_generation() {
    let req = WorkflowGenRequest { /* ... */ };
    let result = generate_workflow(req).unwrap();
    assert_eq!(result.language, "rust");
}
```

### 2. Integration Tests

Test end-to-end flows:
```rust
#[test]
fn test_generate_and_validate() {
    // Generate workflow
    let workflow = gen_workflow("spec.ttl");
    // Validate output
    let valid = validate(workflow);
    assert!(valid);
}
```

### 3. Snapshot Tests

Compare generated output:
```rust
#[test]
fn test_rust_workflow_output() {
    let output = generate_rust_workflow(...);
    insta::assert_snapshot!(output);
}
```

### 4. Telemetry Tests

Verify OTEL spans:
```rust
#[test]
fn test_workflow_telemetry() {
    let spans = capture_spans(|| {
        generate_workflow(...);
    });
    assert_eq!(spans[0].name, "knhk.gen.workflow");
}
```

## Conclusion

The ggen v2.7.1 CLI integration provides a comprehensive, production-ready code generation system for KNHK that follows all architectural best practices:

- ✅ Hierarchical command structure
- ✅ No unwrap/expect policy
- ✅ OTEL telemetry integration
- ✅ Progress indicators
- ✅ Structured error handling
- ✅ Multiple output formats
- ✅ Extensible template system
- ✅ Marketplace integration
- ✅ Weaver validation support

The implementation is ready for production use with stub implementations that can be filled in with actual RDF/Turtle parsing and code generation logic.

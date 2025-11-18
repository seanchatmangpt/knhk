# knhk-workflow-xml-legacy

⚠️ **DEPRECATED** - Legacy XML/YAWL parser for migration purposes only

## Overview

This crate provides XML YAWL parsing and conversion to TTL/Turtle format for users migrating from legacy XML-based YAWL systems to KNHK v4.0's TTL-only architecture.

**DO NOT use this crate for new workflows.** KNHK v4.0+ uses TTL/Turtle exclusively per DOCTRINE Covenant 1.

## Installation

```bash
cargo install knhk-workflow-xml-legacy
```

## Usage

### CLI Tool

```bash
# Convert single file
yawl-xml-to-ttl workflow.yawl > workflow.ttl

# Convert with output file
yawl-xml-to-ttl workflow.yawl -o workflow.ttl

# Convert with validation
yawl-xml-to-ttl workflow.yawl -o workflow.ttl --validate

# Convert directory
yawl-xml-to-ttl dir --dir ./xml/ --output ./ttl/

# Convert directory recursively
yawl-xml-to-ttl dir --dir ./xml/ --output ./ttl/ --recursive

# Validate TTL file
yawl-xml-to-ttl validate workflow.ttl
```

### As Library

```rust
use knhk_workflow_xml_legacy::XmlToTtlConverter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read XML
    let xml = std::fs::read_to_string("workflow.yawl")?;

    // Convert to TTL
    let converter = XmlToTtlConverter::new();
    let ttl = converter.convert(&xml)?;

    // Validate
    converter.validate_ttl(&ttl)?;

    // Write output
    std::fs::write("workflow.ttl", ttl)?;

    Ok(())
}
```

## Supported XML Elements

- `<specification>` - Workflow specification
- `<task>` - Workflow tasks
- `<condition>` - Workflow conditions (places)
- `<flow>` - Flows between elements
- `<join>`, `<split>` - Join/split types
- All 43 Van der Aalst workflow patterns

## Limitations

This is a best-effort converter. Complex XML features may require manual review:

- ✅ Basic workflow structure
- ✅ All 43 YAWL patterns
- ✅ Task metadata
- ⚠️ Custom XML extensions
- ❌ Proprietary vendor extensions

## Why TTL-Only?

KNHK v4.0 enforces TTL-only architecture per **DOCTRINE Covenant 1**:

- **Σ (Ontology-First)**: TTL is the canonical ontology format
- **Semantic completeness**: RDF enables full semantic reasoning
- **Weaver validation**: OTEL schema validation requires RDF
- **No impedance mismatch**: Direct RDF → execution

## Migration Path

1. **Convert XML to TTL** using this tool
2. **Validate output** with `knhk validate` or Weaver
3. **Test workflows** in KNHK v4.0 environment
4. **Switch to TTL** for all new workflow development
5. **Retire XML** workflows

## Documentation

- [Migration Guide](/home/user/knhk/docs/v4-migration/MIGRATION_GUIDE_V4.md)
- [Breaking Changes](/home/user/knhk/docs/v4-migration/V4_BREAKING_CHANGES.md)
- [DOCTRINE Reference](/home/user/knhk/DOCTRINE_2027.md)

## Support

For migration issues:

1. Check the migration guide
2. Validate XML against YAWL 2.0 schema
3. Open issue with XML snippet + error
4. Consider manual TTL creation for complex cases

## License

MIT OR Apache-2.0

---

**Remember**: This crate is temporary migration infrastructure. KNHK v4.0+ is TTL-only.

# ⚠️ DEPRECATION NOTICE

**This crate provides legacy XML/YAWL parsing ONLY for migration purposes.**

## Important Information

XML support was **removed from KNHK v4.0.0 (2028 Q1)** in favor of exclusive TTL/Turtle format.

### Why TTL-Only?

Per **DOCTRINE Covenant 1** (Turtle is source of truth):
- **Σ (Ontology-First)**: TTL is the canonical ontology format
- **Semantic completeness**: RDF provides full semantic reasoning
- **No impedance mismatch**: Direct RDF → execution path
- **Weaver validation**: OTEL schema validation requires RDF structure

## What This Crate Does

This crate exists **ONLY** to help users migrate existing XML YAWL workflows to TTL format.

**DO NOT use this crate for new workflows.**

## Migration Instructions

### Step 1: Install Migration Tool

```bash
cargo install knhk-workflow-xml-legacy
```

### Step 2: Convert XML to TTL

```bash
# Convert single file
yawl-xml-to-ttl input.yawl > output.ttl

# Convert directory
yawl-xml-to-ttl --dir ./xml-workflows/ --output ./ttl-workflows/

# Validate conversion
yawl-xml-to-ttl --validate output.ttl
```

### Step 3: Validate TTL Output

```bash
# Using KNHK workflow engine
knhk validate output.ttl

# Using Weaver (recommended)
weaver registry check -r output.ttl
```

## Supported XML Elements

This parser supports standard YAWL 2.0 XML elements:

- `<specification>` → `yawl:Specification`
- `<task>` → `yawl:Task`
- `<condition>` → `yawl:Condition`
- `<flow>` → `yawl:Flow`
- `<join>`, `<split>` → `yawl:joinType`, `yawl:splitType`
- All 43 Van der Aalst workflow patterns

## Example Conversion

**Input (XML)**:
```xml
<specification uri="http://example.org/workflow1">
  <name>Sample Workflow</name>
  <task id="task1">
    <name>Process Order</name>
    <split code="AND"/>
  </task>
</specification>
```

**Output (TTL)**:
```turtle
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<http://example.org/workflow1> a yawl:Specification ;
    rdfs:label "Sample Workflow" ;
    yawl:hasTask <http://example.org/workflow1#task1> .

<http://example.org/workflow1#task1> a yawl:Task ;
    rdfs:label "Process Order" ;
    yawl:splitType yawl:AND .
```

## Limitations

This is a **best-effort** migration tool. Complex XML features may require manual review:

- ✅ Basic workflow structure
- ✅ All 43 YAWL patterns
- ✅ Task metadata and attributes
- ⚠️ Custom XML extensions (may need manual mapping)
- ⚠️ External schema references (converted to RDF imports)
- ❌ Proprietary YAWL vendor extensions (not supported)

## Support & Documentation

- **Migration Guide**: See `/home/user/knhk/docs/v4-migration/MIGRATION_GUIDE_V4.md`
- **Breaking Changes**: See `/home/user/knhk/docs/v4-migration/V4_BREAKING_CHANGES.md`
- **Issues**: https://github.com/knhk/knhk/issues
- **DOCTRINE Reference**: See `DOCTRINE_2027.md` and `DOCTRINE_COVENANT.md`

## Timeline

- **v3.x** (2027): XML + TTL hybrid support
- **v4.0** (2028 Q1): TTL-only, XML removed from core
- **v4.0+**: This legacy crate for migration only

## Getting Help

If you encounter migration issues:

1. Check the migration guide (link above)
2. Validate your XML against YAWL 2.0 schema
3. Open an issue with XML snippet + error message
4. Consider manual TTL creation for complex cases

---

**Remember**: KNHK v4.0+ is TTL-only. This crate is temporary migration infrastructure.

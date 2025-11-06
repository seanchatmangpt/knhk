# Deprecated API Usage

**Last Updated**: January 2025

## Overview

KNHK uses some deprecated APIs from external dependencies. This document explains why deprecated APIs are used and the migration plan.

## Oxigraph Deprecated APIs

### Location

**File**: `rust/knhk-etl/src/ingest.rs`  
**Functions**: `parse_rdf_turtle()`, `parse_rdf_turtle_stream()`

### Deprecated APIs Used

1. **`oxigraph::sparql::Query::parse()`** - Deprecated in favor of `SparqlEvaluator::parse_query()`
2. **`oxigraph::store::Store::query()`** - Deprecated in favor of `SparqlEvaluator::evaluate()`

### Why Deprecated APIs Are Used

The migration path is incomplete:

1. **`SparqlEvaluator::parse_query()`** returns `PreparedSparqlQuery`, not `Query`
2. **`PreparedSparqlQuery`** cannot be converted to `Query`
3. **No non-deprecated evaluation API** exists that accepts `PreparedSparqlQuery`

**Current Code**:
```rust
#[allow(deprecated)]
let query = Query::parse("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }", None)?;

#[allow(deprecated)]
let results = store.query(query)?;
```

**Attempted Migration** (doesn't work):
```rust
// This doesn't work - PreparedSparqlQuery cannot be used with store.query()
let prepared = SparqlEvaluator::parse_query("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }", None)?;
// No way to evaluate PreparedSparqlQuery with current API
```

### Migration Plan

**Status**: Waiting for oxigraph to provide complete migration path

**When Available**:
1. Update to use `SparqlEvaluator::parse_query()` and `SparqlEvaluator::evaluate()`
2. Remove `#[allow(deprecated)]` attributes
3. Update error handling for new API

**Tracking**: Monitor oxigraph releases for complete migration path

## Impact

### Functionality
- ✅ **No impact**: Deprecated APIs work correctly
- ✅ **No breaking changes**: Behavior is unchanged

### Maintenance
- ⚠️ **Future risk**: Deprecated APIs may be removed in future oxigraph versions
- ⚠️ **Migration required**: Will need to migrate when complete API is available

### Code Quality
- ✅ **Documented**: Deprecated usage is clearly documented
- ✅ **Suppressed warnings**: `#[allow(deprecated)]` used appropriately
- ✅ **Comments explain**: Inline comments explain why deprecated APIs are used

## Related Documentation

- [Current Status](STATUS.md) - Overall implementation status
- [Ingest Stage](../rust/knhk-etl/src/ingest.rs) - Implementation details


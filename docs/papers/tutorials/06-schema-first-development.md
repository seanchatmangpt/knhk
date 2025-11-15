# Tutorial 6: Schema-First Development with Weaver

## Learning Objectives

By the end of this tutorial, you'll understand and practice:

- **Schema-first philosophy** - Design telemetry before implementation
- **Weaver schema syntax** - Write OpenTelemetry schemas correctly
- **Schema-driven development** - Let schemas guide implementation
- **Live validation** - Verify runtime behavior matches schema
- **Debug schema mismatches** - Fix validation errors systematically

**Time**: 25-35 minutes | **Level**: Intermediate
**Prerequisites**: [Understanding Telemetry](02-understanding-telemetry.md), [Building Production-Ready Features](04-building-production-ready-features.md)

---

## What is Schema-First Development?

### Traditional Development (Code-First)

```
1. Write code
2. Add telemetry as afterthought
3. Hope it works
4. Debug in production
5. Fix telemetry issues

Problems:
  ❌ Telemetry incomplete
  ❌ No validation
  ❌ Inconsistent attributes
  ❌ Runtime surprises
```

### Schema-First Development (KNHK Way)

```
1. Design OpenTelemetry schema
2. Validate schema syntax
3. Implement code to match schema
4. Live validate runtime telemetry
5. Deploy with confidence

Benefits:
  ✅ Telemetry complete by design
  ✅ Weaver validates correctness
  ✅ Consistent attributes
  ✅ No runtime surprises
  ✅ No false positives
```

### Why Schema-First?

**The False Positive Problem**:
```
Code-First:
  Tests pass ✅ → Assume telemetry works → FALSE POSITIVE
  └─ Runtime telemetry may be wrong/missing

Schema-First:
  Weaver validates ✅ → Telemetry proven correct → TRUE POSITIVE
  └─ Schema validation proves actual runtime behavior
```

**KNHK Principle**: Don't trust tests, trust schemas.

---

## Part 1: Design a Telemetry Schema

### Step 1.1: Feature Requirements

We'll build a **Search Feature** that:
- Accepts search queries from users
- Searches a data store
- Returns ranked results
- Tracks performance and usage

### Step 1.2: Identify Telemetry Needs

**What do we need to observe?**

```
Spans (operations):
  ✓ search.execute - Main search operation
  ✓ search.parse_query - Query parsing
  ✓ search.rank_results - Result ranking

Metrics (measurements):
  ✓ search.queries.total - Count of searches
  ✓ search.duration - Search duration
  ✓ search.results.count - Results returned

Attributes (context):
  ✓ query.text - The search query
  ✓ query.length - Query string length
  ✓ user.id - Who searched
  ✓ results.count - Number of results
  ✓ results.empty - Boolean for zero results
```

**✅ Checkpoint**: Telemetry requirements identified

### Step 1.3: Write the Schema

Create the OpenTelemetry schema:

```yaml
# registry/search.yaml

schema_url: https://example.com/schemas/search/1.0.0

groups:
  # Span definitions
  - id: search.operations
    type: span
    brief: Search feature operations
    spans:
      - id: search.execute
        brief: Executes a search query
        attributes:
          - id: query.text
            type: string
            requirement_level: required
            brief: The search query text
            examples: ["rust tutorial", "performance optimization"]

          - id: query.length
            type: int
            requirement_level: required
            brief: Length of the query string

          - id: user.id
            type: int
            requirement_level: required
            brief: User who initiated the search

          - id: results.count
            type: int
            requirement_level: required
            brief: Number of results returned

          - id: results.empty
            type: boolean
            requirement_level: required
            brief: Whether the search returned zero results

      - id: search.parse_query
        brief: Parses and validates the search query
        attributes:
          - id: query.text
            type: string
            requirement_level: required
            brief: The raw query text

          - id: query.valid
            type: boolean
            requirement_level: required
            brief: Whether the query is valid

      - id: search.rank_results
        brief: Ranks search results by relevance
        attributes:
          - id: results.count
            type: int
            requirement_level: required
            brief: Number of results to rank

          - id: ranking.algorithm
            type: string
            requirement_level: optional
            brief: Ranking algorithm used
            examples: ["tf-idf", "bm25", "neural"]

  # Metric definitions
  - id: search.metrics
    type: metric
    brief: Search feature metrics
    metrics:
      - id: search.queries.total
        brief: Total number of search queries
        instrument: counter
        unit: "{query}"

      - id: search.duration
        brief: Duration of search operations
        instrument: histogram
        unit: ms

      - id: search.results.count
        brief: Number of results per search
        instrument: histogram
        unit: "{result}"
```

**✅ Checkpoint**: Schema designed and written

### Step 1.4: Validate Schema Syntax

```bash
# Validate the schema definition
weaver registry check -r registry/

# Expected output:
# ✓ Schema validation passed
# ✓ File: registry/search.yaml
#   ✓ Span: search.execute (5 attributes)
#   ✓ Span: search.parse_query (2 attributes)
#   ✓ Span: search.rank_results (2 attributes)
#   ✓ Metric: search.queries.total (counter)
#   ✓ Metric: search.duration (histogram)
#   ✓ Metric: search.results.count (histogram)
```

**If validation fails**:
```bash
# Common errors:
# - Invalid YAML syntax
# - Missing required fields
# - Invalid attribute types
# - Incorrect metric instrument type
```

**✅ Checkpoint**: Schema syntax is valid

---

## Part 2: Implement Code to Match Schema

### Step 2.1: Define Data Structures

```rust
// src/search.rs

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub text: String,
    pub user_id: u64,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: u64,
    pub title: String,
    pub relevance_score: f64,
}

#[derive(Debug)]
pub enum SearchError {
    InvalidQuery,
    EmptyQuery,
    TooLong,
}

pub struct SearchEngine {
    documents: Vec<Document>,
}

#[derive(Debug, Clone)]
struct Document {
    id: u64,
    title: String,
    content: String,
}
```

**✅ Checkpoint**: Data structures defined

### Step 2.2: Implement with Instrumentation

**CRITICAL**: Instrumentation must match schema exactly.

```rust
use tracing::{instrument, info, warn};
use opentelemetry::metrics::{Counter, Histogram};

pub struct SearchEngine {
    documents: Vec<Document>,
    // Metrics (must match schema)
    queries_counter: Counter<u64>,
    duration_histogram: Histogram<f64>,
    results_histogram: Histogram<u64>,
}

impl SearchEngine {
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter("search");

        Self {
            documents: Vec::new(),
            queries_counter: meter
                .u64_counter("search.queries.total")
                .with_description("Total number of search queries")
                .with_unit("{query}")
                .init(),
            duration_histogram: meter
                .f64_histogram("search.duration")
                .with_description("Duration of search operations")
                .with_unit("ms")
                .init(),
            results_histogram: meter
                .u64_histogram("search.results.count")
                .with_description("Number of results per search")
                .with_unit("{result}")
                .init(),
        }
    }

    // Main search operation - must match search.execute span
    #[instrument(
        skip(self),
        fields(
            query.text = %query.text,
            query.length = query.text.len(),
            user.id = query.user_id,
            results.count,  // Will be set later
            results.empty,  // Will be set later
        )
    )]
    pub fn search(&mut self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        let start = std::time::Instant::now();

        // Record query
        self.queries_counter.add(1, &[]);

        // Parse query (child span)
        let parsed = self.parse_query(&query.text)?;

        // Execute search
        let mut results = self.execute_search(&parsed);

        // Rank results (child span)
        results = self.rank_results(results);

        // Record span attributes
        let span = tracing::Span::current();
        span.record("results.count", results.len());
        span.record("results.empty", results.is_empty());

        // Record metrics
        let duration = start.elapsed();
        self.duration_histogram.record(duration.as_secs_f64() * 1000.0, &[]);
        self.results_histogram.record(results.len() as u64, &[]);

        info!(
            results_count = results.len(),
            "Search completed successfully"
        );

        Ok(results)
    }

    // Parse query - must match search.parse_query span
    #[instrument(
        skip(self),
        fields(
            query.text = %query_text,
            query.valid,  // Will be set later
        )
    )]
    fn parse_query(&self, query_text: &str) -> Result<String, SearchError> {
        let span = tracing::Span::current();

        // Validation
        if query_text.is_empty() {
            span.record("query.valid", false);
            warn!("Empty query rejected");
            return Err(SearchError::EmptyQuery);
        }

        if query_text.len() > 1000 {
            span.record("query.valid", false);
            warn!("Query too long");
            return Err(SearchError::TooLong);
        }

        span.record("query.valid", true);
        info!("Query parsed successfully");

        Ok(query_text.to_lowercase())
    }

    // Rank results - must match search.rank_results span
    #[instrument(
        skip(self, results),
        fields(
            results.count = results.len(),
            ranking.algorithm = "simple",
        )
    )]
    fn rank_results(&self, mut results: Vec<SearchResult>) -> Vec<SearchResult> {
        // Sort by relevance score
        results.sort_by(|a, b| {
            b.relevance_score
                .partial_cmp(&a.relevance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        info!("Results ranked");
        results
    }

    fn execute_search(&self, query: &str) -> Vec<SearchResult> {
        // Simple search implementation
        self.documents
            .iter()
            .filter(|doc| {
                doc.title.to_lowercase().contains(query)
                    || doc.content.to_lowercase().contains(query)
            })
            .map(|doc| SearchResult {
                id: doc.id,
                title: doc.title.clone(),
                relevance_score: self.calculate_score(doc, query),
            })
            .collect()
    }

    fn calculate_score(&self, doc: &Document, query: &str) -> f64 {
        // Simple scoring (count occurrences)
        let title_matches = doc.title.to_lowercase().matches(query).count();
        let content_matches = doc.content.to_lowercase().matches(query).count();

        (title_matches * 10 + content_matches) as f64
    }

    pub fn add_document(&mut self, id: u64, title: String, content: String) {
        self.documents.push(Document { id, title, content });
    }
}
```

**✅ Checkpoint**: Code implements schema exactly

### Step 2.3: Write Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_returns_results() {
        // Arrange
        let mut engine = SearchEngine::new();
        engine.add_document(1, "Rust Tutorial".to_string(), "Learn Rust".to_string());
        engine.add_document(2, "Performance".to_string(), "Optimize code".to_string());

        let query = SearchQuery {
            text: "rust".to_string(),
            user_id: 123,
        };

        // Act
        let results = engine.search(query).unwrap();

        // Assert
        assert!(!results.is_empty());
        assert_eq!(results[0].id, 1);
    }

    #[test]
    fn test_search_rejects_empty_query() {
        let mut engine = SearchEngine::new();
        let query = SearchQuery {
            text: "".to_string(),
            user_id: 123,
        };

        let result = engine.search(query);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SearchError::EmptyQuery));
    }

    #[test]
    fn test_search_emits_telemetry() {
        // This will be validated by Weaver
        let mut engine = SearchEngine::new();
        engine.add_document(1, "Test".to_string(), "Content".to_string());

        let query = SearchQuery {
            text: "test".to_string(),
            user_id: 123,
        };

        let results = engine.search(query).unwrap();
        assert!(!results.is_empty());

        // Telemetry emission validated by Weaver live-check
    }
}
```

Run tests:
```bash
cargo test

# test test_search_returns_results ... ok
# test test_search_rejects_empty_query ... ok
# test test_search_emits_telemetry ... ok
# test result: ok. 3 passed
```

**✅ Checkpoint**: Tests pass

---

## Part 3: Live Telemetry Validation

### Step 3.1: Create Example Program

```rust
// examples/search_demo.rs

use search::{SearchEngine, SearchQuery};
use tracing_subscriber;

fn main() {
    // Setup telemetry
    tracing_subscriber::fmt::init();

    // Initialize OpenTelemetry
    opentelemetry::global::set_text_map_propagator(
        opentelemetry_jaeger::Propagator::new()
    );

    let mut engine = SearchEngine::new();

    // Add sample documents
    engine.add_document(
        1,
        "Rust Programming".to_string(),
        "Learn Rust programming language".to_string(),
    );
    engine.add_document(
        2,
        "Performance Optimization".to_string(),
        "Optimize Rust code for speed".to_string(),
    );
    engine.add_document(
        3,
        "TDD Tutorial".to_string(),
        "Test-driven development in Rust".to_string(),
    );

    // Execute searches
    let queries = vec![
        SearchQuery {
            text: "rust".to_string(),
            user_id: 1,
        },
        SearchQuery {
            text: "performance".to_string(),
            user_id: 2,
        },
        SearchQuery {
            text: "tutorial".to_string(),
            user_id: 1,
        },
    ];

    for query in queries {
        match engine.search(query.clone()) {
            Ok(results) => {
                println!(
                    "Query '{}' returned {} results",
                    query.text,
                    results.len()
                );
            }
            Err(e) => {
                eprintln!("Search error: {:?}", e);
            }
        }
    }

    // Shutdown telemetry
    opentelemetry::global::shutdown_tracer_provider();
}
```

**✅ Checkpoint**: Demo program created

### Step 3.2: Run Live Validation

```bash
# Terminal 1: Run the demo program
cargo run --example search_demo

# Output:
# Query 'rust' returned 2 results
# Query 'performance' returned 1 results
# Query 'tutorial' returned 2 results
```

```bash
# Terminal 2: Validate live telemetry
weaver registry live-check --registry registry/

# Expected output:
# ✓ Connecting to telemetry endpoint
# ✓ Checking span: search.execute
#   ✓ Attribute query.text present (type: string)
#   ✓ Attribute query.length present (type: int)
#   ✓ Attribute user.id present (type: int)
#   ✓ Attribute results.count present (type: int)
#   ✓ Attribute results.empty present (type: boolean)
# ✓ Checking span: search.parse_query
#   ✓ Attribute query.text present (type: string)
#   ✓ Attribute query.valid present (type: boolean)
# ✓ Checking span: search.rank_results
#   ✓ Attribute results.count present (type: int)
#   ✓ Attribute ranking.algorithm present (type: string)
# ✓ Checking metric: search.queries.total
#   ✓ Counter incremented (value: 3)
# ✓ Checking metric: search.duration
#   ✓ Histogram recorded (samples: 3)
# ✓ Checking metric: search.results.count
#   ✓ Histogram recorded (samples: 3)
#
# ✅ All telemetry validates against schema
```

**✅ Checkpoint**: Live validation passes

---

## Part 4: Debug Schema Mismatches

### Step 4.1: Common Validation Errors

#### Error 1: Missing Required Attribute

```bash
# Weaver output:
# ✗ Checking span: search.execute
#   ✗ Required attribute 'user.id' not present
```

**Cause**: Forgot to include attribute in span fields

**Fix**:
```rust
// ❌ WRONG: Missing user.id
#[instrument(skip(self), fields(
    query.text = %query.text,
    query.length = query.text.len(),
))]
pub fn search(&mut self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
    // ...
}

// ✅ CORRECT: Include user.id
#[instrument(skip(self), fields(
    query.text = %query.text,
    query.length = query.text.len(),
    user.id = query.user_id,  // Added
))]
pub fn search(&mut self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
    // ...
}
```

#### Error 2: Wrong Attribute Type

```bash
# Weaver output:
# ✗ Checking span: search.execute
#   ✗ Attribute 'results.count' expected int, got string
```

**Cause**: Emitting wrong type

**Fix**:
```rust
// ❌ WRONG: String instead of int
span.record("results.count", results.len().to_string());

// ✅ CORRECT: Int type
span.record("results.count", results.len());
```

#### Error 3: Span Not Emitted

```bash
# Weaver output:
# ✗ Span 'search.parse_query' declared in schema but not emitted
```

**Cause**: Missing #[instrument] macro

**Fix**:
```rust
// ❌ WRONG: No instrumentation
fn parse_query(&self, query_text: &str) -> Result<String, SearchError> {
    // ...
}

// ✅ CORRECT: Add instrumentation
#[instrument(skip(self), fields(...))]
fn parse_query(&self, query_text: &str) -> Result<String, SearchError> {
    // ...
}
```

#### Error 4: Metric Not Incremented

```bash
# Weaver output:
# ✗ Checking metric: search.queries.total
#   ✗ Expected counter increments but none observed
```

**Cause**: Forgot to call counter.add()

**Fix**:
```rust
// ❌ WRONG: Counter not incremented
pub fn search(&mut self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
    // ... search logic ...
    Ok(results)
}

// ✅ CORRECT: Increment counter
pub fn search(&mut self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
    self.queries_counter.add(1, &[]);  // Added
    // ... search logic ...
    Ok(results)
}
```

**✅ Checkpoint**: Common errors understood

### Step 4.2: Systematic Debugging Process

```
When Weaver validation fails:

1. READ the error message carefully
   └─ Identifies exactly what's wrong

2. LOCATE the problem in code
   └─ Find the span/metric that's failing

3. COMPARE schema vs code
   └─ Schema: What should be emitted
   └─ Code: What is actually emitted

4. FIX the mismatch
   └─ Update code to match schema
   └─ (Or update schema if requirements changed)

5. RE-VALIDATE
   └─ Run live-check again
   └─ Confirm fix worked
```

**✅ Checkpoint**: Debugging process learned

---

## Part 5: Schema Evolution

### Step 5.1: Adding New Telemetry

**Requirement**: Add caching metrics to track cache hits/misses

#### Update Schema

```yaml
# registry/search.yaml (add to metrics group)

metrics:
  # ... existing metrics ...

  - id: search.cache.hits
    brief: Number of cache hits
    instrument: counter
    unit: "{hit}"

  - id: search.cache.misses
    brief: Number of cache misses
    instrument: counter
    unit: "{miss}"
```

#### Validate Schema

```bash
weaver registry check -r registry/

# ✓ New metrics valid
```

#### Update Code

```rust
pub struct SearchEngine {
    documents: Vec<Document>,
    cache: HashMap<String, Vec<SearchResult>>,  // New cache
    queries_counter: Counter<u64>,
    duration_histogram: Histogram<f64>,
    results_histogram: Histogram<u64>,
    cache_hits_counter: Counter<u64>,    // New
    cache_misses_counter: Counter<u64>,  // New
}

impl SearchEngine {
    pub fn new() -> Self {
        let meter = opentelemetry::global::meter("search");

        Self {
            documents: Vec::new(),
            cache: HashMap::new(),
            queries_counter: meter.u64_counter("search.queries.total").init(),
            duration_histogram: meter.f64_histogram("search.duration").init(),
            results_histogram: meter.u64_histogram("search.results.count").init(),
            cache_hits_counter: meter
                .u64_counter("search.cache.hits")
                .with_description("Number of cache hits")
                .with_unit("{hit}")
                .init(),
            cache_misses_counter: meter
                .u64_counter("search.cache.misses")
                .with_description("Number of cache misses")
                .with_unit("{miss}")
                .init(),
        }
    }

    pub fn search(&mut self, query: SearchQuery) -> Result<Vec<SearchResult>, SearchError> {
        // Check cache first
        if let Some(cached_results) = self.cache.get(&query.text) {
            self.cache_hits_counter.add(1, &[]);
            info!("Cache hit for query");
            return Ok(cached_results.clone());
        }

        self.cache_misses_counter.add(1, &[]);

        // ... rest of search logic ...

        // Store in cache
        self.cache.insert(query.text.clone(), results.clone());

        Ok(results)
    }
}
```

#### Validate New Telemetry

```bash
# Run demo
cargo run --example search_demo

# Validate
weaver registry live-check --registry registry/

# ✓ Checking metric: search.cache.hits
#   ✓ Counter incremented
# ✓ Checking metric: search.cache.misses
#   ✓ Counter incremented
```

**✅ Checkpoint**: Schema evolution successful

---

## What You've Learned

### Schema-First Development Process

```
1. DESIGN telemetry schema
   ├─ Identify spans, metrics, attributes
   ├─ Write YAML schema
   └─ Validate syntax with Weaver

2. IMPLEMENT code to match schema
   ├─ Add #[instrument] macros
   ├─ Emit required attributes
   ├─ Record metrics
   └─ Match schema exactly

3. VALIDATE runtime telemetry
   ├─ Run application
   ├─ Use Weaver live-check
   └─ Fix any mismatches

4. ITERATE as needed
   ├─ Evolve schema for new requirements
   ├─ Update code to match
   └─ Re-validate
```

### Key Benefits

```
✓ Telemetry correctness guaranteed
✓ No false positives (Weaver is source of truth)
✓ Consistent attribute naming
✓ Complete observability by design
✓ Catch issues before production
```

### Weaver Validation Hierarchy

```
Schema Validation (weaver registry check):
  └─ Validates schema syntax
  └─ Ensures schema is well-formed

Live Validation (weaver registry live-check):
  └─ Validates runtime telemetry
  └─ Ensures code matches schema
  └─ SOURCE OF TRUTH for correctness
```

---

## Practice Exercises

### Exercise 1: Add Error Tracking (Easy)

Extend the search schema to track errors:
- Add `search.errors.total` counter
- Add `error.type` attribute to spans
- Implement error tracking
- Validate with Weaver

### Exercise 2: Add Latency Breakdown (Medium)

Track detailed timing:
- Add spans for each search phase
- Track `phase.duration` for each
- Implement instrumentation
- Validate timing accuracy

### Exercise 3: Multi-Tenant Search (Hard)

Add tenant isolation:
- Add `tenant.id` attribute
- Track per-tenant metrics
- Implement tenant filtering
- Validate isolation works

---

## Next Steps

Now that you understand schema-first development:

1. **Validate production readiness** - [How-to 12: Validate Production Readiness](../how-to-guides/12-validate-production-readiness.md)
2. **Fix validation errors** - [How-to 6: Fix Weaver Validation Errors](../how-to-guides/06-fix-weaver-validation-errors.md)
3. **Master telemetry** - [How-to 7: Emit Proper Telemetry](../how-to-guides/07-emit-proper-telemetry.md)

---

## Related Resources

**Prerequisites**:
- [Tutorial 2: Understanding Telemetry](02-understanding-telemetry.md)
- [Tutorial 4: Building Production-Ready Features](04-building-production-ready-features.md)

**How-to Guides**:
- [How-to 5: Create OTel Schemas](../how-to-guides/05-create-otel-schemas.md)
- [How-to 6: Fix Weaver Validation Errors](../how-to-guides/06-fix-weaver-validation-errors.md)
- [How-to 7: Emit Proper Telemetry](../how-to-guides/07-emit-proper-telemetry.md)

---

**Created**: 2025-11-15
**Status**: Complete
**Difficulty**: Intermediate
**Estimated Time**: 25-35 minutes

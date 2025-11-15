// Telemetry Instrumentation Example
// Demonstrates strategic OpenTelemetry instrumentation
//
// Key Concepts:
// - Telemetry instrumentation pyramid (spans → metrics → logs)
// - Schema-first approach (define before code)
// - Context propagation
// - Performance overhead management
// - Before/after comparison

// NOTE: This is a simplified example. In production, use knhk_otel crate.

use std::time::Instant;

/// Simplified span context for demo
#[derive(Debug, Clone)]
struct SpanContext {
    trace_id: String,
    span_id: String,
    parent_span_id: Option<String>,
}

/// Span status
#[derive(Debug)]
enum SpanStatus {
    Ok,
    Error,
}

/// Simplified tracer for demo
struct Tracer {
    service_name: String,
    spans: Vec<Span>,
}

#[derive(Debug, Clone)]
struct Span {
    name: String,
    context: SpanContext,
    attributes: Vec<(String, String)>,
    start_time: Instant,
    end_time: Option<Instant>,
    status: Option<SpanStatus>,
}

impl Tracer {
    fn new(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            spans: Vec::new(),
        }
    }

    fn start_span(&mut self, name: &str, parent: Option<&SpanContext>) -> SpanContext {
        let span_id = format!("span_{}", self.spans.len() + 1);
        let trace_id = parent
            .map(|p| p.trace_id.clone())
            .unwrap_or_else(|| format!("trace_{}", self.spans.len() + 1));

        let context = SpanContext {
            trace_id,
            span_id,
            parent_span_id: parent.map(|p| p.span_id.clone()),
        };

        let span = Span {
            name: name.to_string(),
            context: context.clone(),
            attributes: Vec::new(),
            start_time: Instant::now(),
            end_time: None,
            status: None,
        };

        self.spans.push(span);
        context
    }

    fn add_attribute(&mut self, ctx: &SpanContext, key: &str, value: &str) {
        if let Some(span) = self
            .spans
            .iter_mut()
            .find(|s| s.context.span_id == ctx.span_id)
        {
            span.attributes.push((key.to_string(), value.to_string()));
        }
    }

    fn end_span(&mut self, ctx: &SpanContext, status: SpanStatus) {
        if let Some(span) = self
            .spans
            .iter_mut()
            .find(|s| s.context.span_id == ctx.span_id)
        {
            span.end_time = Some(Instant::now());
            span.status = Some(status);
        }
    }

    fn export(&self) {
        println!("=== Exported Spans ===");
        for span in &self.spans {
            let duration = span
                .end_time
                .map(|end| end.duration_since(span.start_time))
                .unwrap_or_default();

            println!("Span: {}", span.name);
            println!("  Trace ID: {}", span.context.trace_id);
            println!("  Span ID: {}", span.context.span_id);
            if let Some(parent_id) = &span.context.parent_span_id {
                println!("  Parent Span ID: {}", parent_id);
            }
            println!("  Duration: {:?}", duration);
            println!("  Status: {:?}", span.status);
            println!("  Attributes:");
            for (key, value) in &span.attributes {
                println!("    {}: {}", key, value);
            }
            println!();
        }
    }
}

// ============================================================================
// Example Code: Query Execution
// ============================================================================

#[derive(Debug)]
enum QueryType {
    Ask,
    Select,
    Construct,
}

struct Query {
    query_type: QueryType,
    sparql: String,
}

impl Query {
    fn new_ask(sparql: &str) -> Self {
        Self {
            query_type: QueryType::Ask,
            sparql: sparql.to_string(),
        }
    }
}

// ❌ BEFORE: No instrumentation
mod without_telemetry {
    use super::*;

    pub fn execute_query(query: Query) -> Result<bool, String> {
        // Parse query
        let parsed = parse_query(&query.sparql)?;

        // Execute query
        let result = execute_parsed_query(&parsed)?;

        Ok(result)
    }

    fn parse_query(sparql: &str) -> Result<String, String> {
        // Simulate parsing
        std::thread::sleep(std::time::Duration::from_micros(50));

        if sparql.is_empty() {
            Err("Empty query".to_string())
        } else {
            Ok(sparql.to_uppercase())
        }
    }

    fn execute_parsed_query(_parsed: &str) -> Result<bool, String> {
        // Simulate execution
        std::thread::sleep(std::time::Duration::from_micros(100));
        Ok(true)
    }
}

// ✅ AFTER: With strategic instrumentation
mod with_telemetry {
    use super::*;

    pub fn execute_query(query: Query, tracer: &mut Tracer) -> Result<bool, String> {
        // Start span at service boundary
        let span_ctx = tracer.start_span("knhk.query.execute", None);

        // Add essential attributes
        let query_type_str = format!("{:?}", query.query_type);
        tracer.add_attribute(&span_ctx, "query.type", &query_type_str);
        tracer.add_attribute(&span_ctx, "service.name", "knhk-query-executor");

        // Execute operation
        let result = execute_query_internal(&query, tracer, &span_ctx);

        // Set span status
        match &result {
            Ok(_) => {
                tracer.add_attribute(&span_ctx, "query.result", "success");
                tracer.end_span(&span_ctx, SpanStatus::Ok);
            }
            Err(e) => {
                tracer.add_attribute(&span_ctx, "query.error", e);
                tracer.end_span(&span_ctx, SpanStatus::Error);
            }
        }

        result
    }

    fn execute_query_internal(
        query: &Query,
        tracer: &mut Tracer,
        parent: &SpanContext,
    ) -> Result<bool, String> {
        // Parse query (child span)
        let parse_span = tracer.start_span("knhk.query.parse", Some(parent));
        let parsed = parse_query(&query.sparql);

        match &parsed {
            Ok(p) => {
                tracer.add_attribute(&parse_span, "query.parsed_length", &p.len().to_string());
                tracer.end_span(&parse_span, SpanStatus::Ok);
            }
            Err(e) => {
                tracer.add_attribute(&parse_span, "error.message", e);
                tracer.end_span(&parse_span, SpanStatus::Error);
                return Err(e.clone());
            }
        }

        let parsed = parsed.unwrap();

        // Execute query (child span)
        let exec_span = tracer.start_span("knhk.query.execute_internal", Some(parent));
        let result = execute_parsed_query(&parsed);

        match &result {
            Ok(res) => {
                tracer.add_attribute(&exec_span, "query.result", &res.to_string());
                tracer.end_span(&exec_span, SpanStatus::Ok);
            }
            Err(e) => {
                tracer.add_attribute(&exec_span, "error.message", e);
                tracer.end_span(&exec_span, SpanStatus::Error);
                return Err(e.clone());
            }
        }

        result
    }

    fn parse_query(sparql: &str) -> Result<String, String> {
        // Same implementation as without_telemetry
        std::thread::sleep(std::time::Duration::from_micros(50));

        if sparql.is_empty() {
            Err("Empty query".to_string())
        } else {
            Ok(sparql.to_uppercase())
        }
    }

    fn execute_parsed_query(_parsed: &str) -> Result<bool, String> {
        // Same implementation as without_telemetry
        std::thread::sleep(std::time::Duration::from_micros(100));
        Ok(true)
    }
}

fn main() {
    println!("=== Telemetry Instrumentation Example ===\n");

    // Example 1: Without telemetry
    println!("--- Example 1: Without Telemetry ---");
    let query1 = Query::new_ask("ASK { ?s ?p ?o }");

    let start = Instant::now();
    match without_telemetry::execute_query(query1) {
        Ok(result) => println!("✅ Query result: {}", result),
        Err(e) => println!("❌ Error: {}", e),
    }
    let duration = start.elapsed();
    println!("Duration: {:?}", duration);
    println!("Telemetry: None (no visibility into execution)");
    println!();

    // Example 2: With telemetry
    println!("--- Example 2: With Telemetry ---");
    let query2 = Query::new_ask("ASK { ?s ?p ?o }");
    let mut tracer = Tracer::new("knhk-example");

    let start = Instant::now();
    match with_telemetry::execute_query(query2, &mut tracer) {
        Ok(result) => println!("✅ Query result: {}", result),
        Err(e) => println!("❌ Error: {}", e),
    }
    let duration = start.elapsed();
    println!("Duration: {:?}", duration);
    println!();

    // Export spans
    tracer.export();

    // Example 3: Error case with telemetry
    println!("--- Example 3: Error Case (Empty Query) ---");
    let query3 = Query::new_ask("");
    let mut tracer2 = Tracer::new("knhk-example");

    match with_telemetry::execute_query(query3, &mut tracer2) {
        Ok(result) => println!("✅ Query result: {}", result),
        Err(e) => println!("❌ Error: {}", e),
    }
    println!();

    tracer2.export();

    // Instrumentation pyramid
    println!("=== Telemetry Instrumentation Pyramid ===");
    println!();
    println!("         Spans (few)");
    println!("        /           \\");
    println!("       /    Metrics   \\");
    println!("      /    (moderate)  \\");
    println!("     /                  \\");
    println!("    /     Logs (many)    \\");
    println!("   /_______________________\\");
    println!();
    println!("Spans:   10-20 per request (service boundaries, major operations)");
    println!("Metrics: 50-100 per service (counters, histograms, gauges)");
    println!("Logs:    100-1000 per request (detailed debugging info)");
    println!();

    // Key principles
    println!("=== Strategic Instrumentation Principles ===");
    println!("1. ✅ Schema-first: Define telemetry schema before code");
    println!("2. ✅ Service boundaries: Span at entry/exit points");
    println!("3. ✅ Context propagation: Parent-child relationships");
    println!("4. ✅ Essential attributes: Only what's needed for debugging");
    println!("5. ✅ Error tracking: Always record errors with context");
    println!("6. ✅ Performance budget: ≤5% overhead");
    println!("7. ❌ No telemetry in hot path: ≤8 ticks operations");
    println!("8. ✅ Batch exports: Async, non-blocking");
    println!();

    println!("=== Before/After Comparison ===");
    println!();
    println!("Without Telemetry:");
    println!("  ✅ Faster (no overhead)");
    println!("  ❌ No visibility into execution");
    println!("  ❌ No error context");
    println!("  ❌ No distributed tracing");
    println!("  ❌ Difficult to debug production issues");
    println!();
    println!("With Telemetry:");
    println!("  ✅ Full execution visibility");
    println!("  ✅ Error context and stack traces");
    println!("  ✅ Distributed tracing across services");
    println!("  ✅ Performance profiling");
    println!("  ❌ Small overhead (~5%)");
    println!();

    println!("=== Weaver Schema Example ===");
    println!();
    println!("```yaml");
    println!("groups:");
    println!("  - id: knhk.query.execute");
    println!("    type: span");
    println!("    brief: \"Execute SPARQL query\"");
    println!("    attributes:");
    println!("      - id: query.type");
    println!("        type: string");
    println!("        requirement_level: required");
    println!("        examples: [\"ASK\", \"SELECT\"]");
    println!("      - id: query.result");
    println!("        type: string");
    println!("        requirement_level: recommended");
    println!("```");
    println!();

    println!("=== Run with Real OTEL ===");
    println!("Use knhk_otel crate for production:");
    println!();
    println!("```rust");
    println!("use knhk_otel::{{init_tracer, Tracer, SpanStatus}};");
    println!();
    println!("let _guard = init_tracer(\"knhk\", \"1.0.0\", Some(\"http://localhost:4318\"))");
    println!("    .expect(\"Init tracer\");");
    println!();
    println!("let mut tracer = Tracer::new();");
    println!("let span = tracer.start_span(\"knhk.query.execute\".to_string(), None);");
    println!("// ... execute query ...");
    println!("tracer.end_span(span, SpanStatus::Ok);");
    println!("```");
}

// Key Takeaways:
//
// 1. **Strategic Instrumentation**: Not everything needs telemetry
//    - Service boundaries: Always instrument
//    - Hot path (≤8 ticks): Never instrument
//    - Warm path: Instrument with caution
//    - Cold path: Full instrumentation
//
// 2. **Instrumentation Pyramid**: Different signals for different purposes
//    - Spans: Track request flow (distributed tracing)
//    - Metrics: Quantitative measurements (dashboards, alerts)
//    - Logs: Detailed debugging (troubleshooting)
//
// 3. **Context Propagation**: Parent-child relationships
//    - Root span: Service entry point
//    - Child spans: Internal operations
//    - Maintains causality across distributed system
//
// 4. **Essential Attributes Only**: Minimize overhead
//    - ✅ query.type, query.result (actionable)
//    - ❌ query.full_sparql (too verbose, use logs)
//    - ❌ Large objects (serialize to metrics/logs instead)
//
// 5. **Error Handling**: Always record errors
//    - Span status: Ok or Error
//    - Error attribute: Descriptive message
//    - Maintains error context for debugging
//
// 6. **Schema-First**: Define before coding
//    - Document expected telemetry in schema
//    - Validate with Weaver live-check
//    - Ensures consistency across services
//
// Performance impact:
// - Span creation: ~100 ns
// - Attribute addition: ~50 ns
// - Span export (batched): ~1 µs per span
// - Total overhead: ~5% for well-instrumented services
//
// See also:
// - /home/user/knhk/docs/reference/cards/TELEMETRY_CHECKLIST.md
// - /home/user/knhk/docs/troubleshooting/TELEMETRY_TROUBLESHOOTING.md

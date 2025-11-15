// Warm Path CONSTRUCT8 Example
// Demonstrates CONSTRUCT8 query execution in ≤500µs
//
// Key Concepts:
// - Emit operations (output generation)
// - Pre-allocated buffers
// - Async I/O
// - Caching strategy
// - Performance measurement

use std::time::Instant;

/// Triple representation (standard format, not SoA)
#[derive(Debug, Clone, PartialEq)]
struct Triple {
    subject: String,
    predicate: String,
    object: String,
}

impl Triple {
    fn new(s: &str, p: &str, o: &str) -> Self {
        Self {
            subject: s.to_string(),
            predicate: p.to_string(),
            object: o.to_string(),
        }
    }

    /// Serialize to N-Triples format
    fn serialize(&self) -> String {
        format!(
            "<{}> <{}> <{}> .",
            self.subject, self.predicate, self.object
        )
    }
}

/// Warm path result
#[derive(Debug)]
struct WarmPathResult {
    /// Number of triples emitted
    lanes_written: usize,
    /// Execution latency
    latency_us: u128,
    /// Receipt for provenance
    receipt: Receipt,
}

/// Receipt for provenance tracking
#[derive(Debug, Clone)]
struct Receipt {
    /// Number of triples processed
    lanes: u32,
    /// Span ID for OTEL correlation
    span_id: u64,
    /// Provenance hash
    a_hash: u64,
}

/// Warm path executor
struct WarmPathExecutor {
    /// In-memory graph (simple vector for demo)
    triples: Vec<Triple>,
    /// LRU cache for queries (simplified)
    cache: std::collections::HashMap<String, Vec<Triple>>,
    /// Pre-allocated output buffer
    output_buffer: Vec<u8>,
}

impl WarmPathExecutor {
    /// Create new executor
    fn new() -> Self {
        Self {
            triples: Vec::new(),
            cache: std::collections::HashMap::new(),
            output_buffer: Vec::with_capacity(4096), // 4KB buffer
        }
    }

    /// Add triple to graph
    fn add_triple(&mut self, t: Triple) {
        self.triples.push(t);
    }

    /// Execute CONSTRUCT8 query
    /// Performance target: ≤500µs
    ///
    /// CONSTRUCT8 emits up to 8 triples matching the pattern
    fn execute_construct8(&mut self, pattern: &str) -> Result<WarmPathResult, String> {
        let start = Instant::now();

        // Step 1: Check cache (≤10µs)
        if let Some(cached_triples) = self.cache.get(pattern) {
            let latency = start.elapsed().as_micros();

            return Ok(WarmPathResult {
                lanes_written: cached_triples.len(),
                latency_us: latency,
                receipt: Receipt {
                    lanes: cached_triples.len() as u32,
                    span_id: 12345, // Would be from OTEL
                    a_hash: hash_result(cached_triples),
                },
            });
        }

        // Step 2: Parse pattern (≤50µs)
        let (subject_pattern, predicate_pattern, object_pattern) = parse_pattern(pattern)?;

        // Step 3: Query graph (≤200µs for small graphs)
        let mut matching_triples = Vec::with_capacity(8); // Limit to 8 (CONSTRUCT8)

        for triple in &self.triples {
            if matching_triples.len() >= 8 {
                break; // CONSTRUCT8 = max 8 triples
            }

            if matches_pattern(&triple.subject, &subject_pattern)
                && matches_pattern(&triple.predicate, &predicate_pattern)
                && matches_pattern(&triple.object, &object_pattern)
            {
                matching_triples.push(triple.clone());
            }
        }

        // Step 4: Emit triples to buffer (≤100µs)
        self.output_buffer.clear();
        for triple in &matching_triples {
            let serialized = triple.serialize();
            self.output_buffer.extend_from_slice(serialized.as_bytes());
            self.output_buffer.push(b'\n');
        }

        // Step 5: Cache result (≤10µs)
        self.cache
            .insert(pattern.to_string(), matching_triples.clone());

        // Step 6: Generate receipt
        let receipt = Receipt {
            lanes: matching_triples.len() as u32,
            span_id: 12345, // Would be from OTEL
            a_hash: hash_result(&matching_triples),
        };

        let latency = start.elapsed().as_micros();

        Ok(WarmPathResult {
            lanes_written: matching_triples.len(),
            latency_us: latency,
            receipt,
        })
    }

    /// Get emitted data
    fn get_output(&self) -> &[u8] {
        &self.output_buffer
    }
}

/// Parse CONSTRUCT pattern
/// Format: "CONSTRUCT { ?s ?p ?o }" or "CONSTRUCT { <subject> ?p ?o }"
fn parse_pattern(pattern: &str) -> Result<(String, String, String), String> {
    // Simplified parser (real implementation would use SPARQL parser)
    let pattern = pattern
        .trim()
        .strip_prefix("CONSTRUCT {")
        .and_then(|s| s.strip_suffix("}"))
        .ok_or("Invalid CONSTRUCT pattern")?
        .trim();

    let parts: Vec<&str> = pattern.split_whitespace().collect();
    if parts.len() != 3 {
        return Err("Pattern must have 3 parts (subject, predicate, object)".to_string());
    }

    Ok((
        parts[0].to_string(),
        parts[1].to_string(),
        parts[2].to_string(),
    ))
}

/// Check if value matches pattern
/// Pattern: "?var" (variable, matches anything) or "<iri>" (exact match)
fn matches_pattern(value: &str, pattern: &str) -> bool {
    if pattern.starts_with('?') {
        true // Variable matches anything
    } else {
        value == pattern.trim_matches(|c| c == '<' || c == '>')
    }
}

/// Hash result for provenance
fn hash_result(triples: &[Triple]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for triple in triples {
        triple.subject.hash(&mut hasher);
        triple.predicate.hash(&mut hasher);
        triple.object.hash(&mut hasher);
    }
    hasher.finish()
}

fn main() {
    println!("=== KNHK Warm Path CONSTRUCT8 Example ===\n");

    // Create executor and populate graph
    let mut executor = WarmPathExecutor::new();

    // Add sample triples
    executor.add_triple(Triple::new(
        "http://example.org/Alice",
        "http://example.org/knows",
        "http://example.org/Bob",
    ));
    executor.add_triple(Triple::new(
        "http://example.org/Bob",
        "http://example.org/knows",
        "http://example.org/Charlie",
    ));
    executor.add_triple(Triple::new(
        "http://example.org/Alice",
        "http://example.org/age",
        "30",
    ));
    executor.add_triple(Triple::new(
        "http://example.org/Bob",
        "http://example.org/age",
        "25",
    ));

    println!("Graph populated with 4 triples\n");

    // Example 1: CONSTRUCT all "knows" relationships
    println!("--- CONSTRUCT8 Example 1: All 'knows' relationships ---");
    let pattern = "CONSTRUCT { ?s <http://example.org/knows> ?o }";
    println!("Pattern: {}", pattern);

    match executor.execute_construct8(pattern) {
        Ok(result) => {
            println!("Lanes written: {}", result.lanes_written);
            println!("Latency: {}µs (target: ≤500µs)", result.latency_us);
            println!(
                "Status: {}",
                if result.latency_us <= 500 {
                    "✅ PASS"
                } else {
                    "❌ FAIL"
                }
            );
            println!("Receipt: span_id={}, a_hash={:x}", result.receipt.span_id, result.receipt.a_hash);
            println!("\nEmitted data:");
            println!("{}", String::from_utf8_lossy(executor.get_output()));
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 2: CONSTRUCT all triples about Alice (cache miss)
    println!("\n--- CONSTRUCT8 Example 2: All triples about Alice ---");
    let pattern2 = "CONSTRUCT { <http://example.org/Alice> ?p ?o }";
    println!("Pattern: {}", pattern2);

    match executor.execute_construct8(pattern2) {
        Ok(result) => {
            println!("Lanes written: {}", result.lanes_written);
            println!("Latency: {}µs (target: ≤500µs)", result.latency_us);
            println!(
                "Status: {}",
                if result.latency_us <= 500 {
                    "✅ PASS"
                } else {
                    "❌ FAIL"
                }
            );
            println!("\nEmitted data:");
            println!("{}", String::from_utf8_lossy(executor.get_output()));
        }
        Err(e) => println!("Error: {}", e),
    }

    // Example 3: Repeat query (cache hit)
    println!("\n--- CONSTRUCT8 Example 3: Repeat query (cache hit) ---");
    println!("Pattern: {}", pattern2);

    match executor.execute_construct8(pattern2) {
        Ok(result) => {
            println!("Lanes written: {}", result.lanes_written);
            println!("Latency: {}µs (should be <10µs due to cache)", result.latency_us);
            println!(
                "Cache hit: {}",
                if result.latency_us < 50 {
                    "✅ YES"
                } else {
                    "❌ NO (cache miss)"
                }
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    // Benchmark
    println!("\n=== Benchmark (100 iterations) ===");
    let iterations = 100;
    let mut total_us = 0u128;
    let mut cache_hits = 0;

    for i in 0..iterations {
        // Alternate patterns to test cache
        let pattern = if i % 2 == 0 { pattern } else { pattern2 };

        match executor.execute_construct8(pattern) {
            Ok(result) => {
                total_us += result.latency_us;
                if result.latency_us < 50 {
                    cache_hits += 1;
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    let avg_us = total_us / iterations;
    println!("Average latency: {}µs (target: ≤500µs)", avg_us);
    println!("Cache hit rate: {:.1}%", (cache_hits as f64 / iterations as f64) * 100.0);
    println!(
        "Status: {}",
        if avg_us <= 500 {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );

    println!("\n=== Performance Breakdown ===");
    println!("Cache check:     ≤10µs");
    println!("Pattern parse:   ≤50µs");
    println!("Graph query:     ≤200µs (small graphs)");
    println!("Emit to buffer:  ≤100µs");
    println!("Cache update:    ≤10µs");
    println!("Total:           ≤500µs (warm path budget)");
}

// Key Takeaways:
//
// 1. **Pre-allocated Buffers**: Avoid allocations on hot path
//    - output_buffer pre-allocated to 4KB
//    - Reuse buffer for multiple queries
//
// 2. **Caching Strategy**: LRU cache for repeated queries
//    - Cache hit: <10µs (memory lookup)
//    - Cache miss: 100-500µs (query + parse)
//
// 3. **CONSTRUCT8 Limit**: Max 8 triples emitted
//    - Keeps output bounded
//    - Predictable performance
//
// 4. **Receipt Generation**: Provenance tracking
//    - Hash of results for verification
//    - Span ID for OTEL correlation
//
// 5. **Error Handling**: Result<T, E> pattern
//    - No unwrap() or expect() in production
//    - Descriptive error messages
//
// Performance targets:
// - CONSTRUCT8: ≤500µs (warm path)
// - Cache hit: <10µs
// - Cache miss: 100-500µs
//
// If operations exceed 500µs:
// 1. Check graph size (limit to ≤1000 triples for warm path)
// 2. Check cache hit rate (should be ≥90%)
// 3. Profile with flamegraph
// 4. Consider moving to cold path (>500µs)

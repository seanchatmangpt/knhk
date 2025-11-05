//! Warm path query examples
//! Demonstrates SELECT, ASK queries, RDF loading, and cache performance

use knhk_warm::{WarmPathGraph, execute_select, execute_ask, execute_construct, execute_describe};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Create a warm path graph
    println!("=== Example 1: Creating Warm Path Graph ===");
    let graph = WarmPathGraph::new()?;
    println!("✓ Graph created successfully\n");

    // Example 2: Load RDF data from Turtle string
    println!("=== Example 2: Loading RDF Data ===");
    let turtle_data = r#"
        @prefix ex: <http://example.org/> .
        
        ex:Alice ex:knows ex:Bob .
        ex:Bob ex:knows ex:Charlie .
        ex:Alice ex:age "30"^^<http://www.w3.org/2001/XMLSchema#integer> .
        ex:Bob ex:age "25"^^<http://www.w3.org/2001/XMLSchema#integer> .
    "#;
    
    graph.load_from_turtle(turtle_data)?;
    println!("✓ Loaded {} triples\n", graph.size());

    // Example 3: Execute SELECT query
    println!("=== Example 3: SELECT Query ===");
    let select_query = r#"
        PREFIX ex: <http://example.org/>
        SELECT ?person ?age WHERE {
            ?person ex:age ?age .
        }
    "#;
    
    let start = Instant::now();
    let select_result = execute_select(&graph, select_query)?;
    let duration = start.elapsed();
    
    println!("Query executed in {:?}", duration);
    println!("Found {} bindings:", select_result.bindings.len());
    for binding in &select_result.bindings {
        println!("  {:?}", binding);
    }
    println!();

    // Example 4: Execute ASK query
    println!("=== Example 4: ASK Query ===");
    let ask_query = r#"
        PREFIX ex: <http://example.org/>
        ASK {
            ex:Alice ex:knows ex:Bob .
        }
    "#;
    
    let start = Instant::now();
    let ask_result = execute_ask(&graph, ask_query)?;
    let duration = start.elapsed();
    
    println!("Query executed in {:?}", duration);
    println!("Result: {}\n", ask_result.result);

    // Example 5: Execute CONSTRUCT query
    println!("=== Example 5: CONSTRUCT Query ===");
    let construct_query = r#"
        PREFIX ex: <http://example.org/>
        CONSTRUCT {
            ?person ex:hasFriend ?friend .
        } WHERE {
            ?person ex:knows ?friend .
        }
    "#;
    
    let start = Instant::now();
    let construct_result = execute_construct(&graph, construct_query)?;
    let duration = start.elapsed();
    
    println!("Query executed in {:?}", duration);
    println!("Constructed {} triples:", construct_result.triples.len());
    for triple in &construct_result.triples {
        println!("  {}", triple);
    }
    println!();

    // Example 6: Cache performance demonstration
    println!("=== Example 6: Cache Performance ===");
    let test_query = r#"
        PREFIX ex: <http://example.org/>
        SELECT ?person WHERE {
            ?person ex:knows ?friend .
        }
    "#;
    
    // First execution (cache miss)
    let start = Instant::now();
    let _result1 = execute_select(&graph, test_query)?;
    let first_duration = start.elapsed();
    
    // Second execution (cache hit)
    let start = Instant::now();
    let _result2 = execute_select(&graph, test_query)?;
    let second_duration = start.elapsed();
    
    println!("First execution (cache miss): {:?}", first_duration);
    println!("Second execution (cache hit): {:?}", second_duration);
    println!("Speedup: {:.1}x", 
        first_duration.as_nanos() as f64 / second_duration.as_nanos() as f64);
    println!();

    // Example 7: Query metrics
    println!("=== Example 7: Query Metrics ===");
    let metrics = graph.get_metrics();
    println!("Total queries: {}", metrics.total_queries);
    println!("Cache hits: {}", metrics.cache_hits);
    println!("Cache misses: {}", metrics.cache_misses);
    println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate * 100.0);
    println!();

    // Example 8: Load RDF from file
    println!("=== Example 8: Loading RDF from File ===");
    // Note: This would require a file path
    // graph.load_from_file("path/to/data.ttl")?;
    println!("(File loading example - requires file path)\n");

    // Example 9: Insert triples programmatically
    println!("=== Example 9: Inserting Triples ===");
    graph.insert_triple(
        "http://example.org/Dave",
        "http://example.org/knows",
        "http://example.org/Eve"
    )?;
    println!("✓ Inserted triple: Dave knows Eve");
    println!("Graph size: {} triples\n", graph.size());

    // Example 10: Batch query execution
    println!("=== Example 10: Batch Query Execution ===");
    let queries = vec![
        "SELECT ?s WHERE { ?s <http://example.org/knows> ?o }",
        "ASK { <http://example.org/Alice> <http://example.org/knows> <http://example.org/Bob> }",
    ];
    
    let start = Instant::now();
    for query in &queries {
        if query.trim().to_uppercase().starts_with("SELECT") {
            let _result = execute_select(&graph, query)?;
        } else if query.trim().to_uppercase().starts_with("ASK") {
            let _result = execute_ask(&graph, query)?;
        }
    }
    let batch_duration = start.elapsed();
    
    println!("Executed {} queries in {:?}", queries.len(), batch_duration);
    println!("Average per query: {:?}\n", batch_duration / queries.len() as u32);

    Ok(())
}


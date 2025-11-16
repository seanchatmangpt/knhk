//! Basic GPU pattern matching example
//!
//! Run with: cargo run --release --features gpu --example gpu_basic_pattern_matching

use knhk_workflow_engine::gpu::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize GPU
    let gpu = GpuContext::new()
        .fallback_to_cpu(true)
        .build()
        .await?;

    println!("GPU Acceleration Example");
    println!("========================");
    println!("Using device: {}", gpu.device_info().name);
    println!("Device type: {:?}", gpu.device_info().device_type);
    println!(
        "Memory: {}MB available\n",
        gpu.device_info().available_memory / 1024 / 1024
    );

    // Create sample workflows
    let num_workflows = 100_000;
    let workflows: Vec<WorkflowData> = (0..num_workflows)
        .map(|i| WorkflowData {
            id: i as u64,
            state: (i % 10) as u32,
            flags: (i % 256) as u32,
            data_ptr: 0,
        })
        .collect();

    // Define patterns (all 43 YAWL patterns)
    let patterns: Vec<PatternData> = (0..43)
        .map(|i| PatternData {
            id: i,
            pattern_type: i,
            criteria: 1 << (i % 32),
        })
        .collect();

    println!(
        "Processing {} workflows × {} patterns",
        workflows.len(),
        patterns.len()
    );

    // Batch pattern matching
    let start = std::time::Instant::now();
    let matches = gpu.batch_pattern_match(&workflows, &patterns).await?;
    let elapsed = start.elapsed();

    println!("\nResults:");
    println!("  Time: {:?}", elapsed);
    println!(
        "  Throughput: {:.2} M ops/sec",
        (workflows.len() * patterns.len()) as f64 / elapsed.as_secs_f64() / 1_000_000.0
    );

    // Count matches
    let match_count = matches.iter().filter(|&&m| m).count();
    println!("  Matches: {} out of {} checks", match_count, matches.len());

    Ok(())
}

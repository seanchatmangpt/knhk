//! GPU vs CPU performance comparison
//!
//! Run with: cargo run --release --features gpu --example gpu_performance_comparison

use knhk_workflow_engine::gpu::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("GPU vs CPU Performance Comparison");
    println!("==================================\n");

    // Initialize both CPU and GPU contexts
    let cpu = GpuContext::new()
        .prefer_device(DeviceType::Cpu)
        .build()
        .await?;

    let gpu = GpuContext::new()
        .fallback_to_cpu(false) // Don't fallback to CPU
        .build()
        .await;

    // Create test data
    let num_workflows = 100_000;
    let workflows: Vec<WorkflowData> = (0..num_workflows)
        .map(|i| WorkflowData {
            id: i as u64,
            state: (i % 10) as u32,
            flags: (i % 256) as u32,
            data_ptr: 0,
        })
        .collect();

    let patterns: Vec<PatternData> = (0..43)
        .map(|i| PatternData {
            id: i,
            pattern_type: i,
            criteria: 1 << (i % 32),
        })
        .collect();

    println!("Test Configuration:");
    println!("  Workflows: {}", workflows.len());
    println!("  Patterns: {}", patterns.len());
    println!(
        "  Total operations: {}\n",
        workflows.len() * patterns.len()
    );

    // Benchmark CPU
    println!("CPU Benchmark:");
    println!("  Device: {}", cpu.device_info().name);

    let cpu_start = Instant::now();
    let cpu_results = cpu.batch_pattern_match(&workflows, &patterns).await?;
    let cpu_elapsed = cpu_start.elapsed();

    println!("  Time: {:?}", cpu_elapsed);
    println!(
        "  Throughput: {:.2} M ops/sec\n",
        (workflows.len() * patterns.len()) as f64 / cpu_elapsed.as_secs_f64() / 1_000_000.0
    );

    // Benchmark GPU (if available)
    if let Ok(gpu) = gpu {
        println!("GPU Benchmark:");
        println!("  Device: {}", gpu.device_info().name);
        println!(
            "  Compute: {}",
            gpu.device_info().compute_capability
        );

        let gpu_start = Instant::now();
        let gpu_results = gpu.batch_pattern_match(&workflows, &patterns).await?;
        let gpu_elapsed = gpu_start.elapsed();

        println!("  Time: {:?}", gpu_elapsed);
        println!(
            "  Throughput: {:.2} M ops/sec\n",
            (workflows.len() * patterns.len()) as f64 / gpu_elapsed.as_secs_f64() / 1_000_000.0
        );

        let speedup = cpu_elapsed.as_secs_f64() / gpu_elapsed.as_secs_f64();
        println!("Performance Summary:");
        println!("  Speedup: {:.1}x faster 🚀", speedup);
        println!(
            "  Time saved: {:?}",
            cpu_elapsed.saturating_sub(gpu_elapsed)
        );

        // Verify results match
        assert_eq!(cpu_results.len(), gpu_results.len());
        let matches = cpu_results
            .iter()
            .zip(gpu_results.iter())
            .all(|(c, g)| c == g);
        println!("  Results match: {}", if matches { "✓" } else { "✗" });

        if speedup >= 100.0 {
            println!("\n🎉 Excellent GPU performance! (100x+ speedup)");
        } else if speedup >= 10.0 {
            println!("\n✨ Good GPU performance! (10x+ speedup)");
        } else if speedup >= 2.0 {
            println!("\n👍 Moderate GPU performance (2x+ speedup)");
        } else {
            println!("\n💡 Consider using CPU for this workload size");
        }
    } else {
        println!("GPU not available - using CPU only");
    }

    Ok(())
}

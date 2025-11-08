// w1_pipeline_perf.rs: Benchmark W1 pipeline with Linux perf validation
// Measures cycles/byte, IPC, branch-miss, L1D miss for W1 JSON parsing

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use knhk_hot::w1_pipeline::{
    stage1_structural_index, ShapeCard, SoAPacker, StructuralIndex, TapeBuilder,
};

#[cfg(target_os = "linux")]
use knhk_hot::bench::perf::{benchmark_with_perf, PerfEventManager};

fn bench_w1_stage1_perf(c: &mut Criterion) {
    let json = br#"{"account_id":"12345","transaction_id":"tx-001","amount":100.50,"currency":"USD","timestamp":"2024-01-01T00:00:00Z","status":"pending"}"#;

    c.bench_function("w1_stage1_structural_index", |b| {
        b.iter(|| {
            let mut index = StructuralIndex::new();
            unsafe {
                stage1_structural_index(black_box(json), &mut index);
            }
            black_box(index)
        })
    });

    #[cfg(target_os = "linux")]
    {
        // Linux perf validation
        let result = benchmark_with_perf("w1_stage1", json.len(), || {
            let mut index = StructuralIndex::new();
            unsafe {
                stage1_structural_index(json, &mut index);
            }
        });

        println!("W1 Stage 1 Performance:");
        println!("  macOS time: {} ns", result.macos_time_ns);
        println!("  macOS cycles/byte: {:.2}", result.cycles_per_byte_macos);
        if let Some(ref perf) = result.linux_perf {
            println!("  Linux cycles: {}", perf.cycles);
            println!("  Linux cycles/byte: {:.2}", perf.cycles_per_byte);
            println!("  IPC: {:.2}", perf.ipc);
            println!("  Branch miss rate: {:.4}", perf.branch_miss_rate);
            println!("  L1D miss rate: {:.4}", perf.l1d_miss_rate);
        }
    }
}

fn bench_w1_stage2_perf(c: &mut Criterion) {
    let json = br#"{"account_id":"12345","transaction_id":"tx-001","amount":100.50,"currency":"USD","timestamp":"2024-01-01T00:00:00Z","status":"pending"}"#;

    // Stage 1: Build structural index
    let mut index = StructuralIndex::new();
    unsafe {
        stage1_structural_index(json, &mut index);
    }

    c.bench_function("w1_stage2_build_tape", |b| {
        b.iter(|| {
            let mut builder = TapeBuilder::new();
            builder.build_tape(black_box(json), black_box(&index));
            black_box(builder)
        })
    });

    #[cfg(target_os = "linux")]
    {
        // Linux perf validation
        let result = benchmark_with_perf("w1_stage2", json.len(), || {
            let mut builder = TapeBuilder::new();
            builder.build_tape(json, &index);
        });

        println!("W1 Stage 2 Performance:");
        println!("  macOS time: {} ns", result.macos_time_ns);
        println!("  macOS cycles/byte: {:.2}", result.cycles_per_byte_macos);
        if let Some(ref perf) = result.linux_perf {
            println!("  Linux cycles: {}", perf.cycles);
            println!("  Linux cycles/byte: {:.2}", perf.cycles_per_byte);
            println!("  IPC: {:.2}", perf.ipc);
        }
    }
}

fn bench_w1_soa_packing_perf(c: &mut Criterion) {
    let json = br#"{"account_id":"12345","transaction_id":"tx-001","amount":100.50,"currency":"USD","timestamp":"2024-01-01T00:00:00Z","status":"pending"}"#;

    // Build tape first
    let mut index = StructuralIndex::new();
    unsafe {
        stage1_structural_index(json, &mut index);
    }
    let mut builder = TapeBuilder::new();
    builder.build_tape(json, &index);
    let tape = builder.tape.clone();
    let shape = builder.shape.clone();
    let arena = builder.arena.clone();

    c.bench_function("w1_soa_packing", |b| {
        b.iter(|| {
            let mut packer = SoAPacker::new();
            packer.pack_from_tape(black_box(&tape), black_box(&shape), black_box(&arena));
            black_box(packer)
        })
    });

    #[cfg(target_os = "linux")]
    {
        // Linux perf validation
        let result = benchmark_with_perf("w1_soa_packing", json.len(), || {
            let mut packer = SoAPacker::new();
            packer.pack_from_tape(&tape, &shape, &arena);
        });

        println!("W1 SoA Packing Performance:");
        println!("  macOS time: {} ns", result.macos_time_ns);
        println!("  macOS cycles/byte: {:.2}", result.cycles_per_byte_macos);
        if let Some(ref perf) = result.linux_perf {
            println!("  Linux cycles: {}", perf.cycles);
            println!("  Linux cycles/byte: {:.2}", perf.cycles_per_byte);
            println!("  IPC: {:.2}", perf.ipc);
        }
    }
}

criterion_group!(
    benches,
    bench_w1_stage1_perf,
    bench_w1_stage2_perf,
    bench_w1_soa_packing_perf
);
criterion_main!(benches);

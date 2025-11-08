// rust/knhk-hot/benches/simd_predicates.rs
// Benchmark SIMD predicate matching optimization (Week 2)
// Target: ≥4x speedup over scalar implementation
//
// Measures:
// - Scalar vs SIMD throughput
// - Vectorization efficiency
// - Cache behavior
// - Batch size impact

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

/// Scalar predicate matching (baseline)
fn match_predicates_scalar(predicates: &[u64], target: u64) -> Option<usize> {
    for (idx, &pred) in predicates.iter().enumerate() {
        if pred == target {
            return Some(idx);
        }
    }
    None
}

/// SIMD predicate matching (AVX2 - 4 predicates at once)
#[cfg(target_arch = "x86_64")]
fn match_predicates_simd_avx2(predicates: &[u64], target: u64) -> Option<usize> {
    #[cfg(target_feature = "avx2")]
    unsafe {
        use std::arch::x86_64::*;

        let target_vec = _mm256_set1_epi64x(target as i64);

        for (chunk_idx, chunk) in predicates.chunks_exact(4).enumerate() {
            if chunk.len() < 4 {
                break;
            }

            let pred_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let cmp = _mm256_cmpeq_epi64(pred_vec, target_vec);
            let mask = _mm256_movemask_pd(_mm256_castsi256_pd(cmp));

            if mask != 0 {
                let idx = mask.trailing_zeros() as usize;
                return Some(chunk_idx * 4 + idx);
            }
        }

        // Handle remaining elements
        let processed = (predicates.len() / 4) * 4;
        for (idx, &pred) in predicates[processed..].iter().enumerate() {
            if pred == target {
                return Some(processed + idx);
            }
        }

        None
    }

    #[cfg(not(target_feature = "avx2"))]
    {
        match_predicates_scalar(predicates, target)
    }
}

/// SIMD predicate matching (SSE2 - 2 predicates at once, more portable)
#[cfg(target_arch = "x86_64")]
fn match_predicates_simd_sse2(predicates: &[u64], target: u64) -> Option<usize> {
    #[cfg(target_feature = "sse2")]
    unsafe {
        use std::arch::x86_64::*;

        let target_vec = _mm_set1_epi64x(target as i64);

        for (chunk_idx, chunk) in predicates.chunks_exact(2).enumerate() {
            if chunk.len() < 2 {
                break;
            }

            let pred_vec = _mm_loadu_si128(chunk.as_ptr() as *const __m128i);
            let cmp = _mm_cmpeq_epi64(pred_vec, target_vec);
            let mask = _mm_movemask_pd(_mm_castsi128_pd(cmp));

            if mask != 0 {
                let idx = mask.trailing_zeros() as usize;
                return Some(chunk_idx * 2 + idx);
            }
        }

        // Handle remaining elements
        let processed = (predicates.len() / 2) * 2;
        for (idx, &pred) in predicates[processed..].iter().enumerate() {
            if pred == target {
                return Some(processed + idx);
            }
        }

        None
    }

    #[cfg(not(target_feature = "sse2"))]
    {
        match_predicates_scalar(predicates, target)
    }
}

/// Benchmark scalar predicate matching
fn bench_scalar_predicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_predicates/scalar");

    for size in [4, 8, 16, 32, 64].iter() {
        let predicates: Vec<u64> = (1..=*size).collect();
        let target = *size / 2; // Middle element

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(predicates, target),
            |b, (preds, tgt)| {
                b.iter(|| {
                    let result = match_predicates_scalar(black_box(preds), black_box(*tgt));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark SIMD predicate matching (AVX2)
#[cfg(target_arch = "x86_64")]
fn bench_simd_avx2_predicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_predicates/avx2");

    for size in [4, 8, 16, 32, 64].iter() {
        let predicates: Vec<u64> = (1..=*size).collect();
        let target = *size / 2;

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(predicates, target),
            |b, (preds, tgt)| {
                b.iter(|| {
                    let result = match_predicates_simd_avx2(black_box(preds), black_box(*tgt));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark SIMD predicate matching (SSE2)
#[cfg(target_arch = "x86_64")]
fn bench_simd_sse2_predicates(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_predicates/sse2");

    for size in [4, 8, 16, 32, 64].iter() {
        let predicates: Vec<u64> = (1..=*size).collect();
        let target = *size / 2;

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &(predicates, target),
            |b, (preds, tgt)| {
                b.iter(|| {
                    let result = match_predicates_simd_sse2(black_box(preds), black_box(*tgt));
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark worst-case scenario (not found)
fn bench_predicate_not_found(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_predicates/not_found");

    let predicates: Vec<u64> = (1..=64).collect();
    let target = 999u64; // Not in list

    group.bench_function("scalar", |b| {
        b.iter(|| {
            let result = match_predicates_scalar(black_box(&predicates), black_box(target));
            black_box(result)
        });
    });

    #[cfg(target_arch = "x86_64")]
    group.bench_function("avx2", |b| {
        b.iter(|| {
            let result = match_predicates_simd_avx2(black_box(&predicates), black_box(target));
            black_box(result)
        });
    });

    #[cfg(target_arch = "x86_64")]
    group.bench_function("sse2", |b| {
        b.iter(|| {
            let result = match_predicates_simd_sse2(black_box(&predicates), black_box(target));
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark best-case scenario (first element)
fn bench_predicate_first_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_predicates/first_match");

    let predicates: Vec<u64> = (1..=64).collect();
    let target = 1u64; // First element

    group.bench_function("scalar", |b| {
        b.iter(|| {
            let result = match_predicates_scalar(black_box(&predicates), black_box(target));
            black_box(result)
        });
    });

    #[cfg(target_arch = "x86_64")]
    group.bench_function("avx2", |b| {
        b.iter(|| {
            let result = match_predicates_simd_avx2(black_box(&predicates), black_box(target));
            black_box(result)
        });
    });

    #[cfg(target_arch = "x86_64")]
    group.bench_function("sse2", |b| {
        b.iter(|| {
            let result = match_predicates_simd_sse2(black_box(&predicates), black_box(target));
            black_box(result)
        });
    });

    group.finish();
}

/// Validate 4x speedup target for Week 2
fn validate_simd_speedup() {
    println!("\n{}", "=".repeat(80));
    println!("WEEK 2 TARGET VALIDATION: ≥4x SIMD Speedup");
    println!("{}", "=".repeat(80));

    let predicates: Vec<u64> = (1..=64).collect();
    let iterations = 1_000_000;

    // Scalar baseline
    let mut scalar_hits = 0;
    let start = std::time::Instant::now();
    for i in 0..iterations {
        let target = ((i % 64) + 1) as u64;
        if let Some(_) = match_predicates_scalar(&predicates, target) {
            scalar_hits += 1;
        }
    }
    let scalar_time = start.elapsed();

    println!("\nScalar Implementation:");
    println!("  Time: {:?}", scalar_time);
    println!("  Hits: {}", scalar_hits);
    println!(
        "  Ops/sec: {:.0}",
        iterations as f64 / scalar_time.as_secs_f64()
    );

    #[cfg(target_arch = "x86_64")]
    {
        // SIMD AVX2
        let mut simd_hits = 0;
        let start = std::time::Instant::now();
        for i in 0..iterations {
            let target = ((i % 64) + 1) as u64;
            if let Some(_) = match_predicates_simd_avx2(&predicates, target) {
                simd_hits += 1;
            }
        }
        let simd_time = start.elapsed();

        println!("\nSIMD AVX2 Implementation:");
        println!("  Time: {:?}", simd_time);
        println!("  Hits: {}", simd_hits);
        println!(
            "  Ops/sec: {:.0}",
            iterations as f64 / simd_time.as_secs_f64()
        );

        let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
        println!("\nSpeedup: {:.2}x", speedup);

        if speedup >= 4.0 {
            println!("✅ WEEK 2 TARGET MET: {:.2}x speedup ≥ 4x", speedup);
        } else {
            println!("❌ WEEK 2 TARGET NOT MET: {:.2}x speedup < 4x", speedup);
        }
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        println!("\n⚠️  SIMD benchmarks only available on x86_64 architecture");
    }

    println!("{}", "=".repeat(80));
}

#[cfg(target_arch = "x86_64")]
criterion_group!(
    benches,
    bench_scalar_predicates,
    bench_simd_avx2_predicates,
    bench_simd_sse2_predicates,
    bench_predicate_not_found,
    bench_predicate_first_match
);

#[cfg(not(target_arch = "x86_64"))]
criterion_group!(
    benches,
    bench_scalar_predicates,
    bench_predicate_not_found,
    bench_predicate_first_match
);

criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_predicate_match() {
        let predicates = vec![1, 2, 3, 4, 5];
        assert_eq!(match_predicates_scalar(&predicates, 3), Some(2));
        assert_eq!(match_predicates_scalar(&predicates, 999), None);
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_simd_avx2_predicate_match() {
        let predicates = vec![1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(match_predicates_simd_avx2(&predicates, 5), Some(4));
        assert_eq!(match_predicates_simd_avx2(&predicates, 999), None);
    }

    #[test]
    fn run_speedup_validation() {
        validate_simd_speedup();
    }
}

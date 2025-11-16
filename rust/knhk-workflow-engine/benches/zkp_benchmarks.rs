//! Performance benchmarks for zero-knowledge proof systems

#[cfg(feature = "zkp")]
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

#[cfg(feature = "zkp")]
use knhk_workflow_engine::zkp::*;

#[cfg(feature = "zkp")]
fn bench_proof_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("proof_generation");

    // Prepare inputs
    let private_inputs = PrivateInputs::new()
        .add("current_state", vec![1u8; 32])
        .add("input_data", vec![2u8; 32])
        .add("transition_type", vec![0]);

    let public_inputs = PublicInputs::new()
        .add("workflow_id", b"benchmark_workflow".to_vec());

    let runtime = tokio::runtime::Runtime::new().unwrap();

    for system in [ProofSystem::Groth16, ProofSystem::Plonk, ProofSystem::Stark] {
        group.bench_with_input(
            BenchmarkId::new("prove", format!("{:?}", system)),
            &system,
            |b, &proof_system| {
                b.iter(|| {
                    runtime.block_on(async {
                        let config = ProverConfig {
                            security_level: 128,
                            enable_telemetry: false,
                            parallel_proving: false,
                        };

                        let prover = ZkProver::new(proof_system)
                            .with_circuit("state_transition")
                            .with_config(config)
                            .build();

                        // Note: Will fail without actual circuit implementation
                        // But measures the overhead of the API
                        let _ = black_box(prover);
                    });
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "zkp")]
fn bench_privacy_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_operations");

    let data = vec![1u8; 1024]; // 1KB of data
    let salt = b"benchmark_salt";
    let key = b"benchmark_key";

    group.bench_function("anonymize_data", |b| {
        b.iter(|| {
            black_box(privacy::anonymize_data(&data, salt));
        });
    });

    group.bench_function("pseudonymize_data", |b| {
        b.iter(|| {
            black_box(privacy::pseudonymize_data(&data, key));
        });
    });

    group.bench_function("add_laplace_noise", |b| {
        b.iter(|| {
            black_box(privacy::add_laplace_noise(100.0, 1.0, 1.0));
        });
    });

    group.bench_function("add_gaussian_noise", |b| {
        b.iter(|| {
            black_box(privacy::add_gaussian_noise(100.0, 1.0));
        });
    });

    group.finish();
}

#[cfg(feature = "zkp")]
fn bench_homomorphic_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("homomorphic_encryption");

    let he = privacy::HomomorphicEncryption::generate_keys().unwrap();
    let plaintext = 42u64;
    let ciphertext = he.encrypt(plaintext);

    group.bench_function("generate_keys", |b| {
        b.iter(|| {
            black_box(privacy::HomomorphicEncryption::generate_keys());
        });
    });

    group.bench_function("encrypt", |b| {
        b.iter(|| {
            black_box(he.encrypt(plaintext));
        });
    });

    group.bench_function("decrypt", |b| {
        b.iter(|| {
            black_box(he.decrypt(&ciphertext));
        });
    });

    let c1 = he.encrypt(10);
    let c2 = he.encrypt(20);

    group.bench_function("homomorphic_add", |b| {
        b.iter(|| {
            black_box(privacy::HomomorphicEncryption::add(&c1, &c2));
        });
    });

    group.bench_function("homomorphic_multiply_plain", |b| {
        b.iter(|| {
            black_box(privacy::HomomorphicEncryption::multiply_plain(&c1, 5));
        });
    });

    group.finish();
}

#[cfg(feature = "zkp")]
fn bench_k_anonymity(c: &mut Criterion) {
    let mut group = c.benchmark_group("k_anonymity");

    group.bench_function("add_record", |b| {
        b.iter(|| {
            let mut k_anon = privacy::KAnonymity::new(5);
            for i in 0..100 {
                k_anon.add_record(vec![i % 10], vec![i]);
            }
            black_box(k_anon);
        });
    });

    group.bench_function("is_k_anonymous", |b| {
        let mut k_anon = privacy::KAnonymity::new(5);
        for i in 0..100 {
            k_anon.add_record(vec![i % 10], vec![i]);
        }

        b.iter(|| {
            black_box(k_anon.is_k_anonymous());
        });
    });

    group.finish();
}

#[cfg(feature = "zkp")]
fn bench_hash_operations(c: &mut Criterion) {
    use sha3::{Sha3_256, Digest};

    let mut group = c.benchmark_group("hash_operations");

    let data = vec![1u8; 1024]; // 1KB

    group.bench_function("sha3_256_1kb", |b| {
        b.iter(|| {
            let mut hasher = Sha3_256::new();
            hasher.update(&data);
            black_box(hasher.finalize());
        });
    });

    let large_data = vec![1u8; 1024 * 1024]; // 1MB

    group.bench_function("sha3_256_1mb", |b| {
        b.iter(|| {
            let mut hasher = Sha3_256::new();
            hasher.update(&large_data);
            black_box(hasher.finalize());
        });
    });

    group.finish();
}

#[cfg(feature = "zkp")]
criterion_group!(
    benches,
    bench_proof_generation,
    bench_privacy_operations,
    bench_homomorphic_encryption,
    bench_k_anonymity,
    bench_hash_operations,
);

#[cfg(feature = "zkp")]
criterion_main!(benches);

#[cfg(not(feature = "zkp"))]
fn main() {
    println!("ZKP benchmarks require the 'zkp' feature");
}

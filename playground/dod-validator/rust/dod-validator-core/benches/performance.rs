//! Performance benchmarks for DoD Validator
//!
//! Validates hot path performance claims (≤8 ticks = ≤2ns)

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use dod_validator_core::ValidationEngine;
use dod_validator_hot::{HotPathValidator, DodPattern};
use std::path::PathBuf;

fn bench_hot_path_pattern_matching(c: &mut Criterion) {
    let validator = HotPathValidator::new();
    let patterns = vec![0x556E7772617050u64; 8]; // Hash of ".unwrap()"
    let code_hash = 0x556E7772617050u64;
    
    c.bench_function("hot_path_match_pattern", |b| {
        b.iter(|| {
            let _ = black_box(validator.match_pattern(
                black_box(&patterns),
                black_box(DodPattern::Unwrap),
                black_box(code_hash),
            ));
        });
    });
}

fn bench_pattern_extraction(c: &mut Criterion) {
    use dod_validator_core::pattern_extractor::PatternExtractor;
    
    let extractor = PatternExtractor::new();
    let test_code = r#"
fn main() {
    let x: Option<i32> = Some(42);
    let value = x.unwrap();
    let result = x.expect("error");
    // TODO: Add error handling
}
"#;
    
    let temp_file = std::env::temp_dir().join("bench_test.rs");
    std::fs::write(&temp_file, test_code).unwrap();
    
    c.bench_function("pattern_extraction", |b| {
        b.iter(|| {
            let _ = black_box(extractor.extract_from_file(black_box(&temp_file)));
        });
    });
    
    std::fs::remove_file(&temp_file).ok();
}

fn bench_full_validation_single_file(c: &mut Criterion) {
    let mut engine = ValidationEngine::new().unwrap();
    let test_code = r#"
fn main() {
    let x: Option<i32> = Some(42);
    let value = x.unwrap();
}
"#;
    
    let temp_file = std::env::temp_dir().join("bench_validation.rs");
    std::fs::write(&temp_file, test_code).unwrap();
    
    c.bench_function("full_validation_single_file", |b| {
        b.iter(|| {
            let _ = black_box(engine.validate_all(black_box(&temp_file)));
        });
    });
    
    std::fs::remove_file(&temp_file).ok();
}

fn bench_code_context_extraction(c: &mut Criterion) {
    let mut engine = ValidationEngine::new().unwrap();
    let test_code = "fn test() {\n    let x = Some(42);\n    let v = x.unwrap();\n}\n";
    
    let temp_file = std::env::temp_dir().join("bench_context.rs");
    std::fs::write(&temp_file, test_code).unwrap();
    
    c.bench_function("code_context_extraction", |b| {
        b.iter(|| {
            let _ = black_box(engine.validate_all(black_box(&temp_file)));
        });
    });
    
    std::fs::remove_file(&temp_file).ok();
}

criterion_group!(
    benches,
    bench_hot_path_pattern_matching,
    bench_pattern_extraction,
    bench_full_validation_single_file,
    bench_code_context_extraction
);
criterion_main!(benches);


///! Pattern Library Benchmarks
//! Measures performance of pattern matching and library operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use std::time::Duration;

/// Pattern representation
#[derive(Debug, Clone)]
struct Pattern {
    id: String,
    template: String,
    variables: Vec<String>,
    guards: Vec<Guard>,
}

#[derive(Debug, Clone)]
struct Guard {
    condition: String,
    threshold: f64,
}

/// Pattern library
struct PatternLibrary {
    patterns: HashMap<String, Pattern>,
    index: HashMap<String, Vec<String>>, // tag -> pattern IDs
}

impl PatternLibrary {
    fn new() -> Self {
        Self {
            patterns: HashMap::new(),
            index: HashMap::new(),
        }
    }

    fn add_pattern(&mut self, pattern: Pattern, tags: Vec<String>) {
        let pattern_id = pattern.id.clone();
        self.patterns.insert(pattern_id.clone(), pattern);

        for tag in tags {
            self.index
                .entry(tag)
                .or_insert_with(Vec::new)
                .push(pattern_id.clone());
        }
    }

    fn find_by_tag(&self, tag: &str) -> Vec<&Pattern> {
        self.index
            .get(tag)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.patterns.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    fn match_pattern(&self, pattern_id: &str, data: &HashMap<String, f64>) -> bool {
        if let Some(pattern) = self.patterns.get(pattern_id) {
            // Check all guards
            pattern.guards.iter().all(|guard| {
                data.get(&guard.condition)
                    .map(|value| *value >= guard.threshold)
                    .unwrap_or(false)
            })
        } else {
            false
        }
    }
}

fn create_test_library(size: usize) -> PatternLibrary {
    let mut library = PatternLibrary::new();

    for i in 0..size {
        let pattern = Pattern {
            id: format!("pattern_{}", i),
            template: format!("template_{}", i),
            variables: vec![format!("var_{}", i)],
            guards: vec![
                Guard {
                    condition: "latency".to_string(),
                    threshold: (i as f64) * 0.1,
                },
                Guard {
                    condition: "throughput".to_string(),
                    threshold: (i as f64) * 100.0,
                },
            ],
        };

        let tags = vec![
            format!("tag_{}", i % 10),
            format!("category_{}", i % 5),
        ];

        library.add_pattern(pattern, tags);
    }

    library
}

fn benchmark_pattern_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_lookup");

    for size in [10, 100, 1000].iter() {
        let library = create_test_library(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    black_box(library.find_by_tag("tag_5"))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_pattern_matching(c: &mut Criterion) {
    let library = create_test_library(100);
    let mut data = HashMap::new();
    data.insert("latency".to_string(), 5.0);
    data.insert("throughput".to_string(), 5000.0);

    c.bench_function("pattern_match", |b| {
        b.iter(|| {
            black_box(library.match_pattern("pattern_50", &data))
        });
    });
}

fn benchmark_pattern_add(c: &mut Criterion) {
    c.bench_function("pattern_add", |b| {
        b.iter(|| {
            let mut library = PatternLibrary::new();
            let pattern = Pattern {
                id: "test_pattern".to_string(),
                template: "test_template".to_string(),
                variables: vec!["var1".to_string()],
                guards: vec![Guard {
                    condition: "latency".to_string(),
                    threshold: 10.0,
                }],
            };
            library.add_pattern(pattern, vec!["tag1".to_string()]);
            black_box(library)
        });
    });
}

fn benchmark_pattern_batch_matching(c: &mut Criterion) {
    let library = create_test_library(100);
    let mut data = HashMap::new();
    data.insert("latency".to_string(), 5.0);
    data.insert("throughput".to_string(), 5000.0);

    c.bench_function("pattern_batch_match", |b| {
        b.iter(|| {
            let pattern_ids: Vec<String> = (0..10).map(|i| format!("pattern_{}", i * 10)).collect();
            let results: Vec<bool> = pattern_ids
                .iter()
                .map(|id| library.match_pattern(id, &data))
                .collect();
            black_box(results)
        });
    });
}

fn benchmark_pattern_index_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_index_scan");

    for size in [10, 100, 1000].iter() {
        let library = create_test_library(*size);

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, _| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for i in 0..10 {
                        let tag = format!("tag_{}", i);
                        results.extend(library.find_by_tag(&tag));
                    }
                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

criterion_group! {
    name = pattern_benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(500);
    targets = benchmark_pattern_lookup,
              benchmark_pattern_matching,
              benchmark_pattern_add,
              benchmark_pattern_batch_matching,
              benchmark_pattern_index_scan
}

criterion_main!(pattern_benches);

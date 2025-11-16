//! Compilation Benchmarks
//!
//! Measures ontology parsing, pattern validation, code generation,
//! and descriptor optimization performance.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;
use std::collections::{HashMap, HashSet};

/// Ontology parsing benchmark
pub struct OntologyParser {
    triples: Vec<Triple>,
    prefixes: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Triple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
}

impl OntologyParser {
    pub fn new() -> Self {
        let prefixes = [
            ("yawl", "http://example.org/yawl#"),
            ("rdf", "http://www.w3.org/1999/02/22-rdf-syntax-ns#"),
            ("rdfs", "http://www.w3.org/2000/01/rdf-schema#"),
            ("owl", "http://www.w3.org/2002/07/owl#"),
        ].iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Self {
            triples: Vec::new(),
            prefixes,
        }
    }

    pub fn parse_turtle(&self, turtle: &str) -> Vec<Triple> {
        let mut triples = Vec::new();

        // Simplified Turtle parsing
        for line in turtle.lines() {
            if line.starts_with('@prefix') {
                continue;
            }
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse simple triple pattern: subject predicate object .
            let parts: Vec<&str> = line.trim_end_matches('.').split_whitespace().collect();
            if parts.len() >= 3 {
                triples.push(Triple {
                    subject: self.expand_uri(parts[0]),
                    predicate: self.expand_uri(parts[1]),
                    object: parts[2..].join(" "),
                });
            }
        }

        triples
    }

    fn expand_uri(&self, uri: &str) -> String {
        if let Some(colon_pos) = uri.find(':') {
            let prefix = &uri[..colon_pos];
            if let Some(namespace) = self.prefixes.get(prefix) {
                return format!("{}{}", namespace, &uri[colon_pos + 1..]);
            }
        }
        uri.to_string()
    }
}

/// Pattern validator benchmark
pub struct PatternValidator {
    valid_patterns: HashSet<String>,
    permutation_matrix: Vec<Vec<bool>>,
}

impl PatternValidator {
    pub fn new() -> Self {
        let mut valid_patterns = HashSet::new();
        for i in 0..256 {
            valid_patterns.insert(format!("pattern_{}", i));
        }

        let permutation_matrix = vec![vec![false; 256]; 256];

        Self {
            valid_patterns,
            permutation_matrix,
        }
    }

    pub fn validate_pattern(&self, pattern: &str) -> ValidationResult {
        let start = std::time::Instant::now();

        // Check if pattern exists
        if !self.valid_patterns.contains(pattern) {
            return ValidationResult {
                valid: false,
                errors: vec!["Pattern not found".to_string()],
                validation_time: start.elapsed(),
            };
        }

        // Validate pattern structure
        let errors = self.validate_structure(pattern);

        ValidationResult {
            valid: errors.is_empty(),
            errors,
            validation_time: start.elapsed(),
        }
    }

    fn validate_structure(&self, pattern: &str) -> Vec<String> {
        let mut errors = Vec::new();

        // Simplified validation rules
        if !pattern.starts_with("pattern_") {
            errors.push("Pattern must start with 'pattern_'".to_string());
        }

        if pattern.len() > 100 {
            errors.push("Pattern name too long".to_string());
        }

        errors
    }

    pub fn validate_permutation(&self, from: usize, to: usize) -> bool {
        if from < self.permutation_matrix.len() && to < self.permutation_matrix[0].len() {
            // Simulate complex permutation validation
            let mut valid = true;
            for i in 0..10 {
                valid &= (from + i) % 2 == (to + i) % 2;
            }
            valid
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub validation_time: Duration,
}

/// Code generator benchmark
pub struct CodeGenerator {
    templates: HashMap<String, CodeTemplate>,
    optimization_level: OptimizationLevel,
}

#[derive(Debug, Clone)]
pub struct CodeTemplate {
    pub name: String,
    pub template: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum OptimizationLevel {
    None,
    Basic,
    Aggressive,
}

impl CodeGenerator {
    pub fn new() -> Self {
        let mut templates = HashMap::new();

        templates.insert("pattern_dispatch".to_string(), CodeTemplate {
            name: "pattern_dispatch".to_string(),
            template: "match pattern_id {{ {} }}".to_string(),
            parameters: vec!["pattern_id".to_string()],
        });

        templates.insert("guard_evaluation".to_string(), CodeTemplate {
            name: "guard_evaluation".to_string(),
            template: "if {} {{ {} }}".to_string(),
            parameters: vec!["condition".to_string(), "body".to_string()],
        });

        Self {
            templates,
            optimization_level: OptimizationLevel::Basic,
        }
    }

    pub fn generate_code(&self, pattern_count: usize) -> GeneratedCode {
        let mut code = String::new();
        let start = std::time::Instant::now();

        // Generate pattern matching code
        code.push_str("pub fn dispatch_pattern(pattern_id: u64) -> Result<(), Error> {\n");
        code.push_str("    match pattern_id {\n");

        for i in 0..pattern_count {
            code.push_str(&format!("        {} => execute_pattern_{}(),\n", i, i));
        }

        code.push_str("        _ => Err(Error::InvalidPattern)\n");
        code.push_str("    }\n");
        code.push_str("}\n\n");

        // Generate pattern implementations
        for i in 0..pattern_count {
            code.push_str(&format!("fn execute_pattern_{}() -> Result<(), Error> {{\n", i));
            code.push_str("    // Pattern implementation\n");
            code.push_str("    Ok(())\n");
            code.push_str("}\n\n");
        }

        let code_size = code.len();
        let generation_time = start.elapsed();

        // Apply optimizations
        let optimized_code = self.optimize_code(code);

        GeneratedCode {
            code: optimized_code,
            size_bytes: code_size,
            generation_time,
            pattern_count,
        }
    }

    fn optimize_code(&self, code: String) -> String {
        match self.optimization_level {
            OptimizationLevel::None => code,
            OptimizationLevel::Basic => {
                // Basic optimizations
                code.replace("    ", "  ") // Reduce indentation
            }
            OptimizationLevel::Aggressive => {
                // Aggressive optimizations
                code.replace("    ", " ")
                    .replace("\n\n", "\n")
            }
        }
    }
}

#[derive(Debug)]
pub struct GeneratedCode {
    pub code: String,
    pub size_bytes: usize,
    pub generation_time: Duration,
    pub pattern_count: usize,
}

/// Descriptor compiler benchmark
pub struct DescriptorCompiler {
    compression_enabled: bool,
    signature_enabled: bool,
}

impl DescriptorCompiler {
    pub fn new() -> Self {
        Self {
            compression_enabled: true,
            signature_enabled: true,
        }
    }

    pub fn compile_descriptor(&self, patterns: &[String]) -> CompiledDescriptor {
        let start = std::time::Instant::now();

        let mut descriptor = Vec::new();

        // Header
        descriptor.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // Magic bytes
        descriptor.extend_from_slice(&(patterns.len() as u32).to_le_bytes());

        // Pattern table
        for pattern in patterns {
            let pattern_bytes = pattern.as_bytes();
            descriptor.extend_from_slice(&(pattern_bytes.len() as u32).to_le_bytes());
            descriptor.extend_from_slice(pattern_bytes);
        }

        // Compress if enabled
        let compressed = if self.compression_enabled {
            self.compress(&descriptor)
        } else {
            descriptor.clone()
        };

        // Sign if enabled
        let signature = if self.signature_enabled {
            self.sign(&compressed)
        } else {
            vec![0; 64]
        };

        CompiledDescriptor {
            data: compressed,
            signature,
            original_size: descriptor.len(),
            compressed_size: compressed.len(),
            compilation_time: start.elapsed(),
        }
    }

    fn compress(&self, data: &[u8]) -> Vec<u8> {
        // Simulate compression (simple RLE)
        let mut compressed = Vec::new();
        compressed.push(0xC0); // Compression marker

        let mut i = 0;
        while i < data.len() {
            let byte = data[i];
            let mut count = 1;

            while i + count < data.len() && data[i + count] == byte && count < 255 {
                count += 1;
            }

            compressed.push(count as u8);
            compressed.push(byte);
            i += count;
        }

        compressed
    }

    fn sign(&self, data: &[u8]) -> Vec<u8> {
        // Simulate signature generation
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        let hash = hasher.finish();

        let mut signature = vec![0u8; 64];
        signature[0..8].copy_from_slice(&hash.to_le_bytes());

        signature
    }
}

#[derive(Debug)]
pub struct CompiledDescriptor {
    pub data: Vec<u8>,
    pub signature: Vec<u8>,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compilation_time: Duration,
}

/// Benchmark ontology parsing
fn bench_ontology_parsing(c: &mut Criterion) {
    let parser = OntologyParser::new();

    let small_ontology = generate_turtle_ontology(10);
    let medium_ontology = generate_turtle_ontology(100);
    let large_ontology = generate_turtle_ontology(1000);

    let mut group = c.benchmark_group("ontology_parsing");

    group.throughput(Throughput::Bytes(small_ontology.len() as u64));
    group.bench_function("small", |b| {
        b.iter(|| {
            let triples = parser.parse_turtle(black_box(&small_ontology));
            black_box(triples.len())
        });
    });

    group.throughput(Throughput::Bytes(medium_ontology.len() as u64));
    group.bench_function("medium", |b| {
        b.iter(|| {
            let triples = parser.parse_turtle(black_box(&medium_ontology));
            black_box(triples.len())
        });
    });

    group.throughput(Throughput::Bytes(large_ontology.len() as u64));
    group.bench_function("large", |b| {
        b.iter(|| {
            let triples = parser.parse_turtle(black_box(&large_ontology));
            black_box(triples.len())
        });
    });

    group.finish();
}

fn generate_turtle_ontology(triple_count: usize) -> String {
    let mut ontology = String::new();

    ontology.push_str("@prefix yawl: <http://example.org/yawl#> .\n");
    ontology.push_str("@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n\n");

    for i in 0..triple_count {
        ontology.push_str(&format!("yawl:pattern_{} rdf:type yawl:Pattern .\n", i));
    }

    ontology
}

/// Benchmark pattern validation
fn bench_pattern_validation(c: &mut Criterion) {
    let validator = PatternValidator::new();

    c.bench_function("single_pattern_validation", |b| {
        b.iter(|| {
            let result = validator.validate_pattern(black_box("pattern_42"));
            black_box(result.valid)
        });
    });

    c.bench_function("permutation_validation", |b| {
        b.iter(|| {
            let valid = validator.validate_permutation(black_box(10), black_box(20));
            black_box(valid)
        });
    });

    // Batch validation
    let mut group = c.benchmark_group("batch_validation");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut valid_count = 0;
                for i in 0..size {
                    if validator.validate_pattern(&format!("pattern_{}", i)).valid {
                        valid_count += 1;
                    }
                }
                black_box(valid_count)
            });
        });
    }

    group.finish();
}

/// Benchmark code generation
fn bench_code_generation(c: &mut Criterion) {
    let generator = CodeGenerator::new();

    let mut group = c.benchmark_group("code_generation");

    for pattern_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("patterns", pattern_count),
            pattern_count,
            |b, &count| {
                b.iter(|| {
                    let generated = generator.generate_code(count);
                    black_box(generated.size_bytes)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark descriptor compilation
fn bench_descriptor_compilation(c: &mut Criterion) {
    let compiler = DescriptorCompiler::new();

    let mut group = c.benchmark_group("descriptor_compilation");

    for size in [10, 100, 1000].iter() {
        let patterns: Vec<String> = (0..*size)
            .map(|i| format!("pattern_{}_implementation", i))
            .collect();

        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &patterns, |b, patterns| {
            b.iter(|| {
                let descriptor = compiler.compile_descriptor(black_box(patterns));
                black_box(descriptor.compressed_size)
            });
        });
    }

    group.finish();
}

/// Benchmark signature verification
fn bench_signature_verification(c: &mut Criterion) {
    let compiler = DescriptorCompiler::new();
    let patterns: Vec<String> = (0..100).map(|i| format!("pattern_{}", i)).collect();
    let descriptor = compiler.compile_descriptor(&patterns);

    c.bench_function("signature_verification", |b| {
        b.iter(|| {
            // Simulate signature verification
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&descriptor.data[..], &mut hasher);
            let hash = std::hash::Hasher::finish(&hasher);
            black_box(hash == u64::from_le_bytes([
                descriptor.signature[0],
                descriptor.signature[1],
                descriptor.signature[2],
                descriptor.signature[3],
                descriptor.signature[4],
                descriptor.signature[5],
                descriptor.signature[6],
                descriptor.signature[7],
            ]))
        });
    });
}

/// Benchmark compilation pipeline
fn bench_compilation_pipeline(c: &mut Criterion) {
    let parser = OntologyParser::new();
    let validator = PatternValidator::new();
    let generator = CodeGenerator::new();
    let compiler = DescriptorCompiler::new();

    c.bench_function("full_compilation_pipeline", |b| {
        b.iter(|| {
            // Parse ontology
            let ontology = generate_turtle_ontology(50);
            let triples = parser.parse_turtle(&ontology);

            // Validate patterns
            let mut valid_patterns = Vec::new();
            for i in 0..50 {
                let pattern = format!("pattern_{}", i);
                if validator.validate_pattern(&pattern).valid {
                    valid_patterns.push(pattern);
                }
            }

            // Generate code
            let code = generator.generate_code(valid_patterns.len());

            // Compile descriptor
            let descriptor = compiler.compile_descriptor(&valid_patterns);

            black_box((triples.len(), code.size_bytes, descriptor.compressed_size))
        });
    });
}

criterion_group!(
    compilation_benches,
    bench_ontology_parsing,
    bench_pattern_validation,
    bench_code_generation,
    bench_descriptor_compilation,
    bench_signature_verification,
    bench_compilation_pipeline
);

criterion_main!(compilation_benches);
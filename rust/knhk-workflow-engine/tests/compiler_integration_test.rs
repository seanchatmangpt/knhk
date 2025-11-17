//! Integration tests for Phase 4 Descriptor Compiler

use knhk_workflow_engine::compiler::{CompilationResult, CompilerConfig, DescriptorCompiler};
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

/// Create test Turtle file with valid workflow
fn create_test_turtle() -> String {
    r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

# Test Workflow
:TestWorkflow a yawl:WorkflowSpecification ;
    rdfs:label "Test Workflow" ;
    yawl:version "1.0.0" .

# Tasks
:Task1 a yawl:Task ;
    rdfs:label "Task 1" ;
    yawl:taskType yawl:AtomicTask ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" ;
    yawl:hasGuard :Guard1 .

:Task2 a yawl:Task ;
    rdfs:label "Task 2" ;
    yawl:taskType yawl:AtomicTask ;
    yawl:splitType "XOR" ;
    yawl:joinType "XOR" ;
    yawl:hasTimeout 1000 .

:Task3 a yawl:Task ;
    rdfs:label "Task 3" ;
    yawl:taskType "MultipleInstance" ;
    yawl:splitType "OR" ;
    yawl:joinType "OR" ;
    yawl:multiInstanceCount 5 .

# Guards
:Guard1 a yawl:Guard ;
    yawl:expression "x > 0 && y < 100" ;
    yawl:guardType "precondition" .

# Variables
:Var1 a yawl:Variable ;
    yawl:variableName "x" ;
    yawl:dataType "integer" ;
    yawl:initialValue "10" .

:Var2 a yawl:Variable ;
    yawl:variableName "y" ;
    yawl:dataType "float" ;
    yawl:initialValue "50.5" .

# Flows
:Flow1 a yawl:Flow ;
    yawl:source :Task1 ;
    yawl:target :Task2 .

:Flow2 a yawl:Flow ;
    yawl:source :Task2 ;
    yawl:target :Task3 .
"#
    .to_string()
}

#[tokio::test]
async fn test_compiler_creation() {
    let compiler = DescriptorCompiler::new();
    assert!(compiler.config.strict_validation);
}

#[tokio::test]
async fn test_successful_compilation() {
    // Create temp file with Turtle content
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    // Create compiler
    let mut compiler = DescriptorCompiler::new();

    // Compile
    let result = compiler.compile(file.path()).await.unwrap();

    // Verify result
    assert!(result.descriptor.len() > 0);
    assert!(result.metadata.pattern_count > 0);
    assert!(result.signature.is_some());
}

#[tokio::test]
async fn test_compilation_without_optimization() {
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let config = CompilerConfig {
        enable_optimizations: false,
        ..Default::default()
    };

    let mut compiler = DescriptorCompiler::with_config(config);
    let result = compiler.compile(file.path()).await.unwrap();

    assert_eq!(result.metadata.optimization_stats.dead_code_eliminated, 0);
    assert_eq!(result.metadata.optimization_stats.constants_folded, 0);
}

#[tokio::test]
async fn test_compilation_without_signing() {
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let config = CompilerConfig {
        enable_signing: false,
        ..Default::default()
    };

    let mut compiler = DescriptorCompiler::with_config(config);
    let result = compiler.compile(file.path()).await.unwrap();

    assert!(result.signature.is_none());
}

#[tokio::test]
async fn test_deterministic_compilation() {
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let mut compiler = DescriptorCompiler::new();

    // Compile twice
    let result1 = compiler.compile(file.path()).await.unwrap();
    let result2 = compiler.compile(file.path()).await.unwrap();

    // Should produce same descriptor hash
    assert_eq!(
        result1.metadata.descriptor_hash, result2.metadata.descriptor_hash,
        "Compilation should be deterministic"
    );
}

#[tokio::test]
async fn test_invalid_turtle() {
    let invalid_turtle = "This is not valid Turtle";
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, invalid_turtle).unwrap();

    let mut compiler = DescriptorCompiler::new();
    let result = compiler.compile(file.path()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_pattern_validation() {
    // Create Turtle with invalid pattern combination
    let turtle = r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

:TestWorkflow a yawl:WorkflowSpecification .

:InvalidTask a yawl:Task ;
    yawl:splitType "INVALID" ;
    yawl:joinType "INVALID" .
"#;

    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let config = CompilerConfig {
        strict_validation: true,
        ..Default::default()
    };

    let mut compiler = DescriptorCompiler::with_config(config);
    let result = compiler.compile(file.path()).await;

    // Should succeed but map to default pattern
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_guard_compilation() {
    let turtle = r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

:TestWorkflow a yawl:WorkflowSpecification .

:GuardedTask a yawl:Task ;
    yawl:hasGuard :ComplexGuard .

:ComplexGuard a yawl:Guard ;
    yawl:expression "(x > 5 && y < 10) || (z == true && w != null)" ;
    yawl:guardType "precondition" .
"#;

    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let mut compiler = DescriptorCompiler::new();
    let result = compiler.compile(file.path()).await.unwrap();

    assert!(result.metadata.guard_count > 0);
}

#[tokio::test]
async fn test_multiple_instance_pattern() {
    let turtle = r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

:TestWorkflow a yawl:WorkflowSpecification .

:MITask a yawl:Task ;
    yawl:taskType "MultipleInstance" ;
    yawl:multiInstanceCount 10 .
"#;

    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let mut compiler = DescriptorCompiler::new();
    let result = compiler.compile(file.path()).await.unwrap();

    assert!(result.metadata.pattern_count > 0);
}

#[tokio::test]
async fn test_compilation_metadata() {
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let mut compiler = DescriptorCompiler::new();
    let result = compiler.compile(file.path()).await.unwrap();

    // Check metadata fields
    assert!(result.metadata.timestamp > 0);
    assert!(!result.metadata.compiler_version.is_empty());
    assert!(result.metadata.pattern_count > 0);
}

#[tokio::test]
async fn test_signature_verification() {
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let mut compiler = DescriptorCompiler::new();
    let result = compiler
        .compile_with_verification(file.path())
        .await
        .unwrap();

    // Should have verified signature
    assert!(result.signature.is_some());
}

#[tokio::test]
async fn test_complex_workflow() {
    let turtle = r#"
@prefix yawl: <http://bitflow.ai/ontology/yawl/v2#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

:ComplexWorkflow a yawl:WorkflowSpecification ;
    rdfs:label "Complex Test Workflow" .

# All basic pattern types
:SequenceTask a yawl:Task ;
    yawl:splitType "AND" ;
    yawl:joinType "AND" .

:ParallelTask a yawl:Task ;
    yawl:splitType "AND" ;
    yawl:joinType "XOR" .

:ChoiceTask a yawl:Task ;
    yawl:splitType "XOR" ;
    yawl:joinType "XOR" .

:MultiChoiceTask a yawl:Task ;
    yawl:splitType "OR" ;
    yawl:joinType "OR" .

:LoopTask a yawl:Task ;
    yawl:taskType "Loop" .

:MITask a yawl:Task ;
    yawl:taskType "MultipleInstance" .

:CancelTask a yawl:Task ;
    yawl:taskType "Cancel" .
"#;

    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let mut compiler = DescriptorCompiler::new();
    let result = compiler.compile(file.path()).await.unwrap();

    assert!(result.metadata.pattern_count >= 7);
}

#[tokio::test]
async fn test_optimization_stats() {
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let config = CompilerConfig {
        enable_optimizations: true,
        ..Default::default()
    };

    let mut compiler = DescriptorCompiler::with_config(config);
    let result = compiler.compile(file.path()).await.unwrap();

    // Should have some optimization stats
    assert!(result.metadata.optimization_stats.size_reduction_percent >= 0.0);
}

#[tokio::test]
async fn test_parallel_compilation() {
    let turtle = create_test_turtle();
    let mut file = NamedTempFile::new().unwrap();
    fs::write(&file, turtle).unwrap();

    let config = CompilerConfig {
        parallel_compilation: true,
        ..Default::default()
    };

    let mut compiler = DescriptorCompiler::with_config(config);
    let result = compiler.compile(file.path()).await;

    assert!(result.is_ok());
}

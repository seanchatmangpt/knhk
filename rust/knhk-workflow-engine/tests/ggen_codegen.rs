//! Code Generation Test Suite - Chicago TDD Style
//!
//! Tests multi-language code generation with real compilation validation.
//! Focuses on generated code correctness, type safety, and actual compilation.
//!
//! Test Coverage (30+ tests):
//! - Rust code generation (structs, enums, APIs, tests)
//! - Python code generation (dataclasses, Pydantic, FastAPI)
//! - Type safety and field correctness
//! - Generated code compiles/validates in target language
//! - AAA pattern in generated tests
//! - Documentation generation

use knhk_workflow_engine::ggen::codegen::{
    create_generator, CodeGenerator, GeneratedCode, GenerationContext, PythonGenerator,
    RustGenerator,
};
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// Test Data Builders
// ============================================================================

fn create_rust_struct_context() -> GenerationContext {
    let mut context = GenerationContext::new();
    context.insert("struct_name".to_string(), "User".to_string());
    context.insert(
        "fields".to_string(),
        "pub name: String,\n    pub age: u32".to_string(),
    );
    context
}

fn create_rust_enum_context() -> GenerationContext {
    let mut context = GenerationContext::new();
    context.insert("enum_name".to_string(), "Status".to_string());
    context.insert("variants".to_string(), "Active,\n    Inactive".to_string());
    context
}

fn create_python_class_context() -> GenerationContext {
    let mut context = GenerationContext::new();
    context.insert("class_name".to_string(), "User".to_string());
    context.insert("fields".to_string(), "name: str\n    age: int".to_string());
    context
}

// ============================================================================
// Generator Creation Tests (4 tests)
// ============================================================================

#[test]
fn test_rust_generator_creates_successfully() {
    // Arrange: Create temporary template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Create Rust generator
    let result = RustGenerator::new(&template_dir);

    // Assert: Generator is created successfully
    assert!(
        result.is_ok(),
        "Rust generator should be created successfully"
    );
    let generator = result.expect("Failed to create generator");
    assert_eq!(generator.language(), "rust", "Language should be 'rust'");
    assert_eq!(
        generator.file_extension(),
        ".rs",
        "Extension should be '.rs'"
    );
}

#[test]
fn test_python_generator_creates_successfully() {
    // Arrange: Create temporary template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Create Python generator
    let result = PythonGenerator::new(&template_dir);

    // Assert: Generator is created successfully
    assert!(
        result.is_ok(),
        "Python generator should be created successfully"
    );
    let generator = result.expect("Failed to create generator");
    assert_eq!(
        generator.language(),
        "python",
        "Language should be 'python'"
    );
    assert_eq!(
        generator.file_extension(),
        ".py",
        "Extension should be '.py'"
    );
}

#[test]
fn test_create_generator_factory_creates_rust_generator() {
    // Arrange: Create temporary template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Use factory to create Rust generator
    let result = create_generator("rust", &template_dir);

    // Assert: Rust generator is created
    assert!(result.is_ok(), "Factory should create Rust generator");
    let generator = result.expect("Failed to create generator");
    assert_eq!(generator.language(), "rust");
}

#[test]
fn test_create_generator_factory_returns_error_for_unsupported_language() {
    // Arrange: Create temporary template directory
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let template_dir = temp_dir.path().join("templates");
    std::fs::create_dir_all(&template_dir).expect("Failed to create template dir");

    // Act: Request unsupported language
    let result = create_generator("unsupported_lang", &template_dir);

    // Assert: Error is returned
    assert!(result.is_err(), "Unsupported language should return error");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unsupported language"));
}

// ============================================================================
// Rust Struct Generation Tests (8 tests)
// ============================================================================

#[test]
fn test_rust_struct_generation_includes_struct_name() {
    // Arrange: Create Rust generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_struct_context();

    // Act: Generate struct
    let result = generator.generate_struct(&context);

    // Assert: Generated code contains struct name
    assert!(result.is_ok(), "Struct generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("pub struct User"),
        "Should contain struct definition"
    );
}

#[test]
fn test_rust_struct_generation_includes_all_fields() {
    // Arrange: Create Rust generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_struct_context();

    // Act: Generate struct
    let result = generator.generate_struct(&context);

    // Assert: Generated code contains all fields
    assert!(result.is_ok(), "Struct generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("pub name: String"),
        "Should contain name field"
    );
    assert!(code.contains("pub age: u32"), "Should contain age field");
}

#[test]
fn test_rust_struct_generation_includes_derive_macros() {
    // Arrange: Create Rust generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_struct_context();

    // Act: Generate struct
    let result = generator.generate_struct(&context);

    // Assert: Generated code has derive macros
    assert!(result.is_ok(), "Struct generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains(
            "#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]"
        ),
        "Should contain derive macros"
    );
}

#[test]
fn test_rust_struct_generation_includes_documentation() {
    // Arrange: Create Rust generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_struct_context();

    // Act: Generate struct
    let result = generator.generate_struct(&context);

    // Assert: Generated code has documentation
    assert!(result.is_ok(), "Struct generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("/// Generated struct:"),
        "Should contain documentation comment"
    );
}

#[test]
fn test_rust_struct_generation_fails_without_required_context() {
    // Arrange: Create Rust generator with incomplete context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = GenerationContext::new(); // Empty context

    // Act: Attempt to generate struct
    let result = generator.generate_struct(&context);

    // Assert: Generation fails with meaningful error
    assert!(
        result.is_err(),
        "Generation should fail without required context"
    );
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Missing struct_name"));
}

#[test]
fn test_rust_enum_generation_includes_enum_name() {
    // Arrange: Create Rust generator and enum context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_enum_context();

    // Act: Generate enum
    let result = generator.generate_enum(&context);

    // Assert: Generated code contains enum name
    assert!(result.is_ok(), "Enum generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("pub enum Status"),
        "Should contain enum definition"
    );
}

#[test]
fn test_rust_enum_generation_includes_all_variants() {
    // Arrange: Create Rust generator and enum context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_enum_context();

    // Act: Generate enum
    let result = generator.generate_enum(&context);

    // Assert: Generated code contains all variants
    assert!(result.is_ok(), "Enum generation should succeed");
    let code = result.expect("Generation failed");
    assert!(code.contains("Active"), "Should contain Active variant");
    assert!(code.contains("Inactive"), "Should contain Inactive variant");
}

#[test]
fn test_rust_enum_generation_includes_copy_trait() {
    // Arrange: Create Rust generator and enum context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_enum_context();

    // Act: Generate enum
    let result = generator.generate_enum(&context);

    // Assert: Generated enum has Copy trait
    assert!(result.is_ok(), "Enum generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("Copy"),
        "Enum should derive Copy for simple enums"
    );
}

// ============================================================================
// Rust API Generation Tests (5 tests)
// ============================================================================

#[test]
fn test_rust_axum_handler_generation_includes_handler_function() {
    // Arrange: Create Rust generator and API context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("handler_name".to_string(), "get_user".to_string());
    context.insert("route".to_string(), "/api/users".to_string());

    // Act: Generate API handler
    let result = generator.generate_axum_handler(&context);

    // Assert: Generated code contains handler function
    assert!(result.is_ok(), "Handler generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("async fn get_user_handler"),
        "Should contain handler function"
    );
}

#[test]
fn test_rust_axum_handler_generation_includes_axum_imports() {
    // Arrange: Create Rust generator and API context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("handler_name".to_string(), "get_user".to_string());

    // Act: Generate API handler
    let result = generator.generate_axum_handler(&context);

    // Assert: Generated code has Axum imports
    assert!(result.is_ok(), "Handler generation should succeed");
    let code = result.expect("Generation failed");
    assert!(code.contains("use axum::"), "Should contain Axum imports");
}

#[test]
fn test_rust_axum_handler_generation_includes_response_type() {
    // Arrange: Create Rust generator and API context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("handler_name".to_string(), "get_user".to_string());

    // Act: Generate API handler
    let result = generator.generate_axum_handler(&context);

    // Assert: Generated code has response type
    assert!(result.is_ok(), "Handler generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("struct Response"),
        "Should contain Response type"
    );
}

#[test]
fn test_rust_axum_handler_generation_includes_route_registration() {
    // Arrange: Create Rust generator and API context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("handler_name".to_string(), "get_user".to_string());
    context.insert("route".to_string(), "/api/users".to_string());

    // Act: Generate API handler
    let result = generator.generate_axum_handler(&context);

    // Assert: Generated code has route registration
    assert!(result.is_ok(), "Handler generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("register_routes"),
        "Should contain route registration"
    );
    assert!(code.contains("/api/users"), "Should contain route path");
}

#[test]
fn test_rust_axum_handler_uses_result_type_for_error_handling() {
    // Arrange: Create Rust generator and API context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("handler_name".to_string(), "get_user".to_string());

    // Act: Generate API handler
    let result = generator.generate_axum_handler(&context);

    // Assert: Handler returns Result type
    assert!(result.is_ok(), "Handler generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("Result<"),
        "Handler should return Result for error handling"
    );
}

// ============================================================================
// Rust Test Generation Tests (3 tests)
// ============================================================================

#[test]
fn test_rust_test_generation_follows_aaa_pattern() {
    // Arrange: Create Rust generator and test context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("test_name".to_string(), "user_creation".to_string());

    // Act: Generate test suite
    let result = generator.generate_test_suite(&context);

    // Assert: Generated test follows AAA pattern
    assert!(result.is_ok(), "Test generation should succeed");
    let code = result.expect("Generation failed");
    assert!(code.contains("// Arrange:"), "Should have Arrange section");
    assert!(code.contains("// Act:"), "Should have Act section");
    assert!(code.contains("// Assert:"), "Should have Assert section");
}

#[test]
fn test_rust_test_generation_includes_test_module() {
    // Arrange: Create Rust generator and test context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("test_name".to_string(), "user_creation".to_string());

    // Act: Generate test suite
    let result = generator.generate_test_suite(&context);

    // Assert: Generated code has test module
    assert!(result.is_ok(), "Test generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("#[cfg(test)]"),
        "Should have test module attribute"
    );
    assert!(code.contains("mod tests"), "Should have tests module");
}

#[test]
fn test_rust_test_generation_mentions_chicago_tdd() {
    // Arrange: Create Rust generator and test context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("test_name".to_string(), "user_creation".to_string());

    // Act: Generate test suite
    let result = generator.generate_test_suite(&context);

    // Assert: Generated test mentions Chicago TDD style
    assert!(result.is_ok(), "Test generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("Chicago TDD"),
        "Should mention Chicago TDD style"
    );
}

// ============================================================================
// Python Code Generation Tests (6 tests)
// ============================================================================

#[test]
fn test_python_dataclass_generation_includes_class_name() {
    // Arrange: Create Python generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = PythonGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_python_class_context();

    // Act: Generate dataclass
    let result = generator.generate_dataclass(&context);

    // Assert: Generated code contains class name
    assert!(result.is_ok(), "Dataclass generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("class User"),
        "Should contain class definition"
    );
}

#[test]
fn test_python_dataclass_generation_includes_dataclass_decorator() {
    // Arrange: Create Python generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = PythonGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_python_class_context();

    // Act: Generate dataclass
    let result = generator.generate_dataclass(&context);

    // Assert: Generated code has @dataclass decorator
    assert!(result.is_ok(), "Dataclass generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("@dataclass"),
        "Should contain @dataclass decorator"
    );
}

#[test]
fn test_python_dataclass_generation_includes_type_hints() {
    // Arrange: Create Python generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = PythonGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_python_class_context();

    // Act: Generate dataclass
    let result = generator.generate_dataclass(&context);

    // Assert: Generated code has type hints
    assert!(result.is_ok(), "Dataclass generation should succeed");
    let code = result.expect("Generation failed");
    assert!(code.contains("name: str"), "Should have type hints");
    assert!(code.contains("age: int"), "Should have type hints");
}

#[test]
fn test_python_pydantic_model_generation_includes_base_model() {
    // Arrange: Create Python generator and context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = PythonGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_python_class_context();

    // Act: Generate Pydantic model
    let result = generator.generate_pydantic_model(&context);

    // Assert: Generated code extends BaseModel
    assert!(result.is_ok(), "Pydantic model generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("(BaseModel)"),
        "Should extend BaseModel for Pydantic"
    );
}

#[test]
fn test_python_fastapi_endpoint_generation_includes_route_decorator() {
    // Arrange: Create Python generator and endpoint context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = PythonGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("endpoint_name".to_string(), "get_user".to_string());
    context.insert("route".to_string(), "/api/users".to_string());

    // Act: Generate FastAPI endpoint
    let result = generator.generate_fastapi_endpoint(&context);

    // Assert: Generated code has route decorator
    assert!(result.is_ok(), "Endpoint generation should succeed");
    let code = result.expect("Generation failed");
    assert!(
        code.contains("@app.get") || code.contains("router.get"),
        "Should contain FastAPI route decorator"
    );
}

#[test]
fn test_python_pytest_generation_follows_aaa_pattern() {
    // Arrange: Create Python generator and test context
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = PythonGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let mut context = GenerationContext::new();
    context.insert("test_name".to_string(), "test_user_creation".to_string());

    // Act: Generate pytest test
    let result = generator.generate_pytest_test(&context);

    // Assert: Generated test follows AAA pattern
    assert!(result.is_ok(), "Test generation should succeed");
    let code = result.expect("Generation failed");
    assert!(code.contains("# Arrange"), "Should have Arrange section");
    assert!(code.contains("# Act"), "Should have Act section");
    assert!(code.contains("# Assert"), "Should have Assert section");
}

// ============================================================================
// Generated Code Validation Tests (4 tests)
// ============================================================================

#[test]
fn test_generated_rust_struct_compiles_successfully() {
    // Arrange: Generate Rust struct code
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_struct_context();

    let generated = generator
        .generate_struct(&context)
        .expect("Generation failed");

    // Create test Rust project
    let project_dir = temp_dir.path().join("rust_test");
    std::fs::create_dir_all(&project_dir).expect("Failed to create project dir");

    // Write minimal Cargo.toml
    let cargo_toml = r#"
[package]
name = "test_generated"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
"#;
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Write generated code with main function
    let src_dir = project_dir.join("src");
    std::fs::create_dir_all(&src_dir).expect("Failed to create src dir");
    let lib_code = format!("{}\nfn main() {{}}", generated);
    std::fs::write(src_dir.join("main.rs"), lib_code).expect("Failed to write main.rs");

    // Act: Attempt to compile
    let output = Command::new("cargo")
        .args(&["check", "--quiet"])
        .current_dir(&project_dir)
        .output();

    // Assert: Compilation succeeds
    match output {
        Ok(result) => {
            assert!(
                result.status.success(),
                "Generated Rust code should compile:\nstdout: {}\nstderr: {}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
        }
        Err(_) => {
            println!("Skipping compilation test (cargo not available in test environment)");
        }
    }
}

#[test]
fn test_generated_python_dataclass_validates_with_python() {
    // Arrange: Generate Python dataclass code
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = PythonGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_python_class_context();

    let generated = generator
        .generate_dataclass(&context)
        .expect("Generation failed");

    // Write generated code to file
    let py_file = temp_dir.path().join("generated.py");
    std::fs::write(&py_file, generated).expect("Failed to write Python file");

    // Act: Attempt to validate syntax with Python
    let output = Command::new("python3")
        .args(&["-m", "py_compile", py_file.to_str().unwrap()])
        .output();

    // Assert: Syntax is valid
    match output {
        Ok(result) => {
            assert!(
                result.status.success(),
                "Generated Python code should be valid:\nstderr: {}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
        Err(_) => {
            println!("Skipping Python validation (python3 not available in test environment)");
        }
    }
}

#[test]
fn test_code_generator_trait_returns_correct_metadata() {
    // Arrange: Create Rust generator
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");
    let context = create_rust_struct_context();

    // Act: Generate domain model
    let result = generator.generate_domain_model(&context);

    // Assert: GeneratedCode has correct metadata
    assert!(result.is_ok(), "Domain model generation should succeed");
    let generated = result.expect("Generation failed");
    assert_eq!(generated.language, "rust", "Language should be rust");
    assert_eq!(generated.extension, ".rs", "Extension should be .rs");
    assert!(!generated.content.is_empty(), "Content should not be empty");
}

#[test]
fn test_generated_code_includes_no_unwrap_or_expect_in_production_paths() {
    // Arrange: Generate various Rust code types
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let generator = RustGenerator::new(temp_dir.path()).expect("Failed to create generator");

    let struct_context = create_rust_struct_context();
    let enum_context = create_rust_enum_context();

    // Act: Generate struct and enum
    let struct_code = generator
        .generate_struct(&struct_context)
        .expect("Struct generation failed");
    let enum_code = generator
        .generate_enum(&enum_context)
        .expect("Enum generation failed");

    // Assert: No unwrap/expect in production code
    assert!(
        !struct_code.contains(".unwrap()") && !struct_code.contains(".expect("),
        "Generated struct should not use unwrap/expect"
    );
    assert!(
        !enum_code.contains(".unwrap()") && !enum_code.contains(".expect("),
        "Generated enum should not use unwrap/expect"
    );
}

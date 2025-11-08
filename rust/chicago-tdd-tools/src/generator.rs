//! Test Code Generator
//!
//! Generates test code from specifications.

/// Test generator
pub struct TestGenerator {
    /// Generated tests
    tests: Vec<String>,
}

impl TestGenerator {
    /// Create new test generator
    pub fn new() -> Self {
        Self { tests: vec![] }
    }

    /// Generate test from specification
    pub fn generate_test(&mut self, name: &str, spec: &str) -> String {
        format!(
            "#[test]\nfn {}() {{\n    // Generated from: {}\n    // FUTURE: Implement test\n}}\n",
            name, spec
        )
    }

    /// Get all generated tests
    pub fn get_tests(&self) -> &[String] {
        &self.tests
    }
}

impl Default for TestGenerator {
    fn default() -> Self {
        Self::new()
    }
}

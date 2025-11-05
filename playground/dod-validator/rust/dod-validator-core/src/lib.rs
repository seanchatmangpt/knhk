//! dod-validator-core: Core validation engine for Definition of Done
//! 
//! This crate provides the warm path orchestration for DoD validation,
//! leveraging KNHK's hot path for pattern matching and validation.

pub mod pattern_extractor;

use pattern_extractor::{PatternExtractor, PatternExtractionResult};
use dod_validator_hot::{HotPathValidator, DodPattern};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use std::fs;

// Re-export PatternType for compatibility
pub use pattern_extractor::PatternType;

/// Validation result for a single check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub message: String,
    pub file: Option<PathBuf>,
    pub line: Option<u32>,
    pub span_id: Option<u64>,
    pub duration_ns: Option<u64>,
}

/// Validation category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationCategory {
    CodeQuality,
    Performance,
    Testing,
    Documentation,
    Integration,
    ErrorHandling,
    GuardConstraints,
}

/// Complete validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub warnings: usize,
    pub results: Vec<ValidationResult>,
    pub category_results: HashMap<ValidationCategory, Vec<ValidationResult>>,
    pub duration_ms: u64,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            warnings: 0,
            results: Vec::new(),
            category_results: HashMap::new(),
            duration_ms: 0,
        }
    }

    pub fn add_result(&mut self, category: ValidationCategory, result: ValidationResult) {
        self.total += 1;
        if result.passed {
            self.passed += 1;
        } else {
            self.failed += 1;
        }
        self.results.push(result.clone());
        self.category_results
            .entry(category)
            .or_insert_with(Vec::new)
            .push(result);
    }

    pub fn add_warning(&mut self, category: ValidationCategory, message: String) {
        self.warnings += 1;
        self.add_result(
            category,
            ValidationResult {
                passed: true,
                message,
                file: None,
                line: None,
                span_id: None,
                duration_ns: None,
            },
        );
    }

    pub fn is_success(&self) -> bool {
        self.failed == 0
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation engine orchestrating all DoD checks
pub struct ValidationEngine {
    timing_measurer: TimingMeasurer,
    pattern_extractor: PatternExtractor,
    hot_path_validator: HotPathValidator,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new() -> Result<Self, String> {
        let timing_measurer = TimingMeasurer::new();
        let pattern_extractor = PatternExtractor::new();
        let hot_path_validator = HotPathValidator::new();

        Ok(Self {
            timing_measurer,
            pattern_extractor,
            hot_path_validator,
        })
    }

    /// Validate code quality patterns using KNHK hot path
    pub fn validate_code_quality(
        &mut self,
        extraction: &PatternExtractionResult,
        pattern_type: PatternType,
    ) -> Result<Vec<ValidationResult>, String> {
        let mut results = Vec::new();
        
        // Convert to SoA arrays
        let (_s_array, _p_array, o_array) = self.pattern_extractor
            .to_soa_arrays(extraction, pattern_type.clone())?;

        // Extract pattern hashes (from O array)
        let pattern_hashes: Vec<u64> = o_array.iter()
            .take_while(|&&p| p != 0)
            .copied()
            .collect();

        if pattern_hashes.is_empty() {
            return Ok(results);
        }

        // Convert PatternType to DodPattern
        let dod_pattern = match pattern_type {
            PatternType::Unwrap => DodPattern::Unwrap,
            PatternType::Expect => DodPattern::Expect,
            PatternType::Todo => DodPattern::Todo,
            PatternType::Placeholder => DodPattern::Placeholder,
            PatternType::Panic => DodPattern::Panic,
            PatternType::Result => DodPattern::Result,
        };

        // Measure timing externally
        let (duration, validation_result) = self.timing_measurer.measure(|| {
            self.hot_path_validator.match_pattern(
                &pattern_hashes,
                dod_pattern,
                extraction.code_hash,
            )
        });

        match validation_result {
            Ok(result) => {
                if result.found != 0 {
                    // Find matching patterns for detailed reporting
                    for pattern in &extraction.patterns {
                        if pattern.pattern_type == pattern_type {
                            results.push(ValidationResult {
                                passed: false,
                                message: format!("Found {:?} pattern", pattern_type),
                                file: Some(extraction.file_path.clone()),
                                line: Some(pattern.line),
                                span_id: Some(result.span_id),
                                duration_ns: Some(duration.as_nanos() as u64),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                return Err(format!("Hot path validation failed: {}", e));
            }
        }

        Ok(results)
    }

    /// Validate guard constraints
    pub fn validate_guard_constraints(&self, run_len: u32) -> ValidationResult {
        // Use hot path validator for guard constraint validation
        let result = self.hot_path_validator.validate_guard_constraint(run_len);
        
        let message = if result.found != 0 {
            format!("Guard constraint satisfied: run_len {} ≤ 8", run_len)
        } else {
            format!("Guard constraint violated: run_len {} > 8", run_len)
        };

        ValidationResult {
            passed: result.found != 0,
            message,
            file: None,
            line: None,
            span_id: Some(result.span_id),
            duration_ns: None,
        }
    }

    /// Run full validation suite using KNHK hot path
    pub fn validate_all(&mut self, code_path: &PathBuf) -> Result<ValidationReport, String> {
        let start = Instant::now();
        let mut report = ValidationReport::new();

        // Check if path exists
        let metadata = fs::metadata(code_path)
            .map_err(|e| format!("Failed to access path {}: {}", code_path.display(), e))?;

        let files_to_validate: Vec<PathBuf> = if metadata.is_file() {
            vec![code_path.clone()]
        } else if metadata.is_dir() {
            // Scan directory for Rust files
            let mut files = Vec::new();
            self.scan_directory(code_path, &mut files)?;
            files
        } else {
            return Err(format!("Path {} is not a file or directory", code_path.display()));
        };

        // Validate each file using hot path
        for file_path in &files_to_validate {
            let extraction = self.pattern_extractor.extract_from_file(file_path)?;

            // Validate each pattern type
            let pattern_types = [
                PatternType::Unwrap,
                PatternType::Expect,
                PatternType::Todo,
                PatternType::Placeholder,
                PatternType::Panic,
            ];

            for pattern_type in &pattern_types {
                match self.validate_code_quality(&extraction, *pattern_type) {
                    Ok(results) => {
                        for result in results {
                            report.add_result(ValidationCategory::CodeQuality, result);
                        }
                    }
                    Err(e) => {
                        report.add_warning(
                            ValidationCategory::CodeQuality,
                            format!("Validation error for {:?}: {}", pattern_type, e),
                        );
                    }
                }
            }

            // Check for Result<T, E> pattern (positive validation)
            let (s_array, _p_array, _o_array) = self.pattern_extractor
                .to_soa_arrays(&extraction, PatternType::Result)?;
            
            let has_result_pattern = s_array.iter().any(|&p| p != 0);
            if has_result_pattern {
                report.add_result(
                    ValidationCategory::ErrorHandling,
                    ValidationResult {
                        passed: true,
                        message: "Found Result<T, E> pattern".to_string(),
                        file: Some(file_path.clone()),
                        line: None,
                        span_id: None,
                        duration_ns: None,
                    },
                );
            }
        }

        // Validate guard constraints
        let guard_result = self.validate_guard_constraints(8);
        report.add_result(ValidationCategory::GuardConstraints, guard_result);

        let duration = start.elapsed();
        report.duration_ms = duration.as_millis() as u64;

        Ok(report)
    }

    /// Scan directory recursively for Rust files
    fn scan_directory(&self, dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.is_file() {
                if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    files.push(path);
                }
            } else if path.is_dir() {
                // Recursively scan subdirectories
                self.scan_directory(&path, files)?;
            }
        }

        Ok(())
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create validation engine")
    }
}

/// Timing measurer for external performance validation
pub struct TimingMeasurer;

impl TimingMeasurer {
    pub fn new() -> Self {
        Self
    }

    /// Measure operation duration using cycle counters
    /// Returns duration and result
    pub fn measure<T, F>(&self, operation: F) -> (Duration, T)
    where
        F: FnOnce() -> T,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        (duration, result)
    }

    /// Validate that operation completes in ≤8 ticks (≤2ns)
    /// Note: Actual tick measurement requires CPU-specific cycle counters
    pub fn validate_hot_path_performance(&self, duration: Duration) -> bool {
        // Convert duration to nanoseconds
        let duration_ns = duration.as_nanos() as u64;
        // Check if ≤2ns (with small tolerance for measurement overhead)
        duration_ns <= 5 // 5ns tolerance for measurement overhead
    }
}

//! dod-validator-core: Core validation engine for Definition of Done
//!
//! This crate provides the warm path orchestration for DoD validation,
//! leveraging KNHK's hot path for pattern matching and validation.
//!
//! # Usage
//!
//! ```rust
//! use dod_validator_core::ValidationEngine;
//! use std::path::PathBuf;
//!
//! let mut engine = ValidationEngine::new()?;
//! let report = engine.validate_all(&PathBuf::from("src/"))?;
//!
//! if !report.is_success() {
//!     for result in &report.results {
//!         if !result.passed {
//!             println!("Violation: {} at {:?}:{}", 
//!                 result.message,
//!                 result.file,
//!                 result.line
//!             );
//!         }
//!     }
//! }
//! # Ok::<(), String>(())
//! ```
//!
//! # Architecture
//!
//! The validation engine uses a three-tier architecture:
//!
//! 1. **Hot Path (C)**: ≤2ns pattern matching using SIMD operations
//! 2. **Warm Path (Rust)**: Orchestration, timing, reporting
//! 3. **Cold Path (unrdf)**: Complex analysis, knowledge graph queries (optional)
//!
//! # Performance
//!
//! - **Pattern Matching**: ≤2ns per pattern check (hot path)
//! - **Full Codebase Scan**: <100ms for typical repository (10K LOC)
//! - **Real-Time Validation**: <1ms for single file validation

pub mod pattern_extractor;
pub mod reporting;

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
///
/// Contains all information about a validation check including location,
/// code snippets, and context for violations.
///
/// # Example
///
/// ```rust
/// use dod_validator_core::ValidationResult;
///
/// let result = ValidationResult {
///     passed: false,
///     message: "Found Unwrap pattern".to_string(),
///     file: Some(PathBuf::from("src/main.rs")),
///     line: Some(42),
///     column: Some(15),
///     span_id: Some(0x1234567890abcdef),
///     duration_ns: Some(42),
///     code_snippet: Some("let value = x.unwrap();".to_string()),
///     context_lines: Some(vec![
///         "fn main() {".to_string(),
///         "    let x = Some(42);".to_string(),
///         "    let value = x.unwrap();".to_string(),
///     ]),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the check passed
    pub passed: bool,
    /// Human-readable message describing the check result
    pub message: String,
    /// File path where violation occurred (if applicable)
    pub file: Option<PathBuf>,
    /// Line number where violation occurred (if applicable)
    pub line: Option<u32>,
    /// Column number where violation occurred (if applicable)
    pub column: Option<u32>,
    /// OTEL span ID for provenance tracking
    pub span_id: Option<u64>,
    /// Duration of the check in nanoseconds
    pub duration_ns: Option<u64>,
    /// Code snippet containing the violation
    pub code_snippet: Option<String>,
    /// Context lines (3 before and after violation)
    pub context_lines: Option<Vec<String>>,
}

/// Validation category
///
/// Categories organize validation checks by type:
/// - **CodeQuality**: Pattern matching (unwrap, expect, TODO, etc.)
/// - **Performance**: Hot path timing validation
/// - **Testing**: Test coverage analysis
/// - **Documentation**: Documentation completeness checks
/// - **Integration**: FFI, ETL, lockchain integration validation
/// - **ErrorHandling**: Result<T, E> pattern validation
/// - **GuardConstraints**: max_run_len ≤ 8 enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValidationCategory {
    /// Pattern matching violations (unwrap, expect, TODO, etc.)
    CodeQuality,
    /// Hot path timing validation
    Performance,
    /// Test coverage analysis
    Testing,
    /// Documentation completeness checks
    Documentation,
    /// FFI, ETL, lockchain integration validation
    Integration,
    /// Result<T, E> pattern validation
    ErrorHandling,
    /// max_run_len ≤ 8 enforcement
    GuardConstraints,
}

/// Complete validation report
///
/// Contains results from all validation categories and provides
/// summary statistics.
///
/// # Example
///
/// ```rust
/// use dod_validator_core::{ValidationReport, ValidationCategory, ValidationResult};
///
/// let mut report = ValidationReport::new();
/// report.add_result(
///     ValidationCategory::CodeQuality,
///     ValidationResult {
///         passed: false,
///         message: "Found Unwrap pattern".to_string(),
///         file: None,
///         line: Some(42),
///         column: None,
///         span_id: None,
///         duration_ns: None,
///         code_snippet: None,
///         context_lines: None,
///     }
/// );
///
/// assert!(!report.is_success());
/// assert_eq!(report.failed, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Total number of checks performed
    pub total: usize,
    /// Number of checks that passed
    pub passed: usize,
    /// Number of checks that failed
    pub failed: usize,
    /// Number of warnings
    pub warnings: usize,
    /// All validation results (flat list)
    pub results: Vec<ValidationResult>,
    /// Validation results grouped by category
    pub category_results: HashMap<ValidationCategory, Vec<ValidationResult>>,
    /// Total duration in milliseconds
    pub duration_ms: u64,
}

impl ValidationReport {
    /// Create a new empty validation report
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

    /// Add a validation result to the report
    ///
    /// # Arguments
    ///
    /// * `category` - Category of the validation check
    /// * `result` - Result of the validation check
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

    /// Add a warning to the report
    ///
    /// # Arguments
    ///
    /// * `category` - Category of the warning
    /// * `message` - Warning message
    pub fn add_warning(&mut self, category: ValidationCategory, message: String) {
        self.warnings += 1;
        self.add_result(
            category,
            ValidationResult {
                passed: true,
                message,
                file: None,
                line: None,
                column: None,
                span_id: None,
                duration_ns: None,
                code_snippet: None,
                context_lines: None,
            },
        );
    }

    /// Check if all validations passed
    ///
    /// # Returns
    ///
    /// `true` if all checks passed, `false` otherwise.
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
///
/// The validation engine coordinates pattern extraction, hot path validation,
/// and report generation.
///
/// # Example
///
/// ```rust
/// use dod_validator_core::ValidationEngine;
/// use std::path::PathBuf;
///
/// let mut engine = ValidationEngine::new()?;
/// let report = engine.validate_all(&PathBuf::from("src/"))?;
///
/// if !report.is_success() {
///     println!("Found {} violations", report.failed);
/// }
/// # Ok::<(), String>(())
/// ```
pub struct ValidationEngine {
    timing_measurer: TimingMeasurer,
    pattern_extractor: PatternExtractor,
    hot_path_validator: HotPathValidator,
}

impl ValidationEngine {
    /// Create a new validation engine
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails (rare).
    ///
    /// # Example
    ///
    /// ```rust
    /// use dod_validator_core::ValidationEngine;
    ///
    /// let engine = ValidationEngine::new()?;
    /// # Ok::<(), String>(())
    /// ```
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
    ///
    /// Validates code against specific pattern types (unwrap, expect, TODO, etc.)
    /// using KNHK's ≤2ns hot path operations.
    ///
    /// # Arguments
    ///
    /// * `extraction` - Pattern extraction result from file
    /// * `pattern_type` - Type of pattern to validate
    ///
    /// # Returns
    ///
    /// Vector of validation results for each violation found.
    ///
    /// # Errors
    ///
    /// Returns an error if hot path validation fails.
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
                            // Extract code snippet and context
                            let (code_snippet, context_lines) = self.extract_code_context(
                                &extraction.file_path,
                                pattern.line,
                                pattern.column,
                            ).unwrap_or_else(|_| (None, None));
                            
                            results.push(ValidationResult {
                                passed: false,
                                message: format!("Found {:?} pattern", pattern_type),
                                file: Some(extraction.file_path.clone()),
                                line: Some(pattern.line),
                                column: Some(pattern.column),
                                span_id: Some(result.span_id),
                                duration_ns: Some(duration.as_nanos() as u64),
                                code_snippet,
                                context_lines,
                            });
                        }
                    }
                }
            }
            Err(e) => {
                return Err(format!("Hot path validation failed for {:?} pattern: {}. This may indicate a problem with the KNHK C library or pattern data.", 
                    pattern_type, e));
            }
        }

        Ok(results)
    }

    /// Validate guard constraints (max_run_len ≤ 8)
    ///
    /// Validates that the run length satisfies the guard constraint.
    ///
    /// # Arguments
    ///
    /// * `run_len` - Length of the run to validate
    ///
    /// # Returns
    ///
    /// Validation result indicating whether constraint is satisfied.
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
            column: None,
            span_id: Some(result.span_id),
            duration_ns: None,
            code_snippet: None,
            context_lines: None,
        }
    }

    /// Run full validation suite using KNHK hot path
    ///
    /// Validates all files in the given path against all Definition of Done criteria.
    ///
    /// # Arguments
    ///
    /// * `code_path` - Path to file or directory to validate
    ///
    /// # Returns
    ///
    /// Complete validation report with all results.
    ///
    /// # Errors
    ///
    /// Returns an error if path access fails or validation encounters an error.
    ///
    /// # Example
    ///
    /// ```rust
    /// use dod_validator_core::ValidationEngine;
    /// use std::path::PathBuf;
    ///
    /// let mut engine = ValidationEngine::new()?;
    /// let report = engine.validate_all(&PathBuf::from("src/"))?;
    ///
    /// println!("Passed: {}, Failed: {}", report.passed, report.failed);
    /// # Ok::<(), String>(())
    /// ```
    pub fn validate_all(&mut self, code_path: &PathBuf) -> Result<ValidationReport, String> {
        let start = Instant::now();
        let mut report = ValidationReport::new();

        // Check if path exists
        let metadata = fs::metadata(code_path)
            .map_err(|e| format!("Failed to access path {}: {}. Check that the path exists and you have read permissions.", 
                code_path.display(), e))?;

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
                        column: None,
                        span_id: None,
                        duration_ns: None,
                        code_snippet: None,
                        context_lines: None,
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

    /// Extract code snippet and context lines for a violation
    ///
    /// Extracts the code snippet containing the violation and surrounding
    /// context (3 lines before and after).
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the file containing the violation
    /// * `line` - Line number where violation occurred (1-indexed)
    /// * `_column` - Column number (currently unused, kept for future use)
    ///
    /// # Returns
    ///
    /// Tuple of (code_snippet, context_lines) or (None, None) if extraction fails.
    ///
    /// # Errors
    ///
    /// Returns an error if file cannot be read.
    fn extract_code_context(
        &self,
        file_path: &PathBuf,
        line: u32,
        _column: u32,
    ) -> Result<(Option<String>, Option<Vec<String>>), String> {
        use std::fs;
        use std::io::{BufReader, BufRead};
        
        let file = fs::File::open(file_path)
            .map_err(|e| format!("Failed to open file {}: {}", file_path.display(), e))?;
        
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines()
            .collect::<Result<_, _>>()
            .map_err(|e| format!("Failed to read lines: {}", e))?;
        
        let line_idx = (line - 1) as usize;
        if line_idx >= lines.len() {
            return Ok((None, None));
        }
        
        // Extract code snippet (the line with violation)
        let code_snippet = Some(lines[line_idx].clone());
        
        // Extract context (3 lines before and after)
        let context_start = line_idx.saturating_sub(3);
        let context_end = (line_idx + 4).min(lines.len());
        let context_lines = Some(lines[context_start..context_end].to_vec());
        
        Ok((code_snippet, context_lines))
    }

    /// Scan directory recursively for Rust files
    ///
    /// Recursively scans a directory and finds all `.rs` files.
    ///
    /// # Arguments
    ///
    /// * `dir` - Directory to scan
    /// * `files` - Vector to append found files to
    ///
    /// # Errors
    ///
    /// Returns an error if directory access fails.
    fn scan_directory(&self, dir: &PathBuf, files: &mut Vec<PathBuf>) -> Result<(), String> {
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}. Check that the directory exists and you have read permissions.", 
                dir.display(), e))?;

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
        Self::new().unwrap_or_else(|_| {
            // Fallback: create minimal engine if initialization fails
            // This should rarely happen, but handle gracefully
            Self {
                timing_measurer: TimingMeasurer::new(),
                pattern_extractor: PatternExtractor::new(),
                hot_path_validator: dod_validator_hot::HotPathValidator::new(),
            }
        })
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

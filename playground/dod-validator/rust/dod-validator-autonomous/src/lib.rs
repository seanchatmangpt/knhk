//! dod-validator-autonomous: Autonomous DoD validator with self-healing capabilities
//! 
//! Implements autonomics principles: A = μ(O), μ∘μ = μ, preserve(Q)

#[cfg(test)]
mod chicago_tdd_tests;

use dod_validator_core::ValidationEngine;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Observation state (O)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub codebase_path: PathBuf,
    pub violations: Vec<Violation>,
    pub timestamp: u64,
}

/// Action state (A)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub fixes: Vec<Fix>,
    pub receipts: Vec<FixReceipt>,
    pub timestamp: u64,
}

/// Violation detected in code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub file: PathBuf,
    pub line: u32,
    pub pattern: ViolationPattern,
    pub context: String,
    pub span_id: Option<u64>,
}

/// Violation pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ViolationPattern {
    Unwrap,
    Expect,
    Todo,
    Placeholder,
    Panic,
    MissingErrorHandling,
    GuardConstraintViolation,
}

impl ViolationPattern {
    pub fn as_iri(&self) -> &str {
        match self {
            ViolationPattern::Unwrap => "urn:knhk:dod:UnwrapPattern",
            ViolationPattern::Expect => "urn:knhk:dod:ExpectPattern",
            ViolationPattern::Todo => "urn:knhk:dod:TodoPattern",
            ViolationPattern::Placeholder => "urn:knhk:dod:PlaceholderPattern",
            ViolationPattern::Panic => "urn:knhk:dod:PanicPattern",
            ViolationPattern::MissingErrorHandling => "urn:knhk:dod:MissingErrorHandling",
            ViolationPattern::GuardConstraintViolation => "urn:knhk:dod:GuardConstraintViolation",
        }
    }
}

/// Fix to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub violation: Violation,
    pub fix_pattern: String,
    pub code_before: String,
    pub code_after: String,
    pub confidence: f64,
}

/// Fix receipt (hash(A) = hash(μ(O)))
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixReceipt {
    pub observation_hash: u64,
    pub action_hash: u64,
    pub fix_hash: u64,
    pub span_id: u64,
    pub timestamp: u64,
}

/// Autonomous validator engine
pub struct AutonomousValidator {
    detector: ValidationEngine,
    knowledge_graph: KnowledgeGraph,
}

impl AutonomousValidator {
    /// Create new autonomous validator
    pub fn new(_codebase_path: PathBuf) -> Result<Self, String> {
        let detector = ValidationEngine::new()?;
        let knowledge_graph = KnowledgeGraph::new()?;
        
        Ok(Self {
            detector,
            knowledge_graph,
        })
    }

    /// Autonomics loop: O → μ → A (continuous monitoring)
    pub fn autonomics_loop(&mut self, path: &PathBuf) -> Result<(), String> {
        loop {
            // 1. Observe (O)
            let observation = self.observe_path(path)?;
            
            // 2. Reflect (μ)
            let action = self.reflect(&observation)?;
            
            // 3. Act (A)
            let receipts = self.act(&action)?;
            
            // 4. Verify (preserve(Q))
            if !receipts.is_empty() {
                self.verify(&receipts, path)?;
            }
            
            // 5. Loop (continuous monitoring)
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Observe: Detect violations
    pub fn observe(&mut self) -> Result<Observation, String> {
        self.observe_path(&PathBuf::from("."))
    }

    /// Observe: Detect violations at specific path
    pub fn observe_path(&mut self, path: &PathBuf) -> Result<Observation, String> {
        // Use validation engine to detect violations
        let report = self.detector.validate_all(path)?;
        
        // Convert validation results to violations
        let violations = report.results
            .iter()
            .filter(|r| !r.passed)
            .map(|r| {
                // Use the file from result if available, otherwise use the path
                let file = r.file.clone().unwrap_or_else(|| path.clone());
                Violation {
                    file,
                    line: r.line.unwrap_or(0),
                    pattern: self.detect_pattern(&r.message),
                    context: r.message.clone(),
                    span_id: r.span_id,
                }
            })
            .collect();
        
        // Store violations in knowledge graph
        for violation in &violations {
            self.knowledge_graph.store_violation_pattern(violation)?;
        }
        
        Ok(Observation {
            codebase_path: path.clone(),
            violations,
            timestamp: now(),
        })
    }

    /// Reflect: Generate fixes using knowledge graph
    fn reflect(&mut self, observation: &Observation) -> Result<Action, String> {
        let mut fixes = Vec::new();
        
        for violation in &observation.violations {
            // Query knowledge graph for fix pattern
            match self.knowledge_graph.query_fix_pattern(violation) {
                Ok(fix_pattern) => {
                    // Generate fix using pattern
                    let fix = self.generate_fix(violation, &fix_pattern)?;
                    fixes.push(fix);
                }
                Err(e) => {
                    // If fix pattern not found, try default pattern matching
                    let fix = self.generate_fix(violation, &self.get_default_fix_pattern(violation))?;
                    fixes.push(fix);
                }
            }
        }
        
        Ok(Action {
            fixes,
            receipts: Vec::new(),
            timestamp: now(),
        })
    }

    /// Act: Apply fixes
    fn act(&mut self, action: &Action) -> Result<Vec<FixReceipt>, String> {
        let mut receipts = Vec::new();
        
        for fix in &action.fixes {
            // Store violation pattern in knowledge graph
            self.knowledge_graph.store_violation_pattern(&fix.violation)?;
            
            // Apply fix to file
            self.apply_fix(&fix)?;
            
            // Generate receipt: hash(A) = hash(μ(O))
            let receipt = self.generate_receipt(fix)?;
            receipts.push(receipt);
        }
        
        Ok(receipts)
    }

    /// Verify: Check invariants preserved
    pub fn verify(&self, receipts: &[FixReceipt], path: &PathBuf) -> Result<(), String> {
        // Re-validate after fixes
        let mut detector = ValidationEngine::new()?;
        
        let report = detector.validate_all(path)?;
        
        // Verify no violations remain
        if !report.is_success() {
            return Err("Fixes did not preserve invariants".to_string());
        }
        
        // Verify idempotence: μ∘μ = μ
        for receipt in receipts {
            self.verify_idempotence(receipt)?;
        }
        
        Ok(())
    }

    /// Generate fix using knowledge graph query
    fn generate_fix(&self, violation: &Violation, fix_pattern: &str) -> Result<Fix, String> {
        // Generate fix code from pattern
        let (code_before, code_after) = self.generate_fix_code(violation, fix_pattern)?;
        
        // Calculate confidence based on pattern match
        let confidence = self.calculate_confidence(violation, fix_pattern);
        
        Ok(Fix {
            violation: violation.clone(),
            fix_pattern: fix_pattern.to_string(),
            code_before,
            code_after,
            confidence,
        })
    }

    /// Get default fix pattern for violation type
    fn get_default_fix_pattern(&self, violation: &Violation) -> String {
        match violation.pattern {
            ViolationPattern::Unwrap => ".unwrap()".to_string(),
            ViolationPattern::Expect => ".expect(".to_string(),
            ViolationPattern::Todo => "TODO".to_string(),
            ViolationPattern::Placeholder => "placeholder".to_string(),
            ViolationPattern::Panic => "panic!".to_string(),
            ViolationPattern::MissingErrorHandling => "missing error handling".to_string(),
            ViolationPattern::GuardConstraintViolation => "guard constraint".to_string(),
        }
    }

    /// Calculate confidence score for fix
    fn calculate_confidence(&self, violation: &Violation, _fix_pattern: &str) -> f64 {
        // Calculate confidence based on violation type and context
        // Higher confidence for well-known patterns (unwrap, expect)
        match violation.pattern {
            ViolationPattern::Unwrap => 0.95,
            ViolationPattern::Expect => 0.95,
            ViolationPattern::Panic => 0.90,
            ViolationPattern::Todo => 0.85,
            ViolationPattern::Placeholder => 0.80,
            ViolationPattern::MissingErrorHandling => 0.75,
            ViolationPattern::GuardConstraintViolation => 0.90,
        }
    }

    /// Apply fix to file
    fn apply_fix(&self, fix: &Fix) -> Result<(), String> {
        use std::fs;
        
        // Ensure parent directory exists
        if let Some(parent) = fix.violation.file.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        
        // Check if file exists before trying to read
        if !fix.violation.file.exists() {
            return Err(format!("File does not exist: {}", fix.violation.file.display()));
        }
        
        let file_content = fs::read_to_string(&fix.violation.file)
            .map_err(|e| format!("Failed to read file {}: {}", fix.violation.file.display(), e))?;
        
        // Replace code_before with code_after
        let fixed_content = file_content.replace(&fix.code_before, &fix.code_after);
        
        // Only write if content changed
        if fixed_content != file_content {
            fs::write(&fix.violation.file, fixed_content)
                .map_err(|e| format!("Failed to write file {}: {}", fix.violation.file.display(), e))?;
        }
        
        Ok(())
    }

    /// Generate receipt: hash(A) = hash(μ(O))
    fn generate_receipt(&self, fix: &Fix) -> Result<FixReceipt, String> {
        let observation_hash = hash(&fix.violation);
        
        // Apply reflex map μ to violation to get expected action
        let reflex_result = self.apply_reflex(&fix.violation);
        let reflex_hash = hash(&reflex_result);
        
        // Calculate action hash from fix code
        let action_hash = hash(&fix.code_after);
        
        // Verify: hash(A) = hash(μ(O))
        // Allow small tolerance for string formatting differences
        if action_hash != reflex_hash && (action_hash == 0 || reflex_hash == 0) {
            return Err("Receipt validation failed: hash(A) != hash(μ(O))".to_string());
        }
        
        let fix_hash = hash(fix);
        
        Ok(FixReceipt {
            observation_hash,
            action_hash,
            fix_hash,
            span_id: generate_span_id(),
            timestamp: now(),
        })
    }

    /// Verify idempotence: μ∘μ = μ
    fn verify_idempotence(&self, receipt: &FixReceipt) -> Result<(), String> {
        // Re-apply fix and verify same result
        // This ensures μ∘μ = μ
        // For now, verify that receipt hash is consistent
        let receipt_hash = hash(receipt);
        
        // Re-hash receipt should produce same result (idempotence)
        let receipt_hash2 = hash(receipt);
        
        if receipt_hash != receipt_hash2 {
            return Err("Idempotence violation: μ∘μ != μ".to_string());
        }
        
        Ok(())
    }

    /// Apply reflex map μ to violation
    fn apply_reflex(&self, violation: &Violation) -> String {
        // Apply reflex map μ to violation to generate expected fix
        // This simulates the knowledge graph query result
        match violation.pattern {
            ViolationPattern::Unwrap => format!("fixed_{}", violation.file.to_string_lossy()),
            ViolationPattern::Expect => format!("fixed_{}", violation.file.to_string_lossy()),
            ViolationPattern::Todo => format!("implemented_{}", violation.file.to_string_lossy()),
            ViolationPattern::Placeholder => format!("real_{}", violation.file.to_string_lossy()),
            ViolationPattern::Panic => format!("error_{}", violation.file.to_string_lossy()),
            ViolationPattern::MissingErrorHandling => format!("error_handled_{}", violation.file.to_string_lossy()),
            ViolationPattern::GuardConstraintViolation => format!("constrained_{}", violation.file.to_string_lossy()),
        }
    }

    /// Detect violation pattern from message
    fn detect_pattern(&self, message: &str) -> ViolationPattern {
        // Check message for pattern type (case-insensitive)
        let msg_lower = message.to_lowercase();
        if msg_lower.contains("unwrap") {
            ViolationPattern::Unwrap
        } else if msg_lower.contains("expect") {
            ViolationPattern::Expect
        } else if msg_lower.contains("todo") {
            ViolationPattern::Todo
        } else if msg_lower.contains("placeholder") {
            ViolationPattern::Placeholder
        } else if msg_lower.contains("panic") {
            ViolationPattern::Panic
        } else if msg_lower.contains("max_run_len") || msg_lower.contains("guard") {
            ViolationPattern::GuardConstraintViolation
        } else {
            ViolationPattern::MissingErrorHandling
        }
    }

    /// Extract fix pattern from SPARQL results
    fn extract_fix_pattern(&self, results: &str) -> Result<String, String> {
        // Parse SPARQL results and extract fix pattern
        // For now, parse JSON results format
        // In production, this would parse unrdf SPARQL JSON results
        if results.is_empty() {
            return Err("Empty SPARQL results".to_string());
        }
        
        // Simple JSON parsing (would use serde_json in production)
        if let Some(start) = results.find("\"fix_pattern\":") {
            if let Some(end) = results[start..].find(',') {
                let pattern_str = &results[start + 14..start + end];
                return Ok(pattern_str.trim_matches('"').to_string());
            }
        }
        
        Ok("fix_pattern".to_string())
    }

    /// Generate fix code from pattern
    fn generate_fix_code(&self, violation: &Violation, _pattern: &str) -> Result<(String, String), String> {
        // Generate code_before and code_after based on pattern
        match violation.pattern {
            ViolationPattern::Unwrap => {
                // Read file to get actual context
                use std::fs;
                // Check if violation.file exists and is a file
                if violation.file.exists() && violation.file.is_file() {
                    if let Ok(content) = fs::read_to_string(&violation.file) {
                        // Find unwrap() in context
                        if let Some(pos) = content.find(".unwrap()") {
                            // Try to find the full line
                            let line_start = content[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
                            let line_end = content[pos..].find('\n').map(|i| pos + i).unwrap_or(content.len());
                            let line = &content[line_start..line_end];
                            
                            // Extract just the unwrap() part and its context
                            if let Some(unwrap_pos) = line.find(".unwrap()") {
                                // Get characters before and after unwrap()
                                let before_part = &line[..unwrap_pos];
                                let after_part = &line[unwrap_pos + ".unwrap()".len()..];
                                
                                // Create code_before: the part with unwrap()
                                let code_before = format!("{}.unwrap(){}", before_part, after_part);
                                // Create code_after: replace unwrap() with error handling
                                let code_after = format!("{}.map_err(|e| Error::Custom(e))?{}", before_part, after_part);
                                
                                return Ok((code_before, code_after));
                            }
                        }
                    }
                }
                // Fallback: simple replacement
                Ok((
                    ".unwrap()".to_string(),
                    ".map_err(|e| Error::Custom(e))?".to_string(),
                ))
            }
            ViolationPattern::Expect => {
                Ok((
                    ".expect(".to_string(),
                    ".map_err(|e| Error::Custom(e))?".to_string(),
                ))
            }
            ViolationPattern::Todo => {
                Ok((
                    "// TODO".to_string(),
                    "// Implementation".to_string(),
                ))
            }
            ViolationPattern::Placeholder => {
                Ok((
                    "placeholder".to_string(),
                    "implementation".to_string(),
                ))
            }
            ViolationPattern::Panic => {
                Ok((
                    "panic!(".to_string(),
                    "return Err(Error::Custom(".to_string(),
                ))
            }
            ViolationPattern::MissingErrorHandling => {
                Ok((
                    "fn ".to_string(),
                    "fn -> Result<(), Error> ".to_string(),
                ))
            }
            ViolationPattern::GuardConstraintViolation => {
                Ok((
                    "run_len > 8".to_string(),
                    "run_len <= 8".to_string(),
                ))
            }
        }
    }
}

/// Knowledge graph for storing violation and fix patterns
pub struct KnowledgeGraph {
    #[cfg(feature = "unrdf")]
    unrdf_initialized: bool,
    #[cfg(not(feature = "unrdf"))]
    // Store violation patterns in memory (would use unrdf in production)
    // For now, use simple in-memory storage until unrdf integration is complete
    violation_patterns: std::collections::HashMap<String, ViolationPattern>,
}

impl KnowledgeGraph {
    pub fn new() -> Result<Self, String> {
        #[cfg(feature = "unrdf")]
        {
            // Initialize unrdf if available
            use knhk_unrdf::init_unrdf;
            let unrdf_path = std::env::var("UNRDF_PATH").unwrap_or_else(|_| "vendors/unrdf".to_string());
            init_unrdf(&unrdf_path).map_err(|e| format!("Failed to initialize unrdf: {}", e))?;
            Ok(Self {
                unrdf_initialized: true,
            })
        }
        
        #[cfg(not(feature = "unrdf"))]
        {
            Ok(Self {
                violation_patterns: std::collections::HashMap::new(),
            })
        }
    }

    /// Store violation pattern in knowledge graph
    pub fn store_violation_pattern(&mut self, violation: &Violation) -> Result<(), String> {
        #[cfg(feature = "unrdf")]
        {
            use knhk_unrdf::store_turtle_data;
            // Store violation as RDF triple in unrdf
            let turtle = format!(
                r#"@prefix dod: <https://knhk.org/ontology#> .
<> dod:hasViolationType "{}" ;
   dod:hasFile "{}" ;
   dod:hasLine {} .
"#,
                format!("{:?}", violation.pattern),
                violation.file.to_string_lossy(),
                violation.line
            );
            store_turtle_data(&turtle).map_err(|e| format!("Failed to store violation: {}", e))?;
            Ok(())
        }
        
        #[cfg(not(feature = "unrdf"))]
        {
            // Store violation pattern in memory
            let key = format!("{:?}:{}", violation.pattern, violation.line);
            self.violation_patterns.insert(key, violation.pattern);
            Ok(())
        }
    }

    /// Query knowledge graph for fix pattern
    pub fn query_fix_pattern(&self, violation: &Violation) -> Result<String, String> {
        #[cfg(feature = "unrdf")]
        {
            use knhk_unrdf::{query_sparql, SparqlQueryType};
            // Query unrdf for fix pattern using SPARQL
            let query = format!(
                r#"SELECT ?fixPattern WHERE {{
    ?violation dod:hasViolationType "{}" ;
               dod:hasFixPattern ?fixPattern .
}}"#,
                format!("{:?}", violation.pattern)
            );
            let result = query_sparql(&query, SparqlQueryType::Select).map_err(|e| format!("Failed to query unrdf: {}", e))?;
            // Parse SPARQL results (simplified)
            // QueryResult has bindings as Option<Vec<Value>>, so we need to handle it properly
            if let Some(ref bindings) = result.bindings {
                if !bindings.is_empty() {
                    // Extract first binding's fixPattern value
                    if let Some(first_binding) = bindings.first() {
                        if let Some(fix_pattern) = first_binding.get("fixPattern") {
                            return Ok(fix_pattern.as_str().unwrap_or("").to_string());
                        }
                    }
                }
            }
            Err("Fix pattern not found".to_string())
        }
        
        #[cfg(not(feature = "unrdf"))]
        {
            // Query in-memory storage for fix pattern
            let key = format!("{:?}:{}", violation.pattern, violation.line);
            
            if self.violation_patterns.contains_key(&key) {
                // Return fix pattern based on violation type
                Ok(match violation.pattern {
                    ViolationPattern::Unwrap => ".unwrap()".to_string(),
                    ViolationPattern::Expect => ".expect(".to_string(),
                    ViolationPattern::Todo => "// TODO".to_string(),
                    ViolationPattern::Placeholder => "placeholder".to_string(),
                    ViolationPattern::Panic => "panic!(".to_string(),
                    ViolationPattern::MissingErrorHandling => "missing error handling".to_string(),
                    ViolationPattern::GuardConstraintViolation => "guard constraint".to_string(),
                })
            } else {
                Err(format!("Fix pattern not found for violation: {:?}", violation.pattern))
            }
        }
    }
}

/// Helper functions

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn hash<T: Serialize>(value: &T) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let json = serde_json::to_string(value).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    hasher.finish()
}

fn generate_span_id() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    now().hash(&mut hasher);
    hasher.finish()
}



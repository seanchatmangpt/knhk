//! Neural Pattern Learning System for ggen v2.7.1
//!
//! Implements a hyper-advanced pattern learning system that:
//! - Learns from previously generated code patterns
//! - Recommends optimal patterns for new generation requests
//! - Tracks pattern quality and success rates
//! - Provides adaptive learning based on feedback
//!
//! # Architecture
//!
//! - **Pattern Recognition Engine**: Extracts and matches code patterns
//! - **Pattern Memory System**: LRU cache + persistent storage (sled)
//! - **Code Quality Metrics**: Tracks pattern success rates
//! - **Learning Feedback Loop**: Updates pattern weights based on outcomes
//! - **Pattern Recommendation**: Suggests best patterns with confidence scores
//!
//! # Example
//!
//! ```rust,no_run
//! use knhk_workflow_engine::ggen::neural_patterns::{NeuralPatternLearner, Pattern};
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut learner = NeuralPatternLearner::new(Path::new("./pattern_db"))?;
//!
//! // Learn from generated code
//! let code = "fn process_data(input: Vec<u8>) -> Result<Output, Error> { ... }";
//! learner.learn_from_code(code, 0.95)?;
//!
//! // Get pattern recommendations
//! let recommendations = learner.recommend_patterns("create error handling function")?;
//! for rec in &recommendations {
//!     println!("Pattern: {} (confidence: {:.2})", rec.pattern.id, rec.confidence);
//! }
//! # Ok(())
//! # }
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use lru::LruCache;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, info, instrument, warn};

/// Maximum number of patterns to keep in LRU cache
const PATTERN_CACHE_SIZE: usize = 1000;

/// Pattern decay rate (per day) - recent patterns prioritized
const PATTERN_DECAY_RATE: f64 = 0.01;

/// Minimum success rate for a pattern to remain active
const MIN_SUCCESS_RATE: f64 = 0.3;

/// Minimum usage count before retiring a failing pattern
const MIN_USAGE_FOR_RETIREMENT: u64 = 10;

/// Target programming language for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetLanguage {
    /// Rust programming language
    Rust,
    /// Python programming language
    Python,
    /// JavaScript programming language
    JavaScript,
    /// Go programming language
    Go,
    /// TypeScript programming language
    TypeScript,
    /// Generic/Unknown language
    Generic,
}

impl TargetLanguage {
    /// Detect language from code content
    #[instrument(skip(code))]
    fn detect_from_code(code: &str) -> Self {
        if code.contains("fn ") && code.contains("->") {
            Self::Rust
        } else if code.contains("def ") && code.contains(":") {
            Self::Python
        } else if code.contains("function ") || code.contains("const ") {
            Self::JavaScript
        } else if code.contains("func ") && code.contains("package ") {
            Self::Go
        } else if code.contains("interface ") && code.contains(": ") {
            Self::TypeScript
        } else {
            Self::Generic
        }
    }
}

/// Code pattern with metadata and usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    /// Unique pattern identifier
    pub id: String,
    /// Code template for this pattern
    pub code_template: String,
    /// Context requirements for using this pattern
    pub context_requirements: Vec<String>,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Number of times pattern has been used
    pub usage_count: u64,
    /// Target programming language
    pub language: TargetLanguage,
    /// Pattern keywords for matching
    keywords: Vec<String>,
    /// Last used timestamp (seconds since epoch)
    last_used: u64,
    /// Total quality score accumulated
    total_quality: f64,
}

impl Pattern {
    /// Create a new pattern from code
    #[instrument(skip(code))]
    pub fn from_code(code: &str, language: TargetLanguage) -> Self {
        let id = Self::generate_id(code);
        let keywords = Self::extract_keywords(code);

        Pattern {
            id,
            code_template: code.to_string(),
            context_requirements: Vec::new(),
            success_rate: 0.5, // Start with neutral score
            usage_count: 0,
            language,
            keywords,
            last_used: Self::current_timestamp(),
            total_quality: 0.0,
        }
    }

    /// Generate unique ID from code using SHA256 hash
    fn generate_id(code: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        let result = hasher.finalize();
        format!("pat_{}", hex::encode(&result[..8]))
    }

    /// Extract keywords from code for pattern matching
    fn extract_keywords(code: &str) -> Vec<String> {
        let mut keywords = Vec::new();

        // Common programming keywords to extract
        let patterns = [
            "fn ",
            "def ",
            "function ",
            "class ",
            "struct ",
            "impl ",
            "trait ",
            "async ",
            "await ",
            "Result<",
            "Option<",
            "Vec<",
            "HashMap<",
            "error",
            "handle",
            "process",
            "parse",
            "validate",
            "convert",
        ];

        for pattern in &patterns {
            if code.contains(pattern) {
                keywords.push(pattern.trim().to_string());
            }
        }

        keywords
    }

    /// Get current timestamp in seconds since epoch
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }

    /// Calculate time-decayed score (recent patterns prioritized)
    #[instrument(skip(self))]
    pub fn calculate_score(&self) -> f64 {
        let age_days = (Self::current_timestamp() - self.last_used) / 86400;
        let decay = (-PATTERN_DECAY_RATE * age_days as f64).exp();
        self.success_rate * decay * (1.0 + (self.usage_count as f64).ln())
    }

    /// Update pattern with new quality feedback
    #[instrument(skip(self))]
    pub fn update_quality(&mut self, quality: f64) {
        self.usage_count += 1;
        self.total_quality += quality;
        self.success_rate = self.total_quality / self.usage_count as f64;
        self.last_used = Self::current_timestamp();

        debug!(
            pattern_id = %self.id,
            success_rate = %self.success_rate,
            usage_count = %self.usage_count,
            "Updated pattern quality"
        );
    }

    /// Check if pattern should be retired (poor performance)
    pub fn should_retire(&self) -> bool {
        self.usage_count >= MIN_USAGE_FOR_RETIREMENT && self.success_rate < MIN_SUCCESS_RATE
    }

    /// Calculate similarity score with task description (0.0 to 1.0)
    #[instrument(skip(self, task))]
    fn similarity_with_task(&self, task: &str) -> f64 {
        let task_lower = task.to_lowercase();
        let mut matches = 0;
        let mut total = self.keywords.len().max(1);

        for keyword in &self.keywords {
            if task_lower.contains(&keyword.to_lowercase()) {
                matches += 1;
            }
        }

        // Bonus for language match
        let lang_match = match self.language {
            TargetLanguage::Rust if task_lower.contains("rust") => 0.2,
            TargetLanguage::Python if task_lower.contains("python") => 0.2,
            TargetLanguage::JavaScript
                if task_lower.contains("javascript") || task_lower.contains("js") =>
            {
                0.2
            }
            TargetLanguage::Go if task_lower.contains("go") || task_lower.contains("golang") => 0.2,
            _ => 0.0,
        };

        (matches as f64 / total as f64) + lang_match
    }
}

/// Pattern recommendation with confidence score and rationale
#[derive(Debug, Clone)]
pub struct PatternRecommendation {
    /// Recommended pattern
    pub pattern: Pattern,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Human-readable rationale for recommendation
    pub rationale: String,
    /// Alternative patterns to consider
    pub alternative_patterns: Vec<Pattern>,
}

/// Pattern statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatistics {
    /// Total number of patterns in library
    pub total_patterns: usize,
    /// Number of active patterns
    pub active_patterns: usize,
    /// Number of retired patterns
    pub retired_patterns: usize,
    /// Average success rate across all patterns
    pub avg_success_rate: f64,
    /// Total pattern usages
    pub total_usages: u64,
    /// Patterns by language
    pub patterns_by_language: HashMap<String, usize>,
}

/// Generation context for applying patterns
#[derive(Debug, Clone, Default)]
pub struct Context {
    /// Context variables
    pub variables: HashMap<String, String>,
}

impl Context {
    /// Create new empty context
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Insert a variable
    pub fn insert(&mut self, key: String, value: String) {
        self.variables.insert(key, value);
    }
}

/// Neural pattern learner - main learning system
pub struct NeuralPatternLearner {
    /// LRU cache of most-used patterns (in-memory)
    pattern_cache: Arc<RwLock<LruCache<String, Pattern>>>,
    /// Persistent storage for all patterns
    storage: Arc<RwLock<Option<sled::Db>>>,
    /// Pattern library (all patterns keyed by ID)
    pattern_library: Arc<RwLock<HashMap<String, Pattern>>>,
    /// Retired patterns (poor performers)
    retired_patterns: Arc<RwLock<HashMap<String, Pattern>>>,
}

impl NeuralPatternLearner {
    /// Create a new neural pattern learner
    #[instrument(skip(storage_path))]
    pub fn new(storage_path: &Path) -> WorkflowResult<Self> {
        info!("Initializing neural pattern learner at {:?}", storage_path);

        // Initialize sled database
        let db = sled::open(storage_path).map_err(|e| {
            WorkflowError::Internal(format!("Failed to open pattern storage: {}", e))
        })?;

        // Create LRU cache
        let cache_size = NonZeroUsize::new(PATTERN_CACHE_SIZE)
            .ok_or_else(|| WorkflowError::Internal("Invalid cache size".to_string()))?;
        let pattern_cache = Arc::new(RwLock::new(LruCache::new(cache_size)));

        let storage = Arc::new(RwLock::new(Some(db)));
        let pattern_library = Arc::new(RwLock::new(HashMap::new()));
        let retired_patterns = Arc::new(RwLock::new(HashMap::new()));

        let learner = Self {
            pattern_cache,
            storage,
            pattern_library,
            retired_patterns,
        };

        // Load existing patterns from storage
        learner.load_from_storage()?;

        info!("Neural pattern learner initialized successfully");
        Ok(learner)
    }

    /// Learn from generated code with quality feedback
    #[instrument(skip(self, code))]
    pub fn learn_from_code(&mut self, code: &str, quality: f64) -> WorkflowResult<()> {
        if quality < 0.0 || quality > 1.0 {
            return Err(WorkflowError::Validation(
                "Quality score must be between 0.0 and 1.0".to_string(),
            ));
        }

        let language = TargetLanguage::detect_from_code(code);
        let pattern_id = Pattern::generate_id(code);

        debug!(pattern_id = %pattern_id, quality = %quality, "Learning from code");

        // Check if pattern exists
        let mut library = self.pattern_library.write();
        if let Some(existing) = library.get_mut(&pattern_id) {
            existing.update_quality(quality);

            // Update cache
            let mut cache = self.pattern_cache.write();
            cache.put(pattern_id.clone(), existing.clone());

            info!(pattern_id = %pattern_id, "Updated existing pattern");
        } else {
            // Create new pattern
            let mut pattern = Pattern::from_code(code, language);
            pattern.update_quality(quality);

            // Store in library and cache
            library.insert(pattern_id.clone(), pattern.clone());
            let mut cache = self.pattern_cache.write();
            cache.put(pattern_id.clone(), pattern);

            info!(pattern_id = %pattern_id, "Created new pattern");
        }

        // Persist to storage
        self.persist_pattern(&pattern_id)?;

        Ok(())
    }

    /// Recommend patterns for a given task
    #[instrument(skip(self, task))]
    pub fn recommend_patterns(&self, task: &str) -> WorkflowResult<Vec<PatternRecommendation>> {
        debug!(task = %task, "Generating pattern recommendations");

        let library = self.pattern_library.read();
        let mut scored_patterns: Vec<(Pattern, f64)> = Vec::new();

        // Score all patterns against task
        for pattern in library.values() {
            if pattern.should_retire() {
                continue; // Skip patterns that should be retired
            }

            let quality_score = pattern.calculate_score();
            let similarity_score = pattern.similarity_with_task(task);
            let combined_score = quality_score * 0.6 + similarity_score * 0.4;

            scored_patterns.push((pattern.clone(), combined_score));
        }

        // Sort by score (descending)
        scored_patterns.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top 5 recommendations
        let recommendations: Vec<PatternRecommendation> = scored_patterns
            .iter()
            .take(5)
            .map(|(pattern, score)| {
                let alternatives = scored_patterns
                    .iter()
                    .skip(1)
                    .take(3)
                    .map(|(p, _)| p.clone())
                    .collect();

                PatternRecommendation {
                    pattern: pattern.clone(),
                    confidence: *score,
                    rationale: format!(
                        "Pattern {} has {:.1}% success rate with {} uses, similarity score: {:.2}",
                        pattern.id,
                        pattern.success_rate * 100.0,
                        pattern.usage_count,
                        pattern.similarity_with_task(task)
                    ),
                    alternative_patterns: alternatives,
                }
            })
            .collect();

        info!(
            task = %task,
            recommendation_count = %recommendations.len(),
            "Generated pattern recommendations"
        );

        Ok(recommendations)
    }

    /// Apply a pattern to a context to generate code
    #[instrument(skip(self, pattern, context))]
    pub fn apply_pattern(&self, pattern: &Pattern, context: &Context) -> WorkflowResult<String> {
        debug!(pattern_id = %pattern.id, "Applying pattern");

        let mut code = pattern.code_template.clone();

        // Simple template variable substitution
        for (key, value) in &context.variables {
            let placeholder = format!("{{{{{}}}}}", key);
            code = code.replace(&placeholder, value);
        }

        Ok(code)
    }

    /// Get pattern statistics
    #[instrument(skip(self))]
    pub fn get_pattern_stats(&self) -> WorkflowResult<PatternStatistics> {
        let library = self.pattern_library.read();
        let retired = self.retired_patterns.read();

        let total_patterns = library.len();
        let retired_patterns = retired.len();
        let active_patterns = total_patterns - retired_patterns;

        let mut total_success_rate = 0.0;
        let mut total_usages = 0u64;
        let mut patterns_by_language: HashMap<String, usize> = HashMap::new();

        for pattern in library.values() {
            total_success_rate += pattern.success_rate;
            total_usages += pattern.usage_count;

            let lang = format!("{:?}", pattern.language);
            *patterns_by_language.entry(lang).or_insert(0) += 1;
        }

        let avg_success_rate = if total_patterns > 0 {
            total_success_rate / total_patterns as f64
        } else {
            0.0
        };

        Ok(PatternStatistics {
            total_patterns,
            active_patterns,
            retired_patterns,
            avg_success_rate,
            total_usages,
            patterns_by_language,
        })
    }

    /// Save all patterns to persistent storage
    #[instrument(skip(self))]
    pub fn save_learning(&self) -> WorkflowResult<()> {
        info!("Saving pattern learning to persistent storage");

        let library = self.pattern_library.read();

        for (id, pattern) in library.iter() {
            self.persist_pattern(id)?;
        }

        // Flush storage
        let storage = self.storage.read();
        if let Some(ref db) = *storage {
            db.flush()
                .map_err(|e| WorkflowError::Internal(format!("Failed to flush storage: {}", e)))?;
        }

        info!("Pattern learning saved successfully");
        Ok(())
    }

    /// Retire patterns that consistently fail
    #[instrument(skip(self))]
    pub fn retire_failing_patterns(&mut self) -> WorkflowResult<usize> {
        let mut library = self.pattern_library.write();
        let mut retired = self.retired_patterns.write();

        let mut retired_count = 0;
        let mut to_retire = Vec::new();

        for (id, pattern) in library.iter() {
            if pattern.should_retire() {
                to_retire.push(id.clone());
            }
        }

        for id in to_retire {
            if let Some(pattern) = library.remove(&id) {
                warn!(
                    pattern_id = %id,
                    success_rate = %pattern.success_rate,
                    usage_count = %pattern.usage_count,
                    "Retiring failing pattern"
                );
                retired.insert(id, pattern);
                retired_count += 1;
            }
        }

        info!(retired_count = %retired_count, "Retired failing patterns");
        Ok(retired_count)
    }

    /// Load patterns from persistent storage
    #[instrument(skip(self))]
    fn load_from_storage(&self) -> WorkflowResult<()> {
        let storage = self.storage.read();
        let Some(ref db) = *storage else {
            return Ok(());
        };

        let mut library = self.pattern_library.write();
        let mut loaded_count = 0;

        for item in db.iter() {
            let (key, value) = item.map_err(|e| {
                WorkflowError::Internal(format!("Failed to read from storage: {}", e))
            })?;

            let pattern: Pattern = bincode::deserialize(&value).map_err(|e| {
                WorkflowError::Internal(format!("Failed to deserialize pattern: {}", e))
            })?;

            let id = String::from_utf8(key.to_vec()).map_err(|e| {
                WorkflowError::Internal(format!("Invalid pattern ID in storage: {}", e))
            })?;

            library.insert(id, pattern);
            loaded_count += 1;
        }

        info!(loaded_count = %loaded_count, "Loaded patterns from storage");
        Ok(())
    }

    /// Persist a single pattern to storage
    #[instrument(skip(self, pattern_id))]
    fn persist_pattern(&self, pattern_id: &str) -> WorkflowResult<()> {
        let library = self.pattern_library.read();
        let Some(pattern) = library.get(pattern_id) else {
            return Err(WorkflowError::Internal(format!(
                "Pattern {} not found in library",
                pattern_id
            )));
        };

        let storage = self.storage.read();
        let Some(ref db) = *storage else {
            return Ok(());
        };

        let serialized = bincode::serialize(pattern)
            .map_err(|e| WorkflowError::Internal(format!("Failed to serialize pattern: {}", e)))?;

        db.insert(pattern_id.as_bytes(), serialized)
            .map_err(|e| WorkflowError::Internal(format!("Failed to persist pattern: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_pattern_creation() {
        let code = r#"fn process_data(input: Vec<u8>) -> Result<Output, Error> { Ok(Output) }"#;
        let pattern = Pattern::from_code(code, TargetLanguage::Rust);

        assert_eq!(pattern.language, TargetLanguage::Rust);
        assert_eq!(pattern.success_rate, 0.5);
        assert_eq!(pattern.usage_count, 0);
        assert!(!pattern.keywords.is_empty());
    }

    #[test]
    fn test_pattern_quality_update() {
        let code = r#"fn test() { }"#;
        let mut pattern = Pattern::from_code(code, TargetLanguage::Rust);

        pattern.update_quality(0.9);
        assert_eq!(pattern.usage_count, 1);
        assert_eq!(pattern.success_rate, 0.9);

        pattern.update_quality(0.8);
        assert_eq!(pattern.usage_count, 2);
        assert_eq!(pattern.success_rate, 0.85);
    }

    #[test]
    fn test_learner_creation() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let learner = NeuralPatternLearner::new(temp_dir.path());

        assert!(learner.is_ok());
    }

    #[test]
    fn test_learn_from_code() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut learner =
            NeuralPatternLearner::new(temp_dir.path()).expect("Failed to create learner");

        let code = r#"fn error_handler(e: Error) -> Result<(), Error> { Err(e) }"#;
        let result = learner.learn_from_code(code, 0.95);

        assert!(result.is_ok());
    }

    #[test]
    fn test_pattern_recommendations() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut learner =
            NeuralPatternLearner::new(temp_dir.path()).expect("Failed to create learner");

        // Learn from some code
        let code1 = r#"fn handle_error(e: Error) -> Result<(), Error> { Err(e) }"#;
        let code2 =
            r#"async fn process_async(data: Vec<u8>) -> Result<Output, Error> { Ok(Output) }"#;

        learner
            .learn_from_code(code1, 0.9)
            .expect("Failed to learn");
        learner
            .learn_from_code(code2, 0.85)
            .expect("Failed to learn");

        // Get recommendations
        let recommendations = learner
            .recommend_patterns("create error handling function")
            .expect("Failed to get recommendations");

        assert!(!recommendations.is_empty());
    }

    #[test]
    fn test_pattern_statistics() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let mut learner =
            NeuralPatternLearner::new(temp_dir.path()).expect("Failed to create learner");

        let code = r#"fn test() { }"#;
        learner.learn_from_code(code, 0.8).expect("Failed to learn");

        let stats = learner.get_pattern_stats().expect("Failed to get stats");

        assert_eq!(stats.total_patterns, 1);
        assert!(stats.avg_success_rate > 0.0);
    }

    #[test]
    fn test_apply_pattern() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let learner = NeuralPatternLearner::new(temp_dir.path()).expect("Failed to create learner");

        let code = r#"fn {{function_name}}(input: {{input_type}}) -> {{output_type}} { todo!() }"#;
        let pattern = Pattern::from_code(code, TargetLanguage::Rust);

        let mut context = Context::new();
        context.insert("function_name".to_string(), "process".to_string());
        context.insert("input_type".to_string(), "String".to_string());
        context.insert("output_type".to_string(), "Result<(), Error>".to_string());

        let result = learner.apply_pattern(&pattern, &context);

        assert!(result.is_ok());
        let generated = result.expect("Failed to apply pattern");
        assert!(generated.contains("fn process"));
        assert!(generated.contains("input: String"));
    }
}

// kernel/knowledge_integration.rs - MAPE-K hook integration and learning feedback
// Phase 3: Persistent learning integration with MAPE-K
// DOCTRINE: Covenant 3 (MAPE-K Is Recursive) - Learning feeds back without blocking

use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::collections::{HashMap, VecDeque};
use parking_lot::{Mutex, RwLock};
use dashmap::DashMap;
use tracing::{debug, error, info, warn};
use serde::{Serialize, Deserialize};
use bincode;

/// MAPE-K phases
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum MAPEKPhase {
    Monitor,
    Analyze,
    Plan,
    Execute,
    Knowledge,
}

/// Hook point in the execution flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPoint {
    pub id: String,
    pub phase: MAPEKPhase,
    pub location: String,
    pub trigger_condition: TriggerCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    Always,
    OnError,
    OnSuccess,
    OnThreshold(f64),
    OnPattern(String),
    Custom(String),
}

/// Learning pattern discovered by the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnedPattern {
    pub id: String,
    pub pattern_type: PatternType,
    pub confidence: f64,
    pub occurrences: u64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub features: Vec<Feature>,
    pub outcomes: Vec<Outcome>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Performance,
    Error,
    Load,
    Behavioral,
    Temporal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub value: FeatureValue,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeatureValue {
    Numeric(f64),
    Categorical(String),
    Binary(bool),
    Vector(Vec<f64>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub success: bool,
    pub latency_us: u64,
    pub resource_usage: f64,
    pub quality_score: f64,
}

/// Knowledge base for persistent learning
pub struct KnowledgeBase {
    patterns: Arc<DashMap<String, LearnedPattern>>,
    predictions: Arc<DashMap<String, Prediction>>,
    feedback_queue: Arc<Mutex<VecDeque<FeedbackItem>>>,
    model_store: Arc<RwLock<HashMap<String, PredictiveModel>>>,
    learning_enabled: Arc<AtomicBool>,
    stats: Arc<KnowledgeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub id: String,
    pub timestamp: u64,
    pub predicted_value: f64,
    pub confidence: f64,
    pub model_id: String,
    pub actual_value: Option<f64>,
    pub error: Option<f64>,
}

#[derive(Debug, Clone)]
struct FeedbackItem {
    pub prediction_id: String,
    pub actual_value: f64,
    pub timestamp: u64,
    pub context: HashMap<String, String>,
}

#[derive(Debug)]
struct KnowledgeStats {
    patterns_learned: AtomicU64,
    predictions_made: AtomicU64,
    successful_predictions: AtomicU64,
    feedback_processed: AtomicU64,
    model_updates: AtomicU64,
    learning_iterations: AtomicU64,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        Self {
            patterns: Arc::new(DashMap::new()),
            predictions: Arc::new(DashMap::new()),
            feedback_queue: Arc::new(Mutex::new(VecDeque::with_capacity(1000))),
            model_store: Arc::new(RwLock::new(HashMap::new())),
            learning_enabled: Arc::new(AtomicBool::new(true)),
            stats: Arc::new(KnowledgeStats {
                patterns_learned: AtomicU64::new(0),
                predictions_made: AtomicU64::new(0),
                successful_predictions: AtomicU64::new(0),
                feedback_processed: AtomicU64::new(0),
                model_updates: AtomicU64::new(0),
                learning_iterations: AtomicU64::new(0),
            }),
        }
    }

    /// Record a new pattern
    pub fn record_pattern(&self, pattern: LearnedPattern) {
        let pattern_id = pattern.id.clone();

        self.patterns
            .entry(pattern_id.clone())
            .and_modify(|p| {
                p.occurrences += 1;
                p.last_seen = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                p.confidence = (p.confidence * p.occurrences as f64 + pattern.confidence)
                    / (p.occurrences + 1) as f64;
            })
            .or_insert_with(|| {
                self.stats.patterns_learned.fetch_add(1, Ordering::Relaxed);
                pattern
            });

        debug!("Recorded pattern: {}", pattern_id);
    }

    /// Make a prediction
    pub fn predict(&self, model_id: &str, features: &[Feature]) -> Option<Prediction> {
        let models = self.model_store.read();
        let model = models.get(model_id)?;

        let prediction_value = model.predict(features);
        let confidence = model.confidence(features);

        let prediction = Prediction {
            id: format!("pred-{}", uuid::Uuid::new_v4()),
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            predicted_value: prediction_value,
            confidence,
            model_id: model_id.to_string(),
            actual_value: None,
            error: None,
        };

        self.predictions.insert(prediction.id.clone(), prediction.clone());
        self.stats.predictions_made.fetch_add(1, Ordering::Relaxed);

        Some(prediction)
    }

    /// Provide feedback on a prediction
    pub fn provide_feedback(&self, prediction_id: String, actual_value: f64) {
        if let Some(mut prediction) = self.predictions.get_mut(&prediction_id) {
            prediction.actual_value = Some(actual_value);
            prediction.error = Some((prediction.predicted_value - actual_value).abs());

            if prediction.error.unwrap() < 0.1 {
                self.stats.successful_predictions.fetch_add(1, Ordering::Relaxed);
            }
        }

        let feedback = FeedbackItem {
            prediction_id,
            actual_value,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            context: HashMap::new(),
        };

        self.feedback_queue.lock().push_back(feedback);
        self.stats.feedback_processed.fetch_add(1, Ordering::Relaxed);
    }

    /// Process feedback queue and update models
    pub fn process_feedback(&self) {
        if !self.learning_enabled.load(Ordering::Acquire) {
            return;
        }

        let mut queue = self.feedback_queue.lock();
        let feedback_items: Vec<_> = queue.drain(..).collect();
        drop(queue);

        if feedback_items.is_empty() {
            return;
        }

        let mut models = self.model_store.write();

        for feedback in feedback_items {
            if let Some(prediction) = self.predictions.get(&feedback.prediction_id) {
                if let Some(model) = models.get_mut(&prediction.model_id) {
                    model.update(prediction.predicted_value, feedback.actual_value);
                    self.stats.model_updates.fetch_add(1, Ordering::Relaxed);
                }
            }
        }

        self.stats.learning_iterations.fetch_add(1, Ordering::Relaxed);
    }

    /// Get pattern by ID
    pub fn get_pattern(&self, pattern_id: &str) -> Option<LearnedPattern> {
        self.patterns.get(pattern_id).map(|p| p.clone())
    }

    /// Find patterns matching criteria
    pub fn find_patterns(&self, pattern_type: PatternType, min_confidence: f64) -> Vec<LearnedPattern> {
        self.patterns
            .iter()
            .filter(|p| p.pattern_type == pattern_type && p.confidence >= min_confidence)
            .map(|p| p.clone())
            .collect()
    }

    /// Enable or disable learning
    pub fn set_learning_enabled(&self, enabled: bool) {
        self.learning_enabled.store(enabled, Ordering::Release);
    }

    /// Get statistics
    pub fn get_stats(&self) -> KnowledgeStatistics {
        KnowledgeStatistics {
            patterns_learned: self.stats.patterns_learned.load(Ordering::Relaxed),
            predictions_made: self.stats.predictions_made.load(Ordering::Relaxed),
            successful_predictions: self.stats.successful_predictions.load(Ordering::Relaxed),
            feedback_processed: self.stats.feedback_processed.load(Ordering::Relaxed),
            model_updates: self.stats.model_updates.load(Ordering::Relaxed),
            learning_iterations: self.stats.learning_iterations.load(Ordering::Relaxed),
            total_patterns: self.patterns.len(),
            total_models: self.model_store.read().len(),
        }
    }
}

/// Predictive model for learning
#[derive(Debug, Clone)]
pub struct PredictiveModel {
    pub id: String,
    pub model_type: ModelType,
    weights: Vec<f64>,
    bias: f64,
    learning_rate: f64,
    samples: VecDeque<(f64, f64)>,
    max_samples: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Linear,
    Exponential,
    MovingAverage,
    Neural,
}

impl PredictiveModel {
    pub fn new(id: String, model_type: ModelType) -> Self {
        Self {
            id,
            model_type,
            weights: vec![0.5; 10], // Initial weights
            bias: 0.0,
            learning_rate: 0.01,
            samples: VecDeque::with_capacity(1000),
            max_samples: 1000,
        }
    }

    pub fn predict(&self, features: &[Feature]) -> f64 {
        match self.model_type {
            ModelType::Linear => {
                let mut sum = self.bias;
                for (i, feature) in features.iter().enumerate() {
                    if i < self.weights.len() {
                        sum += self.weights[i] * self.feature_to_value(feature);
                    }
                }
                sum
            }
            ModelType::MovingAverage => {
                if self.samples.is_empty() {
                    0.0
                } else {
                    let window = 10.min(self.samples.len());
                    let recent: Vec<_> = self.samples.iter().rev().take(window).collect();
                    recent.iter().map(|(_, v)| v).sum::<f64>() / window as f64
                }
            }
            _ => {
                // Simplified implementation for other types
                self.bias
            }
        }
    }

    pub fn confidence(&self, _features: &[Feature]) -> f64 {
        // Simplified confidence calculation
        let sample_ratio = self.samples.len() as f64 / self.max_samples as f64;
        sample_ratio.min(1.0) * 0.9 + 0.1
    }

    pub fn update(&mut self, predicted: f64, actual: f64) {
        // Store sample
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back((predicted, actual));

        // Update weights based on error
        let error = actual - predicted;

        match self.model_type {
            ModelType::Linear => {
                // Simple gradient descent update
                self.bias += self.learning_rate * error;
                for weight in &mut self.weights {
                    *weight += self.learning_rate * error * 0.1; // Simplified update
                }
            }
            _ => {
                // Other model types might have different update rules
            }
        }
    }

    fn feature_to_value(&self, feature: &Feature) -> f64 {
        match &feature.value {
            FeatureValue::Numeric(v) => *v,
            FeatureValue::Binary(b) => if *b { 1.0 } else { 0.0 },
            FeatureValue::Vector(v) => v.iter().sum::<f64>() / v.len() as f64,
            FeatureValue::Categorical(_) => 0.5, // Simplified
        }
    }
}

/// MAPE-K integration manager
pub struct MAPEKIntegration {
    hooks: Arc<DashMap<String, HookPoint>>,
    knowledge_base: Arc<KnowledgeBase>,
    execution_history: Arc<Mutex<VecDeque<ExecutionRecord>>>,
    success_tracker: Arc<SuccessTracker>,
    pattern_detector: Arc<PatternDetector>,
}

#[derive(Debug, Clone)]
struct ExecutionRecord {
    hook_id: String,
    phase: MAPEKPhase,
    timestamp: u64,
    duration_us: u64,
    success: bool,
    features: Vec<Feature>,
}

impl MAPEKIntegration {
    pub fn new(knowledge_base: Arc<KnowledgeBase>) -> Self {
        Self {
            hooks: Arc::new(DashMap::new()),
            knowledge_base,
            execution_history: Arc::new(Mutex::new(VecDeque::with_capacity(10000))),
            success_tracker: Arc::new(SuccessTracker::new()),
            pattern_detector: Arc::new(PatternDetector::new()),
        }
    }

    /// Register a hook point
    pub fn register_hook(&self, hook: HookPoint) {
        self.hooks.insert(hook.id.clone(), hook);
    }

    /// Execute hook with learning
    pub fn execute_hook<F, T>(&self, hook_id: &str, f: F) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let hook = self.hooks.get(hook_id)
            .ok_or_else(|| format!("Hook {} not found", hook_id))?;

        let start = Instant::now();
        let result = f();
        let duration = start.elapsed().as_micros() as u64;

        // Record execution
        let record = ExecutionRecord {
            hook_id: hook_id.to_string(),
            phase: hook.phase,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            duration_us: duration,
            success: result.is_ok(),
            features: self.extract_features(&hook.phase, duration),
        };

        self.execution_history.lock().push_back(record.clone());

        // Update success tracking
        self.success_tracker.record(hook_id, result.is_ok());

        // Detect patterns
        if let Some(pattern) = self.pattern_detector.detect(&record) {
            self.knowledge_base.record_pattern(pattern);
        }

        // Learn from execution
        self.learn_from_execution(&record);

        result
    }

    fn extract_features(&self, phase: &MAPEKPhase, duration_us: u64) -> Vec<Feature> {
        vec![
            Feature {
                name: "phase".to_string(),
                value: FeatureValue::Categorical(format!("{:?}", phase)),
                weight: 0.3,
            },
            Feature {
                name: "duration_us".to_string(),
                value: FeatureValue::Numeric(duration_us as f64),
                weight: 0.5,
            },
            Feature {
                name: "timestamp_hour".to_string(),
                value: FeatureValue::Numeric(
                    (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() / 3600 % 24) as f64
                ),
                weight: 0.2,
            },
        ]
    }

    fn learn_from_execution(&self, record: &ExecutionRecord) {
        // Make prediction for next execution
        if let Some(prediction) = self.knowledge_base.predict("latency_model", &record.features) {
            // Store prediction ID for later feedback
            debug!("Predicted latency: {}us", prediction.predicted_value);
        }

        // Process any pending feedback
        self.knowledge_base.process_feedback();
    }

    /// Get hook execution statistics
    pub fn get_hook_stats(&self, hook_id: &str) -> Option<HookStatistics> {
        self.success_tracker.get_stats(hook_id)
    }
}

/// Success rate tracker
struct SuccessTracker {
    success_counts: DashMap<String, (u64, u64)>, // (successes, total)
}

impl SuccessTracker {
    fn new() -> Self {
        Self {
            success_counts: DashMap::new(),
        }
    }

    fn record(&self, hook_id: &str, success: bool) {
        self.success_counts
            .entry(hook_id.to_string())
            .and_modify(|(s, t)| {
                if success {
                    *s += 1;
                }
                *t += 1;
            })
            .or_insert((if success { 1 } else { 0 }, 1));
    }

    fn get_stats(&self, hook_id: &str) -> Option<HookStatistics> {
        self.success_counts.get(hook_id).map(|entry| {
            let (successes, total) = *entry.value();
            HookStatistics {
                hook_id: hook_id.to_string(),
                total_executions: total,
                successful_executions: successes,
                success_rate: if total > 0 { successes as f64 / total as f64 } else { 0.0 },
            }
        })
    }
}

/// Pattern detector for discovering execution patterns
struct PatternDetector {
    min_occurrences: u64,
    confidence_threshold: f64,
}

impl PatternDetector {
    fn new() -> Self {
        Self {
            min_occurrences: 5,
            confidence_threshold: 0.7,
        }
    }

    fn detect(&self, record: &ExecutionRecord) -> Option<LearnedPattern> {
        // Simplified pattern detection
        if record.duration_us > 1000 {
            Some(LearnedPattern {
                id: format!("pattern-{}", uuid::Uuid::new_v4()),
                pattern_type: PatternType::Performance,
                confidence: 0.8,
                occurrences: 1,
                first_seen: record.timestamp,
                last_seen: record.timestamp,
                features: record.features.clone(),
                outcomes: vec![
                    Outcome {
                        success: record.success,
                        latency_us: record.duration_us,
                        resource_usage: 0.5,
                        quality_score: if record.success { 1.0 } else { 0.0 },
                    },
                ],
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeStatistics {
    pub patterns_learned: u64,
    pub predictions_made: u64,
    pub successful_predictions: u64,
    pub feedback_processed: u64,
    pub model_updates: u64,
    pub learning_iterations: u64,
    pub total_patterns: usize,
    pub total_models: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookStatistics {
    pub hook_id: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub success_rate: f64,
}

// UUID generation helper
mod uuid {
    pub struct Uuid;

    impl Uuid {
        pub fn new_v4() -> String {
            format!("{:x}-{:x}-{:x}-{:x}",
                rand::random::<u32>(),
                rand::random::<u16>(),
                rand::random::<u16>(),
                rand::random::<u32>()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base() {
        let kb = KnowledgeBase::new();

        let pattern = LearnedPattern {
            id: "test-pattern".to_string(),
            pattern_type: PatternType::Performance,
            confidence: 0.85,
            occurrences: 1,
            first_seen: 100,
            last_seen: 100,
            features: vec![],
            outcomes: vec![],
        };

        kb.record_pattern(pattern.clone());

        let retrieved = kb.get_pattern("test-pattern");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().confidence, 0.85);
    }

    #[test]
    fn test_predictive_model() {
        let mut model = PredictiveModel::new("test".to_string(), ModelType::Linear);

        let features = vec![
            Feature {
                name: "f1".to_string(),
                value: FeatureValue::Numeric(1.0),
                weight: 0.5,
            },
        ];

        let prediction = model.predict(&features);
        model.update(prediction, 2.0);

        assert!(model.samples.len() == 1);
    }

    #[test]
    fn test_mapek_integration() {
        let kb = Arc::new(KnowledgeBase::new());
        let integration = MAPEKIntegration::new(kb);

        let hook = HookPoint {
            id: "test-hook".to_string(),
            phase: MAPEKPhase::Monitor,
            location: "test".to_string(),
            trigger_condition: TriggerCondition::Always,
        };

        integration.register_hook(hook);

        let result = integration.execute_hook("test-hook", || {
            Ok::<(), String>(())
        });

        assert!(result.is_ok());

        let stats = integration.get_hook_stats("test-hook");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().success_rate, 1.0);
    }

    #[test]
    fn test_pattern_detection() {
        let detector = PatternDetector::new();

        let record = ExecutionRecord {
            hook_id: "test".to_string(),
            phase: MAPEKPhase::Execute,
            timestamp: 100,
            duration_us: 2000,
            success: true,
            features: vec![],
        };

        let pattern = detector.detect(&record);
        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().pattern_type, PatternType::Performance);
    }
}
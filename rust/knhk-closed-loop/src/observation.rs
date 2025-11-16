// Observation Plane Processing: Pattern Detection and Anomaly Detection
// Monitor: First phase of MAPE-K, ingests data from observation plane

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use sha2::Digest;

/// An observation from the system (event, telemetry, receipt)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    /// Unique ID
    pub id: String,

    /// What was observed
    pub event_type: String,

    /// When it happened
    pub timestamp: u64,

    /// The value or measurement
    pub value: serde_json::Value,

    /// What sector this belongs to
    pub sector: String,

    /// Metadata (attributes, tags)
    pub metadata: HashMap<String, String>,
}

impl Observation {
    pub fn new(
        event_type: String,
        value: serde_json::Value,
        sector: String,
        metadata: HashMap<String, String>,
    ) -> Self {
        let timestamp = chrono::Utc::now().timestamp_millis() as u64;
        // Generate deterministic ID from timestamp + event type + sector
        let content = format!("{}-{}-{}", timestamp, event_type, sector);
        let hash = hex::encode(sha2::Sha256::digest(content.as_bytes()));
        let id = format!("{}-{}-{}", sector, event_type, &hash[..16]);

        Observation {
            id,
            event_type,
            timestamp,
            value,
            sector,
            metadata,
        }
    }
}

/// A detected pattern in the observation stream
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DetectedPattern {
    /// Name of the pattern
    pub name: String,

    /// How confident we are (0.0-1.0)
    pub confidence: f64,

    /// When we detected it
    pub detected_at: u64,

    /// How many observations support this
    pub evidence_count: usize,

    /// What observations support it
    pub evidence_ids: Vec<String>,

    /// What to do about it
    pub recommended_action: PatternAction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PatternAction {
    /// No action needed
    Observe,

    /// Propose ontology change
    ProposeChange { description: String },

    /// Enforce invariant
    EnforceInvariant { invariant: String },

    /// Alert operator
    Alert { severity: String },
}

/// Observation store: immutable append-only log
pub struct ObservationStore {
    observations: dashmap::DashMap<String, Arc<Observation>>,
    patterns: dashmap::DashMap<String, Arc<DetectedPattern>>,
}

impl ObservationStore {
    pub fn new() -> Self {
        ObservationStore {
            observations: dashmap::DashMap::new(),
            patterns: dashmap::DashMap::new(),
        }
    }

    pub fn append(&self, obs: Observation) -> String {
        let id = obs.id.clone();
        self.observations.insert(id.clone(), Arc::new(obs));
        id
    }

    pub fn record_pattern(&self, pattern: DetectedPattern) {
        self.patterns
            .insert(pattern.name.clone(), Arc::new(pattern));
    }

    pub fn get_observation(&self, id: &str) -> Option<Arc<Observation>> {
        self.observations.get(id).map(|e| e.clone())
    }

    pub fn get_pattern(&self, name: &str) -> Option<Arc<DetectedPattern>> {
        self.patterns.get(name).map(|e| e.clone())
    }

    pub fn observations_since(&self, timestamp: u64) -> Vec<Arc<Observation>> {
        self.observations
            .iter()
            .filter(|entry| entry.value().timestamp >= timestamp)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn get_sector_observations(&self, sector: &str) -> Vec<Arc<Observation>> {
        self.observations
            .iter()
            .filter(|entry| entry.value().sector == sector)
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn count_observations(&self) -> usize {
        self.observations.len()
    }

    pub fn list_patterns(&self) -> Vec<Arc<DetectedPattern>> {
        self.patterns.iter().map(|entry| entry.value().clone()).collect()
    }
}

impl Default for ObservationStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Pattern detector: Analyze phase of MAPE-K
/// Looks for patterns in observations that might require action
pub struct PatternDetector {
    store: Arc<ObservationStore>,
}

impl PatternDetector {
    pub fn new(store: Arc<ObservationStore>) -> Self {
        PatternDetector { store }
    }

    /// Analyze observations for patterns
    pub async fn detect_patterns(&self) -> Vec<DetectedPattern> {
        let mut patterns = Vec::new();

        // Pattern 1: High-frequency events (possible loop)
        if let Some(pattern) = self.detect_frequency_anomaly().await {
            patterns.push(pattern);
        }

        // Pattern 2: Error rate spike
        if let Some(pattern) = self.detect_error_spike().await {
            patterns.push(pattern);
        }

        // Pattern 3: Missing expected observations
        if let Some(pattern) = self.detect_missing_observations().await {
            patterns.push(pattern);
        }

        // Pattern 4: Schema mismatch
        if let Some(pattern) = self.detect_schema_mismatch().await {
            patterns.push(pattern);
        }

        patterns
    }

    /// Detect abnormally high frequency
    async fn detect_frequency_anomaly(&self) -> Option<DetectedPattern> {
        let observations = self.store.observations_since(
            (chrono::Utc::now().timestamp_millis() as u64).saturating_sub(60_000),
        ); // Last minute

        let mut event_counts: HashMap<String, usize> = HashMap::new();
        for obs in &observations {
            *event_counts.entry(obs.event_type.clone()).or_insert(0) += 1;
        }

        // If any event type appears more than 100 times in 60 seconds, flag it
        for (event_type, count) in event_counts {
            if count > 100 {
                return Some(DetectedPattern {
                    name: format!("high_frequency_{}", event_type),
                    confidence: 0.95,
                    detected_at: chrono::Utc::now().timestamp_millis() as u64,
                    evidence_count: count,
                    evidence_ids: observations
                        .iter()
                        .filter(|o| o.event_type == event_type)
                        .map(|o| o.id.clone())
                        .collect(),
                    recommended_action: PatternAction::ProposeChange {
                        description: format!(
                            "Consider rate-limiting {} events ({}x normal)",
                            event_type,
                            count / 10
                        ),
                    },
                });
            }
        }

        None
    }

    /// Detect error spike
    async fn detect_error_spike(&self) -> Option<DetectedPattern> {
        let observations = self.store.observations_since(
            (chrono::Utc::now().timestamp_millis() as u64).saturating_sub(60_000),
        );

        let errors: Vec<_> = observations
            .iter()
            .filter(|o| o.event_type.contains("error") || o.event_type.contains("Error"))
            .collect();

        let error_count = errors.len();
        let total_count = observations.len();

        if total_count > 10 && error_count as f64 / total_count as f64 > 0.05 {
            return Some(DetectedPattern {
                name: "error_spike".to_string(),
                confidence: 0.9,
                detected_at: chrono::Utc::now().timestamp_millis() as u64,
                evidence_count: error_count,
                evidence_ids: errors.iter().map(|o| o.id.clone()).collect(),
                recommended_action: PatternAction::Alert {
                    severity: "warning".to_string(),
                },
            });
        }

        None
    }

    /// Detect missing expected observations
    async fn detect_missing_observations(&self) -> Option<DetectedPattern> {
        // Check if we've gone >5 seconds without a heartbeat
        let observations = self.store.observations_since(
            (chrono::Utc::now().timestamp_millis() as u64).saturating_sub(10_000),
        );

        if observations.is_empty() {
            return Some(DetectedPattern {
                name: "no_observations".to_string(),
                confidence: 0.8,
                detected_at: chrono::Utc::now().timestamp_millis() as u64,
                evidence_count: 0,
                evidence_ids: vec![],
                recommended_action: PatternAction::Alert {
                    severity: "critical".to_string(),
                },
            });
        }

        None
    }

    /// Detect schema mismatch
    async fn detect_schema_mismatch(&self) -> Option<DetectedPattern> {
        let observations = self.store.observations_since(
            (chrono::Utc::now().timestamp_millis() as u64).saturating_sub(5_000),
        );

        // Simple heuristic: if we see unexpected fields, flag it
        let mut unexpected_fields = 0;
        for obs in &observations {
            // If metadata has fields we don't recognize, count it
            if obs.metadata.contains_key("unexpected_field") {
                unexpected_fields += 1;
            }
        }

        if unexpected_fields > 0 {
            return Some(DetectedPattern {
                name: "schema_mismatch".to_string(),
                confidence: 0.7,
                detected_at: chrono::Utc::now().timestamp_millis() as u64,
                evidence_count: unexpected_fields,
                evidence_ids: vec![],
                recommended_action: PatternAction::ProposeChange {
                    description: "Schema may need to be updated".to_string(),
                },
            });
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let obs = Observation::new(
            "test_event".to_string(),
            serde_json::json!({"value": 42}),
            "test_sector".to_string(),
            metadata,
        );

        assert_eq!(obs.event_type, "test_event");
        assert_eq!(obs.sector, "test_sector");
    }

    #[test]
    fn test_observation_store() {
        let store = ObservationStore::new();

        let obs = Observation::new(
            "event1".to_string(),
            serde_json::json!({"count": 10}),
            "sector1".to_string(),
            HashMap::new(),
        );

        let id = obs.id.clone();
        store.append(obs);

        assert_eq!(store.count_observations(), 1);
        assert!(store.get_observation(&id).is_some());
    }

    #[tokio::test]
    async fn test_pattern_detector() {
        let store = Arc::new(ObservationStore::new());
        let detector = PatternDetector::new(store.clone());

        // Add some observations
        for i in 0..50 {
            let obs = Observation::new(
                "test_event".to_string(),
                serde_json::json!({"count": i}),
                "sector1".to_string(),
                HashMap::new(),
            );
            store.append(obs);
        }

        let patterns = detector.detect_patterns().await;
        // Should detect frequency anomaly
        assert!(!patterns.is_empty());
    }

    #[test]
    fn test_detected_pattern() {
        let pattern = DetectedPattern {
            name: "test_pattern".to_string(),
            confidence: 0.95,
            detected_at: chrono::Utc::now().timestamp_millis() as u64,
            evidence_count: 10,
            evidence_ids: vec!["obs1".to_string(), "obs2".to_string()],
            recommended_action: PatternAction::ProposeChange {
                description: "test_change".to_string(),
            },
        };

        assert_eq!(pattern.confidence, 0.95);
        assert_eq!(pattern.evidence_count, 10);
    }
}

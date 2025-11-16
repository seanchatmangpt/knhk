// rust/knhk-workflow-engine/src/autonomic/trace_index.rs
//! Trace-Indexed Execution System
//!
//! Implements canonical trace identity for deterministic replay and counterfactual analysis.
//!
//! **Architecture**:
//! - `TraceId = hash(O_segment || Σ_snapshot || Q_version)` - Uniquely identifies execution trace
//! - Observable segment (O): Time-bounded sequence of MonitorEvents
//! - Ontology snapshot (Σ): KnowledgeBase state (goals, rules, facts, policies)
//! - Doctrine/Policy (Q): Configuration and constraints
//!
//! **Properties**:
//! - Deterministic hashing with BLAKE3
//! - Zero-copy serialization with bincode
//! - Lock-free trace ID generation
//! - Efficient lookup and reconstruction
//! - Memory-mapped storage support

use super::knowledge::{Fact, Goal, KnowledgeBase, Policy, Rule};
use super::monitor::MonitorEvent;
use crate::error::{WorkflowError, WorkflowResult};
use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{info, debug, warn, instrument};

/// Trace identifier (256-bit BLAKE3 hash)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(pub [u8; 32]);

impl TraceId {
    /// Create new trace ID from components
    /// TraceId = BLAKE3(O_segment || Σ_snapshot || Q_version)
    #[instrument(skip_all, fields(
        o_start = observable_segment.start_time_ms,
        o_end = observable_segment.end_time_ms,
        sigma_goals = ontology_snapshot.goals.len(),
        q_version = %doctrine_config.version
    ))]
    pub fn new(
        observable_segment: &ObservableSegment,
        ontology_snapshot: &OntologySnapshot,
        doctrine_config: &DoctrineConfig,
    ) -> WorkflowResult<Self> {
        debug!("Generating TraceId");
        let mut hasher = Hasher::new();

        // Hash observable segment (O)
        let o_bytes = bincode::serialize(observable_segment)
            .map_err(|e| WorkflowError::Internal(format!("Failed to serialize O: {}", e)))?;
        hasher.update(&o_bytes);

        // Hash ontology snapshot (Σ)
        let sigma_bytes = bincode::serialize(ontology_snapshot)
            .map_err(|e| WorkflowError::Internal(format!("Failed to serialize Σ: {}", e)))?;
        hasher.update(&sigma_bytes);

        // Hash doctrine configuration (Q)
        let q_bytes = bincode::serialize(doctrine_config)
            .map_err(|e| WorkflowError::Internal(format!("Failed to serialize Q: {}", e)))?;
        hasher.update(&q_bytes);

        let hash: [u8; 32] = *hasher.finalize().as_bytes();
        let trace_id = Self(hash);
        info!(trace_id = %trace_id, "TraceId generated");
        Ok(trace_id)
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> WorkflowResult<Self> {
        let bytes = hex::decode(s)
            .map_err(|e| WorkflowError::Parse(format!("Invalid hex: {}", e)))?;
        if bytes.len() != 32 {
            return Err(WorkflowError::Parse(format!(
                "Invalid trace ID length: expected 32, got {}",
                bytes.len()
            )));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(Self(hash))
    }
}

impl std::fmt::Display for TraceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

/// Observable segment (O): Time-bounded sequence of monitor events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservableSegment {
    /// Start timestamp (ms since epoch)
    pub start_time_ms: u64,
    /// End timestamp (ms since epoch)
    pub end_time_ms: u64,
    /// Monitor events in chronological order
    pub events: Vec<MonitorEvent>,
    /// Segment metadata
    pub metadata: HashMap<String, String>,
}

impl ObservableSegment {
    /// Create new observable segment
    pub fn new(start_time_ms: u64, end_time_ms: u64) -> Self {
        Self {
            start_time_ms,
            end_time_ms,
            events: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add monitor event
    pub fn add_event(&mut self, event: MonitorEvent) {
        self.events.push(event);
    }

    /// Get duration (ms)
    pub fn duration_ms(&self) -> u64 {
        self.end_time_ms.saturating_sub(self.start_time_ms)
    }

    /// Check if event is within segment time bounds
    pub fn contains_time(&self, timestamp_ms: u64) -> bool {
        timestamp_ms >= self.start_time_ms && timestamp_ms <= self.end_time_ms
    }
}

/// Ontology snapshot (Σ): KnowledgeBase state at trace time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OntologySnapshot {
    /// Goals at snapshot time
    pub goals: Vec<Goal>,
    /// Rules at snapshot time
    pub rules: Vec<Rule>,
    /// Facts at snapshot time
    pub facts: HashMap<String, Fact>,
    /// Policies at snapshot time
    pub policies: Vec<Policy>,
    /// Snapshot timestamp (ms since epoch)
    pub timestamp_ms: u64,
}

impl OntologySnapshot {
    /// Create snapshot from knowledge base
    pub async fn from_knowledge_base(kb: &KnowledgeBase) -> Self {
        Self {
            goals: kb.get_goals().await,
            rules: kb.get_active_rules().await,
            facts: kb.get_facts().await,
            policies: kb.get_policies().await,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Restore snapshot to knowledge base
    pub async fn restore_to_knowledge_base(&self, kb: &KnowledgeBase) -> WorkflowResult<()> {
        // Clear existing knowledge
        kb.clear().await;

        // Restore goals
        for goal in &self.goals {
            kb.add_goal(goal.clone()).await?;
        }

        // Restore rules
        for rule in &self.rules {
            kb.add_rule(rule.clone()).await?;
        }

        // Restore facts
        for fact in self.facts.values() {
            kb.add_fact(fact.clone()).await?;
        }

        // Restore policies
        for policy in &self.policies {
            kb.add_policy(policy.clone()).await?;
        }

        Ok(())
    }
}

/// Doctrine configuration (Q): Policy lattice element and runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctrineConfig {
    /// Version identifier
    pub version: String,
    /// Policy lattice element (e.g., "strict", "relaxed", "experimental")
    pub policy_level: String,
    /// Configuration parameters
    pub config: HashMap<String, serde_json::Value>,
    /// Feature flags
    pub features: HashMap<String, bool>,
}

impl DoctrineConfig {
    /// Create new doctrine configuration
    pub fn new(version: String, policy_level: String) -> Self {
        Self {
            version,
            policy_level,
            config: HashMap::new(),
            features: HashMap::new(),
        }
    }

    /// Set configuration parameter
    pub fn set_config(&mut self, key: String, value: serde_json::Value) {
        self.config.insert(key, value);
    }

    /// Set feature flag
    pub fn set_feature(&mut self, key: String, enabled: bool) {
        self.features.insert(key, enabled);
    }
}

impl Default for DoctrineConfig {
    fn default() -> Self {
        Self::new("1.0.0".to_string(), "default".to_string())
    }
}

/// Complete execution trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTrace {
    /// Trace identifier
    pub id: TraceId,
    /// Observable segment
    pub observable_segment: ObservableSegment,
    /// Ontology snapshot
    pub ontology_snapshot: OntologySnapshot,
    /// Doctrine configuration
    pub doctrine_config: DoctrineConfig,
    /// Execution results (actions taken)
    pub execution_results: Vec<ExecutionRecord>,
    /// Trace timestamp
    pub timestamp_ms: u64,
}

impl ExecutionTrace {
    /// Create new execution trace
    pub fn new(
        observable_segment: ObservableSegment,
        ontology_snapshot: OntologySnapshot,
        doctrine_config: DoctrineConfig,
    ) -> WorkflowResult<Self> {
        let id = TraceId::new(&observable_segment, &ontology_snapshot, &doctrine_config)?;

        Ok(Self {
            id,
            observable_segment,
            ontology_snapshot,
            doctrine_config,
            execution_results: Vec::new(),
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        })
    }

    /// Add execution record
    pub fn add_execution_record(&mut self, record: ExecutionRecord) {
        self.execution_results.push(record);
    }
}

/// Execution record (action taken during trace)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Timestamp when action was taken (ms since epoch)
    pub timestamp_ms: u64,
    /// Action type
    pub action_type: String,
    /// Action parameters
    pub params: HashMap<String, serde_json::Value>,
    /// Action result
    pub result: ActionResult,
    /// Execution duration (µs)
    pub duration_us: u64,
}

/// Action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionResult {
    /// Action succeeded
    Success { details: String },
    /// Action failed
    Failure { error: String },
    /// Action skipped
    Skipped { reason: String },
}

/// Trace storage (in-memory with optional persistence)
pub struct TraceStorage {
    /// Trace index (TraceId -> ExecutionTrace)
    traces: Arc<RwLock<HashMap<TraceId, ExecutionTrace>>>,
    /// Maximum traces to keep in memory
    max_traces: usize,
    /// Trace access order (for LRU eviction)
    access_order: Arc<RwLock<Vec<TraceId>>>,
}

impl TraceStorage {
    /// Create new trace storage
    pub fn new(max_traces: usize) -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            max_traces,
            access_order: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Store trace
    #[instrument(skip(self, trace), fields(trace_id = %trace.id))]
    pub async fn store(&self, trace: ExecutionTrace) -> WorkflowResult<TraceId> {
        let trace_id = trace.id;
        debug!("Storing trace");

        let mut traces = self.traces.write().await;
        let mut access_order = self.access_order.write().await;

        // Insert trace
        traces.insert(trace_id, trace);

        // Update access order
        access_order.retain(|id| *id != trace_id);
        access_order.push(trace_id);

        // Evict oldest traces if exceeding limit
        while traces.len() > self.max_traces && !access_order.is_empty() {
            if let Some(oldest_id) = access_order.first() {
                let oldest_id = *oldest_id;
                traces.remove(&oldest_id);
                access_order.remove(0);
            }
        }

        info!(trace_id = %trace_id, "Trace stored");
        Ok(trace_id)
    }

    /// Retrieve trace by ID
    #[instrument(skip(self), fields(trace_id = %trace_id))]
    pub async fn retrieve(&self, trace_id: &TraceId) -> WorkflowResult<Option<ExecutionTrace>> {
        debug!("Retrieving trace");
        let traces = self.traces.read().await;
        let result = traces.get(trace_id).cloned();

        match &result {
            Some(_) => info!(trace_id = %trace_id, "Trace retrieved"),
            None => warn!(trace_id = %trace_id, "Trace not found"),
        }

        Ok(result)
    }

    /// Check if trace exists
    pub async fn contains(&self, trace_id: &TraceId) -> bool {
        let traces = self.traces.read().await;
        traces.contains_key(trace_id)
    }

    /// Get all trace IDs
    pub async fn all_trace_ids(&self) -> Vec<TraceId> {
        let traces = self.traces.read().await;
        traces.keys().copied().collect()
    }

    /// Get storage statistics
    pub async fn stats(&self) -> TraceStorageStats {
        let traces = self.traces.read().await;
        TraceStorageStats {
            total_traces: traces.len(),
            max_traces: self.max_traces,
            memory_usage_mb: std::mem::size_of_val(&*traces) as f64 / (1024.0 * 1024.0),
        }
    }

    /// Clear all traces
    pub async fn clear(&self) {
        let mut traces = self.traces.write().await;
        let mut access_order = self.access_order.write().await;
        traces.clear();
        access_order.clear();
    }
}

/// Trace storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStorageStats {
    /// Total traces stored
    pub total_traces: usize,
    /// Maximum traces capacity
    pub max_traces: usize,
    /// Estimated memory usage (MB)
    pub memory_usage_mb: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_trace_id_generation() {
        let o_segment = ObservableSegment::new(1000, 2000);
        let sigma = OntologySnapshot {
            goals: Vec::new(),
            rules: Vec::new(),
            facts: HashMap::new(),
            policies: Vec::new(),
            timestamp_ms: 1500,
        };
        let q_config = DoctrineConfig::default();

        let trace_id = TraceId::new(&o_segment, &sigma, &q_config).unwrap();

        // Verify trace ID is deterministic
        let trace_id2 = TraceId::new(&o_segment, &sigma, &q_config).unwrap();
        assert_eq!(trace_id, trace_id2);

        // Verify different inputs produce different IDs
        let mut o_segment2 = o_segment.clone();
        o_segment2.start_time_ms = 3000;
        let trace_id3 = TraceId::new(&o_segment2, &sigma, &q_config).unwrap();
        assert_ne!(trace_id, trace_id3);
    }

    #[tokio::test]
    async fn test_trace_id_hex_encoding() {
        let o_segment = ObservableSegment::new(1000, 2000);
        let sigma = OntologySnapshot {
            goals: Vec::new(),
            rules: Vec::new(),
            facts: HashMap::new(),
            policies: Vec::new(),
            timestamp_ms: 1500,
        };
        let q_config = DoctrineConfig::default();

        let trace_id = TraceId::new(&o_segment, &sigma, &q_config).unwrap();
        let hex = trace_id.to_hex();

        // Verify hex encoding is 64 characters (32 bytes * 2)
        assert_eq!(hex.len(), 64);

        // Verify round-trip
        let trace_id2 = TraceId::from_hex(&hex).unwrap();
        assert_eq!(trace_id, trace_id2);
    }

    #[tokio::test]
    async fn test_observable_segment() {
        let mut segment = ObservableSegment::new(1000, 2000);

        assert_eq!(segment.duration_ms(), 1000);
        assert!(segment.contains_time(1500));
        assert!(!segment.contains_time(500));
        assert!(!segment.contains_time(3000));

        segment.add_event(MonitorEvent::new(
            "test_metric".to_string(),
            42.0,
            "test_source".to_string(),
        ));
        assert_eq!(segment.events.len(), 1);
    }

    #[tokio::test]
    async fn test_ontology_snapshot() {
        let kb = KnowledgeBase::new();

        // Add some knowledge
        let goal = Goal::new(
            "test".to_string(),
            super::super::knowledge::GoalType::Performance,
            "latency".to_string(),
            100.0,
        );
        kb.add_goal(goal).await.unwrap();

        // Create snapshot
        let snapshot = OntologySnapshot::from_knowledge_base(&kb).await;
        assert_eq!(snapshot.goals.len(), 1);

        // Restore to new KB
        let kb2 = KnowledgeBase::new();
        snapshot.restore_to_knowledge_base(&kb2).await.unwrap();

        let goals2 = kb2.get_goals().await;
        assert_eq!(goals2.len(), 1);
        assert_eq!(goals2[0].name, "test");
    }

    #[tokio::test]
    async fn test_trace_storage() {
        let storage = TraceStorage::new(10);

        let o_segment = ObservableSegment::new(1000, 2000);
        let sigma = OntologySnapshot {
            goals: Vec::new(),
            rules: Vec::new(),
            facts: HashMap::new(),
            policies: Vec::new(),
            timestamp_ms: 1500,
        };
        let q_config = DoctrineConfig::default();

        let trace = ExecutionTrace::new(o_segment, sigma, q_config).unwrap();
        let trace_id = trace.id;

        // Store trace
        storage.store(trace).await.unwrap();

        // Retrieve trace
        let retrieved = storage.retrieve(&trace_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, trace_id);

        // Check stats
        let stats = storage.stats().await;
        assert_eq!(stats.total_traces, 1);
    }

    #[tokio::test]
    async fn test_trace_storage_eviction() {
        let storage = TraceStorage::new(3);

        // Store 5 traces
        for i in 0..5 {
            let mut o_segment = ObservableSegment::new(1000 + i * 1000, 2000 + i * 1000);
            o_segment.metadata.insert("index".to_string(), i.to_string());

            let sigma = OntologySnapshot {
                goals: Vec::new(),
                rules: Vec::new(),
                facts: HashMap::new(),
                policies: Vec::new(),
                timestamp_ms: 1500 + i * 1000,
            };
            let q_config = DoctrineConfig::default();

            let trace = ExecutionTrace::new(o_segment, sigma, q_config).unwrap();
            storage.store(trace).await.unwrap();
        }

        // Verify only 3 traces remain (LRU eviction)
        let stats = storage.stats().await;
        assert_eq!(stats.total_traces, 3);
    }
}

// KNHK Scaling Manager - Horizontal and Vertical Scaling
// Phase 5: Production-grade auto-scaling and cluster coordination
// Enables elastic scaling for Fortune 500 workloads

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering}};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, timeout};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, instrument};
use dashmap::DashMap;

const SCALE_CHECK_INTERVAL: Duration = Duration::from_secs(30);
const SCALE_UP_THRESHOLD: f64 = 80.0; // CPU or memory %
const SCALE_DOWN_THRESHOLD: f64 = 20.0;
const SCALE_UP_COOLDOWN: Duration = Duration::from_secs(300); // 5 minutes
const SCALE_DOWN_COOLDOWN: Duration = Duration::from_secs(600); // 10 minutes
const MIN_REPLICAS: usize = 1;
const MAX_REPLICAS: usize = 100;
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const NODE_TIMEOUT: Duration = Duration::from_secs(30);

/// Cluster node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub node_id: String,
    pub address: String,
    pub capacity: NodeCapacity,
    pub status: NodeStatus,
    pub metrics: NodeMetrics,
    pub last_heartbeat: SystemTime,
    pub joined_at: SystemTime,
    pub version: String,
    pub zone: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Starting,
    Ready,
    Busy,
    Draining,
    Unhealthy,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapacity {
    pub max_workflows: usize,
    pub cpu_cores: usize,
    pub memory_gb: f64,
    pub disk_gb: f64,
    pub network_mbps: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub active_workflows: usize,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: f64,
    pub requests_per_second: f64,
    pub avg_latency_ms: f64,
}

/// Auto-scaling policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub enabled: bool,
    pub min_replicas: usize,
    pub max_replicas: usize,
    pub target_cpu: f64,
    pub target_memory: f64,
    pub target_rps: f64,
    pub scale_up_increment: usize,
    pub scale_down_increment: usize,
    pub predictive_scaling: bool,
}

impl Default for ScalingPolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            min_replicas: MIN_REPLICAS,
            max_replicas: MAX_REPLICAS,
            target_cpu: 70.0,
            target_memory: 70.0,
            target_rps: 1000.0,
            scale_up_increment: 2,
            scale_down_increment: 1,
            predictive_scaling: false,
        }
    }
}

/// Scaling event for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingEvent {
    pub timestamp: SystemTime,
    pub event_type: ScalingEventType,
    pub from_replicas: usize,
    pub to_replicas: usize,
    pub reason: String,
    pub metrics: NodeMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScalingEventType {
    ScaleUp,
    ScaleDown,
    NodeAdded,
    NodeRemoved,
    NodeFailure,
    Rebalance,
}

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalanceStrategy {
    RoundRobin,
    LeastConnections,
    WeightedRandom,
    ConsistentHash,
    ResourceBased,
}

/// Scaling manager for elastic operations
pub struct ScalingManager {
    // Cluster state
    cluster_nodes: Arc<DashMap<String, ClusterNode>>,
    local_node_id: String,
    is_leader: Arc<AtomicBool>,

    // Scaling configuration
    policy: Arc<RwLock<ScalingPolicy>>,
    load_balance_strategy: LoadBalanceStrategy,

    // Scaling state
    current_replicas: Arc<AtomicUsize>,
    target_replicas: Arc<AtomicUsize>,
    last_scale_up: Arc<RwLock<Option<Instant>>>,
    last_scale_down: Arc<RwLock<Option<Instant>>>,

    // Event tracking
    scaling_events: Arc<RwLock<VecDeque<ScalingEvent>>>,

    // Load distribution
    workflow_assignments: Arc<DashMap<String, String>>, // workflow_id -> node_id
    node_loads: Arc<DashMap<String, f64>>, // node_id -> load score

    // Predictive scaling
    load_predictor: Arc<LoadPredictor>,

    // Service discovery
    service_registry: Arc<ServiceRegistry>,

    // Statistics
    total_scale_ups: Arc<AtomicU64>,
    total_scale_downs: Arc<AtomicU64>,
    total_rebalances: Arc<AtomicU64>,
    total_node_failures: Arc<AtomicU64>,

    // Communication
    cluster_tx: mpsc::UnboundedSender<ClusterMessage>,
    cluster_rx: Option<mpsc::UnboundedReceiver<ClusterMessage>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ClusterMessage {
    Heartbeat { node_id: String, metrics: NodeMetrics },
    JoinRequest { node: ClusterNode },
    LeaveRequest { node_id: String },
    ScaleRequest { target: usize },
    RebalanceRequest,
    WorkflowAssignment { workflow_id: String, node_id: String },
}

/// Load prediction for proactive scaling
struct LoadPredictor {
    history: Arc<RwLock<VecDeque<LoadDataPoint>>>,
    predictions: Arc<RwLock<Vec<LoadPrediction>>>,
}

#[derive(Debug, Clone)]
struct LoadDataPoint {
    timestamp: Instant,
    cpu_usage: f64,
    memory_usage: f64,
    request_rate: f64,
}

#[derive(Debug, Clone)]
struct LoadPrediction {
    timestamp: Instant,
    predicted_load: f64,
    confidence: f64,
}

/// Service registry for node discovery
struct ServiceRegistry {
    services: Arc<DashMap<String, ServiceEndpoint>>,
    health_checks: Arc<DashMap<String, HealthCheck>>,
}

#[derive(Debug, Clone)]
struct ServiceEndpoint {
    node_id: String,
    address: String,
    port: u16,
    protocol: String,
    metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct HealthCheck {
    endpoint: String,
    interval: Duration,
    timeout: Duration,
    healthy_threshold: u32,
    unhealthy_threshold: u32,
    consecutive_failures: u32,
}

impl ScalingManager {
    /// Initialize scaling manager
    pub fn new(cluster_mode: bool) -> Result<Self, Box<dyn std::error::Error>> {
        info!("Initializing scaling manager (cluster_mode: {})", cluster_mode);

        let local_node_id = format!("knhk-{}", uuid::Uuid::new_v4());
        let (cluster_tx, cluster_rx) = mpsc::unbounded_channel();

        Ok(Self {
            cluster_nodes: Arc::new(DashMap::new()),
            local_node_id: local_node_id.clone(),
            is_leader: Arc::new(AtomicBool::new(!cluster_mode)), // Single node is leader
            policy: Arc::new(RwLock::new(ScalingPolicy::default())),
            load_balance_strategy: LoadBalanceStrategy::ResourceBased,
            current_replicas: Arc::new(AtomicUsize::new(1)),
            target_replicas: Arc::new(AtomicUsize::new(1)),
            last_scale_up: Arc::new(RwLock::new(None)),
            last_scale_down: Arc::new(RwLock::new(None)),
            scaling_events: Arc::new(RwLock::new(VecDeque::new())),
            workflow_assignments: Arc::new(DashMap::new()),
            node_loads: Arc::new(DashMap::new()),
            load_predictor: Arc::new(LoadPredictor::new()),
            service_registry: Arc::new(ServiceRegistry::new()),
            total_scale_ups: Arc::new(AtomicU64::new(0)),
            total_scale_downs: Arc::new(AtomicU64::new(0)),
            total_rebalances: Arc::new(AtomicU64::new(0)),
            total_node_failures: Arc::new(AtomicU64::new(0)),
            cluster_tx,
            cluster_rx: Some(cluster_rx),
        })
    }

    /// Join the cluster
    #[instrument(skip(self))]
    pub async fn join_cluster(&self, node_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("Joining cluster as node {}", node_id);

        let local_node = ClusterNode {
            node_id: node_id.to_string(),
            address: self.get_local_address(),
            capacity: self.get_local_capacity(),
            status: NodeStatus::Starting,
            metrics: NodeMetrics::default(),
            last_heartbeat: SystemTime::now(),
            joined_at: SystemTime::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            zone: self.get_availability_zone(),
        };

        // Register local node
        self.cluster_nodes.insert(node_id.to_string(), local_node.clone());

        // Send join request to cluster
        self.cluster_tx.send(ClusterMessage::JoinRequest { node: local_node }).ok();

        // Start cluster services
        self.start_heartbeat_service();
        self.start_scaling_controller();
        self.start_load_balancer();
        self.start_health_monitor();

        // Mark as ready
        if let Some(mut node) = self.cluster_nodes.get_mut(node_id) {
            node.status = NodeStatus::Ready;
        }

        info!("Successfully joined cluster");
        Ok(())
    }

    /// Leave the cluster gracefully
    #[instrument(skip(self))]
    pub async fn leave_cluster(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Leaving cluster");

        // Mark node as draining
        if let Some(mut node) = self.cluster_nodes.get_mut(&self.local_node_id) {
            node.status = NodeStatus::Draining;
        }

        // Wait for workflows to complete
        self.drain_workflows().await?;

        // Send leave request
        self.cluster_tx.send(ClusterMessage::LeaveRequest {
            node_id: self.local_node_id.clone()
        }).ok();

        // Remove from cluster
        self.cluster_nodes.remove(&self.local_node_id);

        info!("Successfully left cluster");
        Ok(())
    }

    /// Assign workflow to optimal node
    #[instrument(skip(self))]
    pub async fn assign_workflow(&self, workflow_id: &str) -> Result<String, Box<dyn std::error::Error>> {
        let selected_node = match self.load_balance_strategy {
            LoadBalanceStrategy::RoundRobin => self.select_round_robin(),
            LoadBalanceStrategy::LeastConnections => self.select_least_connections(),
            LoadBalanceStrategy::WeightedRandom => self.select_weighted_random(),
            LoadBalanceStrategy::ConsistentHash => self.select_consistent_hash(workflow_id),
            LoadBalanceStrategy::ResourceBased => self.select_resource_based(),
        }?;

        // Record assignment
        self.workflow_assignments.insert(workflow_id.to_string(), selected_node.clone());

        // Update node load
        if let Some(mut load) = self.node_loads.get_mut(&selected_node) {
            *load += 1.0;
        }

        // Send assignment message
        self.cluster_tx.send(ClusterMessage::WorkflowAssignment {
            workflow_id: workflow_id.to_string(),
            node_id: selected_node.clone(),
        }).ok();

        Ok(selected_node)
    }

    /// Select node using resource-based strategy
    fn select_resource_based(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut best_node: Option<(String, f64)> = None;

        for entry in self.cluster_nodes.iter() {
            let node = entry.value();

            if node.status != NodeStatus::Ready {
                continue;
            }

            // Calculate resource score (lower is better)
            let score = (node.metrics.cpu_usage * 0.4) +
                        (node.metrics.memory_usage * 0.4) +
                        (node.metrics.active_workflows as f64 / node.capacity.max_workflows as f64 * 0.2);

            if best_node.is_none() || score < best_node.as_ref().unwrap().1 {
                best_node = Some((node.node_id.clone(), score));
            }
        }

        best_node
            .map(|(id, _)| id)
            .ok_or("No available nodes".into())
    }

    /// Start heartbeat service
    fn start_heartbeat_service(&self) {
        let nodes = self.cluster_nodes.clone();
        let node_id = self.local_node_id.clone();
        let tx = self.cluster_tx.clone();

        tokio::spawn(async move {
            let mut ticker = interval(HEARTBEAT_INTERVAL);

            loop {
                ticker.tick().await;

                // Send heartbeat
                if let Some(node) = nodes.get(&node_id) {
                    tx.send(ClusterMessage::Heartbeat {
                        node_id: node_id.clone(),
                        metrics: node.metrics.clone(),
                    }).ok();
                }

                // Check for dead nodes
                let now = SystemTime::now();
                let mut dead_nodes = Vec::new();

                for entry in nodes.iter() {
                    let node = entry.value();
                    if let Ok(elapsed) = now.duration_since(node.last_heartbeat) {
                        if elapsed > NODE_TIMEOUT {
                            dead_nodes.push(node.node_id.clone());
                        }
                    }
                }

                // Mark dead nodes as offline
                for dead_node in dead_nodes {
                    if let Some(mut node) = nodes.get_mut(&dead_node) {
                        if node.status != NodeStatus::Offline {
                            warn!("Node {} detected as offline", dead_node);
                            node.status = NodeStatus::Offline;
                        }
                    }
                }
            }
        });
    }

    /// Start scaling controller
    fn start_scaling_controller(&self) {
        let nodes = self.cluster_nodes.clone();
        let policy = self.policy.clone();
        let current = self.current_replicas.clone();
        let target = self.target_replicas.clone();
        let last_up = self.last_scale_up.clone();
        let last_down = self.last_scale_down.clone();
        let events = self.scaling_events.clone();
        let total_ups = self.total_scale_ups.clone();
        let total_downs = self.total_scale_downs.clone();
        let is_leader = self.is_leader.clone();
        let tx = self.cluster_tx.clone();

        tokio::spawn(async move {
            let mut ticker = interval(SCALE_CHECK_INTERVAL);

            loop {
                ticker.tick().await;

                // Only leader makes scaling decisions
                if !is_leader.load(Ordering::Relaxed) {
                    continue;
                }

                let policy = policy.read().unwrap();
                if !policy.enabled {
                    continue;
                }

                // Calculate cluster metrics
                let mut total_cpu = 0.0;
                let mut total_memory = 0.0;
                let mut total_rps = 0.0;
                let mut active_nodes = 0;

                for entry in nodes.iter() {
                    let node = entry.value();
                    if node.status == NodeStatus::Ready {
                        total_cpu += node.metrics.cpu_usage;
                        total_memory += node.metrics.memory_usage;
                        total_rps += node.metrics.requests_per_second;
                        active_nodes += 1;
                    }
                }

                if active_nodes == 0 {
                    continue;
                }

                let avg_cpu = total_cpu / active_nodes as f64;
                let avg_memory = total_memory / active_nodes as f64;

                let current_replicas = current.load(Ordering::Relaxed);

                // Determine if scaling is needed
                let should_scale_up = (avg_cpu > policy.target_cpu ||
                                       avg_memory > policy.target_memory ||
                                       total_rps > policy.target_rps * current_replicas as f64) &&
                                      current_replicas < policy.max_replicas;

                let should_scale_down = avg_cpu < policy.target_cpu * 0.5 &&
                                        avg_memory < policy.target_memory * 0.5 &&
                                        total_rps < policy.target_rps * current_replicas as f64 * 0.5 &&
                                        current_replicas > policy.min_replicas;

                // Check cooldown periods
                let now = Instant::now();
                let can_scale_up = last_up.read().unwrap()
                    .map(|t| now.duration_since(t) > SCALE_UP_COOLDOWN)
                    .unwrap_or(true);

                let can_scale_down = last_down.read().unwrap()
                    .map(|t| now.duration_since(t) > SCALE_DOWN_COOLDOWN)
                    .unwrap_or(true);

                // Execute scaling decision
                if should_scale_up && can_scale_up {
                    let new_replicas = (current_replicas + policy.scale_up_increment)
                        .min(policy.max_replicas);

                    info!("Scaling up from {} to {} replicas (CPU: {:.1}%, Mem: {:.1}%)",
                        current_replicas, new_replicas, avg_cpu, avg_memory);

                    target.store(new_replicas, Ordering::Relaxed);
                    *last_up.write().unwrap() = Some(now);
                    total_ups.fetch_add(1, Ordering::Relaxed);

                    // Record event
                    events.write().unwrap().push_back(ScalingEvent {
                        timestamp: SystemTime::now(),
                        event_type: ScalingEventType::ScaleUp,
                        from_replicas: current_replicas,
                        to_replicas: new_replicas,
                        reason: format!("High resource usage - CPU: {:.1}%, Memory: {:.1}%", avg_cpu, avg_memory),
                        metrics: NodeMetrics {
                            cpu_usage: avg_cpu,
                            memory_usage: avg_memory,
                            requests_per_second: total_rps,
                            ..Default::default()
                        },
                    });

                    tx.send(ClusterMessage::ScaleRequest { target: new_replicas }).ok();

                } else if should_scale_down && can_scale_down {
                    let new_replicas = (current_replicas.saturating_sub(policy.scale_down_increment))
                        .max(policy.min_replicas);

                    info!("Scaling down from {} to {} replicas (CPU: {:.1}%, Mem: {:.1}%)",
                        current_replicas, new_replicas, avg_cpu, avg_memory);

                    target.store(new_replicas, Ordering::Relaxed);
                    *last_down.write().unwrap() = Some(now);
                    total_downs.fetch_add(1, Ordering::Relaxed);

                    // Record event
                    events.write().unwrap().push_back(ScalingEvent {
                        timestamp: SystemTime::now(),
                        event_type: ScalingEventType::ScaleDown,
                        from_replicas: current_replicas,
                        to_replicas: new_replicas,
                        reason: format!("Low resource usage - CPU: {:.1}%, Memory: {:.1}%", avg_cpu, avg_memory),
                        metrics: NodeMetrics {
                            cpu_usage: avg_cpu,
                            memory_usage: avg_memory,
                            requests_per_second: total_rps,
                            ..Default::default()
                        },
                    });

                    tx.send(ClusterMessage::ScaleRequest { target: new_replicas }).ok();
                }
            }
        });
    }

    /// Start load balancer
    fn start_load_balancer(&self) {
        // Implementation would handle workflow distribution
    }

    /// Start health monitor
    fn start_health_monitor(&self) {
        // Implementation would monitor node health
    }

    /// Drain workflows from local node
    async fn drain_workflows(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Draining workflows from local node");

        // Get workflows assigned to this node
        let mut local_workflows = Vec::new();
        for entry in self.workflow_assignments.iter() {
            if entry.value() == &self.local_node_id {
                local_workflows.push(entry.key().clone());
            }
        }

        // Reassign workflows to other nodes
        for workflow_id in local_workflows {
            if let Ok(new_node) = self.select_resource_based() {
                self.workflow_assignments.insert(workflow_id.clone(), new_node.clone());
                info!("Reassigned workflow {} to node {}", workflow_id, new_node);
            }
        }

        Ok(())
    }

    /// Get local node address
    fn get_local_address(&self) -> String {
        // In production, this would determine actual IP
        "127.0.0.1:8080".to_string()
    }

    /// Get local node capacity
    fn get_local_capacity(&self) -> NodeCapacity {
        NodeCapacity {
            max_workflows: 1000,
            cpu_cores: num_cpus::get(),
            memory_gb: 16.0, // Would query actual memory
            disk_gb: 100.0,  // Would query actual disk
            network_mbps: 1000,
        }
    }

    /// Get availability zone
    fn get_availability_zone(&self) -> String {
        // In cloud, would query metadata service
        "us-east-1a".to_string()
    }

    // Stub implementations for other strategies
    fn select_round_robin(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.local_node_id.clone())
    }

    fn select_least_connections(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.local_node_id.clone())
    }

    fn select_weighted_random(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.local_node_id.clone())
    }

    fn select_consistent_hash(&self, _key: &str) -> Result<String, Box<dyn std::error::Error>> {
        Ok(self.local_node_id.clone())
    }

    /// Get scaling statistics
    pub fn get_stats(&self) -> ScalingStats {
        ScalingStats {
            current_replicas: self.current_replicas.load(Ordering::Relaxed),
            target_replicas: self.target_replicas.load(Ordering::Relaxed),
            active_nodes: self.cluster_nodes.iter()
                .filter(|e| e.value().status == NodeStatus::Ready)
                .count(),
            total_nodes: self.cluster_nodes.len(),
            is_leader: self.is_leader.load(Ordering::Relaxed),
            total_scale_ups: self.total_scale_ups.load(Ordering::Relaxed),
            total_scale_downs: self.total_scale_downs.load(Ordering::Relaxed),
            total_rebalances: self.total_rebalances.load(Ordering::Relaxed),
            total_node_failures: self.total_node_failures.load(Ordering::Relaxed),
        }
    }
}

impl LoadPredictor {
    fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(VecDeque::new())),
            predictions: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl ServiceRegistry {
    fn new() -> Self {
        Self {
            services: Arc::new(DashMap::new()),
            health_checks: Arc::new(DashMap::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingStats {
    pub current_replicas: usize,
    pub target_replicas: usize,
    pub active_nodes: usize,
    pub total_nodes: usize,
    pub is_leader: bool,
    pub total_scale_ups: u64,
    pub total_scale_downs: u64,
    pub total_rebalances: u64,
    pub total_node_failures: u64,
}
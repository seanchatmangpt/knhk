//! YAWL 3-Phase Resource Allocation
//!
//! Implements YAWL's 3-phase resource allocation with TRIZ hyper-advanced patterns:
//! - TRIZ Principle 40: Composite Materials - Multiple allocation strategies
//! - TRIZ Principle 35: Parameter Changes - Dynamic allocation parameters
//!
//! Based on: org.yawlfoundation.yawl.resourcing.ResourceManager
//!
//! # 3-Phase Allocation
//!
//! 1. **Phase 1: Offer** - Select eligible participants
//!    - Role-based (primary role + additional roles)
//!    - Capability-based (required skills)
//!    - Position-based (organizational hierarchy)
//!    - Organizational group
//!
//! 2. **Phase 2: Allocate** - Select one participant
//!    - Round robin
//!    - Random
//!    - Shortest queue
//!    - Least busy
//!    - Fastest completion history
//!
//! 3. **Phase 3: Start** - Determine when to start
//!    - User-initiated
//!    - System-initiated
//!    - Concurrent start

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// User ID
pub type UserId = String;

/// Resource ID
pub type ResourceId = String;

/// Role ID
pub type RoleId = String;

/// Capability ID
pub type CapabilityId = String;

/// Phase 1: Offer - Eligible participants
#[derive(Debug, Clone)]
pub struct OfferPhase {
    /// Eligible user IDs
    pub eligible_users: Vec<UserId>,
    /// Selection criteria used
    pub criteria: OfferCriteria,
}

/// Offer criteria (TRIZ Principle 40: Composite Materials)
#[derive(Debug, Clone)]
pub enum OfferCriteria {
    /// Role-based selection
    RoleBased {
        primary_role: RoleId,
        additional_roles: Vec<RoleId>,
    },
    /// Capability-based selection
    CapabilityBased {
        required_capabilities: Vec<CapabilityId>,
    },
    /// Position-based selection
    PositionBased {
        position_level: u32,
        department: Option<String>,
    },
    /// Organizational group selection
    OrgGroupBased {
        group_id: String,
    },
    /// Composite criteria (TRIZ Principle 40)
    Composite {
        criteria: Vec<OfferCriteria>,
        operator: CompositeOperator,
    },
}

/// Composite operator for combining criteria (TRIZ Principle 40)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeOperator {
    /// AND - All criteria must match
    And,
    /// OR - Any criterion must match
    Or,
}

/// Phase 2: Allocate - Select one participant
#[derive(Debug, Clone)]
pub struct AllocatePhase {
    /// Allocated user ID
    pub allocated_user: UserId,
    /// Allocation strategy used
    pub strategy: AllocationStrategy,
}

/// Allocation strategy (TRIZ Principle 40: Composite Materials)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Round robin - Cycle through users
    RoundRobin,
    /// Random - Random selection
    Random,
    /// Shortest queue - User with fewest work items
    ShortestQueue,
    /// Least busy - User with lowest current workload
    LeastBusy,
    /// Fastest completion - User with best completion history
    FastestCompletion,
    /// Composite - Multiple strategies (TRIZ Principle 40)
    Composite {
        strategies: Vec<AllocationStrategy>,
        weights: Vec<f64>,
    },
}

/// Phase 3: Start - Determine when to start
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartPhase {
    /// User-initiated - Manual start
    UserInitiated,
    /// System-initiated - Auto-start when enabled
    SystemInitiated,
    /// Concurrent - Multiple users can start
    Concurrent,
}

/// 3-Phase allocation result
#[derive(Debug, Clone)]
pub struct ThreePhaseAllocation {
    /// Phase 1: Offer
    pub offer: OfferPhase,
    /// Phase 2: Allocate (if allocation happened)
    pub allocate: Option<AllocatePhase>,
    /// Phase 3: Start
    pub start: StartPhase,
    /// Final allocated user (if allocated)
    pub final_user: Option<UserId>,
}

/// Resource metadata
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    /// User ID
    pub user_id: UserId,
    /// Roles
    pub roles: Vec<RoleId>,
    /// Capabilities
    pub capabilities: Vec<CapabilityId>,
    /// Position level
    pub position_level: u32,
    /// Department
    pub department: Option<String>,
    /// Organizational groups
    pub org_groups: Vec<String>,
    /// Current workload (number of active work items)
    pub current_workload: u32,
    /// Average completion time (milliseconds)
    pub avg_completion_time: f64,
    /// Queue length
    pub queue_length: u32,
}

/// 3-Phase Resource Allocator
///
/// TRIZ Principle 40: Composite Materials
/// - Combines multiple allocation strategies
/// - Supports composite criteria and strategies
///
/// TRIZ Principle 35: Parameter Changes
/// - Dynamic allocation parameters based on workload
pub struct ThreePhaseAllocator {
    /// Resource metadata by user ID
    resources: Arc<RwLock<HashMap<UserId, ResourceMetadata>>>,
    /// Round robin counters (TRIZ Principle 35: Parameter Changes)
    round_robin_counters: Arc<RwLock<HashMap<RoleId, usize>>>,
    /// Allocation history for performance tracking
    allocation_history: Arc<RwLock<Vec<(UserId, chrono::DateTime<chrono::Utc>)>>>,
}

impl ThreePhaseAllocator {
    /// Create a new 3-phase allocator
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            round_robin_counters: Arc::new(RwLock::new(HashMap::new())),
            allocation_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a resource
    pub async fn register_resource(&self, metadata: ResourceMetadata) {
        let mut resources = self.resources.write().await;
        resources.insert(metadata.user_id.clone(), metadata);
    }

    /// Phase 1: Offer - Select eligible participants
    pub async fn offer_phase(
        &self,
        criteria: &OfferCriteria,
    ) -> WorkflowResult<OfferPhase> {
        let resources = self.resources.read().await;
        let mut eligible_users = Vec::new();

        match criteria {
            OfferCriteria::RoleBased {
                primary_role,
                additional_roles,
            } => {
                for (user_id, metadata) in resources.iter() {
                    if metadata.roles.contains(primary_role)
                        || additional_roles.iter().any(|r| metadata.roles.contains(r))
                    {
                        eligible_users.push(user_id.clone());
                    }
                }
            }
            OfferCriteria::CapabilityBased { required_capabilities } => {
                for (user_id, metadata) in resources.iter() {
                    if required_capabilities
                        .iter()
                        .all(|cap| metadata.capabilities.contains(cap))
                    {
                        eligible_users.push(user_id.clone());
                    }
                }
            }
            OfferCriteria::PositionBased {
                position_level,
                department,
            } => {
                for (user_id, metadata) in resources.iter() {
                    if metadata.position_level >= *position_level
                        && department.as_ref().map_or(true, |d| {
                            metadata.department.as_ref().map_or(false, |md| md == d)
                        })
                    {
                        eligible_users.push(user_id.clone());
                    }
                }
            }
            OfferCriteria::OrgGroupBased { group_id } => {
                for (user_id, metadata) in resources.iter() {
                    if metadata.org_groups.contains(group_id) {
                        eligible_users.push(user_id.clone());
                    }
                }
            }
            OfferCriteria::Composite { criteria, operator } => {
                // TRIZ Principle 40: Composite Materials - Combine multiple criteria
                let mut user_sets = Vec::new();

                for criterion in criteria {
                    let phase = self.offer_phase(criterion).await?;
                    user_sets.push(HashSet::from_iter(phase.eligible_users));
                }

                let eligible_set = match operator {
                    CompositeOperator::And => {
                        // Intersection - users matching all criteria
                        user_sets
                            .into_iter()
                            .reduce(|acc, set| acc.intersection(&set).cloned().collect())
                            .unwrap_or_default()
                    }
                    CompositeOperator::Or => {
                        // Union - users matching any criterion
                        user_sets
                            .into_iter()
                            .reduce(|acc, set| acc.union(&set).cloned().collect())
                            .unwrap_or_default()
                    }
                };

                eligible_users = eligible_set.into_iter().collect();
            }
        }

        Ok(OfferPhase {
            eligible_users,
            criteria: criteria.clone(),
        })
    }

    /// Phase 2: Allocate - Select one participant
    pub async fn allocate_phase(
        &self,
        eligible_users: &[UserId],
        strategy: AllocationStrategy,
        role_id: Option<RoleId>,
    ) -> WorkflowResult<AllocatePhase> {
        if eligible_users.is_empty() {
            return Err(WorkflowError::ResourceUnavailable(
                "No eligible users for allocation".to_string(),
            ));
        }

        let resources = self.resources.read().await;
        let allocated_user = match strategy {
            AllocationStrategy::RoundRobin => {
                // TRIZ Principle 35: Parameter Changes - Dynamic round robin counter
                let mut counters = self.round_robin_counters.write().await;
                let counter_key = role_id.unwrap_or_else(|| "default".to_string());
                let counter = counters.entry(counter_key).or_insert(0);
                *counter = (*counter + 1) % eligible_users.len();
                eligible_users[*counter].clone()
            }
            AllocationStrategy::Random => {
                // Use deterministic "random" based on current time for reproducibility
                let idx = (chrono::Utc::now().timestamp_millis() as usize) % eligible_users.len();
                eligible_users[idx].clone()
            }
            AllocationStrategy::ShortestQueue => {
                eligible_users
                    .iter()
                    .min_by_key(|user_id| {
                        resources
                            .get(*user_id)
                            .map(|r| r.queue_length)
                            .unwrap_or(u32::MAX)
                    })
                    .ok_or_else(|| WorkflowError::Internal("No users available".to_string()))?
                    .clone()
            }
            AllocationStrategy::LeastBusy => {
                eligible_users
                    .iter()
                    .min_by_key(|user_id| {
                        resources
                            .get(*user_id)
                            .map(|r| r.current_workload)
                            .unwrap_or(u32::MAX)
                    })
                    .ok_or_else(|| WorkflowError::Internal("No users available".to_string()))?
                    .clone()
            }
            AllocationStrategy::FastestCompletion => {
                eligible_users
                    .iter()
                    .min_by(|a, b| {
                        let time_a = resources
                            .get(a)
                            .map(|r| r.avg_completion_time)
                            .unwrap_or(f64::MAX);
                        let time_b = resources
                            .get(b)
                            .map(|r| r.avg_completion_time)
                            .unwrap_or(f64::MAX);
                        time_a.partial_cmp(&time_b).unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .ok_or_else(|| WorkflowError::Internal("No users available".to_string()))?
                    .clone()
            }
            AllocationStrategy::Composite { strategies, weights } => {
                // TRIZ Principle 40: Composite Materials - Weighted combination
                let mut scores: HashMap<UserId, f64> = HashMap::new();

                for (strategy, weight) in strategies.iter().zip(weights.iter()) {
                    let phase = self.allocate_phase(eligible_users, *strategy, role_id.clone()).await?;
                    *scores.entry(phase.allocated_user).or_insert(0.0) += weight;
                }

                scores
                    .into_iter()
                    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                    .ok_or_else(|| WorkflowError::Internal("No users available".to_string()))?
                    .0
            }
        };

        // Record allocation (TRIZ Principle 35: Parameter Changes - Track for dynamic adjustment)
        {
            let mut history = self.allocation_history.write().await;
            history.push((allocated_user.clone(), chrono::Utc::now()));
            // Keep only last 1000 allocations
            if history.len() > 1000 {
                history.remove(0);
            }
        }

        Ok(AllocatePhase {
            allocated_user,
            strategy,
        })
    }

    /// Execute full 3-phase allocation
    pub async fn allocate(
        &self,
        offer_criteria: OfferCriteria,
        allocation_strategy: AllocationStrategy,
        start_phase: StartPhase,
        role_id: Option<RoleId>,
    ) -> WorkflowResult<ThreePhaseAllocation> {
        // Phase 1: Offer
        let offer = self.offer_phase(&offer_criteria).await?;

        // Phase 2: Allocate (if users are eligible)
        let allocate = if !offer.eligible_users.is_empty() {
            Some(
                self.allocate_phase(&offer.eligible_users, allocation_strategy, role_id)
                    .await?,
            )
        } else {
            None
        };

        // Phase 3: Start (determined by parameter)

        Ok(ThreePhaseAllocation {
            offer,
            final_user: allocate.as_ref().map(|a| a.allocated_user.clone()),
            allocate,
            start: start_phase,
        })
    }

    /// Update resource workload (TRIZ Principle 35: Parameter Changes)
    pub async fn update_workload(&self, user_id: &UserId, workload: u32) {
        if let Some(mut resource) = self.resources.write().await.get_mut(user_id) {
            resource.current_workload = workload;
        }
    }

    /// Update resource queue length (TRIZ Principle 35: Parameter Changes)
    pub async fn update_queue_length(&self, user_id: &UserId, queue_length: u32) {
        if let Some(mut resource) = self.resources.write().await.get_mut(user_id) {
            resource.queue_length = queue_length;
        }
    }
}

impl Default for ThreePhaseAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_three_phase_allocation() {
        let allocator = ThreePhaseAllocator::new();

        // Register test resources
        allocator
            .register_resource(ResourceMetadata {
                user_id: "user1".to_string(),
                roles: vec!["developer".to_string()],
                capabilities: vec!["coding".to_string(), "testing".to_string()],
                position_level: 3,
                department: Some("engineering".to_string()),
                org_groups: vec!["dev-team".to_string()],
                current_workload: 2,
                avg_completion_time: 1000.0,
                queue_length: 1,
            })
            .await;

        allocator
            .register_resource(ResourceMetadata {
                user_id: "user2".to_string(),
                roles: vec!["developer".to_string(), "senior".to_string()],
                capabilities: vec!["coding".to_string()],
                position_level: 4,
                department: Some("engineering".to_string()),
                org_groups: vec!["dev-team".to_string()],
                current_workload: 1,
                avg_completion_time: 800.0,
                queue_length: 0,
            })
            .await;

        // Test Phase 1: Offer (role-based)
        let criteria = OfferCriteria::RoleBased {
            primary_role: "developer".to_string(),
            additional_roles: vec![],
        };

        let offer = allocator.offer_phase(&criteria).await.unwrap();
        assert_eq!(offer.eligible_users.len(), 2);

        // Test Phase 2: Allocate (shortest queue)
        let allocate = allocator
            .allocate_phase(&offer.eligible_users, AllocationStrategy::ShortestQueue, None)
            .await
            .unwrap();
        assert_eq!(allocate.allocated_user, "user2"); // user2 has shorter queue

        // Test full 3-phase allocation
        let result = allocator
            .allocate(
                criteria,
                AllocationStrategy::ShortestQueue,
                StartPhase::UserInitiated,
                None,
            )
            .await
            .unwrap();

        assert!(result.final_user.is_some());
    }
}


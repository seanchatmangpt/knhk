// knhk-warm: Epoch scheduler
// Implements Λ total order scheduling with τ ≤ 8 ticks enforcement

use std::collections::{HashMap, HashSet, VecDeque};
use crate::warm_path::WarmPathError;
use crate::WarmPathResult;

/// Hook ID type
pub type HookId = String;

/// Epoch plan
#[derive(Debug, Clone)]
pub struct EpochPlan {
    /// Epoch ID
    pub epoch_id: String,
    /// Ordered list of hook IDs (Λ order)
    pub lambda: Vec<HookId>,
    /// Tick budget (τ ≤ 8)
    pub tau: u32,
    /// Execution plan
    pub execution_plan: ExecutionPlan,
}

/// Execution plan
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Hooks to execute in order
    pub hooks: Vec<HookId>,
    /// Estimated ticks per hook
    pub tick_estimates: HashMap<HookId, u32>,
    /// Total estimated ticks
    pub total_ticks: u32,
}

/// Epoch scheduler
/// Enforces Λ ≺-total ordering and τ ≤ 8 ticks
pub struct EpochScheduler {
    /// Registered hooks
    hooks: HashMap<HookId, HookMetadata>,
}

/// Hook metadata
#[derive(Debug, Clone)]
struct HookMetadata {
    /// Hook ID
    id: HookId,
    /// Estimated ticks
    estimated_ticks: u32,
    /// Dependencies (hooks that must execute before this one)
    dependencies: Vec<HookId>,
}

impl EpochScheduler {
    /// Create a new epoch scheduler
    pub fn new() -> Self {
        Self {
            hooks: HashMap::new(),
        }
    }

    /// Register a hook with metadata
    pub fn register_hook(
        &mut self,
        hook_id: HookId,
        estimated_ticks: u32,
        dependencies: Vec<HookId>,
    ) -> Result<(), WarmPathError> {
        // Validate estimated_ticks ≤ 8
        if estimated_ticks > 8 {
            return Err(WarmPathError::InvalidInput(format!(
                "Hook {} estimated_ticks {} exceeds Chatman Constant (8)",
                hook_id, estimated_ticks
            )));
        }

        self.hooks.insert(
            hook_id.clone(),
            HookMetadata {
                id: hook_id,
                estimated_ticks,
                dependencies,
            },
        );

        Ok(())
    }

    /// Schedule an epoch with Λ total order
    /// Validates that Λ is ≺-total (no cycles, deterministic order)
    pub fn schedule_epoch(
        &self,
        epoch_id: String,
        hook_ids: &[HookId],
        tau: u32,
        lambda: &[String],
    ) -> Result<EpochPlan, WarmPathError> {
        // Validate τ ≤ 8 (Chatman Constant)
        if tau > 8 {
            return Err(WarmPathError::InvalidInput(format!(
                "Epoch {} tau {} exceeds Chatman Constant (8)",
                epoch_id, tau
            )));
        }

        // Validate Λ is ≺-total (no duplicates, deterministic order)
        self.validate_lambda_order(lambda)?;

        // Select hooks by Λ order
        let selected_hooks = self.select_hooks_by_lambda(hook_ids, lambda)?;

        // Create execution plan
        let execution_plan = self.create_execution_plan(&selected_hooks, tau)?;

        Ok(EpochPlan {
            epoch_id,
            lambda: lambda.to_vec(),
            tau,
            execution_plan,
        })
    }

    /// Validate Λ is ≺-total (deterministic order, no cycles)
    fn validate_lambda_order(&self, lambda: &[String]) -> Result<(), WarmPathError> {
        // Check for duplicates (violates ≺-total)
        let mut seen = HashSet::new();
        for hook_id in lambda {
            if seen.contains(hook_id) {
                return Err(WarmPathError::InvalidInput(format!(
                    "Lambda contains duplicate hook '{}', violates ≺-total ordering",
                    hook_id
                )));
            }
            seen.insert(hook_id.clone());
        }

        // Check for cycles in dependencies
        // Build dependency graph
        let mut graph: HashMap<&String, Vec<&String>> = HashMap::new();
        for hook_id in lambda {
            if let Some(metadata) = self.hooks.get(hook_id) {
                let deps: Vec<&String> = metadata
                    .dependencies
                    .iter()
                    .filter(|dep| lambda.contains(dep))
                    .collect();
                graph.insert(hook_id, deps);
            }
        }

        // Check for cycles using DFS
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for hook_id in lambda {
            if self.has_cycle(&graph, hook_id, &mut visited, &mut rec_stack) {
                return Err(WarmPathError::InvalidInput(format!(
                    "Lambda contains cycle involving hook '{}', violates ≺-total ordering",
                    hook_id
                )));
            }
        }

        Ok(())
    }

    /// Check for cycles in dependency graph
    fn has_cycle(
        &self,
        graph: &HashMap<&String, Vec<&String>>,
        node: &String,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true; // Cycle detected
        }

        if visited.contains(node) {
            return false; // Already processed
        }

        visited.insert(node.clone());
        rec_stack.insert(node.clone());

        if let Some(deps) = graph.get(node) {
            for dep in deps {
                if self.has_cycle(graph, dep, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Select hooks by Λ order
    fn select_hooks_by_lambda(
        &self,
        hook_ids: &[HookId],
        lambda: &[String],
    ) -> Result<Vec<HookId>, WarmPathError> {
        let mut selected = Vec::new();
        let hook_set: HashSet<&String> = hook_ids.iter().collect();

        for hook_id in lambda {
            if hook_set.contains(hook_id) {
                // Verify hook is registered
                if !self.hooks.contains_key(hook_id) {
                    return Err(WarmPathError::InvalidInput(format!(
                        "Hook '{}' in lambda is not registered",
                        hook_id
                    )));
                }
                selected.push(hook_id.clone());
            }
        }

        Ok(selected)
    }

    /// Create execution plan respecting τ ≤ 8
    fn create_execution_plan(
        &self,
        hooks: &[HookId],
        tau: u32,
    ) -> Result<ExecutionPlan, WarmPathError> {
        let mut execution_hooks = Vec::new();
        let mut tick_estimates = HashMap::new();
        let mut total_ticks = 0u32;

        for hook_id in hooks {
            let metadata = self.hooks.get(hook_id).ok_or_else(|| {
                WarmPathError::InvalidInput(format!("Hook '{}' not found", hook_id))
            })?;

            // Check if adding this hook would exceed τ budget
            if total_ticks + metadata.estimated_ticks > tau {
                // Stop adding hooks if budget would be exceeded
                break;
            }

            execution_hooks.push(hook_id.clone());
            tick_estimates.insert(hook_id.clone(), metadata.estimated_ticks);
            total_ticks += metadata.estimated_ticks;
        }

        // Validate total ticks ≤ τ
        if total_ticks > tau {
            return Err(WarmPathError::InvalidInput(format!(
                "Execution plan total_ticks {} exceeds tau {}",
                total_ticks, tau
            )));
        }

        Ok(ExecutionPlan {
            hooks: execution_hooks,
            tick_estimates,
            total_ticks,
        })
    }

    /// Enforce τ budget on execution plan
    pub fn enforce_tau_budget(
        &self,
        plan: &ExecutionPlan,
        tau: u32,
    ) -> Result<(), WarmPathError> {
        if plan.total_ticks > tau {
            return Err(WarmPathError::InvalidInput(format!(
                "Execution plan total_ticks {} exceeds tau {}",
                plan.total_ticks, tau
            )));
        }

        Ok(())
    }
}

impl Default for EpochScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_epoch() {
        let mut scheduler = EpochScheduler::new();
        scheduler
            .register_hook("hook1".to_string(), 2, vec![])
            .unwrap();
        scheduler
            .register_hook("hook2".to_string(), 3, vec!["hook1".to_string()])
            .unwrap();

        let plan = scheduler
            .schedule_epoch(
                "epoch1".to_string(),
                &["hook1".to_string(), "hook2".to_string()],
                8,
                &["hook1".to_string(), "hook2".to_string()],
            )
            .unwrap();

        assert_eq!(plan.tau, 8);
        assert_eq!(plan.lambda.len(), 2);
        assert!(plan.execution_plan.total_ticks <= 8);
    }

    #[test]
    fn test_validate_lambda_order_duplicate() {
        let scheduler = EpochScheduler::new();
        let lambda = vec!["hook1".to_string(), "hook1".to_string()];
        let result = scheduler.validate_lambda_order(&lambda);
        assert!(result.is_err());
    }

    #[test]
    fn test_enforce_tau_budget() {
        let mut scheduler = EpochScheduler::new();
        scheduler
            .register_hook("hook1".to_string(), 5, vec![])
            .unwrap();

        let plan = ExecutionPlan {
            hooks: vec!["hook1".to_string()],
            tick_estimates: {
                let mut m = HashMap::new();
                m.insert("hook1".to_string(), 5);
                m
            },
            total_ticks: 5,
        };

        assert!(scheduler.enforce_tau_budget(&plan, 8).is_ok());
        assert!(scheduler.enforce_tau_budget(&plan, 4).is_err());
    }
}


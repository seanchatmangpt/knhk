// Example: Q-Learning for Workflow Task Selection
//
// This example demonstrates how to use Q-Learning to optimize
// task selection in a workflow execution.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// State: Current workflow execution state
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct WorkflowState {
    pub completed_tasks: u64,      // Bitmap of completed tasks
    pub current_phase: u8,         // Current workflow phase (0-15)
    pub resource_usage: u8,        // Resource utilization (0-100)
    pub failures: u8,              // Number of failures so far
}

// Action: Next task to execute
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct TaskAction {
    pub task_id: u8,
    pub priority: Priority,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

// Transition: (s, a, r, s')
pub struct Transition {
    pub state: WorkflowState,
    pub action: TaskAction,
    pub reward: f32,
    pub next_state: WorkflowState,
}

// Q-Learning Agent
pub struct QLearningAgent {
    /// Q-table: State -> [Action -> Q-value]
    q_table: Arc<RwLock<HashMap<WorkflowState, HashMap<TaskAction, f32>>>>,

    /// Learning rate (α)
    learning_rate: f32,

    /// Discount factor (γ)
    discount_factor: f32,

    /// Exploration rate (ε)
    epsilon: f32,

    /// Epsilon decay
    epsilon_decay: f32,

    /// Minimum epsilon
    epsilon_min: f32,
}

impl QLearningAgent {
    pub fn new() -> Self {
        Self {
            q_table: Arc::new(RwLock::new(HashMap::new())),
            learning_rate: 0.1,
            discount_factor: 0.95,
            epsilon: 1.0,
            epsilon_decay: 0.995,
            epsilon_min: 0.01,
        }
    }

    /// Select action using ε-greedy policy
    pub async fn select_action(&self, state: &WorkflowState, available_actions: &[TaskAction]) -> TaskAction {
        // Exploration
        if fastrand::f32() < self.epsilon {
            return available_actions[fastrand::usize(0..available_actions.len())];
        }

        // Exploitation: argmax Q(s, a)
        let table = self.q_table.read().await;

        if let Some(action_values) = table.get(state) {
            let best_action = available_actions.iter()
                .max_by(|a, b| {
                    let q_a = action_values.get(a).copied().unwrap_or(0.0);
                    let q_b = action_values.get(b).copied().unwrap_or(0.0);
                    q_a.partial_cmp(&q_b).unwrap()
                })
                .copied()
                .unwrap();

            best_action
        } else {
            // Unseen state: random action
            available_actions[fastrand::usize(0..available_actions.len())]
        }
    }

    /// Learn from transition
    pub async fn learn(&mut self, transition: &Transition) {
        let mut table = self.q_table.write().await;

        // Get current Q-value
        let current_q = table
            .get(&transition.state)
            .and_then(|actions| actions.get(&transition.action))
            .copied()
            .unwrap_or(0.0);

        // Get max Q-value for next state
        let max_next_q = table
            .get(&transition.next_state)
            .map(|actions| {
                actions.values()
                    .copied()
                    .fold(f32::NEG_INFINITY, f32::max)
            })
            .unwrap_or(0.0);

        // TD target: r + γ max Q(s', a')
        let target = transition.reward + self.discount_factor * max_next_q;

        // Update: Q(s, a) ← Q(s, a) + α[target - Q(s, a)]
        let new_q = current_q + self.learning_rate * (target - current_q);

        // Store updated Q-value
        table
            .entry(transition.state.clone())
            .or_insert_with(HashMap::new)
            .insert(transition.action, new_q);

        // Decay epsilon
        self.epsilon = (self.epsilon * self.epsilon_decay).max(self.epsilon_min);
    }

    /// Get Q-value for state-action pair
    pub async fn get_q_value(&self, state: &WorkflowState, action: &TaskAction) -> f32 {
        let table = self.q_table.read().await;
        table
            .get(state)
            .and_then(|actions| actions.get(action))
            .copied()
            .unwrap_or(0.0)
    }
}

// Example usage
#[tokio::main]
async fn main() {
    println!("Q-Learning Workflow Optimizer Example\n");

    // Create agent
    let mut agent = QLearningAgent::new();

    // Simulate 1000 workflow executions
    for episode in 0..1000 {
        let mut state = WorkflowState {
            completed_tasks: 0,
            current_phase: 0,
            resource_usage: 50,
            failures: 0,
        };

        let mut total_reward = 0.0;

        // Execute workflow (max 10 tasks)
        for step in 0..10 {
            // Available actions (tasks to execute)
            let available_actions = vec![
                TaskAction { task_id: step, priority: Priority::High },
                TaskAction { task_id: step, priority: Priority::Medium },
                TaskAction { task_id: step, priority: Priority::Low },
            ];

            // Select action
            let action = agent.select_action(&state, &available_actions).await;

            // Simulate execution
            let (reward, next_state) = simulate_execution(&state, &action);
            total_reward += reward;

            // Learn from transition
            let transition = Transition {
                state: state.clone(),
                action,
                reward,
                next_state: next_state.clone(),
            };
            agent.learn(&transition).await;

            // Move to next state
            state = next_state;

            // Check if done
            if state.completed_tasks == 0x3FF {  // All 10 tasks done
                break;
            }
        }

        // Print progress every 100 episodes
        if episode % 100 == 0 {
            println!("Episode {}: Total Reward = {:.2}, Epsilon = {:.3}",
                     episode, total_reward, agent.epsilon);
        }
    }

    println!("\nTraining complete!");
    println!("Final epsilon: {:.3}", agent.epsilon);
}

// Simulate task execution
fn simulate_execution(state: &WorkflowState, action: &TaskAction) -> (f32, WorkflowState) {
    let mut next_state = state.clone();

    // Mark task as completed
    next_state.completed_tasks |= 1 << action.task_id;

    // Update resource usage based on priority
    let resource_delta = match action.priority {
        Priority::High => 20,
        Priority::Medium => 10,
        Priority::Low => 5,
    };
    next_state.resource_usage = (next_state.resource_usage as i16 + resource_delta).clamp(0, 100) as u8;

    // Advance phase
    if action.task_id % 3 == 0 {
        next_state.current_phase += 1;
    }

    // Simulate failure (10% chance)
    let failed = fastrand::f32() < 0.1;
    if failed {
        next_state.failures += 1;
    }

    // Compute reward
    let reward = if failed {
        -10.0  // Penalty for failure
    } else {
        match action.priority {
            Priority::High => 10.0,   // High priority tasks give more reward
            Priority::Medium => 5.0,
            Priority::Low => 2.0,
        }
    };

    (reward, next_state)
}

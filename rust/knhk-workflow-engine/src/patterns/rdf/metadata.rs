//! Pattern metadata definitions and retrieval

use serde::{Deserialize, Serialize};

/// Pattern metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMetadata {
    /// Pattern ID (1-43)
    pub pattern_id: u32,
    /// Pattern name
    pub name: String,
    /// Pattern description
    pub description: String,
    /// Pattern category
    pub category: String,
    /// Pattern complexity
    pub complexity: String,
    /// Pattern dependencies (other pattern IDs)
    pub dependencies: Vec<u32>,
}

impl PatternMetadata {
    /// Create pattern metadata
    pub fn new(
        pattern_id: u32,
        name: String,
        description: String,
        category: String,
        complexity: String,
        dependencies: Vec<u32>,
    ) -> Self {
        Self {
            pattern_id,
            name,
            description,
            category,
            complexity,
            dependencies,
        }
    }
}

impl Default for PatternMetadata {
    fn default() -> Self {
        Self {
            pattern_id: 1,
            name: "Default Pattern".to_string(),
            description: "Default pattern metadata".to_string(),
            category: "Basic Control Flow".to_string(),
            complexity: "Simple".to_string(),
            dependencies: vec![],
        }
    }
}

/// Get all pattern metadata
pub fn get_all_pattern_metadata() -> Vec<PatternMetadata> {
    vec![
        // Basic Control Flow (1-5)
        PatternMetadata::new(
            1,
            "Sequence".to_string(),
            "Execute activities in strict sequential order".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            2,
            "Parallel Split".to_string(),
            "Split execution into multiple parallel branches".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            3,
            "Synchronization".to_string(),
            "Synchronize multiple parallel branches".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![2],
        ),
        PatternMetadata::new(
            4,
            "Exclusive Choice".to_string(),
            "Choose one branch based on condition".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            5,
            "Simple Merge".to_string(),
            "Merge multiple branches into one".to_string(),
            "Basic Control Flow".to_string(),
            "Simple".to_string(),
            vec![4],
        ),
        // Advanced Branching (6-11)
        PatternMetadata::new(
            6,
            "Multi-Choice".to_string(),
            "Choose multiple branches based on conditions".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            7,
            "Structured Synchronizing Merge".to_string(),
            "Synchronize multiple branches with structured merge".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![6],
        ),
        PatternMetadata::new(
            8,
            "Multi-Merge".to_string(),
            "Merge multiple branches without synchronization".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            9,
            "Discriminator".to_string(),
            "Wait for first branch to complete, then continue".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            10,
            "Arbitrary Cycles".to_string(),
            "Support arbitrary cycles in workflow".to_string(),
            "Advanced Branching".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            11,
            "Implicit Termination".to_string(),
            "Terminate when all active branches complete".to_string(),
            "Advanced Branching".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        // Multiple Instance (12-15)
        PatternMetadata::new(
            12,
            "MI Without Sync".to_string(),
            "Multiple instances without synchronization".to_string(),
            "Multiple Instance".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            13,
            "MI With Design-Time Knowledge".to_string(),
            "Multiple instances with known count at design time".to_string(),
            "Multiple Instance".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            14,
            "MI With Runtime Knowledge".to_string(),
            "Multiple instances with count known at runtime".to_string(),
            "Multiple Instance".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            15,
            "MI Without Runtime Knowledge".to_string(),
            "Multiple instances with unknown count".to_string(),
            "Multiple Instance".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        // State-Based (16-18)
        PatternMetadata::new(
            16,
            "Deferred Choice".to_string(),
            "Choose branch based on external event".to_string(),
            "State-Based".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            17,
            "Interleaved Parallel Routing".to_string(),
            "Execute branches in interleaved order".to_string(),
            "State-Based".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            18,
            "Milestone".to_string(),
            "Enable activity when milestone is reached".to_string(),
            "State-Based".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        // Cancellation (19-25)
        PatternMetadata::new(
            19,
            "Cancel Activity".to_string(),
            "Cancel a specific activity".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            20,
            "Cancel Case".to_string(),
            "Cancel entire workflow case".to_string(),
            "Cancellation".to_string(),
            "Simple".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            21,
            "Cancel Region".to_string(),
            "Cancel a region of activities".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            22,
            "Cancel MI Activity".to_string(),
            "Cancel multiple instance activity".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![12, 13, 14, 15],
        ),
        PatternMetadata::new(
            23,
            "Complete MI Activity".to_string(),
            "Complete multiple instance activity".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![12, 13, 14, 15],
        ),
        PatternMetadata::new(
            24,
            "Blocking Discriminator".to_string(),
            "Wait for first branch, block others".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![9],
        ),
        PatternMetadata::new(
            25,
            "Cancelling Discriminator".to_string(),
            "Wait for first branch, cancel others".to_string(),
            "Cancellation".to_string(),
            "Medium".to_string(),
            vec![9],
        ),
        // Advanced Control (26-39)
        PatternMetadata::new(
            26,
            "Stateful Resource Allocation".to_string(),
            "Allocates and manages workflow resources based on process state and availability constraints".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            27,
            "General Synchronizing Merge".to_string(),
            "Converges multiple branches based on runtime conditions determining synchronization requirements".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![26],
        ),
        PatternMetadata::new(
            28,
            "Thread-Safe Blocking Discriminator".to_string(),
            "Enables subsequent activity after first incoming branch completes, blocks remaining branches with thread safety".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![9],
        ),
        PatternMetadata::new(
            29,
            "Structured Cancelling Discriminator".to_string(),
            "Enables subsequent activity after first incoming branch completes, cancels remaining branches with structured cleanup".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![9, 19],
        ),
        PatternMetadata::new(
            30,
            "Structured Partial Join for Multiple Instances".to_string(),
            "Synchronizes subset of active branches from multiple instance activity based on predetermined threshold".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![13, 14],
        ),
        PatternMetadata::new(
            31,
            "Blocking Partial Join for Multiple Instances".to_string(),
            "Synchronizes subset of branches, blocks remaining branches until synchronization threshold met".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![30],
        ),
        PatternMetadata::new(
            32,
            "Cancelling Partial Join for Multiple Instances".to_string(),
            "Synchronizes subset of branches, cancels remaining branches once synchronization threshold met".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![30, 19],
        ),
        PatternMetadata::new(
            33,
            "Generalized AND-Join".to_string(),
            "Converges multiple branches with flexible synchronization semantics based on process structure".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![3, 26],
        ),
        PatternMetadata::new(
            34,
            "Static Partial Join for Multiple Instances".to_string(),
            "Partial join where synchronization cardinality is determined at design time".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![30],
        ),
        PatternMetadata::new(
            35,
            "Cancelling Partial Join with Early Termination".to_string(),
            "Partial join that cancels non-synchronized branches after threshold completion with early termination".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![30, 32],
        ),
        PatternMetadata::new(
            36,
            "Dynamic Partial Join for Multiple Instances".to_string(),
            "Partial join where synchronization cardinality is determined at runtime".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![30],
        ),
        PatternMetadata::new(
            37,
            "Acyclic Synchronizing Merge".to_string(),
            "Synchronizing merge for acyclic process structures with predictable merge semantics".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![26],
        ),
        PatternMetadata::new(
            38,
            "Local Synchronizing Merge".to_string(),
            "Most general form of synchronizing merge supporting arbitrary control flow patterns with local synchronization".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![26, 27],
        ),
        PatternMetadata::new(
            39,
            "Critical Section".to_string(),
            "Ensures mutual exclusion for designated workflow region preventing concurrent execution".to_string(),
            "Advanced Control".to_string(),
            "Complex".to_string(),
            vec![],
        ),
        // Trigger Patterns (40-43)
        PatternMetadata::new(
            40,
            "Transient Trigger".to_string(),
            "Enables activity based on transient external event that occurs at specific point in time".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            41,
            "Persistent Trigger".to_string(),
            "Enables activity based on persistent condition that remains true until explicitly cleared".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            42,
            "Auto-start Task".to_string(),
            "Automatically triggers task execution when enabling conditions are met without manual intervention".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
        PatternMetadata::new(
            43,
            "Fire-and-Forget".to_string(),
            "Initiates activity execution asynchronously without waiting for completion or tracking outcome".to_string(),
            "Trigger".to_string(),
            "Medium".to_string(),
            vec![],
        ),
    ]
}

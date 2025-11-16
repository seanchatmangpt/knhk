//! 43 YAWL Patterns - Compiled Dispatch Table
//!
//! Van der Aalst workflow patterns as μ-kernel instructions

use core::fmt;

/// Pattern ID (0-42, maps to Van der Aalst patterns)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PatternId {
    // Basic Control Flow (1-5)
    Sequence = 0,
    ParallelSplit = 1,
    Synchronization = 2,
    ExclusiveChoice = 3,
    SimpleMerge = 4,

    // Advanced Branching (6-11)
    MultiChoice = 5,
    StructuredSynchronizingMerge = 6,
    MultiMerge = 7,
    Discriminator = 8,
    ArbitraryCycles = 9,
    ImplicitTermination = 10,

    // Multiple Instance (12-15)
    MultipleInstancesWithoutSync = 11,
    MultipleInstancesWithDesignTimeKnowledge = 12,
    MultipleInstancesWithRuntimeKnowledge = 13,
    MultipleInstancesWithoutRuntimeKnowledge = 14,

    // State-Based (16-18)
    DeferredChoice = 15,
    InterleavedParallelRouting = 16,
    Milestone = 17,

    // Cancellation (19-25)
    CancelActivity = 18,
    CancelCase = 19,
    CancelRegion = 20,
    CancelMultipleInstanceActivity = 21,
    CompleteMultipleInstanceActivity = 22,
    BlockingDiscriminator = 23,
    CancellingDiscriminator = 24,

    // Advanced Patterns (26-39)
    StructuredPartialJoin = 25,
    BlockingPartialJoin = 26,
    CancellingPartialJoin = 27,
    GeneralizedAndJoin = 28,
    ThreadMerge = 29,
    ThreadSplit = 30,
    LocalSynchronizingMerge = 31,
    GeneralSynchronizingMerge = 32,
    CriticalSection = 33,
    InterleavedRouting = 34,

    // Trigger Patterns (40-43)
    PersistentTrigger = 35,
    TransientTrigger = 36,
    CancelTrigger = 37,
    CompletionTrigger = 38,

    // Reserved for future patterns
    Reserved39 = 39,
    Reserved40 = 40,
    Reserved41 = 41,
    Reserved42 = 42,
}

impl PatternId {
    /// Get tick cost for this pattern (hot path)
    #[inline(always)]
    pub const fn tick_cost(&self) -> u8 {
        // Costs determined by complexity
        const COSTS: [u8; 43] = [
            1, // Sequence
            2, // ParallelSplit
            3, // Synchronization
            1, // ExclusiveChoice
            2, // SimpleMerge
            3, // MultiChoice
            4, // StructuredSynchronizingMerge
            3, // MultiMerge
            4, // Discriminator
            5, // ArbitraryCycles
            2, // ImplicitTermination
            5, // MultipleInstancesWithoutSync
            6, // MultipleInstancesWithDesignTimeKnowledge
            7, // MultipleInstancesWithRuntimeKnowledge
            6, // MultipleInstancesWithoutRuntimeKnowledge
            3, // DeferredChoice
            5, // InterleavedParallelRouting
            4, // Milestone
            2, // CancelActivity
            3, // CancelCase
            4, // CancelRegion
            5, // CancelMultipleInstanceActivity
            4, // CompleteMultipleInstanceActivity
            5, // BlockingDiscriminator
            5, // CancellingDiscriminator
            6, // StructuredPartialJoin
            6, // BlockingPartialJoin
            6, // CancellingPartialJoin
            7, // GeneralizedAndJoin
            4, // ThreadMerge
            4, // ThreadSplit
            5, // LocalSynchronizingMerge
            6, // GeneralSynchronizingMerge
            5, // CriticalSection
            6, // InterleavedRouting
            5, // PersistentTrigger
            4, // TransientTrigger
            3, // CancelTrigger
            4, // CompletionTrigger
            1, // Reserved
            1, // Reserved
            1, // Reserved
            1, // Reserved
        ];

        COSTS[*self as usize]
    }

    /// Check if pattern is hot-path eligible (≤8 ticks)
    #[inline(always)]
    pub const fn is_hot_path_eligible(&self) -> bool {
        self.tick_cost() <= crate::CHATMAN_CONSTANT as u8
    }

    /// Get pattern name
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Sequence => "Sequence",
            Self::ParallelSplit => "Parallel Split",
            Self::Synchronization => "Synchronization",
            Self::ExclusiveChoice => "Exclusive Choice",
            Self::SimpleMerge => "Simple Merge",
            Self::MultiChoice => "Multi-Choice",
            Self::StructuredSynchronizingMerge => "Structured Synchronizing Merge",
            Self::MultiMerge => "Multi-Merge",
            Self::Discriminator => "Discriminator",
            Self::ArbitraryCycles => "Arbitrary Cycles",
            Self::ImplicitTermination => "Implicit Termination",
            Self::MultipleInstancesWithoutSync => "Multiple Instances without Synchronization",
            Self::MultipleInstancesWithDesignTimeKnowledge => "Multiple Instances with a priori Design-Time Knowledge",
            Self::MultipleInstancesWithRuntimeKnowledge => "Multiple Instances with a priori Runtime Knowledge",
            Self::MultipleInstancesWithoutRuntimeKnowledge => "Multiple Instances without a priori Runtime Knowledge",
            Self::DeferredChoice => "Deferred Choice",
            Self::InterleavedParallelRouting => "Interleaved Parallel Routing",
            Self::Milestone => "Milestone",
            Self::CancelActivity => "Cancel Activity",
            Self::CancelCase => "Cancel Case",
            Self::CancelRegion => "Cancel Region",
            Self::CancelMultipleInstanceActivity => "Cancel Multiple Instance Activity",
            Self::CompleteMultipleInstanceActivity => "Complete Multiple Instance Activity",
            Self::BlockingDiscriminator => "Blocking Discriminator",
            Self::CancellingDiscriminator => "Cancelling Discriminator",
            Self::StructuredPartialJoin => "Structured Partial Join",
            Self::BlockingPartialJoin => "Blocking Partial Join",
            Self::CancellingPartialJoin => "Cancelling Partial Join",
            Self::GeneralizedAndJoin => "Generalized AND-Join",
            Self::ThreadMerge => "Thread Merge",
            Self::ThreadSplit => "Thread Split",
            Self::LocalSynchronizingMerge => "Local Synchronizing Merge",
            Self::GeneralSynchronizingMerge => "General Synchronizing Merge",
            Self::CriticalSection => "Critical Section",
            Self::InterleavedRouting => "Interleaved Routing",
            Self::PersistentTrigger => "Persistent Trigger",
            Self::TransientTrigger => "Transient Trigger",
            Self::CancelTrigger => "Cancel Trigger",
            Self::CompletionTrigger => "Completion Trigger",
            _ => "Reserved",
        }
    }
}

impl fmt::Display for PatternId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.name(), *self as u8)
    }
}

/// Pattern dispatch table entry
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]  // Cache-line aligned
pub struct PatternEntry {
    /// Pattern ID
    pub pattern_id: PatternId,
    /// Phase handlers (Init, Exec, Join, Cancel, etc.)
    pub phases: [u64; 8],  // Function pointers
    /// Tick budget for this pattern
    pub tick_budget: u8,
    /// Guard bitmap (which guards apply)
    pub guard_bitmap: u64,
    /// Reserved for future use
    _reserved: [u8; 7],
}

/// Dispatch table for all 43 patterns
#[repr(C, align(4096))]  // Page-aligned
pub struct DispatchTable {
    /// Pattern entries (256 for alignment, only 43 used)
    entries: [PatternEntry; 256],
}

impl DispatchTable {
    /// Create a new dispatch table
    pub const fn new() -> Self {
        const EMPTY_ENTRY: PatternEntry = PatternEntry {
            pattern_id: PatternId::Sequence,
            phases: [0; 8],
            tick_budget: 0,
            guard_bitmap: 0,
            _reserved: [0; 7],
        };

        Self {
            entries: [EMPTY_ENTRY; 256],
        }
    }

    /// Get pattern entry by ID (O(1) lookup)
    #[inline(always)]
    pub fn get(&self, pattern_id: PatternId) -> Option<&PatternEntry> {
        let index = pattern_id as usize;
        if index < 43 {
            Some(&self.entries[index])
        } else {
            None
        }
    }

    /// Get pattern entry mutably
    #[inline(always)]
    pub fn get_mut(&mut self, pattern_id: PatternId) -> Option<&mut PatternEntry> {
        let index = pattern_id as usize;
        if index < 43 {
            Some(&mut self.entries[index])
        } else {
            None
        }
    }
}

impl Default for DispatchTable {
    fn default() -> Self {
        Self::new()
    }
}

/// Guard ID (for guard evaluation)
pub type GuardId = u16;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_tick_costs() {
        // Sequence should be cheapest
        assert_eq!(PatternId::Sequence.tick_cost(), 1);

        // More complex patterns cost more
        assert!(PatternId::GeneralizedAndJoin.tick_cost() > PatternId::Sequence.tick_cost());
    }

    #[test]
    fn test_hot_path_eligibility() {
        // Sequence is hot-path eligible
        assert!(PatternId::Sequence.is_hot_path_eligible());

        // Most patterns should be hot-path eligible (≤8 ticks)
        let hot_path_count = (0..43)
            .filter(|&i| {
                let pattern: PatternId = unsafe { core::mem::transmute(i as u8) };
                pattern.is_hot_path_eligible()
            })
            .count();

        assert!(hot_path_count >= 30, "At least 30 patterns should be hot-path eligible");
    }

    #[test]
    fn test_dispatch_table() {
        let table = DispatchTable::new();

        // Should be able to get any valid pattern
        assert!(table.get(PatternId::Sequence).is_some());
        assert!(table.get(PatternId::ParallelSplit).is_some());
        assert!(table.get(PatternId::CompletionTrigger).is_some());
    }

    #[test]
    fn test_pattern_display() {
        assert_eq!(format!("{}", PatternId::Sequence), "Sequence(0)");
        assert_eq!(format!("{}", PatternId::ParallelSplit), "Parallel Split(1)");
    }

    #[test]
    fn test_all_patterns_have_names() {
        for i in 0..43 {
            let pattern: PatternId = unsafe { core::mem::transmute(i as u8) };
            let name = pattern.name();
            assert!(!name.is_empty());
            assert!(!name.starts_with("Reserved") || i >= 39);
        }
    }
}

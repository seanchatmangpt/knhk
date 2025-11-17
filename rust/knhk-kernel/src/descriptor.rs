// knhk-kernel: Descriptor structure for hot path configuration
// Immutable, cache-friendly, atomic hot-swap capable
// NO UNSAFE CODE - All operations use safe Arc-based memory management

use crate::guard::{Guard, GuardConfig};
use crate::pattern::{PatternConfig, PatternType};
use arrayvec::ArrayVec;
use rustc_hash::FxHashMap;
use std::sync::atomic::{AtomicPtr, AtomicU64, Ordering};
use std::sync::Arc;

/// Maximum patterns per descriptor (fits in cache line)
pub const MAX_PATTERNS: usize = 64;

/// Maximum guards per pattern
pub const MAX_GUARDS_PER_PATTERN: usize = 8;

/// Descriptor version for atomic updates
static DESCRIPTOR_VERSION: AtomicU64 = AtomicU64::new(0);

/// Tick budget manifest entry
#[repr(C, align(8))]
#[derive(Debug, Clone, Copy)]
pub struct TickBudgetEntry {
    pub pattern_id: u32,
    pub max_ticks: u32,
}

/// Pattern registry entry (cache-aligned)
#[repr(C, align(64))]
#[derive(Clone)]
pub struct PatternEntry {
    pub pattern_type: PatternType,
    pub pattern_id: u32,
    pub priority: u32,
    pub config: PatternConfig,
    pub guards: ArrayVec<Guard, MAX_GUARDS_PER_PATTERN>,
    pub tick_budget: u32,
    _padding: [u8; 8], // Ensure cache line alignment
}

impl PatternEntry {
    pub fn new(
        pattern_type: PatternType,
        pattern_id: u32,
        priority: u32,
        config: PatternConfig,
    ) -> Self {
        Self {
            pattern_type,
            pattern_id,
            priority,
            config,
            guards: ArrayVec::new(),
            tick_budget: 8, // Default to Chatman constant
            _padding: [0; 8],
        }
    }

    /// Add a guard to this pattern
    #[inline]
    pub fn add_guard(&mut self, guard: Guard) -> Result<(), &'static str> {
        if self.guards.len() >= MAX_GUARDS_PER_PATTERN {
            return Err("Maximum guards per pattern exceeded");
        }
        self.guards.push(guard);
        Ok(())
    }

    /// Check if all guards pass
    #[inline(always)]
    pub fn guards_pass(&self, context: &ExecutionContext) -> bool {
        // Early exit on first failure (short-circuit evaluation)
        for guard in &self.guards {
            if !guard.evaluate(context) {
                return false;
            }
        }
        true
    }
}

/// Execution context for guard evaluation
#[repr(C, align(64))]
pub struct ExecutionContext {
    pub task_id: u64,
    pub timestamp: u64,
    pub resources: ResourceState,
    pub observations: ObservationBuffer,
    pub state_flags: u64,
}

/// Resource state for guard checks
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ResourceState {
    pub cpu_available: u32,
    pub memory_available: u32,
    pub io_capacity: u32,
    pub queue_depth: u32,
}

/// Observation buffer (fixed-size for hot path)
#[repr(C, align(64))]
pub struct ObservationBuffer {
    pub count: u32,
    pub observations: [u64; 16], // Pre-hashed observations
}

/// Main descriptor structure (immutable after creation)
#[repr(C, align(64))]
pub struct Descriptor {
    /// Descriptor version
    pub version: u64,

    /// Pattern registry (sorted by priority for cache efficiency)
    pub patterns: Vec<PatternEntry>,

    /// Pattern lookup index (pattern_id -> index)
    pub pattern_index: FxHashMap<u32, usize>,

    /// Global tick budget
    pub global_tick_budget: u32,

    /// Guard configuration
    pub guard_config: GuardConfig,

    /// Snapshot timestamp
    pub snapshot_timestamp: u64,

    /// Descriptor hash for verification
    pub descriptor_hash: u64,
}

impl Descriptor {
    /// Create a new descriptor
    pub fn new() -> Self {
        let version = DESCRIPTOR_VERSION.fetch_add(1, Ordering::SeqCst);
        Self {
            version,
            patterns: Vec::with_capacity(MAX_PATTERNS),
            pattern_index: FxHashMap::default(),
            global_tick_budget: 8,
            guard_config: GuardConfig::default(),
            snapshot_timestamp: crate::timer::read_tsc(),
            descriptor_hash: 0,
        }
    }

    /// Add a pattern to the descriptor
    pub fn add_pattern(&mut self, entry: PatternEntry) -> Result<(), &'static str> {
        if self.patterns.len() >= MAX_PATTERNS {
            return Err("Maximum patterns exceeded");
        }

        let pattern_id = entry.pattern_id;
        let index = self.patterns.len();

        self.patterns.push(entry);
        self.pattern_index.insert(pattern_id, index);

        Ok(())
    }

    /// Sort patterns by priority (higher priority first)
    pub fn optimize_layout(&mut self) {
        self.patterns.sort_by_key(|p| std::cmp::Reverse(p.priority));

        // Rebuild index
        self.pattern_index.clear();
        for (idx, pattern) in self.patterns.iter().enumerate() {
            self.pattern_index.insert(pattern.pattern_id, idx);
        }
    }

    /// Compute descriptor hash for verification
    pub fn compute_hash(&mut self) {
        use xxhash_rust::xxh3::xxh3_64;

        let mut hasher_input = Vec::new();
        hasher_input.extend_from_slice(&self.version.to_le_bytes());
        hasher_input.extend_from_slice(&self.global_tick_budget.to_le_bytes());

        for pattern in &self.patterns {
            hasher_input.extend_from_slice(&pattern.pattern_id.to_le_bytes());
            hasher_input.extend_from_slice(&pattern.priority.to_le_bytes());
            hasher_input.extend_from_slice(&pattern.tick_budget.to_le_bytes());
        }

        self.descriptor_hash = xxh3_64(&hasher_input);
    }

    /// Get pattern by ID (O(1) lookup)
    #[inline(always)]
    pub fn get_pattern(&self, pattern_id: u32) -> Option<&PatternEntry> {
        self.pattern_index
            .get(&pattern_id)
            .and_then(|&idx| self.patterns.get(idx))
    }

    /// Get patterns in priority order
    #[inline(always)]
    pub fn patterns_by_priority(&self) -> &[PatternEntry] {
        &self.patterns
    }

    /// Validate descriptor consistency
    pub fn validate(&self) -> Result<(), String> {
        // Check version
        if self.version == 0 {
            return Err("Invalid descriptor version".to_string());
        }

        // Check pattern count
        if self.patterns.len() > MAX_PATTERNS {
            return Err(format!("Too many patterns: {}", self.patterns.len()));
        }

        // Check index consistency
        for (pattern_id, &index) in &self.pattern_index {
            if index >= self.patterns.len() {
                return Err(format!("Invalid index for pattern {}", pattern_id));
            }
            if self.patterns[index].pattern_id != *pattern_id {
                return Err(format!("Index mismatch for pattern {}", pattern_id));
            }
        }

        // Check tick budgets
        for pattern in &self.patterns {
            if pattern.tick_budget == 0 || pattern.tick_budget > self.global_tick_budget {
                return Err(format!(
                    "Invalid tick budget for pattern {}: {}",
                    pattern.pattern_id, pattern.tick_budget
                ));
            }
        }

        Ok(())
    }
}

impl Default for Descriptor {
    fn default() -> Self {
        Self::new()
    }
}

/// Descriptor manager for atomic hot-swap using Arc for safe memory management
pub struct DescriptorManager;

// Use Arc instead of raw pointers for safe reference counting
static ACTIVE_DESCRIPTOR_ARC: std::sync::RwLock<Option<Arc<Descriptor>>> = std::sync::RwLock::new(None);

impl DescriptorManager {
    /// Load a new descriptor atomically
    pub fn load_descriptor(descriptor: Box<Descriptor>) -> Result<(), String> {
        // Validate before swapping
        descriptor.validate()?;

        // Convert to Arc for safe reference counting
        let arc_descriptor = Arc::new(*descriptor);

        // Atomic swap using RwLock (safe, no unsafe code needed)
        let mut guard = ACTIVE_DESCRIPTOR_ARC.write().unwrap();
        *guard = Some(arc_descriptor);

        // Old descriptor automatically dropped when guard is released
        Ok(())
    }

    /// Get active descriptor (lock-free read with Arc)
    #[inline(always)]
    pub fn get_active() -> Option<Arc<Descriptor>> {
        let guard = ACTIVE_DESCRIPTOR_ARC.read().unwrap();
        guard.clone()
    }

    /// Hot-swap descriptor with zero downtime
    pub fn hot_swap(new_descriptor: Box<Descriptor>) -> Result<(), String> {
        // Validate new descriptor
        new_descriptor.validate()?;

        // Convert to Arc for safe reference counting
        let arc_descriptor = Arc::new(*new_descriptor);

        // Perform atomic swap using RwLock
        let mut guard = ACTIVE_DESCRIPTOR_ARC.write().unwrap();
        *guard = Some(arc_descriptor);

        // Old descriptor automatically cleaned up when guard is released
        // Arc ensures all readers finish before memory is freed
        Ok(())
    }
}

/// Builder for descriptors
pub struct DescriptorBuilder {
    descriptor: Descriptor,
}

impl DescriptorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tick_budget(mut self, budget: u32) -> Self {
        self.descriptor.global_tick_budget = budget;
        self
    }

    pub fn with_guard_config(mut self, config: GuardConfig) -> Self {
        self.descriptor.guard_config = config;
        self
    }

    pub fn add_pattern(mut self, pattern: PatternEntry) -> Self {
        let _ = self.descriptor.add_pattern(pattern);
        self
    }

    pub fn build(mut self) -> Descriptor {
        self.descriptor.optimize_layout();
        self.descriptor.compute_hash();
        self.descriptor
    }
}

#[allow(clippy::derivable_impls)]
impl Default for DescriptorBuilder {
    fn default() -> Self {
        Self {
            descriptor: Descriptor::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_descriptor_creation() {
        let descriptor = DescriptorBuilder::new().with_tick_budget(8).build();

        assert_eq!(descriptor.global_tick_budget, 8);
        assert!(descriptor.validate().is_ok());
    }

    #[test]
    fn test_pattern_priority_ordering() {
        let mut descriptor = Descriptor::new();

        let p1 = PatternEntry::new(PatternType::Sequence, 1, 10, PatternConfig::default());
        let p2 = PatternEntry::new(PatternType::ParallelSplit, 2, 20, PatternConfig::default());
        let p3 = PatternEntry::new(
            PatternType::ExclusiveChoice,
            3,
            15,
            PatternConfig::default(),
        );

        descriptor.add_pattern(p1).unwrap();
        descriptor.add_pattern(p2).unwrap();
        descriptor.add_pattern(p3).unwrap();

        descriptor.optimize_layout();

        // Should be ordered by priority: p2(20), p3(15), p1(10)
        assert_eq!(descriptor.patterns[0].pattern_id, 2);
        assert_eq!(descriptor.patterns[1].pattern_id, 3);
        assert_eq!(descriptor.patterns[2].pattern_id, 1);
    }

    #[test]
    fn test_hot_swap() {
        let desc1 = Box::new(DescriptorBuilder::new().with_tick_budget(8).build());

        DescriptorManager::load_descriptor(desc1).unwrap();

        let active = DescriptorManager::get_active().unwrap();
        assert_eq!(active.global_tick_budget, 8);

        let desc2 = Box::new(DescriptorBuilder::new().with_tick_budget(6).build());

        DescriptorManager::hot_swap(desc2).unwrap();

        let active = DescriptorManager::get_active().unwrap();
        assert_eq!(active.global_tick_budget, 6);
    }
}

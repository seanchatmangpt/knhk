// knhk-kernel: Guard evaluation engine for hot path
// Boolean gates with zero-overhead evaluation

use crate::descriptor::ExecutionContext;
use bitflags::bitflags;

/// Guard types for different conditions
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardType {
    /// Simple predicate check
    Predicate = 0,
    /// Resource availability check
    Resource = 1,
    /// State flag check
    State = 2,
    /// Counter threshold check
    Counter = 3,
    /// Time window check
    TimeWindow = 4,
    /// Compound AND guard
    And = 5,
    /// Compound OR guard
    Or = 6,
    /// NOT guard (negation)
    Not = 7,
}

bitflags! {
    /// State flags for guard evaluation
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct StateFlags: u64 {
        const INITIALIZED = 0b00000001;
        const RUNNING     = 0b00000010;
        const SUSPENDED   = 0b00000100;
        const COMPLETED   = 0b00001000;
        const FAILED      = 0b00010000;
        const CANCELLED   = 0b00100000;
        const TIMEOUT     = 0b01000000;
        const RESOURCE_OK = 0b10000000;
    }
}

/// Guard configuration
#[derive(Debug, Clone, Default)]
pub struct GuardConfig {
    pub max_depth: u32,
    pub enable_caching: bool,
    pub cache_ttl_ticks: u64,
}

/// Predicate types for guard conditions
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Predicate {
    Equal = 0,
    NotEqual = 1,
    LessThan = 2,
    LessThanOrEqual = 3,
    GreaterThan = 4,
    GreaterThanOrEqual = 5,
    BitSet = 6,
    BitClear = 7,
    InRange = 8,
    NotInRange = 9,
}

/// Resource type for resource guards
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Cpu = 0,
    Memory = 1,
    Io = 2,
    Queue = 3,
}

/// Guard structure (compact for cache efficiency)
#[repr(C)]
#[derive(Clone)]
pub struct Guard {
    pub guard_type: GuardType,
    pub predicate: Predicate,
    pub operand_a: u64,
    pub operand_b: u64,
    pub children: Vec<Guard>, // For compound guards
}

impl Guard {
    /// Create a simple predicate guard
    #[inline]
    pub fn predicate(pred: Predicate, a: u64, b: u64) -> Self {
        Self {
            guard_type: GuardType::Predicate,
            predicate: pred,
            operand_a: a,
            operand_b: b,
            children: Vec::new(),
        }
    }

    /// Create a resource guard
    #[inline]
    pub fn resource(resource: ResourceType, threshold: u32) -> Self {
        Self {
            guard_type: GuardType::Resource,
            predicate: Predicate::GreaterThanOrEqual,
            operand_a: resource as u64,
            operand_b: threshold as u64,
            children: Vec::new(),
        }
    }

    /// Create a state flag guard
    #[inline]
    pub fn state(flags: StateFlags) -> Self {
        Self {
            guard_type: GuardType::State,
            predicate: Predicate::BitSet,
            operand_a: flags.bits(),
            operand_b: 0,
            children: Vec::new(),
        }
    }

    /// Create an AND compound guard
    #[inline]
    pub fn and(guards: Vec<Guard>) -> Self {
        Self {
            guard_type: GuardType::And,
            predicate: Predicate::Equal,
            operand_a: 0,
            operand_b: 0,
            children: guards,
        }
    }

    /// Create an OR compound guard
    #[inline]
    pub fn or(guards: Vec<Guard>) -> Self {
        Self {
            guard_type: GuardType::Or,
            predicate: Predicate::Equal,
            operand_a: 0,
            operand_b: 0,
            children: guards,
        }
    }

    /// Create a NOT guard
    #[inline]
    pub fn negate(guard: Guard) -> Self {
        Self {
            guard_type: GuardType::Not,
            predicate: Predicate::Equal,
            operand_a: 0,
            operand_b: 0,
            children: vec![guard],
        }
    }

    /// Evaluate guard against context (hot path optimized)
    #[inline(always)]
    pub fn evaluate(&self, context: &ExecutionContext) -> bool {
        match self.guard_type {
            GuardType::Predicate => self.evaluate_predicate(context),
            GuardType::Resource => self.evaluate_resource(context),
            GuardType::State => self.evaluate_state(context),
            GuardType::Counter => self.evaluate_counter(context),
            GuardType::TimeWindow => self.evaluate_time_window(context),
            GuardType::And => self.evaluate_and(context),
            GuardType::Or => self.evaluate_or(context),
            GuardType::Not => self.evaluate_not(context),
        }
    }

    /// Evaluate predicate guard (branchless where possible)
    #[inline(always)]
    fn evaluate_predicate(&self, context: &ExecutionContext) -> bool {
        let value = self.extract_value(context);

        match self.predicate {
            Predicate::Equal => value == self.operand_b,
            Predicate::NotEqual => value != self.operand_b,
            Predicate::LessThan => value < self.operand_b,
            Predicate::LessThanOrEqual => value <= self.operand_b,
            Predicate::GreaterThan => value > self.operand_b,
            Predicate::GreaterThanOrEqual => value >= self.operand_b,
            Predicate::BitSet => (value & self.operand_b) == self.operand_b,
            Predicate::BitClear => (value & self.operand_b) == 0,
            Predicate::InRange => value >= self.operand_a && value <= self.operand_b,
            Predicate::NotInRange => value < self.operand_a || value > self.operand_b,
        }
    }

    /// Evaluate resource guard
    #[inline(always)]
    fn evaluate_resource(&self, context: &ExecutionContext) -> bool {
        let resource_type = self.operand_a as u8;
        let threshold = self.operand_b as u32;

        let available = match resource_type {
            0 => context.resources.cpu_available,
            1 => context.resources.memory_available,
            2 => context.resources.io_capacity,
            3 => context.resources.queue_depth,
            _ => 0,
        };

        available >= threshold
    }

    /// Evaluate state flag guard
    #[inline(always)]
    fn evaluate_state(&self, context: &ExecutionContext) -> bool {
        let required_flags = self.operand_a;
        (context.state_flags & required_flags) == required_flags
    }

    /// Evaluate counter guard
    #[inline(always)]
    fn evaluate_counter(&self, context: &ExecutionContext) -> bool {
        let counter_value = context.observations.count as u64;

        match self.predicate {
            Predicate::GreaterThanOrEqual => counter_value >= self.operand_b,
            Predicate::LessThanOrEqual => counter_value <= self.operand_b,
            _ => counter_value == self.operand_b,
        }
    }

    /// Evaluate time window guard
    #[inline(always)]
    fn evaluate_time_window(&self, context: &ExecutionContext) -> bool {
        let current_time = context.timestamp;
        let window_start = self.operand_a;
        let window_end = self.operand_b;

        current_time >= window_start && current_time <= window_end
    }

    /// Evaluate AND compound guard (short-circuit)
    #[inline(always)]
    fn evaluate_and(&self, context: &ExecutionContext) -> bool {
        for child in &self.children {
            if !child.evaluate(context) {
                return false;
            }
        }
        true
    }

    /// Evaluate OR compound guard (short-circuit)
    #[inline(always)]
    fn evaluate_or(&self, context: &ExecutionContext) -> bool {
        for child in &self.children {
            if child.evaluate(context) {
                return true;
            }
        }
        false
    }

    /// Evaluate NOT guard
    #[inline(always)]
    fn evaluate_not(&self, context: &ExecutionContext) -> bool {
        if let Some(child) = self.children.first() {
            !child.evaluate(context)
        } else {
            false
        }
    }

    /// Extract value from context based on operand_a
    #[inline(always)]
    fn extract_value(&self, context: &ExecutionContext) -> u64 {
        // operand_a encodes the field selector
        match self.operand_a {
            0 => context.task_id,
            1 => context.timestamp,
            2 => context.state_flags,
            3 => context.observations.count as u64,
            _ => 0,
        }
    }
}

/// Guard evaluator with optimizations
pub struct GuardEvaluator {
    /// Cache for guard results (pattern_id -> (result, timestamp))
    cache: rustc_hash::FxHashMap<u32, (bool, u64)>,
    /// Cache TTL in ticks
    cache_ttl: u64,
}

impl GuardEvaluator {
    pub fn new(cache_ttl: u64) -> Self {
        Self {
            cache: rustc_hash::FxHashMap::default(),
            cache_ttl,
        }
    }

    /// Evaluate guard with caching
    #[inline]
    pub fn evaluate_cached(
        &mut self,
        pattern_id: u32,
        guard: &Guard,
        context: &ExecutionContext,
    ) -> bool {
        // Check cache
        if let Some(&(result, timestamp)) = self.cache.get(&pattern_id) {
            if context.timestamp - timestamp < self.cache_ttl {
                return result;
            }
        }

        // Evaluate and cache
        let result = guard.evaluate(context);
        self.cache.insert(pattern_id, (result, context.timestamp));

        result
    }

    /// Clear expired cache entries
    pub fn clear_expired(&mut self, current_timestamp: u64) {
        self.cache
            .retain(|_, &mut (_, timestamp)| current_timestamp - timestamp < self.cache_ttl);
    }
}

/// Optimized guard compiler (compile guards to machine code)
pub struct GuardCompiler;

impl GuardCompiler {
    /// Compile guard to optimized evaluation function
    pub fn compile(guard: &Guard) -> Box<dyn for<'a> Fn(&'a ExecutionContext) -> bool + '_> {
        match guard.guard_type {
            GuardType::Predicate => {
                let pred = guard.predicate;
                let op_a = guard.operand_a;
                let op_b = guard.operand_b;

                Box::new(move |ctx: &ExecutionContext| {
                    let value = match op_a {
                        0 => ctx.task_id,
                        1 => ctx.timestamp,
                        2 => ctx.state_flags,
                        3 => ctx.observations.count as u64,
                        _ => 0,
                    };

                    match pred {
                        Predicate::Equal => value == op_b,
                        Predicate::NotEqual => value != op_b,
                        Predicate::GreaterThan => value > op_b,
                        Predicate::GreaterThanOrEqual => value >= op_b,
                        _ => false,
                    }
                })
            }
            _ => Box::new(move |ctx| guard.evaluate(ctx)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> ExecutionContext {
        ExecutionContext {
            task_id: 42,
            timestamp: 1000,
            resources: ResourceState {
                cpu_available: 80,
                memory_available: 1024,
                io_capacity: 100,
                queue_depth: 10,
            },
            observations: crate::descriptor::ObservationBuffer {
                count: 5,
                observations: [0; 16],
            },
            state_flags: StateFlags::INITIALIZED.bits() | StateFlags::RUNNING.bits(),
        }
    }

    #[test]
    fn test_predicate_guard() {
        let context = create_test_context();

        let guard = Guard::predicate(Predicate::Equal, 0, 42); // task_id == 42
        assert!(guard.evaluate(&context));

        let guard = Guard::predicate(Predicate::GreaterThan, 1, 500); // timestamp > 500
        assert!(guard.evaluate(&context));

        let guard = Guard::predicate(Predicate::LessThan, 1, 500); // timestamp < 500
        assert!(!guard.evaluate(&context));
    }

    #[test]
    fn test_resource_guard() {
        let context = create_test_context();

        let guard = Guard::resource(ResourceType::Cpu, 50);
        assert!(guard.evaluate(&context)); // CPU available (80) >= 50

        let guard = Guard::resource(ResourceType::Memory, 2048);
        assert!(!guard.evaluate(&context)); // Memory (1024) < 2048
    }

    #[test]
    fn test_compound_guards() {
        let context = create_test_context();

        let g1 = Guard::predicate(Predicate::Equal, 0, 42);
        let g2 = Guard::resource(ResourceType::Cpu, 50);

        let and_guard = Guard::and(vec![g1.clone(), g2.clone()]);
        assert!(and_guard.evaluate(&context));

        let g3 = Guard::resource(ResourceType::Memory, 2048);
        let or_guard = Guard::or(vec![g2, g3]);
        assert!(or_guard.evaluate(&context)); // CPU check passes

        let not_guard = Guard::not(g1);
        assert!(!not_guard.evaluate(&context));
    }

    #[test]
    fn test_state_guard() {
        let context = create_test_context();

        let guard = Guard::state(StateFlags::INITIALIZED | StateFlags::RUNNING);
        assert!(guard.evaluate(&context));

        let guard = Guard::state(StateFlags::COMPLETED);
        assert!(!guard.evaluate(&context));
    }

    #[test]
    fn test_guard_evaluator_caching() {
        let mut evaluator = GuardEvaluator::new(100);
        let context = create_test_context();
        let guard = Guard::predicate(Predicate::Equal, 0, 42);

        // First evaluation
        let result = evaluator.evaluate_cached(1, &guard, &context);
        assert!(result);

        // Should use cache
        let result = evaluator.evaluate_cached(1, &guard, &context);
        assert!(result);

        // Clear expired entries (none should be expired yet)
        evaluator.clear_expired(context.timestamp + 50);
        assert_eq!(evaluator.cache.len(), 1);

        // Clear expired entries (should be expired now)
        evaluator.clear_expired(context.timestamp + 200);
        assert_eq!(evaluator.cache.len(), 0);
    }
}

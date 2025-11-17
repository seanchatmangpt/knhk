//! Swarm-Scale Control: Resource and Priority as Linear/Indexed Types
//!
//! Scarce resources as single-use tokens (linear types).
//! Priority and SLO bands as type indices.
//! Compile-time guarantees that code cannot overspend quota.

use crate::const_assert;
use core::marker::PhantomData;

/// Resource token - cannot be cloned (linear-ish semantics)
pub struct ResourceToken<const AMOUNT: u32> {
    _phantom: PhantomData<()>,
}

impl<const AMOUNT: u32> ResourceToken<AMOUNT> {
    /// Create resource token (private - only via allocator)
    const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Consume token - moves ownership
    pub fn consume(self) -> ConsumedToken<AMOUNT> {
        ConsumedToken {
            _phantom: PhantomData,
        }
    }

    /// Split token into smaller amounts
    pub fn split<const A1: u32, const A2: u32>(self) -> (ResourceToken<A1>, ResourceToken<A2>) {
        const_assert!(A1 + A2 == AMOUNT);
        (ResourceToken::new(), ResourceToken::new())
    }

    /// Get amount
    pub const fn amount() -> u32 {
        AMOUNT
    }

    /// Split token at runtime (for property-based testing)
    /// Note: In production code, use compile-time split() instead
    #[cfg(test)]
    pub fn split_runtime(self, _a1: u32, _a2: u32) -> (Self, Self) {
        // Returns tokens of same size for testing
        // Real implementation would need runtime validation
        (ResourceToken::new(), ResourceToken::new())
    }
}

// Prevent cloning - enforce linear semantics
// ResourceToken cannot be Copy or Clone

/// Consumed token - proof that resource was spent
pub struct ConsumedToken<const AMOUNT: u32> {
    _phantom: PhantomData<()>,
}

impl<const AMOUNT: u32> ConsumedToken<AMOUNT> {
    pub const fn amount() -> u32 {
        AMOUNT
    }
}

/// Resource quota - total budget
pub struct ResourceQuota<const TOTAL: u32> {
    remaining: u32,
}

impl<const TOTAL: u32> ResourceQuota<TOTAL> {
    /// Create quota
    pub const fn new() -> Self {
        Self { remaining: TOTAL }
    }

    /// Allocate resources - returns token if available
    pub fn allocate<const AMOUNT: u32>(&mut self) -> Option<ResourceToken<AMOUNT>> {
        if self.remaining >= AMOUNT {
            self.remaining -= AMOUNT;
            Some(ResourceToken::new())
        } else {
            None
        }
    }

    /// Return consumed token - restore quota
    pub fn reclaim<const AMOUNT: u32>(&mut self, _token: ConsumedToken<AMOUNT>) {
        self.remaining += AMOUNT;
    }

    /// Remaining resources
    pub const fn remaining(&self) -> u32 {
        self.remaining
    }

    /// Runtime allocation (for testing - prefer compile-time allocate())
    #[cfg(test)]
    pub fn allocate_generic<const AMOUNT: u32>(
        &mut self,
        runtime_amount: u32,
    ) -> Option<ResourceToken<AMOUNT>> {
        if self.remaining >= runtime_amount {
            self.remaining -= runtime_amount;
            Some(ResourceToken::new())
        } else {
            None
        }
    }
}

/// Priority class - type-level priority
pub trait PriorityClass: 'static {
    const LEVEL: u8; // 0 = highest
    const NAME: &'static str;
    const MAX_LATENCY_MS: u32;
}

/// P0 - Critical interactive
pub struct P0;
impl PriorityClass for P0 {
    const LEVEL: u8 = 0;
    const NAME: &'static str = "P0";
    const MAX_LATENCY_MS: u32 = 10;
}

/// P1 - High priority
pub struct P1;
impl PriorityClass for P1 {
    const LEVEL: u8 = 1;
    const NAME: &'static str = "P1";
    const MAX_LATENCY_MS: u32 = 100;
}

/// P2 - Normal priority
pub struct P2;
impl PriorityClass for P2 {
    const LEVEL: u8 = 2;
    const NAME: &'static str = "P2";
    const MAX_LATENCY_MS: u32 = 1000;
}

/// P3 - Low priority
pub struct P3;
impl PriorityClass for P3 {
    const LEVEL: u8 = 3;
    const NAME: &'static str = "P3";
    const MAX_LATENCY_MS: u32 = 10000;
}

/// P4 - Background
pub struct P4;
impl PriorityClass for P4 {
    const LEVEL: u8 = 4;
    const NAME: &'static str = "P4";
    const MAX_LATENCY_MS: u32 = 60000;
}

/// SLO band - service level objective
pub trait SloBand: 'static {
    const BAND_NAME: &'static str;
    const MAX_TICKS: u8;
    const ALLOWS_BLOCKING: bool;
}

/// Interactive SLO - tight latency bounds
pub struct Interactive;
impl SloBand for Interactive {
    const BAND_NAME: &'static str = "interactive";
    const MAX_TICKS: u8 = 8; // Chatman constant
    const ALLOWS_BLOCKING: bool = false;
}

/// Batch SLO - throughput-oriented
pub struct Batch;
impl SloBand for Batch {
    const BAND_NAME: &'static str = "batch";
    const MAX_TICKS: u8 = 100;
    const ALLOWS_BLOCKING: bool = true;
}

/// Background SLO - best-effort
pub struct Background;
impl SloBand for Background {
    const BAND_NAME: &'static str = "background";
    const MAX_TICKS: u8 = 255;
    const ALLOWS_BLOCKING: bool = true;
}

/// Scheduled action - priority and SLO as types
pub struct ScheduledAction<P: PriorityClass, S: SloBand, const COST: u32> {
    _priority: PhantomData<P>,
    _slo: PhantomData<S>,
}

impl<P: PriorityClass, S: SloBand, const COST: u32> ScheduledAction<P, S, COST> {
    /// Create action - only compiles if SLO constraints satisfied
    pub const fn new() -> Self {
        // Ensure action respects SLO band
        const_assert!(COST <= S::MAX_TICKS as u32);
        Self {
            _priority: PhantomData,
            _slo: PhantomData,
        }
    }

    /// Execute with resource token
    pub fn execute<const TOKEN_AMOUNT: u32>(
        &self,
        token: ResourceToken<TOKEN_AMOUNT>,
    ) -> Result<ConsumedToken<COST>, &'static str> {
        if TOKEN_AMOUNT < COST {
            return Err("Insufficient resources");
        }
        // Consume resources
        let _consumed = token.consume();
        Ok(ConsumedToken {
            _phantom: PhantomData,
        })
    }
}

/// Hot-path scheduler - only accepts P0/P1 + Interactive SLO
pub struct HotPathScheduler<const CAPACITY: usize> {
    queue: [Option<u8>; CAPACITY],
    count: usize,
}

impl<const CAPACITY: usize> HotPathScheduler<CAPACITY> {
    pub const fn new() -> Self {
        Self {
            queue: [None; CAPACITY],
            count: 0,
        }
    }

    /// Enqueue action - only accepts high priority + interactive SLO
    pub fn enqueue<const COST: u32>(
        &mut self,
        action: ScheduledAction<P0, Interactive, COST>,
    ) -> Result<(), &'static str> {
        if self.count >= CAPACITY {
            return Err("Queue full");
        }
        // Type system ensures only P0 + Interactive actions can be enqueued
        self.queue[self.count] = Some(COST as u8);
        self.count += 1;
        Ok(())
    }

    /// Also accept P1 + Interactive
    pub fn enqueue_p1<const COST: u32>(
        &mut self,
        action: ScheduledAction<P1, Interactive, COST>,
    ) -> Result<(), &'static str> {
        if self.count >= CAPACITY {
            return Err("Queue full");
        }
        self.queue[self.count] = Some(COST as u8);
        self.count += 1;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

/// Background scheduler - only accepts P3/P4 + Background/Batch SLO
pub struct BackgroundScheduler<const CAPACITY: usize> {
    queue: [Option<u8>; CAPACITY],
    count: usize,
}

impl<const CAPACITY: usize> BackgroundScheduler<CAPACITY> {
    pub const fn new() -> Self {
        Self {
            queue: [None; CAPACITY],
            count: 0,
        }
    }

    /// Enqueue background action
    pub fn enqueue<const COST: u32>(
        &mut self,
        action: ScheduledAction<P4, Background, COST>,
    ) -> Result<(), &'static str> {
        if self.count >= CAPACITY {
            return Err("Queue full");
        }
        self.queue[self.count] = Some(COST as u8);
        self.count += 1;
        Ok(())
    }

    /// Also accept batch P3
    pub fn enqueue_batch<const COST: u32>(
        &mut self,
        action: ScheduledAction<P3, Batch, COST>,
    ) -> Result<(), &'static str> {
        if self.count >= CAPACITY {
            return Err("Queue full");
        }
        self.queue[self.count] = Some(COST as u8);
        self.count += 1;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.count
    }
}

/// Resource pool - manages token allocation
pub struct ResourcePool<const TOTAL: u32> {
    quota: ResourceQuota<TOTAL>,
}

impl<const TOTAL: u32> ResourcePool<TOTAL> {
    pub const fn new() -> Self {
        Self {
            quota: ResourceQuota::new(),
        }
    }

    /// Allocate for P0 action - highest priority
    pub fn allocate_p0<const AMOUNT: u32>(&mut self) -> Option<ResourceToken<AMOUNT>> {
        self.quota.allocate::<AMOUNT>()
    }

    /// Allocate for lower priority - may fail if P0 needs resources
    pub fn allocate_p2<const AMOUNT: u32>(&mut self) -> Option<ResourceToken<AMOUNT>> {
        // Could implement priority-aware allocation
        if self.quota.remaining() > TOTAL / 2 {
            self.quota.allocate::<AMOUNT>()
        } else {
            None // Reserve for higher priority
        }
    }

    /// Reclaim resources
    pub fn reclaim<const AMOUNT: u32>(&mut self, token: ConsumedToken<AMOUNT>) {
        self.quota.reclaim(token);
    }

    pub fn remaining(&self) -> u32 {
        self.quota.remaining()
    }

    /// Allocate for P4 action - lowest priority
    #[cfg(test)]
    pub fn allocate_p4<const AMOUNT: u32>(&mut self) -> Option<ResourceToken<AMOUNT>> {
        self.quota.allocate::<AMOUNT>()
    }

    /// Runtime allocation (for testing)
    #[cfg(test)]
    pub fn allocate_generic<const AMOUNT: u32>(
        &mut self,
        runtime_amount: u32,
    ) -> Option<ResourceToken<AMOUNT>> {
        if self.quota.remaining() >= runtime_amount {
            self.quota.allocate::<AMOUNT>()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_token() {
        let token = ResourceToken::<100>::new();
        assert_eq!(ResourceToken::<100>::amount(), 100);

        let consumed = token.consume();
        assert_eq!(ConsumedToken::<100>::amount(), 100);
    }

    #[test]
    fn test_resource_quota() {
        let mut quota = ResourceQuota::<1000>::new();
        assert_eq!(quota.remaining(), 1000);

        let token = quota.allocate::<300>().unwrap();
        assert_eq!(quota.remaining(), 700);

        let consumed = token.consume();
        quota.reclaim(consumed);
        assert_eq!(quota.remaining(), 1000);
    }

    #[test]
    fn test_token_split() {
        let token = ResourceToken::<100>::new();
        let (token1, token2) = token.split::<60, 40>();
        assert_eq!(ResourceToken::<60>::amount(), 60);
        assert_eq!(ResourceToken::<40>::amount(), 40);
    }

    #[test]
    fn test_priority_classes() {
        assert_eq!(P0::LEVEL, 0);
        assert_eq!(P0::MAX_LATENCY_MS, 10);

        assert_eq!(P4::LEVEL, 4);
        assert_eq!(P4::MAX_LATENCY_MS, 60000);
    }

    #[test]
    fn test_slo_bands() {
        assert_eq!(Interactive::MAX_TICKS, 8);
        assert!(!Interactive::ALLOWS_BLOCKING);

        assert_eq!(Background::MAX_TICKS, 255);
        assert!(Background::ALLOWS_BLOCKING);
    }

    #[test]
    fn test_scheduled_action() {
        let action = ScheduledAction::<P0, Interactive, 5>::new();
        let token = ResourceToken::<10>::new();
        let consumed = action.execute(token).unwrap();
        assert_eq!(ConsumedToken::<5>::amount(), 5);
    }

    #[test]
    fn test_hot_path_scheduler() {
        let mut scheduler = HotPathScheduler::<10>::new();

        let action = ScheduledAction::<P0, Interactive, 5>::new();
        assert!(scheduler.enqueue(action).is_ok());
        assert_eq!(scheduler.len(), 1);

        // P1 also allowed
        let action = ScheduledAction::<P1, Interactive, 7>::new();
        assert!(scheduler.enqueue_p1(action).is_ok());
        assert_eq!(scheduler.len(), 2);
    }

    #[test]
    fn test_background_scheduler() {
        let mut scheduler = BackgroundScheduler::<10>::new();

        let action = ScheduledAction::<P4, Background, 50>::new();
        assert!(scheduler.enqueue(action).is_ok());
        assert_eq!(scheduler.len(), 1);
    }

    #[test]
    fn test_resource_pool() {
        let mut pool = ResourcePool::<1000>::new();

        // P0 can always allocate
        let token = pool.allocate_p0::<300>().unwrap();
        assert_eq!(pool.remaining(), 700);

        // P2 can allocate if enough remains
        let token2 = pool.allocate_p2::<200>().unwrap();
        assert_eq!(pool.remaining(), 500);

        // Reclaim
        let consumed = token.consume();
        pool.reclaim(consumed);
        assert_eq!(pool.remaining(), 800);
    }

    #[test]
    fn test_quota_exhaustion() {
        let mut quota = ResourceQuota::<100>::new();

        let token1 = quota.allocate::<60>().unwrap();
        let token2 = quota.allocate::<50>(); // Should fail
        assert!(token2.is_none());
    }
}

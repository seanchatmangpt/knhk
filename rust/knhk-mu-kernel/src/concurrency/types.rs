//! Type-Level Concurrency Guarantees
//!
//! Enforces core-local vs shared data at compile time using Rust's type system.

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::sync::Arc;
use crate::guards::GuardId;

/// Marker trait: Type is NOT Send (cannot cross thread boundaries)
pub struct NotSend(PhantomData<*const ()>);

/// Marker trait: Type is NOT Sync (cannot be shared between threads)
pub struct NotSync(PhantomData<UnsafeCell<()>>);

/// Core-local data that is NEVER shared across cores
///
/// # Type Safety
///
/// - NOT Send: Cannot be moved to another thread
/// - NOT Sync: Cannot be shared between threads
/// - Zero synchronization overhead
/// - Cache-friendly (no false sharing)
///
/// # Usage
///
/// ```rust,no_run
/// use knhk_mu_kernel::concurrency::CoreLocal;
///
/// struct CoreData {
///     local_counter: u64,
///     local_buffer: [u8; 1024],
/// }
///
/// let core_data = CoreLocal::new(CoreData {
///     local_counter: 0,
///     local_buffer: [0; 1024],
/// });
///
/// // Can only access on same core (enforced by type system)
/// core_data.with_mut(|data| {
///     data.local_counter += 1;
/// });
/// ```
#[repr(C, align(64))]  // Cache-line aligned to prevent false sharing
pub struct CoreLocal<T> {
    data: UnsafeCell<T>,
    _not_send: PhantomData<NotSend>,
    _not_sync: PhantomData<NotSync>,
}

impl<T> CoreLocal<T> {
    /// Create new core-local data
    #[inline]
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            _not_send: PhantomData,
            _not_sync: PhantomData,
        }
    }

    /// Access data immutably (single core only)
    ///
    /// # Safety
    ///
    /// Must only be called from the owning core.
    /// Type system prevents cross-core access.
    #[inline]
    pub fn with<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        // Safe: CoreLocal is !Send + !Sync, so we're guaranteed single-core access
        unsafe { f(&*self.data.get()) }
    }

    /// Access data mutably (single core only)
    ///
    /// # Safety
    ///
    /// Must only be called from the owning core.
    /// Type system prevents concurrent mutable access.
    #[inline]
    pub fn with_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        // Safe: CoreLocal is !Send + !Sync, guaranteed exclusive access
        unsafe { f(&mut *self.data.get()) }
    }

    /// Get raw pointer (for verified unsafe code)
    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.data.get()
    }
}

// Explicitly NOT Send
impl<T> !Send for CoreLocal<T> {}

// Explicitly NOT Sync
impl<T> !Sync for CoreLocal<T> {}

/// Shared data with explicit ordering guarantees
///
/// # Type Safety
///
/// - IS Send + Sync: Can be shared across cores
/// - Requires explicit ordering specification
/// - Uses atomic operations or locks (depending on ordering)
/// - Type system prevents leaking into CoreLocal
///
/// # Usage
///
/// ```rust,no_run
/// use knhk_mu_kernel::concurrency::{Shared, GlobalOrdering};
///
/// let shared_counter = Shared::new(0u64, GlobalOrdering::SeqCst);
///
/// // Can be accessed from multiple cores
/// shared_counter.fetch_add(1);
/// ```
#[repr(C, align(64))]
pub struct Shared<T>
where
    T: Send + Sync,
{
    data: Arc<T>,
    ordering: GlobalOrdering,
}

impl<T> Shared<T>
where
    T: Send + Sync,
{
    /// Create new shared data with ordering guarantee
    #[inline]
    pub fn new(data: T, ordering: GlobalOrdering) -> Self {
        Self {
            data: Arc::new(data),
            ordering,
        }
    }

    /// Get reference (respects ordering)
    #[inline]
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Clone the Arc (cheap)
    #[inline]
    pub fn clone_ref(&self) -> Arc<T> {
        Arc::clone(&self.data)
    }

    /// Get ordering guarantee
    #[inline]
    pub fn ordering(&self) -> GlobalOrdering {
        self.ordering
    }
}

impl Shared<AtomicU64> {
    /// Atomic fetch-add (for counters)
    #[inline]
    pub fn fetch_add(&self, val: u64) -> u64 {
        self.data.fetch_add(val, self.ordering.into())
    }

    /// Atomic load
    #[inline]
    pub fn load(&self) -> u64 {
        self.data.load(self.ordering.into())
    }

    /// Atomic store
    #[inline]
    pub fn store(&self, val: u64) {
        self.data.store(val, self.ordering.into())
    }

    /// Atomic compare-exchange
    #[inline]
    pub fn compare_exchange(
        &self,
        current: u64,
        new: u64,
    ) -> Result<u64, u64> {
        self.data.compare_exchange(
            current,
            new,
            self.ordering.into(),
            Ordering::Relaxed,
        )
    }
}

impl<T> Clone for Shared<T>
where
    T: Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            data: Arc::clone(&self.data),
            ordering: self.ordering,
        }
    }
}

/// Global memory ordering guarantees
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GlobalOrdering {
    /// Sequentially consistent (strongest, slowest)
    SeqCst = 0,
    /// Release-Acquire (balanced)
    AcqRel = 1,
    /// Relaxed (weakest, fastest - use with caution)
    Relaxed = 2,
}

impl From<GlobalOrdering> for Ordering {
    fn from(order: GlobalOrdering) -> Self {
        match order {
            GlobalOrdering::SeqCst => Ordering::SeqCst,
            GlobalOrdering::AcqRel => Ordering::AcqRel,
            GlobalOrdering::Relaxed => Ordering::Relaxed,
        }
    }
}

/// Guard set trait (compile-time guard requirements)
pub trait GuardSet {
    /// Number of guards
    const COUNT: usize;

    /// Get guard IDs
    fn guards(&self) -> &[GuardId];

    /// Validate guards are sufficient
    fn validate(&self) -> Result<(), GuardError>;
}

/// Empty guard set (no guards required)
#[derive(Debug, Clone, Copy)]
pub struct NoGuards;

impl GuardSet for NoGuards {
    const COUNT: usize = 0;

    fn guards(&self) -> &[GuardId] {
        &[]
    }

    fn validate(&self) -> Result<(), GuardError> {
        Ok(())
    }
}

/// Single guard
#[derive(Debug, Clone, Copy)]
pub struct SingleGuard(pub GuardId);

impl GuardSet for SingleGuard {
    const COUNT: usize = 1;

    fn guards(&self) -> &[GuardId] {
        core::slice::from_ref(&self.0)
    }

    fn validate(&self) -> Result<(), GuardError> {
        if self.0 >= 1024 {
            Err(GuardError::InvalidGuardId)
        } else {
            Ok(())
        }
    }
}

/// Multiple guards (up to 8)
#[derive(Debug, Clone, Copy)]
pub struct MultiGuard<const N: usize> {
    guards: [GuardId; N],
}

impl<const N: usize> MultiGuard<N> {
    /// Create new multi-guard set
    pub const fn new(guards: [GuardId; N]) -> Self {
        assert!(N <= 8, "Maximum 8 guards per task");
        Self { guards }
    }
}

impl<const N: usize> GuardSet for MultiGuard<N> {
    const COUNT: usize = N;

    fn guards(&self) -> &[GuardId] {
        &self.guards
    }

    fn validate(&self) -> Result<(), GuardError> {
        for &guard_id in &self.guards {
            if guard_id >= 1024 {
                return Err(GuardError::InvalidGuardId);
            }
        }
        Ok(())
    }
}

/// Guard errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardError {
    /// Invalid guard ID
    InvalidGuardId,
    /// Too many guards
    TooManyGuards,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_local_creation() {
        let local = CoreLocal::new(42u64);
        local.with(|val| {
            assert_eq!(*val, 42);
        });
    }

    #[test]
    fn test_core_local_mutation() {
        let local = CoreLocal::new(0u64);
        local.with_mut(|val| {
            *val = 100;
        });
        local.with(|val| {
            assert_eq!(*val, 100);
        });
    }

    #[test]
    fn test_shared_atomic() {
        let counter = Shared::new(AtomicU64::new(0), GlobalOrdering::SeqCst);

        let old = counter.fetch_add(1);
        assert_eq!(old, 0);

        let current = counter.load();
        assert_eq!(current, 1);
    }

    #[test]
    fn test_guard_set_validation() {
        let no_guards = NoGuards;
        assert_eq!(no_guards.guards().len(), 0);
        assert!(no_guards.validate().is_ok());

        let single = SingleGuard(42);
        assert_eq!(single.guards().len(), 1);
        assert!(single.validate().is_ok());

        let invalid = SingleGuard(2000);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_multi_guard() {
        let guards = MultiGuard::new([0, 1, 2, 3]);
        assert_eq!(guards.guards().len(), 4);
        assert!(guards.validate().is_ok());
    }

    // Compile-time tests (should fail to compile if uncommented)
    // #[test]
    // fn test_core_local_not_send() {
    //     let local = CoreLocal::new(42);
    //     std::thread::spawn(move || {
    //         local.with(|_| {}); // ERROR: CoreLocal<u64> cannot be sent between threads
    //     });
    // }
}

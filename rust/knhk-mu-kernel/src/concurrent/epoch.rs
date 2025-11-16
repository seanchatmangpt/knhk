//! Epoch-based memory reclamation
//!
//! This module provides:
//! - Epoch-based reclamation (faster than hazard pointers)
//! - Global epoch tracking with local participation
//! - Deferred destruction with bounded memory growth
//! - Integration with concurrent data structures
//! - Grace period detection for safe reclamation

use crate::concurrent::lib_compat::*;

/// Number of epochs (3 is standard: current, previous, and grace period)
const EPOCH_COUNT: usize = 3;

/// Maximum number of deferred operations per thread before forcing collection
const MAX_DEFERRED: usize = 256;

/// Global epoch counter
static GLOBAL_EPOCH: AtomicUsize = AtomicUsize::new(0);

/// List of all thread-local participants
static PARTICIPANTS: AtomicPtr<Participant> = AtomicPtr::new(ptr::null_mut());

/// Thread-local participant in epoch-based reclamation
struct Participant {
    /// Current epoch this thread is in
    epoch: AtomicUsize,
    /// Whether this thread is active (participating)
    active: AtomicBool,
    /// Next participant in linked list
    next: AtomicPtr<Participant>,
    /// Deferred operations for each epoch
    deferred: RefCell<[VecDeque<Box<dyn FnOnce() + Send>>; EPOCH_COUNT]>,
    /// Pin count (nested pins)
    pin_count: Cell<usize>,
}

impl Participant {
    fn new() -> Self {
        Participant {
            epoch: AtomicUsize::new(0),
            active: AtomicBool::new(false),
            next: AtomicPtr::new(ptr::null_mut()),
            deferred: RefCell::new([VecDeque::new(), VecDeque::new(), VecDeque::new()]),
            pin_count: Cell::new(0),
        }
    }

    /// Enter the current epoch
    fn enter(&self) -> usize {
        let count = self.pin_count.get();
        self.pin_count.set(count + 1);

        if count == 0 {
            let global = GLOBAL_EPOCH.load(Ordering::Relaxed);
            self.epoch.store(global, Ordering::Relaxed);
            self.active.store(true, Ordering::Release);
        }

        count
    }

    /// Leave the current epoch
    fn leave(&self) {
        let count = self.pin_count.get();
        assert!(count > 0, "Unbalanced epoch enter/leave");

        let new_count = count - 1;
        self.pin_count.set(new_count);

        if new_count == 0 {
            self.active.store(false, Ordering::Release);
            self.try_collect();
        }
    }

    /// Defer an operation until safe to execute
    fn defer<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let epoch = self.epoch.load(Ordering::Relaxed);
        let index = epoch % EPOCH_COUNT;

        self.deferred.borrow_mut()[index].push_back(Box::new(f));

        // Check if we should force collection
        let total: usize = self.deferred.borrow().iter().map(|d| d.len()).sum();
        if total >= MAX_DEFERRED {
            self.try_advance_epoch();
        }
    }

    /// Try to advance the global epoch
    fn try_advance_epoch(&self) {
        let global = GLOBAL_EPOCH.load(Ordering::Relaxed);

        // Check if all participants are in current or future epoch
        if self.can_advance(global) {
            GLOBAL_EPOCH.compare_exchange(
                global,
                global + 1,
                Ordering::Release,
                Ordering::Relaxed,
            ).ok();
        }
    }

    /// Check if we can advance from the given epoch
    fn can_advance(&self, epoch: usize) -> bool {
        let mut curr = PARTICIPANTS.load(Ordering::Acquire);

        unsafe {
            while !curr.is_null() {
                let participant = &*curr;

                if participant.active.load(Ordering::Acquire) {
                    let p_epoch = participant.epoch.load(Ordering::Relaxed);
                    if p_epoch < epoch {
                        return false;
                    }
                }

                curr = participant.next.load(Ordering::Acquire);
            }
        }

        true
    }

    /// Try to collect deferred operations
    fn try_collect(&self) {
        let global = GLOBAL_EPOCH.load(Ordering::Relaxed);

        // We can safely execute operations from 2 epochs ago
        if global >= 2 {
            let safe_epoch = global - 2;
            let index = safe_epoch % EPOCH_COUNT;

            let mut deferred = self.deferred.borrow_mut();
            let queue = &mut deferred[index];

            while let Some(op) = queue.pop_front() {
                op();
            }
        }
    }

    /// Force collection of all safe deferred operations
    fn flush(&self) {
        // Advance epoch multiple times to ensure grace period
        for _ in 0..EPOCH_COUNT {
            self.try_advance_epoch();
        }

        // Collect all safe operations
        let global = GLOBAL_EPOCH.load(Ordering::Relaxed);
        for epoch in 0..global.saturating_sub(1) {
            let index = epoch % EPOCH_COUNT;
            let mut deferred = self.deferred.borrow_mut();
            let queue = &mut deferred[index];

            while let Some(op) = queue.pop_front() {
                op();
            }
        }
    }
}

impl Drop for Participant {
    fn drop(&mut self) {
        self.flush();
    }
}

/// Register this thread as a participant
fn register_participant() -> &'static Participant {
    thread_local! {
        static PARTICIPANT: Participant = Participant::new();
    }

    PARTICIPANT.with(|p| {
        // Add to global participant list if not already added
        let p_ptr = p as *const Participant as *mut Participant;

        loop {
            let head = PARTICIPANTS.load(Ordering::Acquire);
            p.next.store(head, Ordering::Relaxed);

            match PARTICIPANTS.compare_exchange_weak(
                head,
                p_ptr,
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }

        unsafe { &*p_ptr }
    })
}

/// Get the current thread's participant
fn current_participant() -> &'static Participant {
    thread_local! {
        static PARTICIPANT: &'static Participant = register_participant();
    }

    PARTICIPANT.with(|p| *p)
}

/// Guard for epoch protection
///
/// While a guard exists, the thread is participating in the current epoch
/// and memory reclamation is deferred for objects in this epoch.
pub struct Guard {
    participant: &'static Participant,
}

impl Guard {
    /// Pin the current thread to the current epoch
    pub fn pin() -> Self {
        let participant = current_participant();
        participant.enter();
        Guard { participant }
    }

    /// Defer destruction of a pointer until safe
    ///
    /// # Safety
    /// The pointer must be valid and not used after this call
    pub unsafe fn defer_destroy<T>(&self, ptr: *mut T) {
        if ptr.is_null() {
            return;
        }

        self.participant.defer(move || {
            drop(Box::from_raw(ptr));
        });
    }

    /// Defer an arbitrary operation until safe
    pub fn defer<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.participant.defer(f);
    }

    /// Repin to the current epoch (for long-running operations)
    pub fn repin(&mut self) {
        self.participant.leave();
        self.participant.enter();
    }

    /// Flush all safe deferred operations
    pub fn flush(&self) {
        self.participant.flush();
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        self.participant.leave();
    }
}

impl Clone for Guard {
    fn clone(&self) -> Self {
        self.participant.enter();
        Guard {
            participant: self.participant,
        }
    }
}

/// Atomic pointer with epoch-based reclamation
pub struct Atomic<T> {
    data: AtomicPtr<T>,
}

impl<T> Atomic<T> {
    /// Create a new atomic pointer
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Atomic {
            data: AtomicPtr::new(ptr),
        }
    }

    /// Create a null atomic pointer
    pub fn null() -> Self {
        Atomic {
            data: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Load the pointer
    pub fn load<'g>(&self, _guard: &'g Guard) -> Option<&'g T> {
        let ptr = self.data.load(Ordering::Acquire);
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &*ptr })
        }
    }

    /// Store a new pointer, deferring destruction of the old one
    pub fn store(&self, value: T, guard: &Guard) {
        let new_ptr = Box::into_raw(Box::new(value));
        let old_ptr = self.data.swap(new_ptr, Ordering::Release);

        if !old_ptr.is_null() {
            unsafe {
                guard.defer_destroy(old_ptr);
            }
        }
    }

    /// Compare and swap
    pub fn compare_exchange(
        &self,
        current: *mut T,
        new: T,
        guard: &Guard,
    ) -> Result<*mut T, *mut T> {
        let new_ptr = Box::into_raw(Box::new(new));

        match self.data.compare_exchange(
            current,
            new_ptr,
            Ordering::Release,
            Ordering::Acquire,
        ) {
            Ok(old) => {
                if !old.is_null() {
                    unsafe {
                        guard.defer_destroy(old);
                    }
                }
                Ok(old)
            }
            Err(actual) => {
                unsafe {
                    drop(Box::from_raw(new_ptr));
                }
                Err(actual)
            }
        }
    }

    /// Swap with a new value
    pub fn swap(&self, new: T, guard: &Guard) -> Option<T> {
        let new_ptr = Box::into_raw(Box::new(new));
        let old_ptr = self.data.swap(new_ptr, Ordering::AcqRel);

        if old_ptr.is_null() {
            None
        } else {
            // Return immediately since we own it
            Some(unsafe { *Box::from_raw(old_ptr) })
        }
    }
}

impl<T> Drop for Atomic<T> {
    fn drop(&mut self) {
        let ptr = self.data.load(Ordering::Acquire);
        if !ptr.is_null() {
            unsafe {
                drop(Box::from_raw(ptr));
            }
        }
    }
}

unsafe impl<T: Send> Send for Atomic<T> {}
unsafe impl<T: Send> Sync for Atomic<T> {}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_guard_basic() {
        let guard = Guard::pin();
        guard.defer(|| {
            // This will be executed later
        });
        drop(guard);
    }

    #[test]
    fn test_atomic_basic() {
        let atomic = Atomic::new(42);
        let guard = Guard::pin();

        assert_eq!(atomic.load(&guard), Some(&42));

        atomic.store(100, &guard);
        assert_eq!(atomic.load(&guard), Some(&100));
    }

    #[test]
    fn test_concurrent_updates() {
        let atomic = Arc::new(Atomic::new(0));
        let mut handles = vec![];

        for i in 0..4 {
            let atomic = Arc::clone(&atomic);
            handles.push(thread::spawn(move || {
                let guard = Guard::pin();
                for j in 0..100 {
                    atomic.store(i * 100 + j, &guard);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let guard = Guard::pin();
        assert!(atomic.load(&guard).is_some());
    }

    #[test]
    fn test_deferred_destruction() {
        static DROPS: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;
        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROPS.fetch_add(1, Ordering::Relaxed);
            }
        }

        let atomic = Atomic::new(DropCounter);

        {
            let guard = Guard::pin();
            atomic.store(DropCounter, &guard);
            atomic.store(DropCounter, &guard);
            guard.flush();
        }

        // Give time for deferred operations
        thread::sleep(std::time::Duration::from_millis(10));

        // At least some drops should have occurred
        assert!(DROPS.load(Ordering::Relaxed) > 0);
    }

    #[test]
    fn test_nested_pins() {
        let guard1 = Guard::pin();
        let guard2 = Guard::pin();

        guard1.defer(|| {});
        guard2.defer(|| {});

        drop(guard2);
        drop(guard1);
    }
}

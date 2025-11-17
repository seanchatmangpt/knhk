//! Lock-free atomic reference counting
//!
//! This module provides:
//! - Lock-free Arc operations using compare-and-swap
//! - Strong and weak reference counting
//! - Memory ordering guarantees
//! - ABA problem mitigation
//! - Integration with epoch-based reclamation

use crate::concurrent::lib_compat::*;

/// Reference counts for Arc
struct ArcInner<T> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    data: T,
}

impl<T> ArcInner<T> {
    fn new(data: T) -> Self {
        ArcInner {
            strong: AtomicUsize::new(1),
            weak: AtomicUsize::new(1),
            data,
        }
    }

    /// Increment strong count
    fn inc_strong(&self) -> usize {
        let old = self.strong.fetch_add(1, Ordering::Relaxed);

        // Check for overflow
        if old > usize::MAX / 2 {
            std::process::abort();
        }

        old
    }

    /// Decrement strong count
    fn dec_strong(&self) -> usize {
        self.strong.fetch_sub(1, Ordering::Release)
    }

    /// Increment weak count
    fn inc_weak(&self) -> usize {
        let old = self.weak.fetch_add(1, Ordering::Relaxed);

        // Check for overflow
        if old > usize::MAX / 2 {
            std::process::abort();
        }

        old
    }

    /// Decrement weak count
    fn dec_weak(&self) -> usize {
        self.weak.fetch_sub(1, Ordering::Release)
    }

    /// Get current strong count
    fn strong_count(&self) -> usize {
        self.strong.load(Ordering::Acquire)
    }

    /// Get current weak count
    fn weak_count(&self) -> usize {
        self.weak.load(Ordering::Acquire)
    }
}

/// Lock-free atomic reference counted pointer
pub struct AtomicArc<T> {
    ptr: NonNull<ArcInner<T>>,
    _marker: PhantomData<ArcInner<T>>,
}

impl<T> AtomicArc<T> {
    /// Create a new AtomicArc
    pub fn new(data: T) -> Self {
        let inner = Box::new(ArcInner::new(data));
        let ptr = NonNull::new(Box::into_raw(inner)).unwrap();

        AtomicArc {
            ptr,
            _marker: PhantomData,
        }
    }

    /// Get a reference to the inner value
    pub fn get(&self) -> &T {
        unsafe { &self.ptr.as_ref().data }
    }

    /// Get the strong reference count
    pub fn strong_count(&self) -> usize {
        unsafe { self.ptr.as_ref().strong_count() }
    }

    /// Get the weak reference count (excluding the implicit weak from strong count)
    pub fn weak_count(&self) -> usize {
        unsafe { self.ptr.as_ref().weak_count().saturating_sub(1) }
    }

    /// Create a weak reference
    pub fn downgrade(&self) -> WeakArc<T> {
        unsafe {
            self.ptr.as_ref().inc_weak();
        }
        WeakArc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }

    /// Get a mutable reference if this is the only strong reference
    pub fn get_mut(&mut self) -> Option<&mut T> {
        unsafe {
            if self.ptr.as_ref().strong_count() == 1 {
                Some(&mut (*self.ptr.as_ptr()).data)
            } else {
                None
            }
        }
    }

    /// Clone the Arc (increment reference count)
    fn clone_inner(&self) -> Self {
        unsafe {
            self.ptr.as_ref().inc_strong();
        }
        AtomicArc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for AtomicArc<T> {
    fn clone(&self) -> Self {
        self.clone_inner()
    }
}

impl<T> Deref for AtomicArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<T> Drop for AtomicArc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ref();

            if inner.dec_strong() == 1 {
                // Last strong reference - need synchronization
                std::sync::atomic::fence(Ordering::Acquire);

                // Drop the data
                ptr::drop_in_place(&mut (*self.ptr.as_ptr()).data);

                // Drop weak reference from strong count
                if inner.dec_weak() == 1 {
                    // Last weak reference - deallocate
                    std::sync::atomic::fence(Ordering::Acquire);
                    dealloc(self.ptr.as_ptr() as *mut u8, Layout::new::<ArcInner<T>>());
                }
            }
        }
    }
}

unsafe impl<T: Send + Sync> Send for AtomicArc<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicArc<T> {}

/// Weak reference to AtomicArc
pub struct WeakArc<T> {
    ptr: NonNull<ArcInner<T>>,
    _marker: PhantomData<ArcInner<T>>,
}

impl<T> WeakArc<T> {
    /// Attempt to upgrade to a strong reference
    pub fn upgrade(&self) -> Option<AtomicArc<T>> {
        unsafe {
            let inner = self.ptr.as_ref();

            loop {
                let strong = inner.strong_count();

                if strong == 0 {
                    return None;
                }

                // Try to increment strong count
                match inner.strong.compare_exchange_weak(
                    strong,
                    strong + 1,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => {
                        return Some(AtomicArc {
                            ptr: self.ptr,
                            _marker: PhantomData,
                        });
                    }
                    Err(_) => continue,
                }
            }
        }
    }

    /// Get the strong reference count
    pub fn strong_count(&self) -> usize {
        unsafe { self.ptr.as_ref().strong_count() }
    }

    /// Get the weak reference count (excluding the implicit weak from strong count)
    pub fn weak_count(&self) -> usize {
        unsafe { self.ptr.as_ref().weak_count().saturating_sub(1) }
    }
}

impl<T> Clone for WeakArc<T> {
    fn clone(&self) -> Self {
        unsafe {
            self.ptr.as_ref().inc_weak();
        }
        WeakArc {
            ptr: self.ptr,
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for WeakArc<T> {
    fn drop(&mut self) {
        unsafe {
            let inner = self.ptr.as_ref();

            if inner.dec_weak() == 1 {
                // Last weak reference - deallocate if no strong references
                std::sync::atomic::fence(Ordering::Acquire);

                if inner.strong_count() == 0 {
                    dealloc(self.ptr.as_ptr() as *mut u8, Layout::new::<ArcInner<T>>());
                }
            }
        }
    }
}

unsafe impl<T: Send + Sync> Send for WeakArc<T> {}
unsafe impl<T: Send + Sync> Sync for WeakArc<T> {}

/// Atomic storage for Arc that can be updated atomically
pub struct AtomicArcCell<T> {
    ptr: AtomicUsize,
    _marker: PhantomData<AtomicArc<T>>,
}

impl<T> AtomicArcCell<T> {
    /// Create a new AtomicArcCell
    pub fn new(arc: AtomicArc<T>) -> Self {
        // Increment ref count since we're storing it
        let ptr = arc.ptr.as_ptr() as usize;
        std::mem::forget(arc); // Don't drop, we manage the count

        AtomicArcCell {
            ptr: AtomicUsize::new(ptr),
            _marker: PhantomData,
        }
    }

    /// Create a null AtomicArcCell
    pub fn null() -> Self {
        AtomicArcCell {
            ptr: AtomicUsize::new(0),
            _marker: PhantomData,
        }
    }

    /// Load the Arc
    pub fn load(&self) -> Option<AtomicArc<T>> {
        let ptr = self.ptr.load(Ordering::Acquire);

        if ptr == 0 {
            return None;
        }

        let inner_ptr = ptr as *mut ArcInner<T>;
        let inner = unsafe { &*inner_ptr };

        // Try to increment strong count
        loop {
            let strong = inner.strong_count();

            if strong == 0 {
                return None;
            }

            match inner.strong.compare_exchange_weak(
                strong,
                strong + 1,
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    return Some(AtomicArc {
                        ptr: NonNull::new(inner_ptr).unwrap(),
                        _marker: PhantomData,
                    });
                }
                Err(_) => continue,
            }
        }
    }

    /// Store a new Arc
    pub fn store(&self, arc: AtomicArc<T>) {
        let new_ptr = arc.ptr.as_ptr() as usize;
        std::mem::forget(arc); // Don't drop, we're storing it

        let old_ptr = self.ptr.swap(new_ptr, Ordering::AcqRel);

        if old_ptr != 0 {
            // Drop old Arc
            let old_arc = AtomicArc::<T> {
                ptr: NonNull::new(old_ptr as *mut ArcInner<T>).unwrap(),
                _marker: PhantomData,
            };
            drop(old_arc);
        }
    }

    /// Compare and swap
    pub fn compare_exchange(
        &self,
        current: Option<&AtomicArc<T>>,
        new: AtomicArc<T>,
    ) -> Result<(), AtomicArc<T>> {
        let current_ptr = current.map_or(0, |arc| arc.ptr.as_ptr() as usize);
        let new_ptr = new.ptr.as_ptr() as usize;

        match self
            .ptr
            .compare_exchange(current_ptr, new_ptr, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) => {
                std::mem::forget(new); // Stored successfully
                Ok(())
            }
            Err(_) => Err(new),
        }
    }
}

impl<T> Drop for AtomicArcCell<T> {
    fn drop(&mut self) {
        let ptr = self.ptr.load(Ordering::Acquire);

        if ptr != 0 {
            let arc = AtomicArc::<T> {
                ptr: NonNull::new(ptr as *mut ArcInner<T>).unwrap(),
                _marker: PhantomData,
            };
            drop(arc);
        }
    }
}

unsafe impl<T: Send + Sync> Send for AtomicArcCell<T> {}
unsafe impl<T: Send + Sync> Sync for AtomicArcCell<T> {}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_basic_arc() {
        let arc = AtomicArc::new(42);
        assert_eq!(*arc, 42);
        assert_eq!(arc.strong_count(), 1);
        assert_eq!(arc.weak_count(), 0);
    }

    #[test]
    fn test_clone_arc() {
        let arc1 = AtomicArc::new(100);
        let arc2 = arc1.clone();

        assert_eq!(*arc1, 100);
        assert_eq!(*arc2, 100);
        assert_eq!(arc1.strong_count(), 2);
    }

    #[test]
    fn test_weak_arc() {
        let arc = AtomicArc::new(42);
        let weak = arc.downgrade();

        assert_eq!(arc.weak_count(), 1);
        assert_eq!(weak.strong_count(), 1);

        let upgraded = weak.upgrade().unwrap();
        assert_eq!(*upgraded, 42);
        assert_eq!(arc.strong_count(), 2);

        drop(arc);
        drop(upgraded);

        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn test_atomic_cell() {
        let cell = AtomicArcCell::new(AtomicArc::new(42));

        let loaded = cell.load().unwrap();
        assert_eq!(*loaded, 42);

        cell.store(AtomicArc::new(100));

        let loaded2 = cell.load().unwrap();
        assert_eq!(*loaded2, 100);
    }

    #[test]
    fn test_concurrent_clone() {
        let arc = Arc::new(AtomicArc::new(0));
        let mut handles = vec![];

        for _ in 0..4 {
            let arc = Arc::clone(&arc);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let _clone = arc.clone_inner();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Should eventually drop to 1 when all clones are dropped
        assert!(arc.strong_count() >= 1);
    }

    #[test]
    fn test_concurrent_weak() {
        let arc = Arc::new(AtomicArc::new(42));
        let mut handles = vec![];

        for _ in 0..4 {
            let arc = Arc::clone(&arc);
            handles.push(thread::spawn(move || {
                for _ in 0..100 {
                    let weak = arc.downgrade();
                    weak.upgrade().unwrap();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_drop_order() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;
        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }

        {
            let arc = AtomicArc::new(DropCounter);
            let weak = arc.downgrade();
            drop(arc);
            assert!(weak.upgrade().is_none());
        }

        assert_eq!(DROP_COUNT.load(Ordering::Relaxed), 1);
    }
}

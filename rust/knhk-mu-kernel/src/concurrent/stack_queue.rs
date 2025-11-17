//! Lock-free stack and queue implementations
//!
//! This module provides:
//! - Treiber stack: Lock-free LIFO stack
//! - Michael-Scott queue: Lock-free FIFO queue
//! - ABA problem mitigation using tagged pointers
//! - Memory ordering optimizations for performance

use crate::concurrent::lib_compat::*;

/// Tagged pointer to prevent ABA problem
#[derive(Clone, Copy)]
struct TaggedPtr<T> {
    data: usize,
    _marker: PhantomData<*mut T>,
}

impl<T> TaggedPtr<T> {
    fn new(ptr: *mut T, tag: usize) -> Self {
        let addr = ptr as usize;
        let tagged = (tag << 48) | (addr & 0x0000_FFFF_FFFF_FFFF);
        TaggedPtr {
            data: tagged,
            _marker: PhantomData,
        }
    }

    fn null() -> Self {
        TaggedPtr {
            data: 0,
            _marker: PhantomData,
        }
    }

    fn ptr(self) -> *mut T {
        (self.data & 0x0000_FFFF_FFFF_FFFF) as *mut T
    }

    fn tag(self) -> usize {
        self.data >> 48
    }

    fn is_null(self) -> bool {
        self.ptr().is_null()
    }

    fn to_usize(self) -> usize {
        self.data
    }

    fn from_usize(data: usize) -> Self {
        TaggedPtr {
            data,
            _marker: PhantomData,
        }
    }
}

/// Node for stack and queue
struct Node<T> {
    data: T,
    next: AtomicUsize,
}

impl<T> Node<T> {
    fn new(data: T) -> Self {
        Node {
            data,
            next: AtomicUsize::new(0),
        }
    }

    fn get_next(&self) -> TaggedPtr<Node<T>> {
        TaggedPtr::from_usize(self.next.load(Ordering::Acquire))
    }

    fn set_next(&self, next: TaggedPtr<Node<T>>) {
        self.next.store(next.to_usize(), Ordering::Release);
    }

    fn cas_next(
        &self,
        old: TaggedPtr<Node<T>>,
        new: TaggedPtr<Node<T>>,
    ) -> Result<TaggedPtr<Node<T>>, TaggedPtr<Node<T>>> {
        self.next
            .compare_exchange(
                old.to_usize(),
                new.to_usize(),
                Ordering::Release,
                Ordering::Acquire,
            )
            .map(TaggedPtr::from_usize)
            .map_err(TaggedPtr::from_usize)
    }
}

/// Treiber lock-free stack
///
/// Properties:
/// - Lock-free push and pop operations
/// - LIFO ordering
/// - ABA problem mitigation via tagged pointers
/// - Memory reclamation via manual management (caller must ensure thread safety)
pub struct TreiberStack<T> {
    head: AtomicUsize,
    _marker: PhantomData<T>,
}

impl<T> TreiberStack<T> {
    /// Create a new empty stack
    pub fn new() -> Self {
        TreiberStack {
            head: AtomicUsize::new(0),
            _marker: PhantomData,
        }
    }

    /// Push a value onto the stack
    pub fn push(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node::new(data)));
        let mut tag = 0;

        loop {
            let head = TaggedPtr::from_usize(self.head.load(Ordering::Acquire));
            tag = head.tag().wrapping_add(1);

            unsafe {
                (*new_node).set_next(head);
            }

            let new_head = TaggedPtr::new(new_node, tag);

            match self.head.compare_exchange_weak(
                head.to_usize(),
                new_head.to_usize(),
                Ordering::Release,
                Ordering::Acquire,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
    }

    /// Pop a value from the stack
    pub fn pop(&self) -> Option<T> {
        loop {
            let head = TaggedPtr::from_usize(self.head.load(Ordering::Acquire));

            if head.is_null() {
                return None;
            }

            unsafe {
                let head_node = &*head.ptr();
                let next = head_node.get_next();
                let new_head = TaggedPtr::new(next.ptr(), head.tag().wrapping_add(1));

                match self.head.compare_exchange_weak(
                    head.to_usize(),
                    new_head.to_usize(),
                    Ordering::Release,
                    Ordering::Acquire,
                ) {
                    Ok(_) => {
                        let node = Box::from_raw(head.ptr());
                        return Some(node.data);
                    }
                    Err(_) => continue,
                }
            }
        }
    }

    /// Peek at the top value without removing it
    pub fn peek(&self) -> Option<&T> {
        let head = TaggedPtr::<Node<T>>::from_usize(self.head.load(Ordering::Acquire));

        if head.is_null() {
            None
        } else {
            unsafe { Some(&(*head.ptr()).data) }
        }
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        let head = TaggedPtr::<Node<T>>::from_usize(self.head.load(Ordering::Acquire));
        head.is_null()
    }
}

impl<T> Drop for TreiberStack<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

unsafe impl<T: Send> Send for TreiberStack<T> {}
unsafe impl<T: Send> Sync for TreiberStack<T> {}

impl<T> Default for TreiberStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Michael-Scott lock-free queue
///
/// Properties:
/// - Lock-free enqueue and dequeue operations
/// - FIFO ordering
/// - Uses dummy node to simplify implementation
/// - ABA problem mitigation via tagged pointers
pub struct MichaelScottQueue<T> {
    head: AtomicUsize,
    tail: AtomicUsize,
    _marker: PhantomData<T>,
}

impl<T> MichaelScottQueue<T> {
    /// Create a new empty queue
    pub fn new() -> Self {
        // Create dummy node
        let dummy = Box::into_raw(Box::new(Node {
            data: unsafe { std::mem::zeroed() },
            next: AtomicUsize::new(0),
        }));

        let dummy_ptr = TaggedPtr::new(dummy, 0);

        MichaelScottQueue {
            head: AtomicUsize::new(dummy_ptr.to_usize()),
            tail: AtomicUsize::new(dummy_ptr.to_usize()),
            _marker: PhantomData,
        }
    }

    /// Enqueue a value
    pub fn enqueue(&self, data: T) {
        let new_node = Box::into_raw(Box::new(Node::new(data)));

        loop {
            let tail = TaggedPtr::from_usize(self.tail.load(Ordering::Acquire));
            let tail_node = unsafe { &*tail.ptr() };
            let next = tail_node.get_next();

            // Check if tail is still consistent
            if tail.to_usize() == self.tail.load(Ordering::Acquire) {
                if next.is_null() {
                    // Tail is pointing to last node, try to link new node
                    let new_next = TaggedPtr::new(new_node, next.tag().wrapping_add(1));

                    if tail_node.cas_next(next, new_next).is_ok() {
                        // Successfully linked, try to swing tail
                        let new_tail = TaggedPtr::new(new_node, tail.tag().wrapping_add(1));
                        self.tail
                            .compare_exchange(
                                tail.to_usize(),
                                new_tail.to_usize(),
                                Ordering::Release,
                                Ordering::Relaxed,
                            )
                            .ok();
                        break;
                    }
                } else {
                    // Tail is not pointing to last node, try to swing tail
                    let new_tail = TaggedPtr::new(next.ptr(), tail.tag().wrapping_add(1));
                    self.tail
                        .compare_exchange(
                            tail.to_usize(),
                            new_tail.to_usize(),
                            Ordering::Release,
                            Ordering::Relaxed,
                        )
                        .ok();
                }
            }
        }
    }

    /// Dequeue a value
    pub fn dequeue(&self) -> Option<T> {
        loop {
            let head = TaggedPtr::from_usize(self.head.load(Ordering::Acquire));
            let tail = TaggedPtr::from_usize(self.tail.load(Ordering::Acquire));
            let head_node = unsafe { &*head.ptr() };
            let next = head_node.get_next();

            // Check if head is still consistent
            if head.to_usize() == self.head.load(Ordering::Acquire) {
                if head.ptr() == tail.ptr() {
                    // Queue is empty or tail is falling behind
                    if next.is_null() {
                        return None;
                    }

                    // Tail is falling behind, try to advance it
                    let new_tail = TaggedPtr::new(next.ptr(), tail.tag().wrapping_add(1));
                    self.tail
                        .compare_exchange(
                            tail.to_usize(),
                            new_tail.to_usize(),
                            Ordering::Release,
                            Ordering::Relaxed,
                        )
                        .ok();
                } else {
                    // Read value before CAS
                    let value = unsafe { ptr::read(&(*next.ptr()).data) };

                    // Try to swing head
                    let new_head = TaggedPtr::new(next.ptr(), head.tag().wrapping_add(1));
                    if self
                        .head
                        .compare_exchange(
                            head.to_usize(),
                            new_head.to_usize(),
                            Ordering::Release,
                            Ordering::Acquire,
                        )
                        .is_ok()
                    {
                        // Successfully dequeued, deallocate old dummy
                        unsafe {
                            drop(Box::from_raw(head.ptr()));
                        }
                        return Some(value);
                    } else {
                        // Failed, forget the value we read
                        std::mem::forget(value);
                    }
                }
            }
        }
    }

    /// Peek at the front value without removing it
    pub fn peek(&self) -> Option<&T> {
        let head = TaggedPtr::<Node<T>>::from_usize(self.head.load(Ordering::Acquire));
        let head_node = unsafe { &*head.ptr() };
        let next = head_node.get_next();

        if next.is_null() {
            None
        } else {
            unsafe { Some(&(*next.ptr()).data) }
        }
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        let head = TaggedPtr::<Node<T>>::from_usize(self.head.load(Ordering::Acquire));
        let head_node = unsafe { &*head.ptr() };
        let next = head_node.get_next();
        next.is_null()
    }
}

impl<T> Drop for MichaelScottQueue<T> {
    fn drop(&mut self) {
        while self.dequeue().is_some() {}

        // Drop dummy node
        let head = TaggedPtr::<Node<T>>::from_usize(self.head.load(Ordering::Acquire));
        unsafe {
            drop(Box::from_raw(head.ptr()));
        }
    }
}

unsafe impl<T: Send> Send for MichaelScottQueue<T> {}
unsafe impl<T: Send> Sync for MichaelScottQueue<T> {}

impl<T> Default for MichaelScottQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
extern crate std;
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_stack_basic() {
        let stack = TreiberStack::new();

        assert!(stack.is_empty());
        stack.push(1);
        stack.push(2);
        stack.push(3);

        assert!(!stack.is_empty());
        assert_eq!(stack.peek(), Some(&3));
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_queue_basic() {
        let queue = MichaelScottQueue::new();

        assert!(queue.is_empty());
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);

        assert!(!queue.is_empty());
        assert_eq!(queue.peek(), Some(&1));
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_stack_concurrent() {
        let stack = Arc::new(TreiberStack::new());
        let mut handles = vec![];

        for t in 0..4 {
            let stack = Arc::clone(&stack);
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    stack.push(t * 100 + i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut count = 0;
        while stack.pop().is_some() {
            count += 1;
        }
        assert_eq!(count, 400);
    }

    #[test]
    fn test_queue_concurrent() {
        let queue = Arc::new(MichaelScottQueue::new());
        let mut handles = vec![];

        for t in 0..4 {
            let queue = Arc::clone(&queue);
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    queue.enqueue(t * 100 + i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let mut count = 0;
        while queue.dequeue().is_some() {
            count += 1;
        }
        assert_eq!(count, 400);
    }

    #[test]
    fn test_stack_mixed() {
        let stack = Arc::new(TreiberStack::new());
        let mut handles = vec![];

        for _ in 0..4 {
            let stack = Arc::clone(&stack);
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    stack.push(i);
                }
            }));
        }

        for _ in 0..4 {
            let stack = Arc::clone(&stack);
            handles.push(thread::spawn(move || {
                for _ in 0..50 {
                    stack.pop();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_queue_mixed() {
        let queue = Arc::new(MichaelScottQueue::new());
        let mut handles = vec![];

        for _ in 0..4 {
            let queue = Arc::clone(&queue);
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    queue.enqueue(i);
                }
            }));
        }

        for _ in 0..4 {
            let queue = Arc::clone(&queue);
            handles.push(thread::spawn(move || {
                for _ in 0..50 {
                    queue.dequeue();
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}

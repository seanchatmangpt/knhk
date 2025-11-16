//! Concurrent Hash Array Mapped Trie (HAMT)
//!
//! This implementation provides:
//! - Lock-free updates using copy-on-write
//! - Persistent data structure with structural sharing
//! - Cache-friendly compact array representation
//! - O(log32 n) expected time for operations
//! - Memory-efficient sparse array representation

use crate::concurrent::lib_compat::*;

/// Number of bits per level (32-way branching)
const BITS: usize = 5;
const BRANCH_FACTOR: usize = 1 << BITS;
const MASK: usize = BRANCH_FACTOR - 1;
const MAX_DEPTH: usize = 64 / BITS;

/// Bitmap indicating which slots are occupied
type Bitmap = u32;

/// Node in the HAMT
enum Node<K, V> {
    /// Branch node with bitmap and children
    Branch {
        bitmap: Bitmap,
        children: Vec<Arc<Node<K, V>>>,
    },
    /// Leaf node with key-value pair
    Leaf { key: K, value: V },
    /// Collision node for hash collisions
    Collision { hash: u64, entries: Vec<(K, V)> },
}

impl<K: Clone, V: Clone> Clone for Node<K, V> {
    fn clone(&self) -> Self {
        match self {
            Node::Branch { bitmap, children } => Node::Branch {
                bitmap: *bitmap,
                children: children.clone(),
            },
            Node::Leaf { key, value } => Node::Leaf {
                key: key.clone(),
                value: value.clone(),
            },
            Node::Collision { hash, entries } => Node::Collision {
                hash: *hash,
                entries: entries.clone(),
            },
        }
    }
}

impl<K, V> Node<K, V> {
    fn new_branch(bitmap: Bitmap, children: Vec<Arc<Node<K, V>>>) -> Self {
        Node::Branch { bitmap, children }
    }

    fn new_leaf(key: K, value: V) -> Self {
        Node::Leaf { key, value }
    }

    fn new_collision(hash: u64, entries: Vec<(K, V)>) -> Self {
        Node::Collision { hash, entries }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Node<K, V> {
    /// Get value for key
    fn get(&self, hash: u64, key: &K, shift: usize) -> Option<&V> {
        match self {
            Node::Branch { bitmap, children } => {
                let idx = index(hash, shift);
                let bit = 1 << idx;

                if *bitmap & bit == 0 {
                    return None;
                }

                let pos = (*bitmap & (bit - 1)).count_ones() as usize;
                children[pos].get(hash, key, shift + BITS)
            }
            Node::Leaf {
                key: leaf_key,
                value,
            } => {
                if leaf_key == key {
                    Some(value)
                } else {
                    None
                }
            }
            Node::Collision {
                hash: coll_hash,
                entries,
            } => {
                if *coll_hash != hash {
                    return None;
                }
                entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
        }
    }

    /// Insert key-value pair (returns new root)
    fn insert(
        self: Arc<Self>,
        hash: u64,
        key: K,
        value: V,
        shift: usize,
    ) -> (Arc<Node<K, V>>, Option<V>) {
        match self.as_ref() {
            Node::Branch { bitmap, children } => {
                let idx = index(hash, shift);
                let bit = 1 << idx;

                if *bitmap & bit == 0 {
                    // Insert new child
                    let pos = (*bitmap & (bit - 1)).count_ones() as usize;
                    let mut new_children = children.clone();
                    new_children.insert(pos, Arc::new(Node::new_leaf(key, value)));

                    let new_node = Arc::new(Node::new_branch(*bitmap | bit, new_children));
                    (new_node, None)
                } else {
                    // Update existing child
                    let pos = (*bitmap & (bit - 1)).count_ones() as usize;
                    let (new_child, old_value) =
                        children[pos].clone().insert(hash, key, value, shift + BITS);

                    let mut new_children = children.clone();
                    new_children[pos] = new_child;

                    let new_node = Arc::new(Node::new_branch(*bitmap, new_children));
                    (new_node, old_value)
                }
            }
            Node::Leaf {
                key: leaf_key,
                value: leaf_value,
            } => {
                if leaf_key == &key {
                    // Replace value
                    (Arc::new(Node::new_leaf(key, value)), Some(leaf_value.clone()))
                } else {
                    // Create branch or collision
                    let leaf_hash = hash_key(leaf_key);
                    if leaf_hash == hash {
                        // Hash collision - create collision node
                        let entries = vec![(leaf_key.clone(), leaf_value.clone()), (key, value)];
                        (Arc::new(Node::new_collision(hash, entries)), None)
                    } else {
                        // Create branch
                        let leaf_idx = index(leaf_hash, shift);
                        let new_idx = index(hash, shift);

                        if leaf_idx == new_idx {
                            // Need to go deeper
                            let child = Arc::new(Node::new_leaf(leaf_key.clone(), leaf_value.clone()));
                            let (new_child, old_value) = child.insert(hash, key, value, shift + BITS);

                            let bitmap = 1 << leaf_idx;
                            let children = vec![new_child];
                            (Arc::new(Node::new_branch(bitmap, children)), old_value)
                        } else {
                            // Create branch with two leaves
                            let bitmap = (1 << leaf_idx) | (1 << new_idx);
                            let children = if leaf_idx < new_idx {
                                vec![
                                    Arc::new(Node::new_leaf(leaf_key.clone(), leaf_value.clone())),
                                    Arc::new(Node::new_leaf(key, value)),
                                ]
                            } else {
                                vec![
                                    Arc::new(Node::new_leaf(key, value)),
                                    Arc::new(Node::new_leaf(leaf_key.clone(), leaf_value.clone())),
                                ]
                            };
                            (Arc::new(Node::new_branch(bitmap, children)), None)
                        }
                    }
                }
            }
            Node::Collision {
                hash: coll_hash,
                entries,
            } => {
                if *coll_hash != hash {
                    // Different hash - create branch
                    let coll_idx = index(*coll_hash, shift);
                    let new_idx = index(hash, shift);

                    if coll_idx == new_idx {
                        // Need to go deeper
                        let child = self.clone();
                        let (new_child, old_value) = child.insert(hash, key, value, shift + BITS);

                        let bitmap = 1 << coll_idx;
                        let children = vec![new_child];
                        (Arc::new(Node::new_branch(bitmap, children)), old_value)
                    } else {
                        let bitmap = (1 << coll_idx) | (1 << new_idx);
                        let children = if coll_idx < new_idx {
                            vec![self.clone(), Arc::new(Node::new_leaf(key, value))]
                        } else {
                            vec![Arc::new(Node::new_leaf(key, value)), self.clone()]
                        };
                        (Arc::new(Node::new_branch(bitmap, children)), None)
                    }
                } else {
                    // Same hash - update collision list
                    let mut new_entries = entries.clone();
                    let old_value = new_entries.iter().position(|(k, _)| k == &key).map(|i| {
                        let old = new_entries[i].1.clone();
                        new_entries[i] = (key.clone(), value.clone());
                        old
                    });

                    if old_value.is_none() {
                        new_entries.push((key, value));
                    }

                    (Arc::new(Node::new_collision(hash, new_entries)), old_value)
                }
            }
        }
    }

    /// Remove key (returns new root and old value)
    fn remove(self: Arc<Self>, hash: u64, key: &K, shift: usize) -> (Option<Arc<Node<K, V>>>, Option<V>) {
        match self.as_ref() {
            Node::Branch { bitmap, children } => {
                let idx = index(hash, shift);
                let bit = 1 << idx;

                if *bitmap & bit == 0 {
                    return (Some(self), None);
                }

                let pos = (*bitmap & (bit - 1)).count_ones() as usize;
                let (new_child, old_value) = children[pos].clone().remove(hash, key, shift + BITS);

                if let Some(new_child) = new_child {
                    let mut new_children = children.clone();
                    new_children[pos] = new_child;
                    (Some(Arc::new(Node::new_branch(*bitmap, new_children))), old_value)
                } else {
                    // Remove child
                    let new_bitmap = *bitmap & !bit;
                    if new_bitmap == 0 {
                        (None, old_value)
                    } else {
                        let mut new_children = children.clone();
                        new_children.remove(pos);
                        (Some(Arc::new(Node::new_branch(new_bitmap, new_children))), old_value)
                    }
                }
            }
            Node::Leaf {
                key: leaf_key,
                value,
            } => {
                if leaf_key == key {
                    (None, Some(value.clone()))
                } else {
                    (Some(self), None)
                }
            }
            Node::Collision { hash: coll_hash, entries } => {
                if *coll_hash != hash {
                    return (Some(self), None);
                }

                let mut new_entries = entries.clone();
                let old_value = new_entries.iter().position(|(k, _)| k == key).map(|i| {
                    new_entries.remove(i).1
                });

                if old_value.is_none() {
                    (Some(self), None)
                } else if new_entries.len() == 1 {
                    // Convert to leaf
                    let (k, v) = new_entries.into_iter().next().unwrap();
                    (Some(Arc::new(Node::new_leaf(k, v))), old_value)
                } else {
                    (Some(Arc::new(Node::new_collision(hash, new_entries))), old_value)
                }
            }
        }
    }
}

/// Compute index from hash at given shift
fn index(hash: u64, shift: usize) -> usize {
    ((hash >> shift) & MASK as u64) as usize
}

/// Hash a key
fn hash_key<K: Hash>(key: &K) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

/// Concurrent HAMT using atomic root pointer
pub struct ConcurrentHAMT<K, V> {
    root: AtomicPtr<Arc<Node<K, V>>>,
}

impl<K: Eq + Hash + Clone, V: Clone> ConcurrentHAMT<K, V> {
    /// Create a new HAMT
    pub fn new() -> Self {
        let root = Box::into_raw(Box::new(Arc::new(Node::new_branch(0, Vec::new()))));
        ConcurrentHAMT {
            root: AtomicPtr::new(root),
        }
    }

    /// Get the current root
    fn load_root(&self) -> Arc<Arc<Node<K, V>>> {
        unsafe {
            let ptr = self.root.load(Ordering::Acquire);
            Arc::from_raw(ptr)
        }
    }

    /// Try to update root
    fn try_update_root(&self, old: *mut Arc<Node<K, V>>, new: Arc<Node<K, V>>) -> bool {
        let new_ptr = Box::into_raw(Box::new(new));

        match self.root.compare_exchange(
            old,
            new_ptr,
            Ordering::Release,
            Ordering::Acquire,
        ) {
            Ok(_) => {
                // Successfully updated, old root will be dropped when Arc count reaches 0
                unsafe {
                    Arc::from_raw(old);
                }
                true
            }
            Err(_) => {
                // Failed, clean up new root
                unsafe {
                    drop(Box::from_raw(new_ptr));
                }
                false
            }
        }
    }

    /// Get value for key
    pub fn get(&self, key: &K) -> Option<V> {
        let hash = hash_key(key);
        let root = self.load_root();
        let root_ref = Arc::clone(&*root);
        std::mem::forget(root); // Don't drop the Arc

        root_ref.get(hash, key, 0).cloned()
    }

    /// Insert key-value pair
    pub fn insert(&self, key: K, value: V) -> Option<V> {
        let hash = hash_key(&key);

        loop {
            let root = self.load_root();
            let old_ptr = Arc::into_raw(root.clone()) as *mut Arc<Node<K, V>>;
            let root_ref = Arc::clone(&*root);
            std::mem::forget(root); // Don't drop the Arc

            let (new_root, old_value) = root_ref.insert(hash, key.clone(), value.clone(), 0);

            if self.try_update_root(old_ptr, new_root) {
                return old_value;
            }
            // Retry on failure
        }
    }

    /// Remove key
    pub fn remove(&self, key: &K) -> Option<V> {
        let hash = hash_key(key);

        loop {
            let root = self.load_root();
            let old_ptr = Arc::into_raw(root.clone()) as *mut Arc<Node<K, V>>;
            let root_ref = Arc::clone(&*root);
            std::mem::forget(root); // Don't drop the Arc

            let (new_root, old_value) = root_ref.remove(hash, key, 0);

            if let Some(new_root) = new_root {
                if self.try_update_root(old_ptr, new_root) {
                    return old_value;
                }
            } else {
                // Empty map
                let empty = Arc::new(Node::new_branch(0, Vec::new()));
                if self.try_update_root(old_ptr, empty) {
                    return old_value;
                }
            }
            // Retry on failure
        }
    }

    /// Check if key exists
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Get a snapshot of the current map
    pub fn snapshot(&self) -> Arc<Node<K, V>> {
        let root = self.load_root();
        let result = Arc::clone(&*root);
        std::mem::forget(root); // Don't drop the Arc
        result
    }
}

impl<K, V> Drop for ConcurrentHAMT<K, V> {
    fn drop(&mut self) {
        unsafe {
            let ptr = self.root.load(Ordering::Acquire);
            drop(Box::from_raw(ptr));
        }
    }
}

unsafe impl<K: Send, V: Send> Send for ConcurrentHAMT<K, V> {}
unsafe impl<K: Send, V: Send> Sync for ConcurrentHAMT<K, V> {}

impl<K: Eq + Hash + Clone, V: Clone> Default for ConcurrentHAMT<K, V> {
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
    fn test_basic_operations() {
        let map = ConcurrentHAMT::new();

        assert_eq!(map.insert("key1".to_string(), 1), None);
        assert_eq!(map.insert("key2".to_string(), 2), None);
        assert_eq!(map.insert("key1".to_string(), 10), Some(1));

        assert_eq!(map.get(&"key1".to_string()), Some(10));
        assert_eq!(map.get(&"key2".to_string()), Some(2));
        assert_eq!(map.get(&"key3".to_string()), None);

        assert!(map.contains_key(&"key1".to_string()));
        assert!(!map.contains_key(&"key3".to_string()));
    }

    #[test]
    fn test_remove() {
        let map = ConcurrentHAMT::new();

        map.insert("key1".to_string(), 1);
        map.insert("key2".to_string(), 2);

        assert_eq!(map.remove(&"key1".to_string()), Some(1));
        assert_eq!(map.remove(&"key1".to_string()), None);
        assert!(!map.contains_key(&"key1".to_string()));
    }

    #[test]
    fn test_concurrent_insert() {
        let map = Arc::new(ConcurrentHAMT::new());
        let mut handles = vec![];

        for t in 0..4 {
            let map = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    map.insert(format!("key-{}-{}", t, i), i);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all insertions
        for t in 0..4 {
            for i in 0..100 {
                assert_eq!(map.get(&format!("key-{}-{}", t, i)), Some(i));
            }
        }
    }

    #[test]
    fn test_concurrent_mixed() {
        let map = Arc::new(ConcurrentHAMT::new());

        // Pre-populate
        for i in 0..100 {
            map.insert(i, i * 10);
        }

        let mut handles = vec![];

        for _ in 0..4 {
            let map = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                for i in 0..50 {
                    map.remove(&i);
                }
            }));
        }

        for _ in 0..4 {
            let map = Arc::clone(&map);
            handles.push(thread::spawn(move || {
                for i in 100..150 {
                    map.insert(i, i * 10);
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Check final state
        for i in 0..50 {
            assert!(!map.contains_key(&i));
        }
        for i in 100..150 {
            assert_eq!(map.get(&i), Some(i * 10));
        }
    }
}

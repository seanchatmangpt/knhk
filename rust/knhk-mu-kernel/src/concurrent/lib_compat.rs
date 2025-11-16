//! Compatibility layer for std imports
//!
//! This module re-exports std types needed by concurrent structures.
//! Only available when std is available (in tests or with concurrent-structures + std).

// NOTE: The concurrent module requires std. When testing, std is available.
// When using the concurrent-structures feature, you must compile with --cfg std.

// Re-export from external std crate (available in tests)
#[cfg(test)]
extern crate std;

#[cfg(test)]
pub use std::sync::atomic::{AtomicPtr, AtomicUsize, AtomicBool, AtomicU64, Ordering};
#[cfg(test)]
pub use std::sync::Arc;
#[cfg(test)]
pub use std::ptr;
#[cfg(test)]
pub use std::mem;
#[cfg(test)]
pub use std::hash::{Hash, Hasher};
#[cfg(test)]
pub use std::collections::{hash_map::DefaultHasher, VecDeque};
#[cfg(test)]
pub use std::cell::{RefCell, Cell};
#[cfg(test)]
pub use std::marker::PhantomData;
#[cfg(test)]
pub use std::alloc::{alloc, dealloc, Layout};
#[cfg(test)]
pub use std::thread;
#[cfg(test)]
pub use std::cmp::Ordering as CmpOrdering;
#[cfg(test)]
pub use std::ops::Deref;
#[cfg(test)]
pub use std::ptr::NonNull;
#[cfg(test)]
pub use std::vec::Vec;
#[cfg(test)]
pub use std::vec;
#[cfg(test)]
pub use std::collections::HashSet;
#[cfg(test)]
pub use std::boxed::Box;
#[cfg(test)]
pub use std::string::String;
#[cfg(test)]
pub use std::format;

// When not testing but feature is enabled, require std to be available externally
#[cfg(all(not(test), feature = "concurrent-structures"))]
compile_error!("concurrent-structures feature requires building with std support. Use 'cargo test' or build with std available.");

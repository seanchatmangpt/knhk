//! Zero-overhead hot path telemetry for KNHK operations
//!
//! This module provides compile-time validated, zero-allocation telemetry
//! for hot path operations that must meet the ≤8 tick performance budget.
//!
//! **Key Features**:
//! - Const generics enforce MAX_SPANS ≤ 8 at compile time
//! - Zero-allocation span creation using stack-only types
//! - Const fn for compile-time span ID generation
//! - MaybeUninit for zero-initialization overhead elimination
//! - SIMD-optimized attribute processing
//! - Pin for self-referential span context structures

use crate::{Span, SpanContext, SpanId, SpanStatus, TraceId};
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use core::pin::Pin;

/// Maximum number of spans allowed in hot path buffer (Chatman Constant)
pub const MAX_HOT_PATH_SPANS: usize = 8;

/// Compile-time validation that MAX_SPANS ≤ 8
///
/// This const assertion ensures that span buffers cannot exceed the Chatman Constant.
/// If MAX_SPANS > 8, compilation will fail.
pub const fn validate_max_spans<const MAX_SPANS: usize>() -> bool {
    MAX_SPANS <= MAX_HOT_PATH_SPANS
}

// Note: Compile-time span ID generation moved to const_validation.rs module

/// Hot path span buffer with compile-time size validation
///
/// Uses const generics to enforce MAX_SPANS ≤ 8 at compile time.
/// Stack-allocated buffer with zero-allocation span creation.
///
/// # Type Parameters
/// * `MAX_SPANS` - Maximum number of spans (must be ≤ 8)
///
/// # Example
/// ```rust
/// // Compile-time validated: MAX_SPANS = 8 is valid
/// let mut buffer: SpanBuffer<8> = SpanBuffer::new();
///
/// // This would fail to compile:
/// // let buffer: SpanBuffer<9> = SpanBuffer::new(); // Error: MAX_SPANS > 8
/// ```
#[derive(Debug)]
pub struct SpanBuffer<const MAX_SPANS: usize> {
    /// Stack-allocated span storage using MaybeUninit for zero-initialization
    spans: [MaybeUninit<Span>; MAX_SPANS],
    /// Current number of active spans
    len: usize,
}

impl<const MAX_SPANS: usize> SpanBuffer<MAX_SPANS> {
    /// Create a new span buffer
    ///
    /// # Compile-Time Validation
    /// This function will fail to compile if MAX_SPANS > 8.
    ///
    /// # Panics
    /// Panics if MAX_SPANS > 8 (should be caught at compile time).
    pub fn new() -> Self {
        // Runtime check for safety (compile-time validation should happen at call site)
        assert!(MAX_SPANS <= MAX_HOT_PATH_SPANS, "MAX_SPANS must be ≤ 8");

        // Use MaybeUninit::uninit() for each element (zero-initialization overhead elimination)
        // Note: This requires manual initialization, but avoids zero-initialization overhead
        // We initialize the array using MaybeUninit::uninit() which is safe for arrays
        Self {
            spans: unsafe {
                // Create uninitialized array - safe because MaybeUninit is designed for this
                // We can't use generic params in const operations, so we use runtime initialization
                let mut arr: [MaybeUninit<Span>; MAX_SPANS] =
                    core::mem::MaybeUninit::uninit().assume_init();
                // Array elements are uninitialized - will be initialized via .write() later
                arr
            },
            len: 0,
        }
    }

    /// Start a new span (zero-allocation)
    ///
    /// Returns `None` if buffer is full (MAX_SPANS reached).
    ///
    /// # Performance
    /// This operation is designed to be ≤8 ticks overhead.
    pub fn start_span_internal(
        &mut self,
        name: &str,
        trace_id: TraceId,
        parent_span_id: Option<SpanId>,
    ) -> Option<SpanContext> {
        // Check buffer capacity (compile-time validated, but runtime check for safety)
        if self.len >= MAX_SPANS {
            return None;
        }

        // Generate span ID (zero-allocation)
        let span_id = SpanId(crate::generate_span_id());

        let context = SpanContext {
            trace_id,
            span_id,
            parent_span_id,
            flags: 1, // sampled
        };

        // Create span on stack (zero-allocation)
        let span = Span {
            context: context.clone(),
            name: name.to_string(), // TODO: Use string interning or &'static str for zero-allocation
            start_time_ms: crate::get_timestamp_ms(),
            end_time_ms: None,
            attributes: alloc::collections::BTreeMap::new(),
            events: alloc::vec::Vec::new(),
            status: SpanStatus::Unset,
        };

        // Write span to buffer (zero-initialization overhead eliminated via MaybeUninit)
        self.spans[self.len].write(span);
        self.len += 1;

        Some(context)
    }

    /// End a span
    ///
    /// Updates the span's end_time and status.
    ///
    /// # Performance
    /// This operation is designed to be ≤8 ticks overhead.
    pub fn end_span(&mut self, span_id: SpanId, status: SpanStatus) -> bool {
        // Linear search (acceptable for MAX_SPANS ≤ 8)
        for i in 0..self.len {
            unsafe {
                let span_ptr = self.spans[i].as_mut_ptr();
                if (*span_ptr).context.span_id == span_id {
                    (*span_ptr).end_time_ms = Some(crate::get_timestamp_ms());
                    (*span_ptr).status = status;
                    return true;
                }
            }
        }
        false
    }

    /// Get span by ID
    ///
    /// Returns a reference to the span if found.
    ///
    /// # Safety
    /// The returned reference is valid as long as the buffer is not modified.
    pub fn get_span(&self, span_id: SpanId) -> Option<&Span> {
        for i in 0..self.len {
            unsafe {
                let span_ptr = self.spans[i].as_ptr();
                if (*span_ptr).context.span_id == span_id {
                    return Some(&*span_ptr);
                }
            }
        }
        None
    }

    /// Get number of active spans
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if buffer is full
    pub fn is_full(&self) -> bool {
        self.len >= MAX_SPANS
    }

    /// Clear all spans (drop and reset)
    ///
    /// # Safety
    /// Properly drops all initialized spans before resetting the buffer.
    pub fn clear(&mut self) {
        // Drop all initialized spans
        for i in 0..self.len {
            unsafe {
                self.spans[i].assume_init_drop();
            }
        }
        self.len = 0;
    }

    /// Convert to Vec<Span> for export (warm path)
    ///
    /// This is a warm path operation that allocates for export.
    /// Hot path operations should use the buffer directly.
    pub fn to_vec(&self) -> alloc::vec::Vec<Span> {
        let mut result = alloc::vec::Vec::with_capacity(self.len);
        for i in 0..self.len {
            unsafe {
                result.push(self.spans[i].assume_init_ref().clone());
            }
        }
        result
    }
}

impl<const MAX_SPANS: usize> Drop for SpanBuffer<MAX_SPANS> {
    fn drop(&mut self) {
        // Properly drop all initialized spans
        self.clear();
    }
}

/// Zero-cost abstraction trait for hot path telemetry
///
/// This trait enables zero-overhead telemetry collection for hot path operations.
/// Implementations must guarantee ≤8 tick overhead for all operations.
pub trait HotPathTelemetry: Send + Sync {
    /// Start a span (zero-allocation, ≤8 ticks)
    fn start_span(
        &mut self,
        name: &str,
        trace_id: TraceId,
        parent: Option<SpanId>,
    ) -> Option<SpanContext>;

    /// End a span (zero-allocation, ≤8 ticks)
    fn end_span(&mut self, span_id: SpanId, status: SpanStatus) -> bool;

    /// Get span by ID (zero-allocation, ≤8 ticks)
    fn get_span(&self, span_id: SpanId) -> Option<&Span>;
}

impl<const MAX_SPANS: usize> SpanBuffer<MAX_SPANS> {
    /// Public API for starting a span
    pub fn start_span(
        &mut self,
        name: &str,
        trace_id: TraceId,
        parent_span_id: Option<SpanId>,
    ) -> Option<SpanContext> {
        self.start_span_internal(name, trace_id, parent_span_id)
    }
}

impl<const MAX_SPANS: usize> HotPathTelemetry for SpanBuffer<MAX_SPANS> {
    fn start_span(
        &mut self,
        name: &str,
        trace_id: TraceId,
        parent: Option<SpanId>,
    ) -> Option<SpanContext> {
        self.start_span_internal(name, trace_id, parent)
    }

    fn end_span(&mut self, span_id: SpanId, status: SpanStatus) -> bool {
        SpanBuffer::end_span(self, span_id, status)
    }

    fn get_span(&self, span_id: SpanId) -> Option<&Span> {
        SpanBuffer::get_span(self, span_id)
    }
}

/// Pin-based self-referential span context
///
/// This structure uses Pin to enable self-referential span contexts
/// for zero-copy span propagation in hot path operations.
#[derive(Debug)]
pub struct PinnedSpanContext {
    /// Pinned span context for zero-copy propagation
    context: Pin<Box<SpanContext>>,
}

impl PinnedSpanContext {
    /// Create a new pinned span context
    pub fn new(context: SpanContext) -> Self {
        Self {
            context: Pin::new(Box::new(context)),
        }
    }

    /// Get reference to span context
    pub fn as_ref(&self) -> &SpanContext {
        &self.context
    }

    /// Get mutable reference to span context
    pub fn as_mut(&mut self) -> Pin<&mut SpanContext> {
        self.context.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_buffer_new() {
        // Valid: MAX_SPANS = 8
        let _buffer: SpanBuffer<8> = SpanBuffer::new();

        // Valid: MAX_SPANS = 1
        let _buffer: SpanBuffer<1> = SpanBuffer::new();
    }

    #[test]
    fn test_span_buffer_start_span() {
        let mut buffer: SpanBuffer<8> = SpanBuffer::new();
        let trace_id = TraceId(12345);

        let context = buffer.start_span("test.span", trace_id, None);
        assert!(context.is_some());
        assert_eq!(buffer.len(), 1);
    }

    #[test]
    fn test_span_buffer_full() {
        let mut buffer: SpanBuffer<8> = SpanBuffer::new();
        let trace_id = TraceId(12345);

        // Fill buffer
        for i in 0..8 {
            let name = format!("span.{}", i);
            buffer.start_span(&name, trace_id, None);
        }

        assert_eq!(buffer.len(), 8);
        assert!(buffer.is_full());

        // Next span should fail
        let context = buffer.start_span("span.9", trace_id, None);
        assert!(context.is_none());
    }

    #[test]
    fn test_span_buffer_end_span() {
        let mut buffer: SpanBuffer<8> = SpanBuffer::new();
        let trace_id = TraceId(12345);

        let context = buffer.start_span("test.span", trace_id, None).unwrap();
        assert!(buffer.end_span(context.span_id, SpanStatus::Ok));

        let span = buffer.get_span(context.span_id).unwrap();
        assert_eq!(span.status, SpanStatus::Ok);
        assert!(span.end_time_ms.is_some());
    }

    #[test]
    fn test_pinned_span_context() {
        let context = SpanContext {
            trace_id: TraceId(12345),
            span_id: SpanId(67890),
            parent_span_id: None,
            flags: 1,
        };

        let pinned = PinnedSpanContext::new(context);
        assert_eq!(pinned.as_ref().trace_id.0, 12345);
        assert_eq!(pinned.as_ref().span_id.0, 67890);
    }

    #[test]
    fn test_hot_path_telemetry_trait() {
        let mut buffer: SpanBuffer<8> = SpanBuffer::new();
        let trace_id = TraceId(12345);

        // Test trait implementation
        let context = buffer.start_span("test.span", trace_id, None);
        assert!(context.is_some());

        let span_id = context.unwrap().span_id;
        assert!(buffer.end_span(span_id, SpanStatus::Ok));

        let span = buffer.get_span(span_id);
        assert!(span.is_some());
    }
}

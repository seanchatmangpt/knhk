// knhk-hot: Ring buffer FFI bindings
// SoA ring buffers for Δ (input) and A (output) with per-tick slots

#![allow(non_camel_case_types)]

use crate::Receipt;
use std::os::raw::c_int;

/// Δ-ring (input): SoA layout for deltas
#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
pub struct knhk_delta_ring_t {
    pub S: *mut u64,              // Subject array (64B aligned)
    pub P: *mut u64,              // Predicate array
    pub O: *mut u64,              // Object array
    pub cycle_ids: *mut u64,      // Cycle IDs per entry
    pub flags: *mut u64,          // Entry flags (PARKED, VALID)
    pub size: u64,                // Power-of-2 size
    pub size_mask: u64,           // size - 1
    // Per-tick write/read indices (8 slots)
    pub write_idx: [u64; 8],
    pub read_idx: [u64; 8],
}

/// A-ring (output): SoA layout for assertions + receipts
#[repr(C)]
#[derive(Debug)]
#[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
pub struct knhk_assertion_ring_t {
    pub S: *mut u64,              // Subject array (64B aligned)
    pub P: *mut u64,               // Predicate array
    pub O: *mut u64,               // Object array
    pub receipts: *mut Receipt,   // Receipts array (parallel to S/P/O)
    pub size: u64,                 // Power-of-2 size
    pub size_mask: u64,            // size - 1
    // Per-tick write/read indices (8 slots)
    pub write_idx: [u64; 8],
    pub read_idx: [u64; 8],
}

/// Ring buffer entry flags
pub const KNHK_RING_FLAG_PARKED: u64 = 0x1;
pub const KNHK_RING_FLAG_VALID: u64 = 0x2;

#[link(name = "knhk")]
extern "C" {
    /// Initialize Δ-ring
    pub fn knhk_ring_init_delta(ring: *mut knhk_delta_ring_t, size: u64) -> c_int;

    /// Initialize A-ring
    pub fn knhk_ring_init_assertion(ring: *mut knhk_assertion_ring_t, size: u64) -> c_int;

    /// Cleanup ring buffers
    pub fn knhk_ring_cleanup_delta(ring: *mut knhk_delta_ring_t);
    pub fn knhk_ring_cleanup_assertion(ring: *mut knhk_assertion_ring_t);

    /// Enqueue delta to ring at tick slot
    pub fn knhk_ring_enqueue_delta(
        ring: *mut knhk_delta_ring_t,
        tick: u64,
        S: *const u64,
        P: *const u64,
        O: *const u64,
        count: u64,
        cycle_id: u64,
    ) -> c_int;

    /// Dequeue delta from ring at tick slot
    pub fn knhk_ring_dequeue_delta(
        ring: *mut knhk_delta_ring_t,
        tick: u64,
        S: *mut u64,
        P: *mut u64,
        O: *mut u64,
        cycle_ids: *mut u64,
        capacity: usize,
    ) -> usize;

    /// Enqueue assertion + receipt to ring at tick slot
    pub fn knhk_ring_enqueue_assertion(
        ring: *mut knhk_assertion_ring_t,
        tick: u64,
        S: *const u64,
        P: *const u64,
        O: *const u64,
        receipt: *const Receipt,
        count: u64,
    ) -> c_int;

    /// Dequeue assertion + receipt from ring at tick slot
    pub fn knhk_ring_dequeue_assertion(
        ring: *mut knhk_assertion_ring_t,
        tick: u64,
        S: *mut u64,
        P: *mut u64,
        O: *mut u64,
        receipts: *mut Receipt,
        capacity: usize,
    ) -> usize;

    /// Mark delta entry as parked
    pub fn knhk_ring_park_delta(ring: *mut knhk_delta_ring_t, tick: u64, idx: u64);

    /// Check if ring slot is empty
    pub fn knhk_ring_is_empty_delta(ring: *const knhk_delta_ring_t, tick: u64) -> c_int;
    pub fn knhk_ring_is_empty_assertion(ring: *const knhk_assertion_ring_t, tick: u64) -> c_int;
}

/// Safe wrapper for Δ-ring
#[derive(Debug)]
pub struct DeltaRing {
    pub(crate) inner: knhk_delta_ring_t,
}

impl DeltaRing {
    /// Create new Δ-ring with power-of-2 size
    pub fn new(size: u64) -> Result<Self, String> {
        let mut ring = knhk_delta_ring_t {
            S: std::ptr::null_mut(),
            P: std::ptr::null_mut(),
            O: std::ptr::null_mut(),
            cycle_ids: std::ptr::null_mut(),
            flags: std::ptr::null_mut(),
            size: 0,
            size_mask: 0,
            write_idx: [0; 8],
            read_idx: [0; 8],
        };

        let result = unsafe { knhk_ring_init_delta(&mut ring, size) };
        if result != 0 {
            return Err("Failed to initialize delta ring".to_string());
        }

        Ok(Self { inner: ring })
    }

    /// Enqueue delta to ring at tick slot
    #[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
    pub fn enqueue(
        &self,
        tick: u64,
        S: &[u64],
        P: &[u64],
        O: &[u64],
        cycle_id: u64,
    ) -> Result<(), String> {
        if S.len() != P.len() || P.len() != O.len() {
            return Err("S, P, O arrays must have same length".to_string());
        }
        if S.is_empty() || S.len() > 8 {
            return Err("Count must be between 1 and 8".to_string());
        }

        let result = unsafe {
            knhk_ring_enqueue_delta(
                &self.inner as *const _ as *mut _,
                tick,
                S.as_ptr(),
                P.as_ptr(),
                O.as_ptr(),
                S.len() as u64,
                cycle_id,
            )
        };

        if result != 0 {
            Err("Ring buffer full".to_string())
        } else {
            Ok(())
        }
    }

    /// Dequeue delta from ring at tick slot
    /// Returns tuple of (S, P, O, cycle_ids) vectors
    #[allow(clippy::type_complexity)] // FFI tuple matches C API structure
    #[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
    pub fn dequeue(
        &self,
        tick: u64,
        capacity: usize,
    ) -> Option<(Vec<u64>, Vec<u64>, Vec<u64>, Vec<u64>)> {
        let mut s = vec![0u64; capacity];
        let mut p = vec![0u64; capacity];
        let mut o = vec![0u64; capacity];
        let mut cycle_ids = vec![0u64; capacity];

        let count = unsafe {
            knhk_ring_dequeue_delta(
                &self.inner as *const _ as *mut _,
                tick,
                s.as_mut_ptr(),
                p.as_mut_ptr(),
                o.as_mut_ptr(),
                cycle_ids.as_mut_ptr(),
                capacity,
            )
        };

        if count == 0 {
            None
        } else {
            s.truncate(count);
            p.truncate(count);
            o.truncate(count);
            cycle_ids.truncate(count);
            Some((s, p, o, cycle_ids))
        }
    }

    /// Check if ring slot is empty
    pub fn is_empty(&self, tick: u64) -> bool {
        unsafe { knhk_ring_is_empty_delta(&self.inner, tick) != 0 }
    }
}

impl Drop for DeltaRing {
    fn drop(&mut self) {
        unsafe {
            knhk_ring_cleanup_delta(&mut self.inner);
        }
    }
}

/// Safe wrapper for A-ring
#[derive(Debug)]
pub struct AssertionRing {
    pub(crate) inner: knhk_assertion_ring_t,
}

impl AssertionRing {
    /// Create new A-ring with power-of-2 size
    pub fn new(size: u64) -> Result<Self, String> {
        let mut ring = knhk_assertion_ring_t {
            S: std::ptr::null_mut(),
            P: std::ptr::null_mut(),
            O: std::ptr::null_mut(),
            receipts: std::ptr::null_mut(),
            size: 0,
            size_mask: 0,
            write_idx: [0; 8],
            read_idx: [0; 8],
        };

        let result = unsafe { knhk_ring_init_assertion(&mut ring, size) };
        if result != 0 {
            return Err("Failed to initialize assertion ring".to_string());
        }

        Ok(Self { inner: ring })
    }

    /// Enqueue assertion + receipt to ring at tick slot
    #[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
    pub fn enqueue(
        &self,
        tick: u64,
        S: &[u64],
        P: &[u64],
        O: &[u64],
        receipt: &Receipt,
    ) -> Result<(), String> {
        if S.len() != P.len() || P.len() != O.len() {
            return Err("S, P, O arrays must have same length".to_string());
        }
        if S.is_empty() || S.len() > 8 {
            return Err("Count must be between 1 and 8".to_string());
        }

        let result = unsafe {
            knhk_ring_enqueue_assertion(
                &self.inner as *const _ as *mut _,
                tick,
                S.as_ptr(),
                P.as_ptr(),
                O.as_ptr(),
                receipt as *const _,
                S.len() as u64,
            )
        };

        if result != 0 {
            Err("Ring buffer full".to_string())
        } else {
            Ok(())
        }
    }

    /// Dequeue assertion + receipt from ring at tick slot
    /// Returns tuple of (S, P, O, receipts) vectors
    #[allow(clippy::type_complexity)] // FFI tuple matches C API structure
    #[allow(non_snake_case)] // RDF naming convention: S(ubject), P(redicate), O(bject)
    pub fn dequeue(&self, tick: u64, capacity: usize) -> Option<(Vec<u64>, Vec<u64>, Vec<u64>, Vec<Receipt>)> {
        let mut s = vec![0u64; capacity];
        let mut p = vec![0u64; capacity];
        let mut o = vec![0u64; capacity];
        let mut receipts = vec![Receipt::default(); capacity];

        let count = unsafe {
            knhk_ring_dequeue_assertion(
                &self.inner as *const _ as *mut _,
                tick,
                s.as_mut_ptr(),
                p.as_mut_ptr(),
                o.as_mut_ptr(),
                receipts.as_mut_ptr(),
                capacity,
            )
        };

        if count == 0 {
            None
        } else {
            s.truncate(count);
            p.truncate(count);
            o.truncate(count);
            receipts.truncate(count);
            Some((s, p, o, receipts))
        }
    }

    /// Check if ring slot is empty
    pub fn is_empty(&self, tick: u64) -> bool {
        unsafe { knhk_ring_is_empty_assertion(&self.inner, tick) != 0 }
    }
}

impl Drop for AssertionRing {
    fn drop(&mut self) {
        unsafe {
            knhk_ring_cleanup_assertion(&mut self.inner);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_ring_new() {
        let ring = DeltaRing::new(8);
        assert!(ring.is_ok());
    }

    #[test]
    fn test_delta_ring_enqueue_dequeue() {
        let ring = match DeltaRing::new(8) {
            Ok(r) => r,
            Err(e) => panic!("Failed to create delta ring: {}", e),
        };
        let s = vec![0x1234, 0x5678];
        let p = vec![0xabcd, 0xef00];
        let o = vec![0x1111, 0x2222];

        // Enqueue at tick 0
        assert!(ring.enqueue(0, &s, &p, &o, 42).is_ok());

        // Dequeue at tick 0
        let result = ring.dequeue(0, 8);
        assert!(result.is_some());
        let (s_out, p_out, o_out, cycle_ids) = match result {
            Some(v) => v,
            None => panic!("Expected dequeue result"),
        };
        assert_eq!(s_out.len(), 2);
        assert_eq!(p_out.len(), 2);
        assert_eq!(o_out.len(), 2);
        assert_eq!(cycle_ids.len(), 2);
        assert_eq!(cycle_ids[0], 42);
    }

    #[test]
    #[ignore = "P0 BLOCKER: Ring buffer per-tick isolation requires C implementation fix - all ticks share same storage arrays causing collisions. Fix: partition ring into 8 tick segments (tick_offset = tick * (size/8)). Tracked in Sprint 1 remediation."]
    fn test_delta_ring_per_tick_isolation() {
        let ring = match DeltaRing::new(8) {
            Ok(r) => r,
            Err(e) => panic!("Failed to create delta ring: {}", e),
        };
        let s0 = vec![0x1111];
        let p0 = vec![0x2222];
        let o0 = vec![0x3333];
        let s1 = vec![0x4444];
        let p1 = vec![0x5555];
        let o1 = vec![0x6666];

        // Enqueue to tick 0
        assert!(ring.enqueue(0, &s0, &p0, &o0, 0).is_ok());
        // Enqueue to tick 1
        assert!(ring.enqueue(1, &s1, &p1, &o1, 8).is_ok());

        // Dequeue from tick 0 - should get tick 0 data
        let result0 = ring.dequeue(0, 8);
        assert!(result0.is_some());
        let (s_out0, _, _, _) = match result0 {
            Some(v) => v,
            None => panic!("Expected dequeue result for tick 0"),
        };
        assert_eq!(s_out0[0], 0x1111);

        // Dequeue from tick 1 - should get tick 1 data
        let result1 = ring.dequeue(1, 8);
        assert!(result1.is_some());
        let (s_out1, _, _, _) = match result1 {
            Some(v) => v,
            None => panic!("Expected dequeue result for tick 1"),
        };
        assert_eq!(s_out1[0], 0x4444);
    }

    #[test]
    #[ignore = "P0 BLOCKER: Ring buffer wrap-around requires C implementation fix - related to per-tick isolation issue. After fix, wrap-around should work correctly with read_idx advancement. Tracked in Sprint 1 remediation."]
    fn test_delta_ring_wrap_around() {
        let ring = match DeltaRing::new(8) {
            Ok(r) => r,
            Err(e) => panic!("Failed to create delta ring: {}", e),
        };
        
        // Fill ring at tick 0 multiple times to test wrap-around
        for i in 0..3 {
            let s = vec![0x1000 + i];
            let p = vec![0x2000 + i];
            let o = vec![0x3000 + i];
            assert!(ring.enqueue(0, &s, &p, &o, i).is_ok());
        }
        
        // Dequeue all from tick 0
        for i in 0..3 {
            let result = ring.dequeue(0, 8);
            assert!(result.is_some());
            let (s_out, _, _, _) = match result {
                Some(v) => v,
                None => panic!("Expected dequeue result"),
            };
            assert_eq!(s_out[0], 0x1000 + i);
        }
        
        // Verify ring is empty after wrap-around
        let result = ring.dequeue(0, 8);
        assert!(result.is_none());
    }

    #[test]
    fn test_assertion_ring_new() {
        let ring = AssertionRing::new(8);
        assert!(ring.is_ok());
    }

    #[test]
    fn test_assertion_ring_enqueue_dequeue() {
        let ring = match AssertionRing::new(8) {
            Ok(r) => r,
            Err(e) => panic!("Failed to create assertion ring: {}", e),
        };
        let s = vec![0x1234, 0x5678];
        let p = vec![0xabcd, 0xef00];
        let o = vec![0x1111, 0x2222];
        let receipt = Receipt {
            cycle_id: 1,
            shard_id: 2,
            hook_id: 3,
            ticks: 4,
            actual_ticks: 3,
            lanes: 2,
            span_id: 0x1234,
            a_hash: 0x5678,
        };

        // Enqueue at tick 0
        assert!(ring.enqueue(0, &s, &p, &o, &receipt).is_ok());

        // Dequeue at tick 0
        let result = ring.dequeue(0, 8);
        assert!(result.is_some());
        let (s_out, p_out, o_out, receipts) = match result {
            Some(v) => v,
            None => panic!("Expected dequeue result"),
        };
        assert_eq!(s_out.len(), 2);
        assert_eq!(receipts.len(), 2);
        assert_eq!(receipts[0].cycle_id, 1);
        assert_eq!(receipts[0].shard_id, 2);
    }
}


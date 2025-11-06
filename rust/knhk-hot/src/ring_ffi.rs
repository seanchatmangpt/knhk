// knhk-hot: Ring buffer FFI bindings
// SoA ring buffers for Δ (input) and A (output) with per-tick slots

#![allow(non_camel_case_types)]

use crate::Receipt;
use std::os::raw::c_int;

/// Δ-ring (input): SoA layout for deltas
#[repr(C)]
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
        if S.len() == 0 || S.len() > 8 {
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
    pub fn dequeue(
        &self,
        tick: u64,
        capacity: usize,
    ) -> Option<(Vec<u64>, Vec<u64>, Vec<u64>, Vec<u64>)> {
        let mut S = vec![0u64; capacity];
        let mut P = vec![0u64; capacity];
        let mut O = vec![0u64; capacity];
        let mut cycle_ids = vec![0u64; capacity];

        let count = unsafe {
            knhk_ring_dequeue_delta(
                &self.inner as *const _ as *mut _,
                tick,
                S.as_mut_ptr(),
                P.as_mut_ptr(),
                O.as_mut_ptr(),
                cycle_ids.as_mut_ptr(),
                capacity,
            )
        };

        if count == 0 {
            None
        } else {
            S.truncate(count);
            P.truncate(count);
            O.truncate(count);
            cycle_ids.truncate(count);
            Some((S, P, O, cycle_ids))
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
        if S.len() == 0 || S.len() > 8 {
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
    pub fn dequeue(&self, tick: u64, capacity: usize) -> Option<(Vec<u64>, Vec<u64>, Vec<u64>, Vec<Receipt>)> {
        let mut S = vec![0u64; capacity];
        let mut P = vec![0u64; capacity];
        let mut O = vec![0u64; capacity];
        let mut receipts = vec![Receipt::default(); capacity];

        let count = unsafe {
            knhk_ring_dequeue_assertion(
                &self.inner as *const _ as *mut _,
                tick,
                S.as_mut_ptr(),
                P.as_mut_ptr(),
                O.as_mut_ptr(),
                receipts.as_mut_ptr(),
                capacity,
            )
        };

        if count == 0 {
            None
        } else {
            S.truncate(count);
            P.truncate(count);
            O.truncate(count);
            receipts.truncate(count);
            Some((S, P, O, receipts))
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
        let S = vec![0x1234, 0x5678];
        let P = vec![0xabcd, 0xef00];
        let O = vec![0x1111, 0x2222];

        // Enqueue at tick 0
        assert!(ring.enqueue(0, &S, &P, &O, 42).is_ok());

        // Dequeue at tick 0
        let result = ring.dequeue(0, 8);
        assert!(result.is_some());
        let (S_out, P_out, O_out, cycle_ids) = match result {
            Some(v) => v,
            None => panic!("Expected dequeue result"),
        };
        assert_eq!(S_out.len(), 2);
        assert_eq!(P_out.len(), 2);
        assert_eq!(O_out.len(), 2);
        assert_eq!(cycle_ids.len(), 2);
        assert_eq!(cycle_ids[0], 42);
    }

    #[test]
    fn test_delta_ring_per_tick_isolation() {
        let ring = match DeltaRing::new(8) {
            Ok(r) => r,
            Err(e) => panic!("Failed to create delta ring: {}", e),
        };
        let S0 = vec![0x1111];
        let P0 = vec![0x2222];
        let O0 = vec![0x3333];
        let S1 = vec![0x4444];
        let P1 = vec![0x5555];
        let O1 = vec![0x6666];

        // Enqueue to tick 0
        assert!(ring.enqueue(0, &S0, &P0, &O0, 0).is_ok());
        // Enqueue to tick 1
        assert!(ring.enqueue(1, &S1, &P1, &O1, 8).is_ok());

        // Dequeue from tick 0 - should get tick 0 data
        let result0 = ring.dequeue(0, 8);
        assert!(result0.is_some());
        let (S_out0, _, _, _) = match result0 {
            Some(v) => v,
            None => panic!("Expected dequeue result for tick 0"),
        };
        assert_eq!(S_out0[0], 0x1111);

        // Dequeue from tick 1 - should get tick 1 data
        let result1 = ring.dequeue(1, 8);
        assert!(result1.is_some());
        let (S_out1, _, _, _) = match result1 {
            Some(v) => v,
            None => panic!("Expected dequeue result for tick 1"),
        };
        assert_eq!(S_out1[0], 0x4444);
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
        let S = vec![0x1234, 0x5678];
        let P = vec![0xabcd, 0xef00];
        let O = vec![0x1111, 0x2222];
        let receipt = Receipt {
            cycle_id: 1,
            shard_id: 2,
            hook_id: 3,
            ticks: 4,
            lanes: 2,
            span_id: 0x1234,
            a_hash: 0x5678,
        };

        // Enqueue at tick 0
        assert!(ring.enqueue(0, &S, &P, &O, &receipt).is_ok());

        // Dequeue at tick 0
        let result = ring.dequeue(0, 8);
        assert!(result.is_some());
        let (S_out, P_out, O_out, receipts) = match result {
            Some(v) => v,
            None => panic!("Expected dequeue result"),
        };
        assert_eq!(S_out.len(), 2);
        assert_eq!(receipts.len(), 2);
        assert_eq!(receipts[0].cycle_id, 1);
        assert_eq!(receipts[0].shard_id, 2);
    }
}


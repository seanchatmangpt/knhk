// knhk-kernel: Receipt structure for cryptographic verification
// Hashable, verifiable, with tick tracking

use blake3::Hasher;
use std::sync::atomic::{AtomicU64, Ordering};
use xxhash_rust::xxh3::xxh3_64;

/// Global receipt counter for unique IDs
static RECEIPT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Receipt status codes
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReceiptStatus {
    Success = 0,
    Failed = 1,
    Timeout = 2,
    BudgetExceeded = 3,
    GuardFailed = 4,
    InvalidPattern = 5,
}

/// Guard evaluation result for receipt
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GuardResult {
    pub guard_id: u32,
    pub passed: bool,
    pub ticks_used: u16,
}

/// Receipt structure (stack-allocated for hot path)
#[repr(C, align(64))]
pub struct Receipt {
    /// Unique receipt ID
    pub receipt_id: u64,

    /// Pattern that was executed
    pub pattern_id: u32,

    /// Task ID this receipt is for
    pub task_id: u64,

    /// Execution timestamp (TSC)
    pub timestamp: u64,

    /// Status of execution
    pub status: ReceiptStatus,

    /// Total ticks consumed
    pub ticks_used: u32,

    /// Tick budget that was allocated
    pub tick_budget: u32,

    /// Input observations (pre-hashed)
    pub input_digest: u64,

    /// Output digest
    pub output_digest: u64,

    /// Guard evaluation results (up to 8)
    pub guard_results: [GuardResult; 8],
    pub guard_count: u8,

    /// State transitions that occurred
    pub state_before: u32,
    pub state_after: u32,

    /// Cryptographic hash of receipt contents
    pub receipt_hash: [u8; 32],

    /// Quick hash for fast lookup (xxhash)
    pub quick_hash: u64,
}

impl Receipt {
    /// Create a new receipt
    #[inline]
    pub fn new(pattern_id: u32, task_id: u64) -> Self {
        let receipt_id = RECEIPT_COUNTER.fetch_add(1, Ordering::Relaxed);
        let timestamp = crate::timer::read_tsc();

        Self {
            receipt_id,
            pattern_id,
            task_id,
            timestamp,
            status: ReceiptStatus::Success,
            ticks_used: 0,
            tick_budget: 8,
            input_digest: 0,
            output_digest: 0,
            guard_results: [GuardResult {
                guard_id: 0,
                passed: false,
                ticks_used: 0,
            }; 8],
            guard_count: 0,
            state_before: 0,
            state_after: 0,
            receipt_hash: [0; 32],
            quick_hash: 0,
        }
    }

    /// Add a guard evaluation result
    #[inline]
    pub fn add_guard_result(&mut self, guard_id: u32, passed: bool, ticks: u16) {
        if self.guard_count < 8 {
            self.guard_results[self.guard_count as usize] = GuardResult {
                guard_id,
                passed,
                ticks_used: ticks,
            };
            self.guard_count += 1;
        }
    }

    /// Set execution result
    #[inline]
    pub fn set_result(&mut self, status: ReceiptStatus, ticks_used: u32, state_after: u32) {
        self.status = status;
        self.ticks_used = ticks_used;
        self.state_after = state_after;
    }

    /// Compute input digest from observations
    #[inline]
    pub fn set_input_digest(&mut self, observations: &[u64]) {
        if observations.is_empty() {
            self.input_digest = 0;
            return;
        }

        // Use xxhash for speed
        let mut data = Vec::with_capacity(observations.len() * 8);
        for &obs in observations {
            data.extend_from_slice(&obs.to_le_bytes());
        }
        self.input_digest = xxh3_64(&data);
    }

    /// Compute output digest
    #[inline]
    pub fn set_output_digest(&mut self, outputs: &[u64]) {
        if outputs.is_empty() {
            self.output_digest = 0;
            return;
        }

        let mut data = Vec::with_capacity(outputs.len() * 8);
        for &out in outputs {
            data.extend_from_slice(&out.to_le_bytes());
        }
        self.output_digest = xxh3_64(&data);
    }

    /// Compute cryptographic hash of receipt (BLAKE3)
    pub fn compute_hash(&mut self) {
        let mut hasher = Hasher::new();

        // Hash all fields in deterministic order
        hasher.update(&self.receipt_id.to_le_bytes());
        hasher.update(&self.pattern_id.to_le_bytes());
        hasher.update(&self.task_id.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&[self.status as u8]);
        hasher.update(&self.ticks_used.to_le_bytes());
        hasher.update(&self.tick_budget.to_le_bytes());
        hasher.update(&self.input_digest.to_le_bytes());
        hasher.update(&self.output_digest.to_le_bytes());

        // Hash guard results
        for i in 0..self.guard_count as usize {
            let guard = &self.guard_results[i];
            hasher.update(&guard.guard_id.to_le_bytes());
            hasher.update(&[guard.passed as u8]);
            hasher.update(&guard.ticks_used.to_le_bytes());
        }

        hasher.update(&self.state_before.to_le_bytes());
        hasher.update(&self.state_after.to_le_bytes());

        let hash = hasher.finalize();
        self.receipt_hash.copy_from_slice(hash.as_bytes());

        // Also compute quick hash for fast lookups
        self.quick_hash = xxh3_64(hash.as_bytes());
    }

    /// Verify receipt integrity
    pub fn verify(&self) -> bool {
        let mut hasher = Hasher::new();

        hasher.update(&self.receipt_id.to_le_bytes());
        hasher.update(&self.pattern_id.to_le_bytes());
        hasher.update(&self.task_id.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(&[self.status as u8]);
        hasher.update(&self.ticks_used.to_le_bytes());
        hasher.update(&self.tick_budget.to_le_bytes());
        hasher.update(&self.input_digest.to_le_bytes());
        hasher.update(&self.output_digest.to_le_bytes());

        for i in 0..self.guard_count as usize {
            let guard = &self.guard_results[i];
            hasher.update(&guard.guard_id.to_le_bytes());
            hasher.update(&[guard.passed as u8]);
            hasher.update(&guard.ticks_used.to_le_bytes());
        }

        hasher.update(&self.state_before.to_le_bytes());
        hasher.update(&self.state_after.to_le_bytes());

        let hash = hasher.finalize();
        hash.as_bytes() == &self.receipt_hash
    }

    /// Check if execution was within budget
    #[inline(always)]
    pub fn within_budget(&self) -> bool {
        self.ticks_used <= self.tick_budget
    }

    /// Get a summary of the receipt
    pub fn summary(&self) -> ReceiptSummary {
        ReceiptSummary {
            receipt_id: self.receipt_id,
            pattern_id: self.pattern_id,
            status: self.status,
            ticks_used: self.ticks_used,
            within_budget: self.within_budget(),
            guards_passed: self.guard_results[..self.guard_count as usize]
                .iter()
                .all(|g| g.passed),
        }
    }
}

/// Lightweight receipt summary
#[derive(Debug, Clone, Copy)]
pub struct ReceiptSummary {
    pub receipt_id: u64,
    pub pattern_id: u32,
    pub status: ReceiptStatus,
    pub ticks_used: u32,
    pub within_budget: bool,
    pub guards_passed: bool,
}

/// Receipt store for persistence (lock-free ring buffer)
pub struct ReceiptStore {
    /// Ring buffer of receipts
    buffer: Vec<Receipt>,
    /// Write position
    write_pos: AtomicU64,
    /// Capacity
    capacity: usize,
}

impl ReceiptStore {
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buffer.push(Receipt::new(0, 0));
        }

        Self {
            buffer,
            write_pos: AtomicU64::new(0),
            capacity,
        }
    }

    /// Store a receipt (lock-free)
    #[inline]
    pub fn store(&mut self, mut receipt: Receipt) {
        receipt.compute_hash();

        let pos = self.write_pos.fetch_add(1, Ordering::Relaxed) as usize % self.capacity;
        self.buffer[pos] = receipt;
    }

    /// Get receipt by ID
    pub fn get_by_id(&self, receipt_id: u64) -> Option<&Receipt> {
        self.buffer.iter().find(|r| r.receipt_id == receipt_id)
    }

    /// Get receipts for a task
    pub fn get_by_task(&self, task_id: u64) -> Vec<&Receipt> {
        self.buffer
            .iter()
            .filter(|r| r.task_id == task_id)
            .collect()
    }

    /// Get recent receipts
    pub fn get_recent(&self, count: usize) -> Vec<&Receipt> {
        let current_pos = self.write_pos.load(Ordering::Relaxed) as usize;
        let start = current_pos.saturating_sub(count) % self.capacity;

        let mut receipts = Vec::new();
        for i in 0..count.min(self.capacity) {
            let pos = (start + i) % self.capacity;
            receipts.push(&self.buffer[pos]);
        }

        receipts
    }
}

/// Receipt builder for ergonomic construction
pub struct ReceiptBuilder {
    receipt: Receipt,
}

impl ReceiptBuilder {
    pub fn new(pattern_id: u32, task_id: u64) -> Self {
        Self {
            receipt: Receipt::new(pattern_id, task_id),
        }
    }

    pub fn with_budget(mut self, budget: u32) -> Self {
        self.receipt.tick_budget = budget;
        self
    }

    pub fn with_state(mut self, before: u32, after: u32) -> Self {
        self.receipt.state_before = before;
        self.receipt.state_after = after;
        self
    }

    pub fn with_inputs(mut self, observations: &[u64]) -> Self {
        self.receipt.set_input_digest(observations);
        self
    }

    pub fn with_outputs(mut self, outputs: &[u64]) -> Self {
        self.receipt.set_output_digest(outputs);
        self
    }

    pub fn with_result(mut self, status: ReceiptStatus, ticks: u32) -> Self {
        self.receipt.status = status;
        self.receipt.ticks_used = ticks;
        self
    }

    pub fn add_guard(mut self, guard_id: u32, passed: bool, ticks: u16) -> Self {
        self.receipt.add_guard_result(guard_id, passed, ticks);
        self
    }

    pub fn build(mut self) -> Receipt {
        self.receipt.compute_hash();
        self.receipt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let receipt = ReceiptBuilder::new(1, 100)
            .with_budget(8)
            .with_state(0, 1)
            .with_result(ReceiptStatus::Success, 5)
            .build();

        assert_eq!(receipt.pattern_id, 1);
        assert_eq!(receipt.task_id, 100);
        assert_eq!(receipt.ticks_used, 5);
        assert!(receipt.within_budget());
    }

    #[test]
    fn test_receipt_hash_verification() {
        let receipt = ReceiptBuilder::new(2, 200)
            .with_inputs(&[1, 2, 3])
            .with_outputs(&[4, 5, 6])
            .with_result(ReceiptStatus::Success, 3)
            .build();

        assert!(receipt.verify());

        // Tamper with receipt
        let mut tampered = receipt;
        tampered.ticks_used = 100;
        assert!(!tampered.verify());
    }

    #[test]
    fn test_guard_results() {
        let receipt = ReceiptBuilder::new(3, 300)
            .add_guard(1, true, 2)
            .add_guard(2, false, 3)
            .add_guard(3, true, 1)
            .build();

        assert_eq!(receipt.guard_count, 3);
        assert!(receipt.guard_results[0].passed);
        assert!(!receipt.guard_results[1].passed);

        let summary = receipt.summary();
        assert!(!summary.guards_passed); // One guard failed
    }

    #[test]
    fn test_receipt_store() {
        let mut store = ReceiptStore::new(100);

        for i in 0..10 {
            let receipt = ReceiptBuilder::new(i, i as u64 * 100)
                .with_result(ReceiptStatus::Success, 4)
                .build();
            store.store(receipt);
        }

        let recent = store.get_recent(5);
        assert_eq!(recent.len(), 5);

        let task_receipts = store.get_by_task(200);
        assert!(!task_receipts.is_empty());
    }

    #[test]
    fn test_digest_computation() {
        let mut receipt = Receipt::new(4, 400);

        receipt.set_input_digest(&[1, 2, 3, 4, 5]);
        assert_ne!(receipt.input_digest, 0);

        receipt.set_output_digest(&[10, 20, 30]);
        assert_ne!(receipt.output_digest, 0);

        // Different inputs should produce different digests
        let mut receipt2 = Receipt::new(5, 500);
        receipt2.set_input_digest(&[6, 7, 8, 9, 10]);
        assert_ne!(receipt.input_digest, receipt2.input_digest);
    }

    #[test]
    fn test_budget_exceeded() {
        let receipt = ReceiptBuilder::new(6, 600)
            .with_budget(8)
            .with_result(ReceiptStatus::BudgetExceeded, 10)
            .build();

        assert!(!receipt.within_budget());
        assert_eq!(receipt.status, ReceiptStatus::BudgetExceeded);
    }
}

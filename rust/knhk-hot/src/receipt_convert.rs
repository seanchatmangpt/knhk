// knhk-hot: Receipt conversion utilities
// Convert between C Receipt and Rust Receipt structures

use crate::Receipt as CReceipt;
use crate::ffi::Receipt as HotReceipt;

/// Receipt structure for ETL (matches knhk-etl::reflex::Receipt)
/// Note: This is a local copy to avoid circular dependency
#[derive(Debug, Clone)]
pub struct EtlReceipt {
    pub id: String,
    pub cycle_id: u64,
    pub shard_id: u64,
    pub hook_id: u64,
    pub ticks: u32,
    pub lanes: u32,
    pub span_id: u64,
    pub a_hash: u64,
}

/// Convert C Receipt to Rust ETL Receipt
pub fn c_receipt_to_etl(c_receipt: &CReceipt) -> EtlReceipt {
    EtlReceipt {
        id: format!("receipt_{}", c_receipt.span_id),
        cycle_id: c_receipt.cycle_id,
        shard_id: c_receipt.shard_id,
        hook_id: c_receipt.hook_id,
        ticks: c_receipt.ticks,
        lanes: c_receipt.lanes,
        span_id: c_receipt.span_id,
        a_hash: c_receipt.a_hash,
    }
}

/// Convert Rust ETL Receipt to C Receipt
pub fn etl_receipt_to_c(etl_receipt: &EtlReceipt) -> CReceipt {
    CReceipt {
        cycle_id: etl_receipt.cycle_id,
        shard_id: etl_receipt.shard_id,
        hook_id: etl_receipt.hook_id,
        ticks: etl_receipt.ticks,
        lanes: etl_receipt.lanes,
        span_id: etl_receipt.span_id,
        a_hash: etl_receipt.a_hash,
    }
}

/// Convert Hot Receipt to Rust ETL Receipt
pub fn hot_receipt_to_etl(hot_receipt: &HotReceipt) -> EtlReceipt {
    EtlReceipt {
        id: format!("receipt_{}", hot_receipt.span_id),
        cycle_id: hot_receipt.cycle_id,
        shard_id: hot_receipt.shard_id,
        hook_id: hot_receipt.hook_id,
        ticks: hot_receipt.ticks,
        lanes: hot_receipt.lanes,
        span_id: hot_receipt.span_id,
        a_hash: hot_receipt.a_hash,
    }
}


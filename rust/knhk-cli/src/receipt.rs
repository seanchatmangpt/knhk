//! Receipt commands - Receipt operations

use clap_noun_verb::Result;
use clap_noun_verb_macros::verb;
use serde::Serialize;
use crate::commands::receipt as receipt_impl;

#[derive(Serialize, Debug)]
struct ReceiptResult {
    id: String,
    ticks: u32,
    lanes: u32,
    span_id: u64,
    a_hash: u64,
    timestamp_ms: u64,
}

/// Get receipt
#[verb] // Noun "receipt" auto-inferred from filename "receipt.rs"
fn get(id: String) -> Result<ReceiptResult> {
    let receipt = receipt_impl::get(id.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to get receipt: {}", e)))?;
    
    Ok(ReceiptResult {
        id: receipt.id,
        ticks: receipt.ticks,
        lanes: receipt.lanes,
        span_id: receipt.span_id,
        a_hash: receipt.a_hash,
        timestamp_ms: receipt.timestamp_ms,
    })
}

#[derive(Serialize, Debug)]
struct MergeResult {
    id: String,
    ticks: u32,
    lanes: u32,
    span_id: u64,
    a_hash: u64,
}

/// Merge receipts
#[verb] // Noun "receipt" auto-inferred
fn merge(ids: String) -> Result<MergeResult> {
    let receipt = receipt_impl::merge(ids.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to merge receipts: {}", e)))?;
    
    Ok(MergeResult {
        id: receipt.id,
        ticks: receipt.ticks,
        lanes: receipt.lanes,
        span_id: receipt.span_id,
        a_hash: receipt.a_hash,
    })
}

#[derive(Serialize, Debug)]
struct ReceiptList {
    receipts: Vec<String>,
}

/// List receipts
#[verb] // Noun "receipt" auto-inferred
fn list() -> Result<ReceiptList> {
    receipt_impl::list()
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to list receipts: {}", e)))
        .map(|receipts| ReceiptList { receipts })
}

#[derive(Serialize, Debug)]
struct VerifyResult {
    id: String,
    valid: bool,
}

/// Verify receipt integrity
#[verb] // Noun "receipt" auto-inferred
fn verify(id: String) -> Result<VerifyResult> {
    let valid = receipt_impl::verify(id.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to verify receipt: {}", e)))?;
    
    Ok(VerifyResult { id, valid })
}

#[derive(Serialize, Debug)]
struct ShowReceiptResult {
    id: String,
    ticks: u32,
    lanes: u32,
    span_id: u64,
    a_hash: u64,
    timestamp_ms: u64,
}

/// Show receipt details
#[verb] // Noun "receipt" auto-inferred
fn show(id: String) -> Result<ShowReceiptResult> {
    let receipt = receipt_impl::show(id.clone())
        .map_err(|e| clap_noun_verb::NounVerbError::new(&format!("Failed to show receipt: {}", e)))?;
    
    Ok(ShowReceiptResult {
        id: receipt.id,
        ticks: receipt.ticks,
        lanes: receipt.lanes,
        span_id: receipt.span_id,
        a_hash: receipt.a_hash,
        timestamp_ms: receipt.timestamp_ms,
    })
}


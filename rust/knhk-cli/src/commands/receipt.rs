// rust/knhk-cli/src/commands/receipt.rs
// Receipt commands - Receipt operations

use crate::receipt_store::ReceiptStore;

/// Get receipt
/// receipt(Id) -> Receipt
pub fn get(id: String) -> Result<crate::receipt_store::store::ReceiptEntry, String> {
    let store = ReceiptStore::new()?;
    store.get(&id)
}

/// Merge receipts
/// merge(Receipts) -> Receipt (Π ⊕)
pub fn merge(ids: String) -> Result<crate::receipt_store::store::ReceiptEntry, String> {
    let store = ReceiptStore::new()?;

    let receipt_ids: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();

    if receipt_ids.is_empty() {
        return Err("No receipt IDs provided".to_string());
    }

    store.merge(&receipt_ids)
}

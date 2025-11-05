// knhk-unrdf: Transaction management
// ACID transaction support for unrdf operations

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::template::TemplateEngine;
use crate::types::{Transaction, TransactionReceipt, TransactionState};
use std::collections::HashMap;
use tera::Context;

/// Begin a new transaction
pub fn begin_transaction(actor: &str) -> UnrdfResult<u32> {
    let state = get_state()?;
    
    let mut next_id = state.next_transaction_id.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transaction ID lock: {}", e)))?;
    
    let transaction_id = *next_id;
    *next_id += 1;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = Transaction {
        id: transaction_id,
        state: TransactionState::Pending,
        additions: Vec::new(),
        removals: Vec::new(),
        actor: actor.to_string(),
        metadata: HashMap::new(),
    };
    
    transactions.insert(transaction_id, transaction);
    
    Ok(transaction_id)
}

/// Add data to a transaction
pub fn transaction_add(transaction_id: u32, turtle_data: &str) -> UnrdfResult<()> {
    let state = get_state()?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            transaction.additions.push(turtle_data.to_string());
            Ok(())
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}

/// Remove data from a transaction
pub fn transaction_remove(transaction_id: u32, turtle_data: &str) -> UnrdfResult<()> {
    let state = get_state()?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            transaction.removals.push(turtle_data.to_string());
            Ok(())
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}

/// Commit a transaction
pub fn commit_transaction(transaction_id: u32) -> UnrdfResult<TransactionReceipt> {
    let state = get_state()?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            // Clone transaction data for script execution
            let additions = transaction.additions.clone();
            let removals = transaction.removals.clone();
            let actor = transaction.actor.clone();
            
            drop(transactions); // Release lock before async operation
            
            // Use Tera template engine
            let template_engine = TemplateEngine::get()?;
            let mut context = Context::new();
            context.insert("additions_json", &serde_json::to_string(&additions)
                .map_err(|e| UnrdfError::InvalidInput(format!("Failed to serialize additions: {}", e)))?);
            context.insert("removals_json", &serde_json::to_string(&removals)
                .map_err(|e| UnrdfError::InvalidInput(format!("Failed to serialize removals: {}", e)))?);
            context.insert("actor", &actor);
            context.insert("transaction_id", &transaction_id);
            
            let script = template_engine.lock()
                .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire template engine lock: {}", e)))?
                .render("transaction-commit", &context)
                .map_err(|e| UnrdfError::InvalidInput(format!("Failed to render transaction-commit template: {}", e)))?;
            
            let receipt_result = state.runtime.block_on(async {
                let output = execute_unrdf_script(&script).await?;
                let receipt: TransactionReceipt = serde_json::from_str(&output)
                    .map_err(|e| UnrdfError::QueryFailed(format!("Failed to parse receipt: {} - output: {}", e, output)))?;
                Ok(receipt)
            })?;
            
            // Update transaction state
            let mut transactions = state.transactions.lock()
                .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
            if let Some(txn) = transactions.get_mut(&transaction_id) {
                if receipt_result.success {
                    txn.state = TransactionState::Committed;
                }
            }
            
            Ok(receipt_result)
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}

/// Rollback a transaction
pub fn rollback_transaction(transaction_id: u32) -> UnrdfResult<()> {
    let state = get_state()?;
    
    let mut transactions = state.transactions.lock()
        .map_err(|e| UnrdfError::InvalidInput(format!("Failed to acquire transactions lock: {}", e)))?;
    
    let transaction = transactions.get_mut(&transaction_id)
        .ok_or_else(|| UnrdfError::InvalidInput(format!("Transaction {} not found", transaction_id)))?;
    
    match transaction.state {
        TransactionState::Pending => {
            transaction.state = TransactionState::RolledBack;
            Ok(())
        }
        _ => Err(UnrdfError::InvalidInput(format!("Transaction {} is not pending", transaction_id)))
    }
}


// knhk-unrdf: Transaction management
// ACID transaction support for unrdf operations

use crate::error::{UnrdfError, UnrdfResult};
use crate::script::execute_unrdf_script;
use crate::state::get_state;
use crate::types::{Transaction, TransactionReceipt, TransactionState};
use std::collections::HashMap;

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
            
            // Escape turtle data
            let escaped_additions: Vec<String> = additions.iter()
                .map(|s| s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$"))
                .collect();
            let escaped_removals: Vec<String> = removals.iter()
                .map(|s| s.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$"))
                .collect();
            
            // Build additions array
            let additions_js = escaped_additions.iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join(",\n                ");
            
            // Build removals array
            let removals_js = escaped_removals.iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join(",\n                ");
            
            let script = format!(
                r#"
                import {{ createDarkMatterCore }} from './src/knowledge-engine/knowledge-substrate-core.mjs';
                import {{ parseTurtle }} from './src/knowledge-engine/parse.mjs';
                
                async function main() {{
                    const system = await createDarkMatterCore({{
                        enableKnowledgeHookManager: true,
                        enableLockchainWriter: false
                    }});
                
                    const additionsData = [
                        {}
                    ];
                    const removalsData = [
                        {}
                    ];
                
                    const additionsQuads = [];
                    for (const turtleData of additionsData) {{
                        const store = await parseTurtle(turtleData);
                        store.forEach(q => additionsQuads.push(q));
                    }}
                
                    const removalsQuads = [];
                    for (const turtleData of removalsData) {{
                        const store = await parseTurtle(turtleData);
                        store.forEach(q => removalsQuads.push(q));
                    }}
                
                    const receipt = await system.executeTransaction({{
                        additions: additionsQuads,
                        removals: removalsQuads,
                        actor: '{}'
                    }});
                
                    console.log(JSON.stringify({{
                        transaction_id: {},
                        success: true,
                        receipt: receipt ? JSON.stringify(receipt) : null
                    }}));
                }}
                
                main().catch(err => {{
                    console.error(JSON.stringify({{
                        transaction_id: {},
                        success: false,
                        error: err.message
                    }}));
                    process.exit(1);
                }});
                "#,
                additions_js,
                removals_js,
                actor,
                transaction_id,
                transaction_id
            );
            
            drop(transactions); // Release lock before async operation
            
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


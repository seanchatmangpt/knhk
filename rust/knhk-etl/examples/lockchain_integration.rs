// rust/knhk-etl/examples/lockchain_integration.rs
// Example of lockchain integration with beat scheduler

use knhk_lockchain::{LockchainStorage, MerkleTree, PeerId, QuorumManager, Receipt};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== KNHK Lockchain Integration Example ===\n");

    // Step 1: Initialize Merkle tree
    println!("1. Initializing Merkle tree for current beat...");
    let mut merkle_tree = MerkleTree::new();

    // Step 2: Add receipts to Merkle tree (simulating beat execution)
    println!("2. Adding receipts from beat execution...");
    let receipts = vec![
        Receipt::new(100, 0, 1, 5, 0x1234567890abcdef),
        Receipt::new(100, 1, 2, 6, 0x2345678901bcdef0),
        Receipt::new(100, 2, 3, 7, 0x3456789012cdef01),
        Receipt::new(100, 3, 4, 4, 0x4567890123def012),
    ];

    for (i, receipt) in receipts.iter().enumerate() {
        let leaf_hash = merkle_tree.add_receipt(receipt);
        println!(
            "   Receipt {}: cycle={}, shard={}, ticks={}, hash={:x?}",
            i,
            receipt.cycle_id,
            receipt.shard_id,
            receipt.actual_ticks,
            &leaf_hash[..8]
        );
    }

    // Step 3: Compute Merkle root at pulse boundary (tick == 0)
    println!("\n3. Computing Merkle root at pulse boundary...");
    let root = merkle_tree.compute_root();
    println!("   Merkle root: {:x?}...{:x?}", &root[..4], &root[28..]);

    // Step 4: Achieve quorum consensus
    println!("\n4. Achieving quorum consensus...");
    let peers = vec![
        PeerId("node1".to_string()),
        PeerId("node2".to_string()),
        PeerId("node3".to_string()),
    ];
    let quorum = QuorumManager::new(peers, 3, PeerId("self".to_string()));

    let cycle = 100;
    let proof = quorum.achieve_consensus(root, cycle)?;
    println!("   Quorum achieved: {} votes collected", proof.vote_count());
    println!("   Proof verification: {}", proof.verify(3));

    // Step 5: Persist to lockchain storage
    println!("\n5. Persisting to lockchain storage...");
    let storage = LockchainStorage::new("/tmp/knhk-lockchain-example")?;
    storage.persist_root(cycle, root, proof)?;
    println!("   Persisted root for cycle {}", cycle);

    // Step 6: Query and verify
    println!("\n6. Querying and verifying stored root...");
    if let Some(entry) = storage.get_root(cycle)? {
        println!("   Retrieved cycle: {}", entry.cycle);
        println!(
            "   Retrieved root: {:x?}...{:x?}",
            &entry.root[..4],
            &entry.root[28..]
        );
        println!("   Proof votes: {}", entry.proof.vote_count());
        println!("   Verified: {}", entry.proof.verify(3));
    }

    // Step 7: Generate and verify Merkle proof for audit
    println!("\n7. Generating Merkle proof for receipt audit...");
    if let Ok(proof) = merkle_tree.generate_proof(0) {
        println!("   Proof for receipt 0:");
        println!(
            "   - Leaf hash: {:x?}...{:x?}",
            &proof.leaf_hash[..4],
            &proof.leaf_hash[28..]
        );
        println!("   - Proof path length: {}", proof.proof_hashes.len());
        println!("   - Verification: {}", proof.verify());
    }

    println!("\n=== Lockchain Integration Complete ===");
    println!("\nKey Properties:");
    println!("- Receipts hashed into Merkle tree per beat");
    println!("- Root computed at pulse (tick == 0)");
    println!("- Quorum consensus ensures Byzantine fault tolerance");
    println!("- Audit trail queryable from persistent storage");
    println!("- Merkle proofs enable individual receipt verification");

    Ok(())
}

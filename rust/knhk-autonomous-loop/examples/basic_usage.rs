//! Basic usage example for the Autonomous Evolution Loop
//!
//! This example demonstrates how to set up and run the autonomous loop
//! with mock dependencies for demonstration purposes.

use knhk_autonomous_loop::*;
use std::time::Duration;
use tokio::time::sleep;

// Mock implementations (in production, use real KNHK components)
mod mocks {
    use async_trait::async_trait;
    use knhk_autonomous_loop::dependencies::*;
    use knhk_autonomous_loop::*;
    use std::sync::Arc;

    pub struct SimpleSnapshotStore;

    #[async_trait]
    impl SnapshotStore for SimpleSnapshotStore {
        async fn current_snapshot(&self) -> Result<SigmaSnapshot> {
            Ok(SigmaSnapshot {
                id: [1u8; 32],
                version: 1,
                validation_receipt: None,
            })
        }

        fn get_snapshot(&self, id: &SigmaSnapshotId) -> Result<SigmaSnapshot> {
            Ok(SigmaSnapshot {
                id: *id,
                version: 1,
                validation_receipt: None,
            })
        }

        fn create_overlay(&self, name: String) -> SnapshotOverlay {
            SnapshotOverlay {
                base_snapshot_id: [1u8; 32],
                name,
                changes: vec![],
            }
        }

        async fn commit_overlay(
            &self,
            _overlay: SnapshotOverlay,
            _receipt: ValidationReceipt,
        ) -> Result<SigmaSnapshotId> {
            Ok([2u8; 32])
        }
    }

    pub struct SimplePatternMiner;

    impl PatternMiner for SimplePatternMiner {
        fn scan(&self, observations: &[ObservationReceipt]) -> Result<DetectedPatterns> {
            println!("📊 Scanning {} observations for patterns", observations.len());

            Ok(DetectedPatterns {
                patterns: vec![
                    Pattern {
                        pattern_type: "frequent_operation".to_string(),
                        confidence: 0.95,
                        occurrences: 42,
                    },
                    Pattern {
                        pattern_type: "new_entity_type".to_string(),
                        confidence: 0.87,
                        occurrences: 15,
                    },
                ],
            })
        }
    }

    pub struct SimpleProposer;

    #[async_trait]
    impl DeltaSigmaProposer for SimpleProposer {
        async fn propose(
            &self,
            patterns: &DetectedPatterns,
        ) -> Result<Vec<ChangeProposal>> {
            println!("💡 Generating proposals from {} patterns", patterns.total_count());

            Ok(vec![ChangeProposal {
                id: "proposal-1".to_string(),
                change_type: "add_entity_class".to_string(),
                description: "Add new entity class based on observed patterns".to_string(),
            }])
        }
    }

    pub struct SimpleValidator;

    #[async_trait]
    impl DeltaSigmaValidator for SimpleValidator {
        async fn validate(
            &self,
            proposal: &ChangeProposal,
        ) -> Result<ValidationReceipt> {
            println!("✅ Validating proposal: {}", proposal.id);

            Ok(ValidationReceipt {
                proposal_id: proposal.id.clone(),
                invariants_q_preserved: true,
                production_ready: true,
                validation_results: ValidationResults {
                    invariants_q_preserved: true,
                    tests_passed: 10,
                    tests_failed: 0,
                },
            })
        }
    }

    pub struct SimplePromotionPipeline;

    #[async_trait]
    impl PromotionPipeline for SimplePromotionPipeline {
        async fn promote_snapshot(&self, snapshot_id: SigmaSnapshotId) -> Result<()> {
            println!("🚀 Promoting snapshot: {}", hex::encode(&snapshot_id));
            Ok(())
        }

        async fn rollback(
            &self,
            from: SigmaSnapshotId,
            to: SigmaSnapshotId,
        ) -> Result<()> {
            println!(
                "⏪ Rolling back from {} to {}",
                hex::encode(&from),
                hex::encode(&to)
            );
            Ok(())
        }
    }

    pub struct SimpleChangeExecutor;

    #[async_trait]
    impl ChangeExecutor for SimpleChangeExecutor {
        async fn apply_proposal_to_overlay(
            &self,
            overlay: &mut SnapshotOverlay,
            proposal: &ChangeProposal,
        ) -> Result<()> {
            println!("🔧 Applying proposal to overlay: {}", proposal.id);
            overlay.changes.push(proposal.clone());
            Ok(())
        }
    }

    pub struct SimpleReceiptLog;

    #[async_trait]
    impl ReceiptLog for SimpleReceiptLog {
        async fn recent_receipts(&self, limit: usize) -> Result<Vec<ObservationReceipt>> {
            println!("📋 Fetching {} recent receipts", limit);

            Ok((0..100.min(limit))
                .map(|i| ObservationReceipt {
                    id: format!("receipt-{}", i),
                    operation: "create_entity".to_string(),
                    attributes: vec![("entity_type".to_string(), "Product".to_string())],
                })
                .collect())
        }
    }

    pub fn create_dependencies() -> LoopDependencies {
        LoopDependencies::new(
            Arc::new(SimpleSnapshotStore),
            Arc::new(SimplePatternMiner),
            Arc::new(SimpleProposer),
            Arc::new(SimpleValidator),
            Arc::new(SimplePromotionPipeline),
            Arc::new(SimpleChangeExecutor),
            Arc::new(SimpleReceiptLog),
        )
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("knhk_autonomous_loop=info")
        .init();

    println!("🌱 Starting Autonomous Evolution Loop Demo\n");

    // Configure the loop
    let config = AutonomousLoopConfig::new()
        .with_cycle_interval(Duration::from_secs(5))
        .with_min_patterns(2)
        .with_auto_promote(true)
        .with_error_threshold(10.0);

    println!("⚙️  Configuration:");
    println!("   - Cycle interval: {:?}", config.cycle_interval);
    println!("   - Min patterns: {}", config.min_patterns_for_proposal);
    println!("   - Auto-promote: {}", config.auto_promote);
    println!();

    // Create dependencies
    let dependencies = mocks::create_dependencies();

    // Start the loop
    let handle = start_autonomous_loop(config, dependencies)?;

    println!("🚀 Loop started!\n");

    // Monitor the loop for 30 seconds
    for i in 1..=6 {
        sleep(Duration::from_secs(5)).await;

        let stats = handle.engine().get_stats().await;
        let health = handle.engine().get_health().await;

        println!("📊 Status Report #{}", i);
        println!("   - Health: {}", health.status());
        println!("   - Total cycles: {}", stats.total_cycles);
        println!("   - Successful: {}", stats.successful_cycles);
        println!("   - Failed: {}", stats.failed_cycles);
        println!("   - Success rate: {:.1}%", stats.success_rate());
        println!("   - Error rate: {:.1}%", stats.error_rate);
        println!("   - Avg duration: {}ms", stats.avg_cycle_duration_ms);
        println!();
    }

    // Get cycle history
    let history = handle.engine().get_history().await;
    println!("📜 Cycle History ({} cycles):", history.len());
    for cycle in history.iter().take(5) {
        println!(
            "   - Cycle {}: {:?} ({} steps)",
            cycle.cycle_id,
            cycle.result,
            cycle.steps.len()
        );
    }
    println!();

    // Stop the loop
    println!("🛑 Stopping loop...");
    handle.stop().await?;

    println!("✅ Loop stopped successfully!");

    Ok(())
}

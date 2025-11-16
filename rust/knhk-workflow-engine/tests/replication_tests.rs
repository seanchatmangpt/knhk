//! State machine replication tests

use knhk_workflow_engine::consensus::*;
use knhk_workflow_engine::case::{CaseId, CaseState};
use serde_json::json;

#[tokio::test]
async fn test_state_machine_creation() {
    let sm = ReplicatedStateMachine::new();

    assert_eq!(sm.last_applied().await.inner(), 0);

    let state = sm.get_state().await;
    assert_eq!(state.cases.len(), 0);
    assert_eq!(state.policies.len(), 0);
    assert_eq!(state.overlays.len(), 0);
}

#[tokio::test]
async fn test_state_machine_create_case() {
    let sm = ReplicatedStateMachine::new();

    let op = StateMachineOp::CreateCase {
        case_id: CaseId::from("case-1".to_string()),
        spec_id: "spec-1".to_string(),
        data: json!({"key": "value"}),
    };

    let data = bincode::serialize(&op).unwrap();

    sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();

    let state = sm.get_state().await;
    assert_eq!(state.cases.len(), 1);

    let case = sm.get_case(&CaseId::from("case-1".to_string())).await.unwrap();
    assert_eq!(case.spec_id, "spec-1");
    assert_eq!(case.state, CaseState::Created);
}

#[tokio::test]
async fn test_state_machine_update_case_state() {
    let sm = ReplicatedStateMachine::new();

    // Create case first
    let create_op = StateMachineOp::CreateCase {
        case_id: CaseId::from("case-1".to_string()),
        spec_id: "spec-1".to_string(),
        data: json!({}),
    };

    sm.apply(LogIndex::new(1), Term::new(1), &bincode::serialize(&create_op).unwrap())
        .await
        .unwrap();

    // Update case state
    let update_op = StateMachineOp::UpdateCaseState {
        case_id: CaseId::from("case-1".to_string()),
        new_state: CaseState::Running,
    };

    sm.apply(LogIndex::new(2), Term::new(1), &bincode::serialize(&update_op).unwrap())
        .await
        .unwrap();

    let case = sm.get_case(&CaseId::from("case-1".to_string())).await.unwrap();
    assert_eq!(case.state, CaseState::Running);
}

#[tokio::test]
async fn test_state_machine_apply_policy() {
    let sm = ReplicatedStateMachine::new();

    let op = StateMachineOp::ApplyPolicy {
        policy_id: "policy-1".to_string(),
        policy_data: b"policy data".to_vec(),
    };

    let data = bincode::serialize(&op).unwrap();

    sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();

    let state = sm.get_state().await;
    assert_eq!(state.policies.len(), 1);
    assert!(state.policies.contains_key("policy-1"));
    assert_eq!(state.policies.get("policy-1").unwrap(), &b"policy data".to_vec());
}

#[tokio::test]
async fn test_state_machine_deploy_overlay() {
    let sm = ReplicatedStateMachine::new();

    let op = StateMachineOp::DeployOverlay {
        overlay_id: "overlay-1".to_string(),
        overlay_data: b"overlay data".to_vec(),
    };

    let data = bincode::serialize(&op).unwrap();

    sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();

    let state = sm.get_state().await;
    assert_eq!(state.overlays.len(), 1);
    assert!(state.overlays.contains_key("overlay-1"));
}

#[tokio::test]
async fn test_state_machine_custom_operation() {
    let sm = ReplicatedStateMachine::new();

    let op = StateMachineOp::Custom {
        operation: "custom-op".to_string(),
        data: b"custom data".to_vec(),
    };

    let data = bincode::serialize(&op).unwrap();

    sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();

    let state = sm.get_state().await;
    assert_eq!(state.custom.len(), 1);
    assert!(state.custom.contains_key("custom-op"));
}

#[tokio::test]
async fn test_state_machine_idempotent_apply() {
    let sm = ReplicatedStateMachine::new();

    let op = StateMachineOp::CreateCase {
        case_id: CaseId::from("case-1".to_string()),
        spec_id: "spec-1".to_string(),
        data: json!({}),
    };

    let data = bincode::serialize(&op).unwrap();

    // Apply same operation twice with same index
    sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();
    sm.apply(LogIndex::new(1), Term::new(1), &data).await.unwrap();

    let state = sm.get_state().await;
    assert_eq!(state.cases.len(), 1); // Should only have one case
}

#[tokio::test]
async fn test_state_machine_snapshot() {
    let sm = ReplicatedStateMachine::new();

    // Apply multiple operations
    let ops = vec![
        StateMachineOp::CreateCase {
            case_id: CaseId::from("case-1".to_string()),
            spec_id: "spec-1".to_string(),
            data: json!({}),
        },
        StateMachineOp::CreateCase {
            case_id: CaseId::from("case-2".to_string()),
            spec_id: "spec-1".to_string(),
            data: json!({}),
        },
        StateMachineOp::ApplyPolicy {
            policy_id: "policy-1".to_string(),
            policy_data: b"policy".to_vec(),
        },
    ];

    for (i, op) in ops.iter().enumerate() {
        let data = bincode::serialize(op).unwrap();
        sm.apply(LogIndex::new(i as u64 + 1), Term::new(1), &data)
            .await
            .unwrap();
    }

    // Create snapshot
    let snapshot = sm.create_snapshot(LogIndex::new(3), Term::new(1)).await.unwrap();
    assert_eq!(snapshot.last_included_index.inner(), 3);
    assert_eq!(snapshot.last_included_term.inner(), 1);

    // Restore snapshot in new state machine
    let sm2 = ReplicatedStateMachine::new();
    sm2.restore_snapshot(&snapshot).await.unwrap();

    let state1 = sm.get_state().await;
    let state2 = sm2.get_state().await;

    assert_eq!(state1.cases.len(), state2.cases.len());
    assert_eq!(state1.policies.len(), state2.policies.len());
    assert_eq!(sm2.last_applied().await.inner(), 3);
}

#[tokio::test]
async fn test_state_machine_sequential_operations() {
    let sm = ReplicatedStateMachine::new();

    // Create case
    let create_op = StateMachineOp::CreateCase {
        case_id: CaseId::from("case-1".to_string()),
        spec_id: "spec-1".to_string(),
        data: json!({"status": "new"}),
    };

    sm.apply(LogIndex::new(1), Term::new(1), &bincode::serialize(&create_op).unwrap())
        .await
        .unwrap();

    // Update case state
    let update_op = StateMachineOp::UpdateCaseState {
        case_id: CaseId::from("case-1".to_string()),
        new_state: CaseState::Running,
    };

    sm.apply(LogIndex::new(2), Term::new(1), &bincode::serialize(&update_op).unwrap())
        .await
        .unwrap();

    // Apply policy
    let policy_op = StateMachineOp::ApplyPolicy {
        policy_id: "policy-1".to_string(),
        policy_data: b"policy".to_vec(),
    };

    sm.apply(LogIndex::new(3), Term::new(1), &bincode::serialize(&policy_op).unwrap())
        .await
        .unwrap();

    let state = sm.get_state().await;
    assert_eq!(state.cases.len(), 1);
    assert_eq!(state.policies.len(), 1);
    assert_eq!(sm.last_applied().await.inner(), 3);

    let case = sm.get_case(&CaseId::from("case-1".to_string())).await.unwrap();
    assert_eq!(case.state, CaseState::Running);
}

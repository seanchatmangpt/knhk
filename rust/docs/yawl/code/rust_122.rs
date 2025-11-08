use knhk_workflow_engine::testing::chicago_tdd::TaskBuilder;
use knhk_workflow_engine::parser::{TaskType, SplitType, JoinType};

let task = TaskBuilder::new("task:approve", "Approve Order")
    .with_type(TaskType::Atomic)
    .with_split_type(SplitType::And)
    .with_join_type(JoinType::And)
    .with_max_ticks(8)
    .add_outgoing_flow("task:notify")
    .build();
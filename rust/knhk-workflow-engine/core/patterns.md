# Workflow Patterns

The KNHK Workflow Engine supports all 43 Van der Aalst workflow patterns.

## Pattern Categories

### Basic Control Flow (1-5)

- **Sequence**: Tasks execute in order
- **Parallel Split**: Tasks execute concurrently
- **Synchronization**: Wait for all parallel tasks
- **Exclusive Choice**: Choose one path
- **Simple Merge**: Merge multiple paths

### Advanced Branching (6-11)

- **Multi-Choice**: Choose multiple paths
- **Structured Synchronizing Merge**: Synchronize multiple paths
- **Multi-Merge**: Merge without synchronization
- **Discriminator**: First completed path wins
- **Arbitrary Cycles**: Loops in workflows
- **Implicit Termination**: Automatic termination

### Multiple Instance (12-15)

- **MI Without Sync**: Multiple instances, no synchronization
- **MI With Design-Time Knowledge**: Known instance count
- **MI With Runtime Knowledge**: Runtime-determined count
- **MI Without Runtime Knowledge**: Dynamic instance creation

### State-Based (16-18)

- **Deferred Choice**: Runtime choice
- **Interleaved Parallel Routing**: Interleaved execution
- **Milestone**: State-based milestone

### Cancellation (19-25)

- **Cancel Activity**: Cancel specific activity
- **Cancel Case**: Cancel entire case
- **Cancel Region**: Cancel region of workflow
- **Cancel MI Activity**: Cancel multiple instances
- **Complete MI Activity**: Complete multiple instances
- **Blocking Discriminator**: Block until first completion
- **Cancelling Discriminator**: Cancel others on first completion

### Advanced Patterns (26-39)

Advanced workflow patterns for complex scenarios.

### Trigger Patterns (40-43)

Event-driven workflow patterns.

## Pattern Usage

```rust
use knhk_workflow_engine::patterns::PatternRegistry;

let registry = PatternRegistry::new();
let pattern_id = PatternId::Sequence;
let result = registry.execute_pattern(pattern_id, context).await?;
```

## Next Steps

- [YAWL Support](yawl.md) - YAWL compatibility
- [Workflow Execution](execution.md) - Execution details


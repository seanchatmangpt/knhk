# YAWL Java to Rust Module Mapping

## Package Structure Mapping

### Core Engine (`org.yawlfoundation.yawl.engine`)

| Java Class | Rust Module | Status | TRIZ Patterns |
|------------|------------|--------|---------------|
| `YEngine.java` | `rust/knhk-workflow-engine/src/engine/y_engine.rs` | ✅ Partial | Principle 28 (Mechanics Substitution), Principle 13 (Inversion) |
| `YNetRunner.java` | `rust/knhk-workflow-engine/src/engine/net_runner.rs` | ✅ Partial | Principle 24 (Intermediary) |
| `YWorkItem.java` | `rust/knhk-workflow-engine/src/engine/y_work_item.rs` | ✅ Partial | Principle 32 (Color Changes) |
| `YCaseNbrStore.java` | `rust/knhk-workflow-engine/src/engine/case_store.rs` | ❌ Missing | - |

### Elements (`org.yawlfoundation.yawl.elements`)

| Java Class | Rust Module | Status | TRIZ Patterns |
|------------|------------|--------|---------------|
| `YNet.java` | `rust/knhk-workflow-engine/src/elements/net.rs` | ⚠️ Needs creation | Principle 40 (Composite Materials) |
| `YTask.java` | `rust/knhk-workflow-engine/src/elements/task.rs` | ⚠️ Needs creation | Principle 32 (Color Changes) |
| `YCondition.java` | `rust/knhk-workflow-engine/src/elements/condition.rs` | ⚠️ Needs creation | - |
| `YFlow.java` | `rust/knhk-workflow-engine/src/elements/flow.rs` | ⚠️ Needs creation | - |

### Interfaces (`org.yawlfoundation.yawl.engine.interfce`)

| Java Interface | Rust Module | Status | TRIZ Patterns |
|----------------|------------|--------|---------------|
| `InterfaceA` | `rust/knhk-workflow-engine/src/api/interface_a.rs` | ❌ Missing | - |
| `InterfaceB` | `rust/knhk-workflow-engine/src/api/interface_b.rs` | ✅ Partial (20%) | Principle 13 (Inversion), Principle 32 (Color Changes) |
| `InterfaceE` | `rust/knhk-workflow-engine/src/api/interface_e.rs` | ❌ Missing | - |
| `InterfaceX` | `rust/knhk-workflow-engine/src/api/interface_x.rs` | ❌ Missing | Principle 37 (Thermal Expansion) |
| `InterfaceS` | `rust/knhk-workflow-engine/src/api/interface_s.rs` | ❌ Missing | Principle 19 (Periodic Action) |

### Resourcing (`org.yawlfoundation.yawl.resourcing`)

| Java Component | Rust Module | Status | TRIZ Patterns |
|----------------|------------|--------|---------------|
| Resource Allocation | `rust/knhk-workflow-engine/src/resourcing/allocation.rs` | ❌ Missing | Principle 40 (Composite Materials), Principle 35 (Parameter Changes) |
| 3-Phase Allocation | `rust/knhk-workflow-engine/src/resourcing/three_phase.rs` | ❌ Missing | Principle 40 (Composite Materials) |
| Filters | `rust/knhk-workflow-engine/src/resourcing/filters.rs` | ❌ Missing | Principle 40 (Composite Materials) |
| Constraints | `rust/knhk-workflow-engine/src/resourcing/constraints.rs` | ❌ Missing | Principle 35 (Parameter Changes) |

### Worklets (`org.yawlfoundation.yawl.worklet`)

| Java Component | Rust Module | Status | TRIZ Patterns |
|----------------|------------|--------|---------------|
| Worklet Repository | `rust/knhk-workflow-engine/src/worklets/repository.rs` | ✅ Partial | Principle 19 (Periodic Action) |
| RDR Selection | `rust/knhk-workflow-engine/src/worklets/rdr.rs` | ✅ Partial | Principle 24 (Intermediary) |
| Worklet Execution | `rust/knhk-workflow-engine/src/worklets/execution.rs` | ⚠️ Needs fix | Principle 24 (Intermediary) |

## Implementation Priority

1. **Core Engine** (Week 1-2) - YEngine, YNetRunner, YWorkItem enhancements
2. **Interface B** (Week 2-3) - Complete 50+ operations
3. **Resource Management** (Week 3-4) - 3-phase allocation, filters, constraints
4. **Worklet System** (Week 4-5) - RDR, execution fixes
5. **Advanced Features** (Week 5-6) - Scheduling, XQuery, Interface X


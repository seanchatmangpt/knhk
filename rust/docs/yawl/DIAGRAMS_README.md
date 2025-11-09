# C4 Architecture Diagrams

**Last Updated**: January 2025  
**Total Diagrams**: 9 PlantUML C4 diagrams

## Overview

This directory contains C4 architecture diagrams documenting the KNHK Workflow Engine architecture. C4 diagrams use a hierarchical approach to describe software architecture at different levels of abstraction.

## C4 Model Levels

The C4 model consists of four levels:

- **C1 - Context**: High-level system context showing external actors and systems
- **C2 - Container**: Application containers and their interactions
- **C3 - Component**: Components within containers and their relationships
- **C4 - Code**: Class-level structure and implementation details

## Diagrams

### C1: Context Diagram

**File**: `C1_Context.puml`

**Purpose**: Shows the KNHK Workflow Engine in its environment, identifying external actors and systems.

**Key Elements**:
- Human actors (users creating cases, submitting work items)
- Legacy YAWL/BPMN systems (external workflow engines)
- External event sources (APIs, Kafka, Webhooks)
- Timebase (NTP/Monotonic Clock)
- State Store (Sled-based persistence)

**When to Use**: 
- System overview presentations
- Understanding system boundaries
- Identifying external dependencies
- Onboarding new team members

**Rendering**: 
```bash
plantuml C1_Context.puml
```

---

### C2: Container Diagram

**File**: `C2_Container.puml`

**Purpose**: Shows the high-level technical building blocks (containers) of the workflow engine.

**Key Elements**:
- API Gateway (REST/gRPC servers)
- Execution Layer (Execution Engine, Pipeline, Work Queue)
- State Layer (State Manager, Sled database)
- Pattern Registry (43 pattern dispatcher)
- Resource Layer (Allocator, Pool Manager)
- Resilience Layer (Circuit Breaker, Retry, Rate Limiter)
- Observability (OTEL) and Provenance (Lockchain)

**When to Use**:
- Architecture reviews
- Deployment planning
- Understanding system components
- Integration discussions

**Rendering**: 
```bash
plantuml C2_Container.puml
```

---

### C3: Component Diagrams

Component-level diagrams show the internal structure of containers. We have four C3 diagrams covering different aspects:

#### C3: Basic and Advanced Control Flow

**File**: `C3_Components_Basic_Advanced.puml`

**Purpose**: Shows pattern components for basic control flow (patterns 1-5) and advanced branching (patterns 6-11).

**Key Elements**:
- Patterns 1-5: Sequence, Parallel Split, Synchronization, Exclusive Choice, Simple Merge
- Patterns 6-11: Multi-Choice, Structured Synchronizing Merge, Multi-Merge, Discriminator, Arbitrary Cycles, Implicit Termination
- Pattern Registry dispatch mechanism

**When to Use**:
- Understanding basic workflow patterns
- Pattern implementation discussions
- Pattern selection guidance

---

#### C3: Multiple Instance and Cancellation

**File**: `C3_Components_MI_Cancellation.puml`

**Purpose**: Shows pattern components for multiple instance patterns (12-15) and cancellation patterns (19-25, 33-37).

**Key Elements**:
- Patterns 12-15: Multiple Instance variants (with/without sync, design-time/runtime knowledge)
- Patterns 19-25: Cancellation patterns (Activity, Case, Region, MI variants)
- Patterns 33-37: Partial and generalized joins
- Resource Allocator integration

**When to Use**:
- Multiple instance workflow design
- Cancellation pattern implementation
- Resource allocation discussions

---

#### C3: State-Based, Triggers, and Human Interaction

**File**: `C3_Components_State_Triggers_Humans.puml`

**Purpose**: Shows pattern components for state-based patterns (16-18), triggers (30-31, 40-43), and advanced control (26-29).

**Key Elements**:
- Patterns 16-18: Deferred Choice, Interleaved Parallel Routing, Milestone
- Patterns 26-29: Discriminators, Structured Loop, Recursion
- Patterns 30-31: Transient and Persistent Triggers
- Patterns 40-43: Termination variants
- Integration with Worklet Repository and Observability

**When to Use**:
- Event-driven workflow design
- Human task integration
- Timer and trigger pattern implementation
- Termination handling discussions

---

#### C3: Legacy Mode Interface

**File**: `C3_Legacy_Mode_Interface.puml`

**Purpose**: Shows how the workflow engine provides YAWL-compatible interfaces while maintaining internal architecture.

**Key Elements**:
- REST/gRPC API for YAWL operations
- Workflow Parser (YAWL/Turtle parsing)
- Pattern Registry (all 43 patterns)
- Workflow Validator
- Legacy YAWL tooling integration

**When to Use**:
- YAWL migration discussions
- Legacy system integration
- API design reviews
- Compatibility planning

---

### C4: Code Level Diagram

**File**: `C4_Code_Level.puml`

**Purpose**: Shows the class-level structure of pattern executors and core engine components.

**Key Elements**:
- PatternRegistry class structure
- PatternExecutor trait and implementations
- All 43 pattern executor classes (P01-P43)
- WorkflowEngine, ExecutionEngine, StateManager classes
- ExecutionPipeline, WorkQueue, ResourcePoolManager
- CircuitBreaker implementation

**When to Use**:
- Code-level architecture discussions
- Implementation planning
- Class relationship understanding
- Developer onboarding

**Note**: This diagram shows the complete class structure for all 43 patterns and core engine components.

---

### Reflex Enterprise Container View

**File**: `Reflex_Enterprise_Container_View.puml`

**Purpose**: Shows how the Workflow Engine integrates into the Reflex Enterprise architecture (R1/W1/C1 tiers).

**Key Elements**:
- R1 Hot Path (Beat Scheduler, Fiber Executor, Ring Buffers, Guard Layer H₁)
- W1 Warm Path (ETL, Guard Layer H₂, μ_spawn calls)
- C1 Cold Path (Analytics, Guard Layer H₃, Subtask Router)
- Control Plane (Beat Orchestrator, Provenance Validator, Metrics Engine)
- Workflow Engine integration points

**When to Use**:
- Enterprise architecture discussions
- Reflex integration planning
- Performance optimization discussions
- Tier allocation decisions

---

## Rendering Diagrams

### Prerequisites

1. **Java** (required for PlantUML)
2. **PlantUML** (diagram rendering tool)

### Installation

**macOS**:
```bash
brew install plantuml
```

**Linux**:
```bash
sudo apt-get install plantuml  # Debian/Ubuntu
```

**Windows**:
Download from [PlantUML website](https://plantuml.com/download)

### Rendering Methods

#### 1. Command Line (PNG)
```bash
cd diagrams/
plantuml *.puml
```

#### 2. Command Line (SVG)
```bash
plantuml -tsvg *.puml
```

#### 3. VS Code Extension
1. Install "PlantUML" extension
2. Open any `.puml` file
3. Press `Alt+D` (Windows/Linux) or `Option+D` (macOS) to preview

#### 4. Online Editor
1. Visit [PlantUML Web Server](http://www.plantuml.com/plantuml/uml/)
2. Copy diagram content
3. Paste into editor
4. View rendered diagram

### C4-PlantUML Library

All diagrams use the C4-PlantUML library which is included via:
```plantuml
!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml
```

This library provides:
- Standard C4 diagram elements (Person, System, Container, Component)
- Consistent styling
- Relationship types (Rel, Rel_Back, etc.)

**Documentation**: [C4-PlantUML GitHub](https://github.com/plantuml-stdlib/C4-PlantUML)

## Diagram Usage Guidelines

### When to Use Each Level

1. **C1 Context**: 
   - Stakeholder presentations
   - System overview documentation
   - External integration discussions

2. **C2 Container**:
   - Architecture reviews
   - Deployment planning
   - Technology stack discussions

3. **C3 Component**:
   - Detailed design discussions
   - Pattern implementation planning
   - Component interaction analysis

4. **C4 Code**:
   - Code review preparation
   - Implementation planning
   - Developer documentation

### Best Practices

- **Start with C1**: Always begin architecture discussions with context
- **Progressive Detail**: Move from C1 → C2 → C3 → C4 as needed
- **Keep Updated**: Update diagrams when architecture changes
- **Version Control**: Commit diagram changes with code changes
- **Consistent Naming**: Use consistent naming across diagrams

## Diagram Maintenance

### Updating Diagrams

1. Edit the `.puml` file
2. Render to verify changes
3. Commit with descriptive message
4. Update this README if diagram purpose changes

### Adding New Diagrams

1. Create new `.puml` file in `diagrams/` directory
2. Follow C4 naming conventions
3. Include C4-PlantUML library
4. Add entry to this README
5. Update main [README.md](README.md)

## Related Documentation

- [README.md](README.md) - Main YAWL artifacts documentation
- [CODE_INDEX.md](CODE_INDEX.md) - Code file index
- [ARCHITECTURE_IMPROVEMENTS.md](ARCHITECTURE_IMPROVEMENTS.md) - Architecture documentation
- [Architecture Guide](../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Architecture Reference](../../docs/architecture.md) - Detailed architecture reference

## References

- [C4 Model](https://c4model.com/) - C4 model methodology
- [C4-PlantUML](https://github.com/plantuml-stdlib/C4-PlantUML) - PlantUML C4 library
- [PlantUML](https://plantuml.com/) - PlantUML documentation


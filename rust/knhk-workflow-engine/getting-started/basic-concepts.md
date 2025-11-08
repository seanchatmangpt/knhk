# Basic Concepts

Core concepts of the KNHK Workflow Engine.

## Workflow Specification

A workflow specification defines the structure of a workflow:

- **Tasks**: Individual units of work
- **Flows**: Connections between tasks
- **Conditions**: Decision points
- **Resources**: Resource allocation

## Workflow Case

A case is an instance of a workflow execution:

- **Case ID**: Unique identifier
- **State**: Current execution state
- **Data**: Case-specific data

## Task Execution

Tasks are executed according to workflow patterns:

- **Atomic Tasks**: Single unit of work
- **Composite Tasks**: Contain sub-workflows
- **Multiple Instance Tasks**: Execute multiple times

## State Management

Workflow state is persisted:

- **State Store**: Sled-based persistence
- **Case State**: Tracks execution progress
- **Task State**: Tracks individual task execution

## Resource Allocation

Resources are allocated to tasks:

- **Resource Pool**: Available resources
- **Allocation Policies**: How resources are assigned
- **Capabilities**: Required capabilities for tasks

## Next Steps

- [Workflow Patterns](core/patterns.md) - Learn about patterns
- [Workflow Execution](core/execution.md) - Understand execution


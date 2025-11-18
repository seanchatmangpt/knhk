/**
 * Sample YAWL Workflows for Testing
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 1: Turtle RDF is the source of truth
 * - Covenant 2: Workflows must satisfy Q invariants
 *
 * These fixtures provide test data for validating:
 * - RDF round-trip serialization
 * - Pattern validation
 * - MAPE-K feedback loops
 * - knhk kernel integration
 */

import type { WorkflowNode, WorkflowEdge } from '@/lib/types';

/**
 * Simple linear workflow: Start -> Task1 -> Task2 -> End
 * Validates: Basic sequence pattern
 */
export const simpleSequenceWorkflow = {
  nodes: [
    {
      id: 'start-1',
      type: 'condition',
      position: { x: 100, y: 100 },
      data: {
        label: 'Start',
        conditionType: 'input',
        splitType: 'none',
        joinType: 'none',
      },
    },
    {
      id: 'task-1',
      type: 'task',
      position: { x: 250, y: 100 },
      data: {
        label: 'Process Order',
        decomposition: 'atomic',
        resources: [],
        properties: {},
      },
    },
    {
      id: 'task-2',
      type: 'task',
      position: { x: 400, y: 100 },
      data: {
        label: 'Ship Order',
        decomposition: 'atomic',
        resources: [],
        properties: {},
      },
    },
    {
      id: 'end-1',
      type: 'condition',
      position: { x: 550, y: 100 },
      data: {
        label: 'End',
        conditionType: 'output',
        splitType: 'none',
        joinType: 'none',
      },
    },
  ] as WorkflowNode[],
  edges: [
    { id: 'e1', source: 'start-1', target: 'task-1' },
    { id: 'e2', source: 'task-1', target: 'task-2' },
    { id: 'e3', source: 'task-2', target: 'end-1' },
  ] as WorkflowEdge[],
  metadata: {
    name: 'Simple Sequence',
    description: 'Basic sequential workflow for testing',
    version: '1.0.0',
  },
};

/**
 * AND-Split/AND-Join workflow
 * Validates: Parallel execution pattern (Pattern 2)
 */
export const parallelWorkflow = {
  nodes: [
    {
      id: 'start-1',
      type: 'condition',
      position: { x: 100, y: 200 },
      data: {
        label: 'Start',
        conditionType: 'input',
        splitType: 'none',
        joinType: 'none',
      },
    },
    {
      id: 'split-1',
      type: 'condition',
      position: { x: 250, y: 200 },
      data: {
        label: 'AND Split',
        conditionType: 'intermediate',
        splitType: 'and',
        joinType: 'none',
      },
    },
    {
      id: 'task-1',
      type: 'task',
      position: { x: 400, y: 100 },
      data: {
        label: 'Check Inventory',
        decomposition: 'atomic',
        resources: [],
        properties: {},
      },
    },
    {
      id: 'task-2',
      type: 'task',
      position: { x: 400, y: 300 },
      data: {
        label: 'Verify Payment',
        decomposition: 'atomic',
        resources: [],
        properties: {},
      },
    },
    {
      id: 'join-1',
      type: 'condition',
      position: { x: 550, y: 200 },
      data: {
        label: 'AND Join',
        conditionType: 'intermediate',
        splitType: 'none',
        joinType: 'and',
      },
    },
    {
      id: 'end-1',
      type: 'condition',
      position: { x: 700, y: 200 },
      data: {
        label: 'End',
        conditionType: 'output',
        splitType: 'none',
        joinType: 'none',
      },
    },
  ] as WorkflowNode[],
  edges: [
    { id: 'e1', source: 'start-1', target: 'split-1' },
    { id: 'e2', source: 'split-1', target: 'task-1' },
    { id: 'e3', source: 'split-1', target: 'task-2' },
    { id: 'e4', source: 'task-1', target: 'join-1' },
    { id: 'e5', source: 'task-2', target: 'join-1' },
    { id: 'e6', source: 'join-1', target: 'end-1' },
  ] as WorkflowEdge[],
  metadata: {
    name: 'Parallel Execution',
    description: 'AND-split/AND-join pattern',
    version: '1.0.0',
  },
};

/**
 * XOR-Split/XOR-Join workflow (Exclusive Choice)
 * Validates: Conditional branching pattern (Pattern 4)
 */
export const exclusiveChoiceWorkflow = {
  nodes: [
    {
      id: 'start-1',
      type: 'condition',
      position: { x: 100, y: 200 },
      data: {
        label: 'Start',
        conditionType: 'input',
        splitType: 'none',
        joinType: 'none',
      },
    },
    {
      id: 'split-1',
      type: 'condition',
      position: { x: 250, y: 200 },
      data: {
        label: 'XOR Split',
        conditionType: 'intermediate',
        splitType: 'xor',
        joinType: 'none',
      },
    },
    {
      id: 'task-1',
      type: 'task',
      position: { x: 400, y: 100 },
      data: {
        label: 'Express Shipping',
        decomposition: 'atomic',
        resources: [],
        properties: { priority: 'high' },
      },
    },
    {
      id: 'task-2',
      type: 'task',
      position: { x: 400, y: 300 },
      data: {
        label: 'Standard Shipping',
        decomposition: 'atomic',
        resources: [],
        properties: { priority: 'normal' },
      },
    },
    {
      id: 'join-1',
      type: 'condition',
      position: { x: 550, y: 200 },
      data: {
        label: 'XOR Join',
        conditionType: 'intermediate',
        splitType: 'none',
        joinType: 'xor',
      },
    },
    {
      id: 'end-1',
      type: 'condition',
      position: { x: 700, y: 200 },
      data: {
        label: 'End',
        conditionType: 'output',
        splitType: 'none',
        joinType: 'none',
      },
    },
  ] as WorkflowNode[],
  edges: [
    { id: 'e1', source: 'start-1', target: 'split-1' },
    { id: 'e2', source: 'split-1', target: 'task-1', data: { condition: 'priority == "high"' } },
    { id: 'e3', source: 'split-1', target: 'task-2', data: { condition: 'priority == "normal"' } },
    { id: 'e4', source: 'task-1', target: 'join-1' },
    { id: 'e5', source: 'task-2', target: 'join-1' },
    { id: 'e6', source: 'join-1', target: 'end-1' },
  ] as WorkflowEdge[],
  metadata: {
    name: 'Exclusive Choice',
    description: 'XOR-split/XOR-join conditional pattern',
    version: '1.0.0',
  },
};

/**
 * Invalid workflow: Missing end condition
 * Validates: Error detection in validation
 */
export const invalidWorkflowMissingEnd = {
  nodes: [
    {
      id: 'start-1',
      type: 'condition',
      position: { x: 100, y: 100 },
      data: {
        label: 'Start',
        conditionType: 'input',
        splitType: 'none',
        joinType: 'none',
      },
    },
    {
      id: 'task-1',
      type: 'task',
      position: { x: 250, y: 100 },
      data: {
        label: 'Process',
        decomposition: 'atomic',
        resources: [],
        properties: {},
      },
    },
  ] as WorkflowNode[],
  edges: [
    { id: 'e1', source: 'start-1', target: 'task-1' },
  ] as WorkflowEdge[],
  metadata: {
    name: 'Invalid - Missing End',
    description: 'Workflow missing required end condition',
    version: '1.0.0',
  },
};

/**
 * Invalid workflow: Unmatched split/join
 * Validates: Pattern validation enforcement
 */
export const invalidWorkflowUnmatchedSplit = {
  nodes: [
    {
      id: 'start-1',
      type: 'condition',
      position: { x: 100, y: 200 },
      data: {
        label: 'Start',
        conditionType: 'input',
        splitType: 'none',
        joinType: 'none',
      },
    },
    {
      id: 'split-1',
      type: 'condition',
      position: { x: 250, y: 200 },
      data: {
        label: 'AND Split',
        conditionType: 'intermediate',
        splitType: 'and',
        joinType: 'none',
      },
    },
    {
      id: 'task-1',
      type: 'task',
      position: { x: 400, y: 100 },
      data: {
        label: 'Task A',
        decomposition: 'atomic',
        resources: [],
        properties: {},
      },
    },
    {
      id: 'task-2',
      type: 'task',
      position: { x: 400, y: 300 },
      data: {
        label: 'Task B',
        decomposition: 'atomic',
        resources: [],
        properties: {},
      },
    },
    {
      id: 'end-1',
      type: 'condition',
      position: { x: 550, y: 200 },
      data: {
        label: 'End',
        conditionType: 'output',
        splitType: 'none',
        joinType: 'none', // Missing AND join!
      },
    },
  ] as WorkflowNode[],
  edges: [
    { id: 'e1', source: 'start-1', target: 'split-1' },
    { id: 'e2', source: 'split-1', target: 'task-1' },
    { id: 'e3', source: 'split-1', target: 'task-2' },
    { id: 'e4', source: 'task-1', target: 'end-1' },
    { id: 'e5', source: 'task-2', target: 'end-1' },
  ] as WorkflowEdge[],
  metadata: {
    name: 'Invalid - Unmatched Split',
    description: 'AND split without corresponding AND join',
    version: '1.0.0',
  },
};

/**
 * Sample RDF Turtle representation
 * Validates: RDF serialization round-trip
 */
export const sampleTurtleWorkflow = `
@prefix yawl: <http://knhk.io/ontology/yawl#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

:workflow-1 a yawl:Specification ;
    yawl:name "Simple Sequence" ;
    yawl:version "1.0.0" ;
    yawl:hasTask :task-1, :task-2 ;
    yawl:hasCondition :start-1, :end-1 ;
    yawl:hasFlow :flow-1, :flow-2, :flow-3 .

:start-1 a yawl:InputCondition ;
    yawl:label "Start" .

:task-1 a yawl:AtomicTask ;
    yawl:label "Process Order" ;
    yawl:decomposition "atomic" .

:task-2 a yawl:AtomicTask ;
    yawl:label "Ship Order" ;
    yawl:decomposition "atomic" .

:end-1 a yawl:OutputCondition ;
    yawl:label "End" .

:flow-1 a yawl:Flow ;
    yawl:source :start-1 ;
    yawl:target :task-1 .

:flow-2 a yawl:Flow ;
    yawl:source :task-1 ;
    yawl:target :task-2 .

:flow-3 a yawl:Flow ;
    yawl:source :task-2 ;
    yawl:target :end-1 .
`;

/**
 * Complex workflow for performance testing
 * Validates: Chicago TDD latency constraints (≤8ms)
 */
export const complexPerformanceWorkflow = {
  nodes: Array.from({ length: 50 }, (_, i) => ({
    id: `node-${i}`,
    type: i === 0 ? 'condition' : i === 49 ? 'condition' : 'task',
    position: { x: (i % 10) * 150, y: Math.floor(i / 10) * 150 },
    data: {
      label: `Node ${i}`,
      ...(i === 0 ? { conditionType: 'input', splitType: 'none', joinType: 'none' } :
          i === 49 ? { conditionType: 'output', splitType: 'none', joinType: 'none' } :
          { decomposition: 'atomic', resources: [], properties: {} }),
    },
  })) as WorkflowNode[],
  edges: Array.from({ length: 49 }, (_, i) => ({
    id: `e${i}`,
    source: `node-${i}`,
    target: `node-${i + 1}`,
  })) as WorkflowEdge[],
  metadata: {
    name: 'Performance Test Workflow',
    description: '50-node workflow for latency testing',
    version: '1.0.0',
  },
};

/**
 * All test workflows for convenience
 */
export const allTestWorkflows = {
  simpleSequence: simpleSequenceWorkflow,
  parallel: parallelWorkflow,
  exclusiveChoice: exclusiveChoiceWorkflow,
  invalidMissingEnd: invalidWorkflowMissingEnd,
  invalidUnmatchedSplit: invalidWorkflowUnmatchedSplit,
  complexPerformance: complexPerformanceWorkflow,
};

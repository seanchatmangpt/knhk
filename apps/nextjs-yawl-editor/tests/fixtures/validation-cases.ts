/**
 * Validation Test Cases
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 2: Q invariants are enforced
 * - Validates pattern permutation matrix
 *
 * Test cases for comprehensive validation coverage:
 * - Valid patterns
 * - Invalid patterns
 * - Edge cases
 * - Performance constraints
 */

export type ValidationTestCase = {
  name: string;
  description: string;
  workflow: any;
  expectedValid: boolean;
  expectedErrors?: string[];
  covenantValidated: string;
};

/**
 * Valid workflow patterns
 */
export const validPatterns: ValidationTestCase[] = [
  {
    name: 'Valid Sequence',
    description: 'Simple sequential workflow',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'task-1' },
        { source: 'task-1', target: 'end-1' },
      ],
    },
    expectedValid: true,
    covenantValidated: 'Covenant 2 - Pattern 1 (Sequence)',
  },
  {
    name: 'Valid AND Split/Join',
    description: 'Parallel execution with matched split/join',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'split-1', type: 'condition', data: { splitType: 'and' } },
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'task-2', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'join-1', type: 'condition', data: { joinType: 'and' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'split-1' },
        { source: 'split-1', target: 'task-1' },
        { source: 'split-1', target: 'task-2' },
        { source: 'task-1', target: 'join-1' },
        { source: 'task-2', target: 'join-1' },
        { source: 'join-1', target: 'end-1' },
      ],
    },
    expectedValid: true,
    covenantValidated: 'Covenant 2 - Pattern 2 (Parallel Split)',
  },
  {
    name: 'Valid XOR Split/Join',
    description: 'Exclusive choice with matched split/join',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'split-1', type: 'condition', data: { splitType: 'xor' } },
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'task-2', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'join-1', type: 'condition', data: { joinType: 'xor' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'split-1' },
        { source: 'split-1', target: 'task-1', data: { condition: 'x > 0' } },
        { source: 'split-1', target: 'task-2', data: { condition: 'x <= 0' } },
        { source: 'task-1', target: 'join-1' },
        { source: 'task-2', target: 'join-1' },
        { source: 'join-1', target: 'end-1' },
      ],
    },
    expectedValid: true,
    covenantValidated: 'Covenant 2 - Pattern 4 (Exclusive Choice)',
  },
];

/**
 * Invalid workflow patterns
 */
export const invalidPatterns: ValidationTestCase[] = [
  {
    name: 'Missing Start Condition',
    description: 'Workflow without input condition',
    workflow: {
      nodes: [
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'task-1', target: 'end-1' },
      ],
    },
    expectedValid: false,
    expectedErrors: ['MISSING_INPUT_CONDITION'],
    covenantValidated: 'Covenant 2 - Q1 (Structural Soundness)',
  },
  {
    name: 'Missing End Condition',
    description: 'Workflow without output condition',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
      ],
      edges: [
        { source: 'start-1', target: 'task-1' },
      ],
    },
    expectedValid: false,
    expectedErrors: ['MISSING_OUTPUT_CONDITION'],
    covenantValidated: 'Covenant 2 - Q1 (Structural Soundness)',
  },
  {
    name: 'Unmatched AND Split',
    description: 'AND-split without corresponding AND-join',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'split-1', type: 'condition', data: { splitType: 'and' } },
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'task-2', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'split-1' },
        { source: 'split-1', target: 'task-1' },
        { source: 'split-1', target: 'task-2' },
        { source: 'task-1', target: 'end-1' },
        { source: 'task-2', target: 'end-1' },
      ],
    },
    expectedValid: false,
    expectedErrors: ['UNMATCHED_SPLIT_JOIN'],
    covenantValidated: 'Covenant 2 - Pattern Validation',
  },
  {
    name: 'Disconnected Nodes',
    description: 'Workflow with unreachable nodes',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'task-2', type: 'task', data: { decomposition: 'atomic' } }, // Disconnected!
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'task-1' },
        { source: 'task-1', target: 'end-1' },
      ],
    },
    expectedValid: false,
    expectedErrors: ['UNREACHABLE_NODES'],
    covenantValidated: 'Covenant 2 - Q1 (Connectedness)',
  },
  {
    name: 'Cyclic Workflow',
    description: 'Workflow with cycle (retrocausation violation)',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'task-2', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'task-1' },
        { source: 'task-1', target: 'task-2' },
        { source: 'task-2', target: 'task-1' }, // Cycle!
        { source: 'task-2', target: 'end-1' },
      ],
    },
    expectedValid: false,
    expectedErrors: ['CYCLE_DETECTED'],
    covenantValidated: 'Covenant 2 - Q1 (No Retrocausation)',
  },
];

/**
 * Edge cases
 */
export const edgeCases: ValidationTestCase[] = [
  {
    name: 'Minimal Valid Workflow',
    description: 'Smallest possible valid workflow',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'end-1' },
      ],
    },
    expectedValid: true,
    covenantValidated: 'Covenant 2 - Minimal Soundness',
  },
  {
    name: 'Empty Workflow',
    description: 'Workflow with no nodes',
    workflow: {
      nodes: [],
      edges: [],
    },
    expectedValid: false,
    expectedErrors: ['EMPTY_WORKFLOW'],
    covenantValidated: 'Covenant 2 - Non-empty Constraint',
  },
  {
    name: 'Multiple Input Conditions',
    description: 'Workflow with more than one start',
    workflow: {
      nodes: [
        { id: 'start-1', type: 'condition', data: { conditionType: 'input' } },
        { id: 'start-2', type: 'condition', data: { conditionType: 'input' } }, // Invalid!
        { id: 'task-1', type: 'task', data: { decomposition: 'atomic' } },
        { id: 'end-1', type: 'condition', data: { conditionType: 'output' } },
      ],
      edges: [
        { source: 'start-1', target: 'task-1' },
        { source: 'start-2', target: 'task-1' },
        { source: 'task-1', target: 'end-1' },
      ],
    },
    expectedValid: false,
    expectedErrors: ['MULTIPLE_INPUT_CONDITIONS'],
    covenantValidated: 'Covenant 2 - Unique Start/End',
  },
];

/**
 * Performance test cases (Chicago TDD)
 */
export const performanceCases = {
  small: {
    name: 'Small Workflow (≤8ms)',
    nodeCount: 10,
    expectedLatency: 8, // ms (Chatman constant)
    covenantValidated: 'Covenant 5 - Chatman Constant (hot path)',
  },
  medium: {
    name: 'Medium Workflow (≤100ms)',
    nodeCount: 50,
    expectedLatency: 100, // ms (warm path)
    covenantValidated: 'Covenant 2 - Q4 (Latency SLO)',
  },
  large: {
    name: 'Large Workflow (≤500ms)',
    nodeCount: 200,
    expectedLatency: 500, // ms (cold path)
    covenantValidated: 'Covenant 2 - Q4 (Latency SLO)',
  },
};

/**
 * All validation test cases
 */
export const allValidationCases = {
  valid: validPatterns,
  invalid: invalidPatterns,
  edge: edgeCases,
  performance: performanceCases,
};

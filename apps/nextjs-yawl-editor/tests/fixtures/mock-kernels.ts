/**
 * Mock knhk-kernel Responses for Testing
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 3: MAPE-K autonomic feedback
 * - Covenant 6: Observable telemetry
 *
 * These mocks simulate knhk-kernel behavior for:
 * - Workflow submission and validation
 * - Execution trace generation
 * - MAPE-K recommendations
 * - Pattern matrix synchronization
 */

import type {
  WorkflowSubmissionResponse,
  WorkflowExecutionTrace,
  MAPEKRecommendation,
  PatternMatrixEntry,
} from '@/lib/knhk/types';

/**
 * Successful workflow submission response
 */
export const mockSuccessfulSubmission: WorkflowSubmissionResponse = {
  success: true,
  workflowId: 'wf-test-123',
  validationStatus: 'valid',
  timestamp: new Date('2025-11-18T00:00:00Z').toISOString(),
  message: 'Workflow submitted successfully',
};

/**
 * Failed workflow submission (validation error)
 */
export const mockFailedSubmission: WorkflowSubmissionResponse = {
  success: false,
  workflowId: null,
  validationStatus: 'invalid',
  timestamp: new Date('2025-11-18T00:00:00Z').toISOString(),
  message: 'Validation failed: Missing end condition',
  errors: [
    {
      code: 'MISSING_END_CONDITION',
      message: 'Workflow must have exactly one output condition',
      severity: 'error',
      location: { nodeId: null },
    },
  ],
};

/**
 * Pattern validation error
 */
export const mockPatternValidationError: WorkflowSubmissionResponse = {
  success: false,
  workflowId: null,
  validationStatus: 'invalid',
  timestamp: new Date('2025-11-18T00:00:00Z').toISOString(),
  message: 'Pattern validation failed',
  errors: [
    {
      code: 'UNMATCHED_SPLIT_JOIN',
      message: 'AND-split at node split-1 has no corresponding AND-join',
      severity: 'error',
      location: { nodeId: 'split-1' },
    },
  ],
};

/**
 * Workflow execution trace
 */
export const mockExecutionTrace: WorkflowExecutionTrace = {
  workflowId: 'wf-test-123',
  executionId: 'exec-456',
  status: 'completed',
  startTime: new Date('2025-11-18T00:00:00Z').toISOString(),
  endTime: new Date('2025-11-18T00:01:00Z').toISOString(),
  steps: [
    {
      stepId: 'step-1',
      nodeId: 'start-1',
      type: 'condition_activated',
      timestamp: new Date('2025-11-18T00:00:00Z').toISOString(),
      duration: 2,
    },
    {
      stepId: 'step-2',
      nodeId: 'task-1',
      type: 'task_started',
      timestamp: new Date('2025-11-18T00:00:05Z').toISOString(),
      duration: 15,
    },
    {
      stepId: 'step-3',
      nodeId: 'task-1',
      type: 'task_completed',
      timestamp: new Date('2025-11-18T00:00:20Z').toISOString(),
      duration: 0,
    },
    {
      stepId: 'step-4',
      nodeId: 'task-2',
      type: 'task_started',
      timestamp: new Date('2025-11-18T00:00:25Z').toISOString(),
      duration: 20,
    },
    {
      stepId: 'step-5',
      nodeId: 'task-2',
      type: 'task_completed',
      timestamp: new Date('2025-11-18T00:00:45Z').toISOString(),
      duration: 0,
    },
    {
      stepId: 'step-6',
      nodeId: 'end-1',
      type: 'condition_activated',
      timestamp: new Date('2025-11-18T00:01:00Z').toISOString(),
      duration: 1,
    },
  ],
  telemetry: {
    totalDuration: 60000,
    tickCount: 6,
    averageTickDuration: 10000,
    maxTickDuration: 20000,
  },
};

/**
 * MAPE-K Monitor phase output
 */
export const mockMAPEKMonitor = {
  phase: 'monitor' as const,
  timestamp: new Date('2025-11-18T00:00:00Z').toISOString(),
  observations: {
    workflowModified: true,
    changedNodes: ['task-1'],
    changedEdges: [],
    validationNeeded: true,
  },
};

/**
 * MAPE-K Analysis phase output
 */
export const mockMAPEKAnalysis = {
  phase: 'analyze' as const,
  timestamp: new Date('2025-11-18T00:00:01Z').toISOString(),
  analysis: {
    patternDetected: 'sequence',
    optimizationOpportunities: [
      {
        type: 'parallelization',
        confidence: 0.85,
        estimatedImprovement: 0.3,
        affectedNodes: ['task-1', 'task-2'],
      },
    ],
    risks: [],
  },
};

/**
 * MAPE-K Plan phase output
 */
export const mockMAPEKPlan: MAPEKRecommendation = {
  id: 'rec-789',
  type: 'optimization',
  confidence: 0.85,
  description: 'Tasks task-1 and task-2 can be executed in parallel',
  actions: [
    {
      type: 'add_node',
      data: {
        id: 'split-1',
        type: 'condition',
        label: 'AND Split',
        conditionType: 'intermediate',
        splitType: 'and',
      },
    },
    {
      type: 'add_node',
      data: {
        id: 'join-1',
        type: 'condition',
        label: 'AND Join',
        conditionType: 'intermediate',
        joinType: 'and',
      },
    },
    {
      type: 'modify_edges',
      data: {
        remove: ['e1'],
        add: [
          { source: 'start-1', target: 'split-1' },
          { source: 'split-1', target: 'task-1' },
          { source: 'split-1', target: 'task-2' },
          { source: 'task-1', target: 'join-1' },
          { source: 'task-2', target: 'join-1' },
          { source: 'join-1', target: 'end-1' },
        ],
      },
    },
  ],
  estimatedImpact: {
    performance: 0.3,
    complexity: -0.1,
  },
};

/**
 * MAPE-K Execute phase confirmation
 */
export const mockMAPEKExecute = {
  phase: 'execute' as const,
  timestamp: new Date('2025-11-18T00:00:02Z').toISOString(),
  status: 'applied',
  recommendationId: 'rec-789',
  appliedActions: 3,
};

/**
 * MAPE-K Knowledge update
 */
export const mockMAPEKKnowledge = {
  phase: 'knowledge' as const,
  timestamp: new Date('2025-11-18T00:00:03Z').toISOString(),
  updates: {
    patternFrequency: {
      sequence: 15,
      parallel: 8,
      exclusive_choice: 5,
    },
    optimizationSuccess: {
      parallelization: 0.82,
      resource_allocation: 0.75,
    },
  },
};

/**
 * Pattern matrix entries from knhk-kernel
 */
export const mockPatternMatrixEntries: PatternMatrixEntry[] = [
  {
    id: 'pattern-1',
    name: 'Sequence',
    description: 'Sequential execution of tasks',
    valid: true,
    constraints: [],
  },
  {
    id: 'pattern-2',
    name: 'Parallel Split',
    description: 'AND-split followed by AND-join',
    valid: true,
    constraints: [
      'Must have matching AND-join',
      'All branches must complete',
    ],
  },
  {
    id: 'pattern-4',
    name: 'Exclusive Choice',
    description: 'XOR-split with conditional branches',
    valid: true,
    constraints: [
      'Must have matching XOR-join',
      'Exactly one branch executes',
    ],
  },
  {
    id: 'pattern-invalid-1',
    name: 'Unmatched Split',
    description: 'Split without corresponding join',
    valid: false,
    constraints: [],
  },
];

/**
 * Performance telemetry (Chicago TDD)
 */
export const mockPerformanceTelemetry = {
  operation: 'workflow_validation',
  latency: 6.5, // Must be ≤8ms (Chatman constant)
  tickCount: 4,
  cpuTime: 5.2,
  memoryUsed: 1024 * 512, // 512KB
  timestamp: new Date('2025-11-18T00:00:00Z').toISOString(),
};

/**
 * Telemetry exceeding SLO (for testing error handling)
 */
export const mockSlowTelemetry = {
  operation: 'workflow_validation',
  latency: 12.3, // EXCEEDS 8ms SLO!
  tickCount: 8,
  cpuTime: 10.8,
  memoryUsed: 1024 * 1024 * 2, // 2MB
  timestamp: new Date('2025-11-18T00:00:00Z').toISOString(),
};

/**
 * Mock HTTP responses for MSW
 */
export const mockKernelResponses = {
  submitWorkflow: {
    success: mockSuccessfulSubmission,
    validationError: mockFailedSubmission,
    patternError: mockPatternValidationError,
  },
  getExecutionTrace: mockExecutionTrace,
  mapeK: {
    monitor: mockMAPEKMonitor,
    analyze: mockMAPEKAnalysis,
    plan: mockMAPEKPlan,
    execute: mockMAPEKExecute,
    knowledge: mockMAPEKKnowledge,
  },
  patterns: mockPatternMatrixEntries,
  telemetry: {
    valid: mockPerformanceTelemetry,
    sloViolation: mockSlowTelemetry,
  },
};

/**
 * Mock network delay (simulates kernel latency)
 */
export const mockNetworkDelay = (ms: number = 100) =>
  new Promise(resolve => setTimeout(resolve, ms));

/**
 * DOCTRINE ALIGNMENT: Π (Projections) + O (Ontology-First)
 * Type definitions for knhk ecosystem integration
 *
 * COVENANT 1: RDF as source of truth
 * COVENANT 6: All operations observable via telemetry
 */

import { z } from 'zod';
import type { YAWLWorkflow, ValidationResult as EditorValidationResult } from '@/lib/types';

// Re-export ValidationResult for use in this module
export type { EditorValidationResult as ValidationResult };

/* ============================================================================
 * knhk-kernel Types
 * ========================================================================== */

/**
 * Unique identifier for workflows in the kernel
 */
export type WorkflowId = string & { readonly __brand: 'WorkflowId' };

/**
 * Unique identifier for workflow execution cases
 */
export type CaseId = string & { readonly __brand: 'CaseId' };

/**
 * YAWL specification format for kernel execution
 */
export interface YAWLSpecification {
  id: WorkflowId;
  name: string;
  version: string;
  xml: string; // YAWL XML format
  rdf?: string; // Optional RDF representation
  metadata: {
    author?: string;
    description?: string;
    created: string;
    modified: string;
    tags?: string[];
  };
}

/**
 * Workflow execution event from kernel
 */
export interface ExecutionEvent {
  id: string;
  caseId: CaseId;
  workflowId: WorkflowId;
  timestamp: number;
  type: 'case.created' | 'case.started' | 'task.enabled' | 'task.started' | 'task.completed' | 'case.completed' | 'case.failed';
  taskId?: string;
  data?: Record<string, unknown>;
  spans?: Array<{
    spanId: string;
    traceId: string;
    name: string;
    duration: number;
  }>;
}

/**
 * Case execution metrics
 */
export interface CaseMetrics {
  caseId: CaseId;
  workflowId: WorkflowId;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  startTime: number;
  endTime?: number;
  duration?: number;
  taskCount: number;
  completedTasks: number;
  failedTasks: number;
  currentTasks: string[];
  performance: {
    averageTaskDuration: number;
    totalWaitTime: number;
    throughput: number;
  };
}

/**
 * Execution trace for learning and analysis
 */
export interface ExecutionTrace {
  caseId: CaseId;
  workflowId: WorkflowId;
  events: ExecutionEvent[];
  metrics: CaseMetrics;
  patterns: {
    detected: string[];
    violations: string[];
  };
}

/* ============================================================================
 * Pattern Matrix Types
 * ========================================================================== */

/**
 * YAWL split types
 */
export type SplitType = 'AND' | 'XOR' | 'OR';

/**
 * YAWL join types
 */
export type JoinType = 'AND' | 'XOR' | 'OR';

/**
 * Pattern modifiers (cancellation, multiple instances, etc.)
 */
export type Modifiers =
  | 'cancel_region'
  | 'cancel_case'
  | 'multiple_instances'
  | 'deferred_choice'
  | 'milestone';

/**
 * Pattern combination in permutation matrix
 */
export interface PatternCombination {
  split: SplitType;
  join: JoinType;
  modifiers: Modifiers[];
  valid: boolean;
  constraints: string[] | undefined;
  examples: string[] | undefined;
}

/**
 * Full pattern permutation matrix
 */
export interface PatternPermutations {
  version: string;
  source: string;
  lastUpdated: string;
  combinations: PatternCombination[];
  index: {
    [key: string]: PatternCombination; // "split:join:mod1,mod2" -> combination
  };
}

/**
 * Cache status for pattern matrix
 */
export interface CacheInfo {
  loaded: boolean;
  version: string | undefined;
  lastUpdated: string | undefined;
  size: number;
  hits: number;
  misses: number;
}

/* ============================================================================
 * MAPE-K Types
 * ========================================================================== */

/**
 * Analysis result from MAPE-K loop
 */
export interface AnalysisResult {
  workflowId: WorkflowId;
  timestamp: number;
  patterns: {
    detected: string[];
    missing: string[];
    violations: string[];
  };
  performance: {
    estimatedDuration: number;
    bottlenecks: string[];
    parallelismScore: number;
  };
  quality: {
    score: number;
    issues: Array<{
      severity: 'error' | 'warning' | 'info';
      message: string;
      location?: string;
    }>;
  };
}

/**
 * Optimization recommendation
 */
export interface Recommendation {
  id: string;
  type: 'refactor' | 'optimize' | 'fix' | 'enhance';
  priority: 'high' | 'medium' | 'low';
  title: string;
  description: string;
  impact: {
    performance?: number; // percentage improvement
    quality?: number; // score improvement
    complexity?: number; // complexity reduction
  };
  changes: Array<{
    nodeId?: string;
    edgeId?: string;
    action: 'add' | 'remove' | 'modify';
    before?: unknown;
    after?: unknown;
  }>;
  rationale: string;
}

/* ============================================================================
 * Telemetry & Schema Types
 * ========================================================================== */

/**
 * OpenTelemetry span for validation
 */
export interface ReadableSpan {
  spanContext: {
    traceId: string;
    spanId: string;
  };
  name: string;
  kind: number;
  startTime: number;
  endTime: number;
  attributes: Record<string, string | number | boolean>;
  events: Array<{
    name: string;
    time: number;
    attributes?: Record<string, unknown>;
  }>;
  status: {
    code: number;
    message?: string;
  };
}

/**
 * Schema validation result from Weaver
 */
export interface SchemaValidation {
  valid: boolean;
  errors: Array<{
    code: string;
    message: string;
    path?: string;
  }>;
  warnings: Array<{
    code: string;
    message: string;
    path?: string;
  }>;
  schemaVersion: string;
}

/**
 * OpenTelemetry schema definition
 */
export interface OTelSchema {
  schemaUrl: string;
  version: string;
  spans: Array<{
    name: string;
    attributes: Record<string, {
      type: 'string' | 'number' | 'boolean';
      required: boolean;
      description?: string;
    }>;
    events?: string[];
  }>;
  metrics: Array<{
    name: string;
    type: 'counter' | 'histogram' | 'gauge';
    unit: string;
    description?: string;
  }>;
}

/**
 * Schema status from Weaver registry
 */
export interface SchemaStatus {
  id: string;
  version: string;
  status: 'valid' | 'invalid' | 'pending';
  registered: string;
  lastChecked: string;
  errors: string[];
}

/**
 * Metric for export
 */
export interface Metric {
  name: string;
  type: 'counter' | 'histogram' | 'gauge';
  value: number;
  timestamp: number;
  attributes: Record<string, string | number | boolean>;
}

/**
 * Operation for receipt logging
 */
export interface Operation {
  id: string;
  type: string;
  timestamp: number;
  actor?: string;
  target?: string;
  params?: Record<string, unknown>;
}

/**
 * Result of an operation
 */
export interface Result {
  success: boolean;
  duration: number;
  error?: string;
  data?: unknown;
  metrics?: Metric[];
}

/* ============================================================================
 * Workflow Exchange Types
 * ========================================================================== */

/**
 * RDF dataset representation
 */
export interface RDFDataset {
  format: 'turtle' | 'ntriples' | 'jsonld';
  content: string;
  prefixes: Record<string, string>;
  graphs?: Record<string, string>;
}

/**
 * knhk workflow format for execution
 */
export interface KNHKWorkflow {
  id: WorkflowId;
  format: 'yawl-xml' | 'bpmn' | 'petri-net';
  specification: string;
  ontology: RDFDataset | undefined;
  metadata: {
    source: 'editor' | 'import' | 'generated';
    author: string | undefined;
    created: string;
    validated: boolean;
    validationReport: EditorValidationResult | undefined;
  };
}

/**
 * Validated specification
 */
export interface ValidatedSpec {
  workflow: YAWLWorkflow;
  validation: EditorValidationResult;
  patterns: string[];
  compliance: {
    matrixConformance: boolean;
    invariantsPreserved: boolean;
    performanceBudget: boolean;
  };
  timestamp: string;
}

/**
 * Audit trail entry
 */
export interface AuditEntry {
  id: string;
  timestamp: string;
  operation: 'create' | 'modify' | 'delete' | 'validate';
  actor?: string;
  changes: Array<{
    path: string;
    before: unknown;
    after: unknown;
    reason?: string;
  }>;
  metadata: {
    workflowId?: WorkflowId;
    caseId?: CaseId;
    traceId?: string;
  };
}

/* ============================================================================
 * API Response Types
 * ========================================================================== */

/**
 * Standard API response wrapper
 */
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
  metadata: {
    timestamp: number;
    requestId: string;
    duration: number;
  };
}

/**
 * Unsubscribe function type
 */
export type Unsubscribe = () => void;

/* ============================================================================
 * Zod Schemas for Runtime Validation
 * ========================================================================== */

export const ExecutionEventSchema = z.object({
  id: z.string(),
  caseId: z.string(),
  workflowId: z.string(),
  timestamp: z.number(),
  type: z.enum([
    'case.created',
    'case.started',
    'task.enabled',
    'task.started',
    'task.completed',
    'case.completed',
    'case.failed',
  ]),
  taskId: z.string().optional(),
  data: z.record(z.unknown()).optional(),
});

export const CaseMetricsSchema = z.object({
  caseId: z.string(),
  workflowId: z.string(),
  status: z.enum(['running', 'completed', 'failed', 'cancelled']),
  startTime: z.number(),
  endTime: z.number().optional(),
  duration: z.number().optional(),
  taskCount: z.number(),
  completedTasks: z.number(),
  failedTasks: z.number(),
  currentTasks: z.array(z.string()),
  performance: z.object({
    averageTaskDuration: z.number(),
    totalWaitTime: z.number(),
    throughput: z.number(),
  }),
});

export const RecommendationSchema = z.object({
  id: z.string(),
  type: z.enum(['refactor', 'optimize', 'fix', 'enhance']),
  priority: z.enum(['high', 'medium', 'low']),
  title: z.string(),
  description: z.string(),
  impact: z.object({
    performance: z.number().optional(),
    quality: z.number().optional(),
    complexity: z.number().optional(),
  }),
  changes: z.array(z.object({
    nodeId: z.string().optional(),
    edgeId: z.string().optional(),
    action: z.enum(['add', 'remove', 'modify']),
    before: z.unknown().optional(),
    after: z.unknown().optional(),
  })),
  rationale: z.string(),
});

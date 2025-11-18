/**
 * DOCTRINE ALIGNMENT: O (Ontology-First)
 * Core type definitions for YAWL Process Editor
 * Ensures type safety and semantic consistency
 */

import { z } from 'zod';

/* ============================================================================
 * YAWL Workflow Types
 * ========================================================================== */

export const YAWLNodeSchema = z.object({
  id: z.string(),
  type: z.enum(['task', 'condition', 'start', 'end', 'split', 'join']),
  label: z.string(),
  position: z.object({
    x: z.number(),
    y: z.number(),
  }),
  data: z.record(z.unknown()).optional(),
});

export const YAWLEdgeSchema = z.object({
  id: z.string(),
  source: z.string(),
  target: z.string(),
  label: z.string().optional(),
  condition: z.string().optional(),
});

export const YAWLWorkflowSchema = z.object({
  id: z.string(),
  name: z.string(),
  version: z.string().default('1.0.0'),
  nodes: z.array(YAWLNodeSchema),
  edges: z.array(YAWLEdgeSchema),
  metadata: z
    .object({
      author: z.string().optional(),
      description: z.string().optional(),
      created: z.string().datetime(),
      modified: z.string().datetime(),
    })
    .optional(),
});

export type YAWLNode = z.infer<typeof YAWLNodeSchema>;
export type YAWLEdge = z.infer<typeof YAWLEdgeSchema>;
export type YAWLWorkflow = z.infer<typeof YAWLWorkflowSchema>;

/* ============================================================================
 * RDF/Turtle Types
 * ========================================================================== */

export interface RDFTriple {
  subject: string;
  predicate: string;
  object: string;
}

export interface OntologyDefinition {
  uri: string;
  prefixes: Record<string, string>;
  classes: string[];
  properties: string[];
  triples: RDFTriple[];
}

/* ============================================================================
 * Pattern Validation Types
 * ========================================================================== */

export const PatternTypeSchema = z.enum([
  'sequence',
  'parallel_split',
  'synchronization',
  'exclusive_choice',
  'simple_merge',
  'multi_choice',
  'structured_synchronizing_merge',
  'multi_merge',
  'structured_discriminator',
  'arbitrary_cycles',
  'implicit_termination',
  'multiple_instances_without_synchronization',
  'multiple_instances_with_a_priori_design_time_knowledge',
  'multiple_instances_with_a_priori_runtime_knowledge',
  'multiple_instances_without_a_priori_runtime_knowledge',
  'deferred_choice',
  'interleaved_parallel_routing',
  'milestone',
  'critical_section',
  'interleaved_routing',
  'cancel_activity',
  'cancel_case',
]);

export type PatternType = z.infer<typeof PatternTypeSchema>;

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
}

export interface ValidationError {
  code: string;
  message: string;
  node?: string;
  severity: 'error';
}

export interface ValidationWarning {
  code: string;
  message: string;
  node?: string;
  severity: 'warning';
}

/* ============================================================================
 * Editor State Types
 * ========================================================================== */

export interface EditorState {
  workflow: YAWLWorkflow | null;
  selectedNodes: string[];
  selectedEdges: string[];
  clipboard: {
    nodes: YAWLNode[];
    edges: YAWLEdge[];
  } | null;
  history: {
    past: YAWLWorkflow[];
    future: YAWLWorkflow[];
  };
  mode: 'edit' | 'view' | 'validate';
}

/* ============================================================================
 * Telemetry Types
 * ========================================================================== */

export interface TelemetryEvent {
  name: string;
  attributes: Record<string, string | number | boolean>;
  timestamp: number;
}

export interface PerformanceMetrics {
  renderTime: number;
  validationTime: number;
  parseTime: number;
  totalNodes: number;
  totalEdges: number;
}

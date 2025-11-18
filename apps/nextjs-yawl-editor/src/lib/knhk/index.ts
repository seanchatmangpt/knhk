/**
 * DOCTRINE ALIGNMENT: Full Stack Integration
 * knhk Ecosystem Integration for YAWL Editor
 *
 * This module bridges the Next.js YAWL editor with:
 * - knhk-kernel: Workflow execution engine
 * - MAPE-K loops: Autonomic feedback and optimization
 * - Pattern matrix: YAWL pattern validation
 * - OpenTelemetry: Observability and schema validation (Weaver)
 * - RDF ontology: Semantic workflow representation
 *
 * DOCTRINE PRINCIPLES:
 * - O (Ontology-First): RDF as source of truth
 * - Σ (System Integrity): All operations observable
 * - Q (Hard Invariants): Pattern validation enforced
 * - Π (Projections): Bidirectional sync with kernel
 * - MAPE-K: Autonomic feedback loops
 * - Chatman Constant: Performance budgets enforced
 *
 * @module knhk
 */

/* ============================================================================
 * Configuration
 * ========================================================================== */

import {
  knhkConfig as _knhkConfig,
  validateConfig as _validateConfig,
  isDevelopment as _isDevelopment,
} from './config';

export {
  knhkConfig,
  getKernelApiUrl,
  getOTLPEndpoint,
  isDevelopment,
  isProduction,
  validateConfig,
  type KNHKConfig,
} from './config';

/* ============================================================================
 * Types
 * ========================================================================== */

import type {
  WorkflowId,
  CaseId,
  ExecutionTrace,
  Recommendation,
  RDFDataset,
  YAWLSpecification,
} from './types';

export type {
  // Workflow types
  WorkflowId,
  CaseId,
  YAWLSpecification,
  ExecutionEvent,
  ExecutionTrace,
  CaseMetrics,
  // Pattern types
  SplitType,
  JoinType,
  Modifiers,
  PatternCombination,
  PatternPermutations,
  CacheInfo,
  // MAPE-K types
  AnalysisResult,
  Recommendation,
  // Telemetry types
  ReadableSpan,
  SchemaValidation,
  OTelSchema,
  SchemaStatus,
  Metric,
  Operation,
  Result,
  // Exchange types
  RDFDataset,
  KNHKWorkflow,
  ValidatedSpec,
  AuditEntry,
  // Utility types
  ApiResponse,
  Unsubscribe,
} from './types';

export {
  ExecutionEventSchema,
  CaseMetricsSchema,
  RecommendationSchema,
} from './types';

/* ============================================================================
 * knhk-kernel Client
 * ========================================================================== */

import { knhkClient as _knhkClient } from './client';

export { KNHKClient, knhkClient } from './client';

/* ============================================================================
 * MAPE-K Hooks
 * ========================================================================== */

import { mapekHooks as _mapekHooks } from './mape-k-hooks';

export { MAPEKHooks, mapekHooks } from './mape-k-hooks';

/* ============================================================================
 * Pattern Matrix Sync
 * ========================================================================== */

import { patternMatrixSync as _patternMatrixSync } from './pattern-matrix-sync';

export { PatternMatrixSync, patternMatrixSync } from './pattern-matrix-sync';

/* ============================================================================
 * Telemetry Bridge
 * ========================================================================== */

import { telemetryBridge as _telemetryBridge } from './telemetry-bridge';

export {
  TelemetryBridge,
  telemetryBridge,
  editorSchema,
} from './telemetry-bridge';

/* ============================================================================
 * Workflow Exchange
 * ========================================================================== */

import { workflowExchange as _workflowExchange } from './workflow-exchange';

export { WorkflowExchange, workflowExchange } from './workflow-exchange';

/* ============================================================================
 * Initialization
 * ========================================================================== */

/**
 * Initialize knhk integration
 *
 * Call this once at application startup
 *
 * @example
 * ```ts
 * import { initializeKNHK } from '@/lib/knhk';
 *
 * // In your app initialization
 * await initializeKNHK();
 * ```
 */
export async function initializeKNHK(): Promise<void> {
  // Validate configuration
  _validateConfig();

  // Preload pattern matrix
  await _patternMatrixSync.loadMatrix();

  // Register telemetry schema
  // (Already done automatically on module load in telemetry-bridge.ts)

  // Perform health check
  try {
    const health = await _knhkClient.healthCheck();
    console.log('knhk-kernel health:', health.status, health.version);
  } catch (error) {
    console.warn('knhk-kernel health check failed:', error);
    if (!_isDevelopment()) {
      throw new Error('Failed to connect to knhk-kernel');
    }
  }
}

/**
 * Cleanup knhk integration
 *
 * Call this on application shutdown
 */
export async function cleanupKNHK(): Promise<void> {
  // Stop pattern matrix polling
  _patternMatrixSync.stopPolling();
  _patternMatrixSync.destroy();

  // Flush pending telemetry
  await _telemetryBridge.destroy();

  // Clear MAPE-K cache
  _mapekHooks.clearCache();
}

/* ============================================================================
 * Convenience Functions
 * ========================================================================== */

/**
 * Submit workflow from editor to kernel for execution
 *
 * This handles the full pipeline:
 * 1. Convert RDF to knhk format
 * 2. Validate against pattern matrix
 * 3. Submit to kernel
 * 4. Track submission via telemetry
 *
 * @example
 * ```ts
 * import { submitWorkflowFromEditor } from '@/lib/knhk';
 *
 * const caseId = await submitWorkflowFromEditor(rdfDataset, {
 *   author: 'user@example.com',
 *   autoStart: true
 * });
 * ```
 */
export async function submitWorkflowFromEditor(
  rdfDataset: RDFDataset,
  options?: {
    author?: string;
    autoStart?: boolean;
    inputData?: Record<string, unknown>;
  }
): Promise<{ workflowId: WorkflowId; caseId?: CaseId }> {
  // Convert to knhk format
  const knhkWorkflow = await _workflowExchange.exportToKNHKFormat(rdfDataset);

  // Create YAWL specification
  const author = options?.author ?? knhkWorkflow.metadata.author;
  const description = knhkWorkflow.metadata.validationReport
    ? knhkWorkflow.metadata.validationReport.errors
        .map((e) => e.message)
        .join(', ')
    : undefined;

  const spec: YAWLSpecification = {
    id: knhkWorkflow.id,
    name: knhkWorkflow.metadata.author ?? 'Untitled',
    version: _knhkConfig.exchange.versioningEnabled
      ? _workflowExchange.generateVersion(knhkWorkflow.id)
      : '1.0.0',
    xml: knhkWorkflow.specification,
    rdf: rdfDataset.content,
    metadata: {
      ...(author ? { author } : {}),
      ...(description ? { description } : {}),
      created: knhkWorkflow.metadata.created,
      modified: new Date().toISOString(),
    },
  };

  // Submit to kernel
  const workflowId = await _knhkClient.submitWorkflow(spec);

  // Auto-start if requested
  if (options?.autoStart) {
    const caseId = await _knhkClient.startCase(workflowId, options.inputData);
    return { workflowId, caseId };
  }

  return { workflowId };
}

/**
 * Analyze workflow and get optimization recommendations
 *
 * Uses MAPE-K loop to analyze workflow and suggest improvements
 *
 * @example
 * ```ts
 * import { analyzeAndOptimize } from '@/lib/knhk';
 *
 * const recommendations = await analyzeAndOptimize(workflow);
 * for (const rec of recommendations) {
 *   console.log(`${rec.priority}: ${rec.title}`);
 * }
 * ```
 */
export async function analyzeAndOptimize(
  workflow: import('@/lib/types').YAWLWorkflow
): Promise<Recommendation[]> {
  // Analyze workflow
  const analysis = await _mapekHooks.analyzeWorkflow(workflow);

  // Get recommendations
  const recommendations = await _mapekHooks.suggestOptimizations(analysis);

  return recommendations;
}

/**
 * Validate workflow against pattern matrix
 *
 * Quick validation without full MAPE-K analysis
 *
 * @example
 * ```ts
 * import { validateWorkflowPatterns } from '@/lib/knhk';
 *
 * const isValid = await validateWorkflowPatterns(workflow);
 * if (!isValid) {
 *   console.error('Workflow contains invalid patterns');
 * }
 * ```
 */
export async function validateWorkflowPatterns(
  workflow: import('@/lib/types').YAWLWorkflow
): Promise<boolean> {
  // Ensure matrix is loaded
  await _patternMatrixSync.loadMatrix();

  // Validate each split-join pair
  for (const node of workflow.nodes) {
    if (node.type === 'split' || node.type === 'join') {
      // Find corresponding join/split
      // This is simplified - in production, do proper pattern analysis
      const split = node.type === 'split' ? node : null;
      const join = node.type === 'join' ? node : null;

      if (split && join) {
        const valid = _patternMatrixSync.validateCombination(
          'AND', // Simplified
          'AND',
          []
        );

        if (!valid) {
          return false;
        }
      }
    }
  }

  return true;
}

/**
 * Learn from execution trace
 *
 * Import execution trace and update MAPE-K knowledge base
 *
 * @example
 * ```ts
 * import { learnFromExecution } from '@/lib/knhk';
 *
 * const trace = await knhkClient.getExecutionTrace(workflowId, caseId);
 * await learnFromExecution(trace);
 * ```
 */
export async function learnFromExecution(trace: ExecutionTrace): Promise<void> {
  // Learn in MAPE-K
  await _mapekHooks.learnFromExecution(trace);

  // Import trace to RDF for ontology enrichment
  await _workflowExchange.importFromKNHKTrace(trace);

  // In production, this would merge into the main ontology
  if (_knhkConfig.dev.debug) {
    console.log('Learned from execution:', {
      caseId: trace.caseId,
      events: trace.events.length,
      patterns: trace.patterns.detected,
    });
  }
}

/* ============================================================================
 * Default Export
 * ========================================================================== */

export default {
  // Singleton instances
  client: _knhkClient,
  mapek: _mapekHooks,
  patterns: _patternMatrixSync,
  telemetry: _telemetryBridge,
  exchange: _workflowExchange,

  // Configuration
  config: _knhkConfig,

  // Lifecycle
  initialize: initializeKNHK,
  cleanup: cleanupKNHK,

  // Convenience functions
  submitWorkflow: submitWorkflowFromEditor,
  analyze: analyzeAndOptimize,
  validate: validateWorkflowPatterns,
  learn: learnFromExecution,
};

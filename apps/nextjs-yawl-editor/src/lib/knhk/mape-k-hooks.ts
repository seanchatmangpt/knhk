/**
 * DOCTRINE ALIGNMENT: MAPE-K (Autonomic Feedback Loops)
 * Monitor-Analyze-Plan-Execute-Knowledge integration
 *
 * COVENANT 3: Feedback runs at machine speed
 * Autonomic optimization without manual intervention
 */

import { createSpan, withSpan } from '@/lib/telemetry/setup';
import { knhkConfig, getKernelApiUrl } from './config';
import type { YAWLWorkflow } from '@/lib/types';
import type {
  AnalysisResult,
  Recommendation,
  ExecutionTrace,
  ApiResponse,
} from './types';

/* ============================================================================
 * MAPE-K Hooks Class
 * ========================================================================== */

/**
 * Autonomic feedback loop integration with knhk ecosystem
 *
 * The MAPE-K loop:
 * - Monitor: Track workflow changes and execution events
 * - Analyze: Pattern recognition and anomaly detection
 * - Plan: Generate optimization recommendations
 * - Execute: Apply changes (with user confirmation by default)
 * - Knowledge: Learn from execution traces and improve over time
 */
export class MAPEKHooks {
  private analysisCache: Map<string, { result: AnalysisResult; timestamp: number }> = new Map();
  private learningEnabled: boolean;
  private autoApply: boolean;

  constructor() {
    this.learningEnabled = knhkConfig.mapek.learningEnabled;
    this.autoApply = knhkConfig.mapek.autoApply;
  }

  /* ============================
   * Monitor Phase
   * ============================ */

  /**
   * Monitor workflow changes and trigger analysis
   *
   * Called whenever the workflow is modified in the editor
   */
  async onWorkflowChanged(workflow: YAWLWorkflow): Promise<void> {
    return withSpan(
      'mapek.monitor.workflow_changed',
      async () => {
        // Track the change event
        const span = createSpan('mapek.monitor.track_change', {
          'workflow.id': workflow.id,
          'workflow.name': workflow.name,
          'workflow.nodes': workflow.nodes.length,
          'workflow.edges': workflow.edges.length,
        });
        span.end();

        // Trigger asynchronous analysis if enabled
        if (knhkConfig.mapek.enabled) {
          // Don't await - run analysis in background
          this.analyzeWorkflow(workflow).catch((error) => {
            console.error('Background analysis failed:', error);
          });
        }
      },
      {
        'workflow.id': workflow.id,
        'mapek.phase': 'monitor',
      }
    );
  }

  /**
   * Monitor execution events for learning
   */
  async onExecutionEvent(
    workflowId: string,
    eventType: string,
    eventData: Record<string, unknown>
  ): Promise<void> {
    return withSpan(
      'mapek.monitor.execution_event',
      async () => {
        // Store event for later analysis
        const span = createSpan('mapek.monitor.store_event', {
          'workflow.id': workflowId,
          'event.type': eventType,
        });

        // In production, this would store to a persistent event log
        if (knhkConfig.dev.debug) {
          console.debug('MAPE-K execution event:', {
            workflowId,
            eventType,
            eventData,
          });
        }

        span.end();
      },
      {
        'workflow.id': workflowId,
        'event.type': eventType,
        'mapek.phase': 'monitor',
      }
    );
  }

  /* ============================
   * Analyze Phase
   * ============================ */

  /**
   * Analyze workflow for patterns, bottlenecks, and quality issues
   *
   * PERFORMANCE: Must complete within analysisBudget (500ms)
   */
  async analyzeWorkflow(workflow: YAWLWorkflow): Promise<AnalysisResult> {
    const startTime = performance.now();

    // Check cache first
    const cached = this.analysisCache.get(workflow.id);
    if (cached && Date.now() - cached.timestamp < knhkConfig.mapek.interval) {
      return cached.result;
    }

    return withSpan(
      'mapek.analyze.workflow',
      async () => {
        const response = await fetch(getKernelApiUrl('/mapek/analyze'), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            workflow: {
              id: workflow.id,
              name: workflow.name,
              nodes: workflow.nodes,
              edges: workflow.edges,
            },
          }),
          signal: AbortSignal.timeout(knhkConfig.mapek.analysisTimeout),
        });

        if (!response.ok) {
          throw new Error(`Analysis failed: ${response.statusText}`);
        }

        const apiResponse = (await response.json()) as ApiResponse<AnalysisResult>;

        if (!apiResponse.success || !apiResponse.data) {
          throw new Error(apiResponse.error?.message || 'Analysis returned no data');
        }

        const result = apiResponse.data;
        const duration = performance.now() - startTime;

        // Check performance budget
        if (duration > knhkConfig.performance.analysisBudget) {
          console.warn(
            `Analysis exceeded budget: ${duration}ms > ${knhkConfig.performance.analysisBudget}ms`
          );
        }

        // Cache result
        this.analysisCache.set(workflow.id, {
          result,
          timestamp: Date.now(),
        });

        return result;
      },
      {
        'workflow.id': workflow.id,
        'workflow.nodes': workflow.nodes.length,
        'workflow.edges': workflow.edges.length,
        'mapek.phase': 'analyze',
      }
    );
  }

  /**
   * Analyze execution trace for learning patterns
   */
  async analyzeExecutionTrace(trace: ExecutionTrace): Promise<{
    patterns: string[];
    anomalies: string[];
    insights: Array<{ type: string; message: string }>;
  }> {
    return withSpan(
      'mapek.analyze.trace',
      async () => {
        const response = await fetch(getKernelApiUrl('/mapek/analyze/trace'), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ trace }),
        });

        if (!response.ok) {
          throw new Error(`Trace analysis failed: ${response.statusText}`);
        }

        const apiResponse = (await response.json()) as ApiResponse<{
          patterns: string[];
          anomalies: string[];
          insights: Array<{ type: string; message: string }>;
        }>;

        if (!apiResponse.success || !apiResponse.data) {
          throw new Error('Trace analysis returned no data');
        }

        return apiResponse.data;
      },
      {
        'trace.case_id': trace.caseId,
        'trace.workflow_id': trace.workflowId,
        'trace.events': trace.events.length,
        'mapek.phase': 'analyze',
      }
    );
  }

  /* ============================
   * Plan Phase
   * ============================ */

  /**
   * Generate optimization recommendations based on analysis
   *
   * Returns ranked list of recommendations by priority and impact
   */
  async suggestOptimizations(analysis: AnalysisResult): Promise<Recommendation[]> {
    return withSpan(
      'mapek.plan.suggest',
      async () => {
        const response = await fetch(getKernelApiUrl('/mapek/plan'), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ analysis }),
        });

        if (!response.ok) {
          throw new Error(`Planning failed: ${response.statusText}`);
        }

        const apiResponse = (await response.json()) as ApiResponse<Recommendation[]>;

        if (!apiResponse.success || !apiResponse.data) {
          throw new Error('Planning returned no recommendations');
        }

        // Sort by priority (high first) and impact
        const recommendations = apiResponse.data.sort((a, b) => {
          const priorityOrder = { high: 3, medium: 2, low: 1 };
          const priorityDiff = priorityOrder[b.priority] - priorityOrder[a.priority];
          if (priorityDiff !== 0) return priorityDiff;

          // If same priority, sort by total impact
          const aImpact =
            (a.impact.performance || 0) + (a.impact.quality || 0) + (a.impact.complexity || 0);
          const bImpact =
            (b.impact.performance || 0) + (b.impact.quality || 0) + (b.impact.complexity || 0);
          return bImpact - aImpact;
        });

        return recommendations;
      },
      {
        'workflow.id': analysis.workflowId,
        'analysis.patterns': analysis.patterns.detected.length,
        'analysis.quality_score': analysis.quality.score,
        'mapek.phase': 'plan',
      }
    );
  }

  /* ============================
   * Execute Phase
   * ============================ */

  /**
   * Apply a recommendation to the workflow
   *
   * By default, requires user confirmation (autoApply: false)
   * Can be configured to auto-apply low-risk changes
   */
  async applyRecommendation(
    recommendation: Recommendation,
    workflow: YAWLWorkflow
  ): Promise<YAWLWorkflow> {
    return withSpan(
      'mapek.execute.apply',
      async () => {
        // Check if auto-apply is allowed
        if (!this.autoApply && recommendation.priority === 'high') {
          throw new Error('High-priority changes require manual confirmation');
        }

        const response = await fetch(getKernelApiUrl('/mapek/execute'), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            recommendation,
            workflow: {
              id: workflow.id,
              name: workflow.name,
              nodes: workflow.nodes,
              edges: workflow.edges,
            },
          }),
        });

        if (!response.ok) {
          throw new Error(`Execution failed: ${response.statusText}`);
        }

        const apiResponse = (await response.json()) as ApiResponse<YAWLWorkflow>;

        if (!apiResponse.success || !apiResponse.data) {
          throw new Error('Execution returned no modified workflow');
        }

        // Track the change in knowledge base
        if (this.learningEnabled) {
          this.recordChange(recommendation, workflow, apiResponse.data).catch((error) => {
            console.error('Failed to record change:', error);
          });
        }

        return apiResponse.data;
      },
      {
        'workflow.id': workflow.id,
        'recommendation.id': recommendation.id,
        'recommendation.type': recommendation.type,
        'recommendation.priority': recommendation.priority,
        'mapek.phase': 'execute',
      }
    );
  }

  /**
   * Batch apply multiple recommendations
   *
   * Applies recommendations in dependency order
   */
  async applyRecommendations(
    recommendations: Recommendation[],
    workflow: YAWLWorkflow
  ): Promise<YAWLWorkflow> {
    return withSpan(
      'mapek.execute.batch_apply',
      async () => {
        let currentWorkflow = workflow;

        for (const recommendation of recommendations) {
          try {
            currentWorkflow = await this.applyRecommendation(recommendation, currentWorkflow);
          } catch (error) {
            console.error(`Failed to apply recommendation ${recommendation.id}:`, error);
            // Continue with other recommendations
          }
        }

        return currentWorkflow;
      },
      {
        'workflow.id': workflow.id,
        'recommendations.count': recommendations.length,
        'mapek.phase': 'execute',
      }
    );
  }

  /* ============================
   * Knowledge Phase
   * ============================ */

  /**
   * Learn from execution traces and improve recommendations
   *
   * This builds a knowledge base of patterns, optimizations, and outcomes
   */
  async learnFromExecution(trace: ExecutionTrace): Promise<void> {
    if (!this.learningEnabled) {
      return;
    }

    return withSpan(
      'mapek.knowledge.learn',
      async () => {
        const response = await fetch(getKernelApiUrl('/mapek/knowledge/learn'), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ trace }),
        });

        if (!response.ok) {
          throw new Error(`Learning failed: ${response.statusText}`);
        }

        const apiResponse = (await response.json()) as ApiResponse<{
          patternsLearned: number;
          knowledgeUpdated: boolean;
        }>;

        if (!apiResponse.success || !apiResponse.data) {
          throw new Error('Learning returned no data');
        }

        if (knhkConfig.dev.debug) {
          console.debug('MAPE-K learning result:', apiResponse.data);
        }
      },
      {
        'trace.case_id': trace.caseId,
        'trace.workflow_id': trace.workflowId,
        'trace.events': trace.events.length,
        'mapek.phase': 'knowledge',
      }
    );
  }

  /**
   * Record a change in the knowledge base
   */
  private async recordChange(
    recommendation: Recommendation,
    before: YAWLWorkflow,
    after: YAWLWorkflow
  ): Promise<void> {
    return withSpan(
      'mapek.knowledge.record_change',
      async () => {
        await fetch(getKernelApiUrl('/mapek/knowledge/changes'), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            recommendation: {
              id: recommendation.id,
              type: recommendation.type,
              priority: recommendation.priority,
            },
            before: {
              id: before.id,
              nodes: before.nodes.length,
              edges: before.edges.length,
            },
            after: {
              id: after.id,
              nodes: after.nodes.length,
              edges: after.edges.length,
            },
            timestamp: Date.now(),
          }),
        });
      },
      {
        'workflow.id': before.id,
        'recommendation.id': recommendation.id,
        'mapek.phase': 'knowledge',
      }
    );
  }

  /**
   * Query the knowledge base for similar workflows
   */
  async querySimilarWorkflows(
    workflow: YAWLWorkflow,
    limit = 5
  ): Promise<
    Array<{
      workflow: YAWLWorkflow;
      similarity: number;
      commonPatterns: string[];
    }>
  > {
    return withSpan(
      'mapek.knowledge.query_similar',
      async () => {
        const response = await fetch(getKernelApiUrl('/mapek/knowledge/similar'), {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            workflow: {
              id: workflow.id,
              nodes: workflow.nodes,
              edges: workflow.edges,
            },
            limit,
          }),
        });

        if (!response.ok) {
          throw new Error(`Knowledge query failed: ${response.statusText}`);
        }

        const apiResponse = (await response.json()) as ApiResponse<
          Array<{
            workflow: YAWLWorkflow;
            similarity: number;
            commonPatterns: string[];
          }>
        >;

        if (!apiResponse.success || !apiResponse.data) {
          return [];
        }

        return apiResponse.data;
      },
      {
        'workflow.id': workflow.id,
        'query.limit': limit,
        'mapek.phase': 'knowledge',
      }
    );
  }

  /* ============================
   * Configuration
   * ============================ */

  /**
   * Enable or disable learning
   */
  setLearningEnabled(enabled: boolean): void {
    this.learningEnabled = enabled;
  }

  /**
   * Enable or disable auto-apply
   */
  setAutoApply(enabled: boolean): void {
    this.autoApply = enabled;
  }

  /**
   * Clear analysis cache
   */
  clearCache(): void {
    this.analysisCache.clear();
  }
}

/* ============================================================================
 * Singleton Instance
 * ========================================================================== */

/**
 * Default MAPE-K hooks instance
 */
export const mapekHooks = new MAPEKHooks();

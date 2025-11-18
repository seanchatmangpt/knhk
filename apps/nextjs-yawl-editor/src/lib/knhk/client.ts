/**
 * DOCTRINE ALIGNMENT: Π (Projections) + O (Observation)
 * knhk-kernel client for workflow execution
 *
 * COVENANT 6: All operations observable via telemetry
 * Every API call emits spans for observability
 */

import { createSpan, recordError, withSpan } from '@/lib/telemetry/setup';
import { knhkConfig, getKernelApiUrl } from './config';
import type {
  WorkflowId,
  CaseId,
  YAWLSpecification,
  ExecutionEvent,
  ExecutionTrace,
  CaseMetrics,
  PatternPermutations,
  ApiResponse,
  ValidationResult,
} from './types';

/* ============================================================================
 * HTTP Client with Retry Logic
 * ========================================================================== */

/**
 * Execute HTTP request with exponential backoff retry
 */
async function fetchWithRetry<T>(
  url: string,
  options: RequestInit,
  attempt = 1
): Promise<T> {
  const span = createSpan('http.request', {
    'http.url': url,
    'http.method': options.method || 'GET',
    'http.attempt': attempt,
  });

  try {
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      signal: AbortSignal.timeout(knhkConfig.kernel.timeout),
    });

    span.setAttribute('http.status_code', response.status);

    if (!response.ok) {
      // Check if we should retry (5xx errors or 429)
      const shouldRetry =
        (response.status >= 500 || response.status === 429) &&
        attempt < knhkConfig.kernel.retries;

      if (shouldRetry) {
        const delay =
          knhkConfig.kernel.retryDelay * Math.pow(knhkConfig.kernel.retryBackoff, attempt - 1);
        span.setAttribute('retry.delay', delay);
        span.end();

        await new Promise((resolve) => setTimeout(resolve, delay));
        return fetchWithRetry<T>(url, options, attempt + 1);
      }

      const error = await response.text();
      throw new Error(`HTTP ${response.status}: ${error}`);
    }

    const data = await response.json();
    span.end();
    return data as T;
  } catch (error) {
    recordError(span, error as Error);
    throw error;
  }
}

/* ============================================================================
 * KNHKClient Class
 * ========================================================================== */

/**
 * Client for interacting with knhk-kernel
 */
export class KNHKClient {
  constructor(_baseUrl?: string) {
    // baseUrl is currently unused as we use getKernelApiUrl() from config
    // Kept for future extensibility
  }

  /* ============================
   * Workflow Operations
   * ============================ */

  /**
   * Submit a workflow specification to the kernel for execution
   *
   * @example
   * ```ts
   * const workflowId = await client.submitWorkflow({
   *   id: 'wf-123' as WorkflowId,
   *   name: 'Order Processing',
   *   version: '1.0.0',
   *   xml: yawlXmlString,
   *   metadata: { author: 'user', created: new Date().toISOString(), modified: new Date().toISOString() }
   * });
   * ```
   */
  async submitWorkflow(spec: YAWLSpecification): Promise<WorkflowId> {
    return withSpan(
      'kernel.workflow.submit',
      async () => {
        const response = await fetchWithRetry<ApiResponse<{ workflowId: WorkflowId }>>(
          getKernelApiUrl('/workflows'),
          {
            method: 'POST',
            body: JSON.stringify(spec),
          }
        );

        if (!response.success || !response.data) {
          throw new Error(response.error?.message || 'Failed to submit workflow');
        }

        return response.data.workflowId;
      },
      {
        'workflow.id': spec.id,
        'workflow.name': spec.name,
        'workflow.version': spec.version,
      }
    );
  }

  /**
   * Retrieve a workflow specification by ID
   */
  async getWorkflow(id: WorkflowId): Promise<YAWLSpecification> {
    return withSpan(
      'kernel.workflow.get',
      async () => {
        const response = await fetchWithRetry<ApiResponse<YAWLSpecification>>(
          getKernelApiUrl(`/workflows/${id}`),
          { method: 'GET' }
        );

        if (!response.success || !response.data) {
          throw new Error(response.error?.message || 'Failed to get workflow');
        }

        return response.data;
      },
      { 'workflow.id': id }
    );
  }

  /**
   * Validate a workflow specification
   *
   * CRITICAL: Must complete within validationBudget (100ms)
   * Performance requirement from Q4 (Hard Invariants)
   */
  async validateWorkflow(spec: YAWLSpecification): Promise<ValidationResult> {
    const startTime = performance.now();

    return withSpan(
      'kernel.workflow.validate',
      async () => {
        const response = await fetchWithRetry<ApiResponse<ValidationResult>>(
          getKernelApiUrl('/workflows/validate'),
          {
            method: 'POST',
            body: JSON.stringify(spec),
          }
        );

        const duration = performance.now() - startTime;

        // Check performance budget
        if (duration > knhkConfig.performance.validationBudget) {
          console.warn(
            `Validation exceeded budget: ${duration}ms > ${knhkConfig.performance.validationBudget}ms`
          );
        }

        if (!response.success || !response.data) {
          throw new Error(response.error?.message || 'Validation failed');
        }

        return response.data;
      },
      {
        'workflow.id': spec.id,
        'workflow.version': spec.version,
      }
    );
  }

  /**
   * List all workflows in the kernel
   */
  async listWorkflows(filters?: {
    author?: string;
    tags?: string[];
    status?: string;
  }): Promise<YAWLSpecification[]> {
    return withSpan('kernel.workflow.list', async () => {
      const params = new URLSearchParams();
      if (filters?.author) params.set('author', filters.author);
      if (filters?.tags) params.set('tags', filters.tags.join(','));
      if (filters?.status) params.set('status', filters.status);

      const url = getKernelApiUrl('/workflows');
      const fullUrl = params.toString() ? `${url}?${params}` : url;

      const response = await fetchWithRetry<ApiResponse<YAWLSpecification[]>>(fullUrl, {
        method: 'GET',
      });

      if (!response.success || !response.data) {
        throw new Error(response.error?.message || 'Failed to list workflows');
      }

      return response.data;
    });
  }

  /**
   * Delete a workflow from the kernel
   */
  async deleteWorkflow(id: WorkflowId): Promise<void> {
    return withSpan(
      'kernel.workflow.delete',
      async () => {
        const response = await fetchWithRetry<ApiResponse<void>>(
          getKernelApiUrl(`/workflows/${id}`),
          { method: 'DELETE' }
        );

        if (!response.success) {
          throw new Error(response.error?.message || 'Failed to delete workflow');
        }
      },
      { 'workflow.id': id }
    );
  }

  /* ============================
   * Execution Monitoring
   * ============================ */

  /**
   * Start a new workflow case (instance)
   */
  async startCase(
    workflowId: WorkflowId,
    inputData?: Record<string, unknown>
  ): Promise<CaseId> {
    return withSpan(
      'kernel.case.start',
      async () => {
        const response = await fetchWithRetry<ApiResponse<{ caseId: CaseId }>>(
          getKernelApiUrl(`/workflows/${workflowId}/cases`),
          {
            method: 'POST',
            body: JSON.stringify({ inputData }),
          }
        );

        if (!response.success || !response.data) {
          throw new Error(response.error?.message || 'Failed to start case');
        }

        return response.data.caseId;
      },
      {
        'workflow.id': workflowId,
        'case.has_input': !!inputData,
      }
    );
  }

  /**
   * Get execution trace for a workflow case
   *
   * This includes all events, metrics, and detected patterns
   */
  async getExecutionTrace(workflowId: WorkflowId, caseId?: CaseId): Promise<ExecutionTrace> {
    return withSpan(
      'kernel.execution.trace',
      async () => {
        const endpoint = caseId
          ? `/workflows/${workflowId}/cases/${caseId}/trace`
          : `/workflows/${workflowId}/trace`;

        const response = await fetchWithRetry<ApiResponse<ExecutionTrace>>(
          getKernelApiUrl(endpoint),
          { method: 'GET' }
        );

        if (!response.success || !response.data) {
          throw new Error(response.error?.message || 'Failed to get execution trace');
        }

        return response.data;
      },
      {
        'workflow.id': workflowId,
        'case.id': caseId || 'all',
      }
    );
  }

  /**
   * Get metrics for a specific case
   */
  async getCaseMetrics(caseId: CaseId): Promise<CaseMetrics> {
    return withSpan(
      'kernel.case.metrics',
      async () => {
        const response = await fetchWithRetry<ApiResponse<CaseMetrics>>(
          getKernelApiUrl(`/cases/${caseId}/metrics`),
          { method: 'GET' }
        );

        if (!response.success || !response.data) {
          throw new Error(response.error?.message || 'Failed to get case metrics');
        }

        return response.data;
      },
      { 'case.id': caseId }
    );
  }

  /**
   * Stream execution events in real-time
   *
   * @returns EventSource for server-sent events
   */
  streamExecutionEvents(
    workflowId: WorkflowId,
    onEvent: (event: ExecutionEvent) => void,
    onError?: (error: Error) => void
  ): EventSource {
    const url = getKernelApiUrl(`/workflows/${workflowId}/events/stream`);

    const eventSource = new EventSource(url);

    eventSource.onmessage = (e) => {
      try {
        const event = JSON.parse(e.data) as ExecutionEvent;
        onEvent(event);
      } catch (error) {
        console.error('Failed to parse execution event:', error);
        onError?.(error as Error);
      }
    };

    eventSource.onerror = (e) => {
      console.error('EventSource error:', e);
      onError?.(new Error('EventSource connection error'));
    };

    return eventSource;
  }

  /**
   * Cancel a running case
   */
  async cancelCase(caseId: CaseId, reason?: string): Promise<void> {
    return withSpan(
      'kernel.case.cancel',
      async () => {
        const response = await fetchWithRetry<ApiResponse<void>>(
          getKernelApiUrl(`/cases/${caseId}/cancel`),
          {
            method: 'POST',
            body: JSON.stringify({ reason }),
          }
        );

        if (!response.success) {
          throw new Error(response.error?.message || 'Failed to cancel case');
        }
      },
      {
        'case.id': caseId,
        'cancel.reason': reason || 'user_requested',
      }
    );
  }

  /* ============================
   * Pattern Validation
   * ============================ */

  /**
   * Get the pattern permutation matrix from kernel
   *
   * This is used as a fallback if local matrix is unavailable
   */
  async getPatternMatrix(): Promise<PatternPermutations> {
    return withSpan('kernel.pattern.matrix', async () => {
      const response = await fetchWithRetry<ApiResponse<PatternPermutations>>(
        getKernelApiUrl('/patterns/matrix'),
        { method: 'GET' }
      );

      if (!response.success || !response.data) {
        throw new Error(response.error?.message || 'Failed to get pattern matrix');
      }

      return response.data;
    });
  }

  /**
   * Validate a split-join pattern combination
   *
   * CRITICAL: Hot path operation, must complete in ≤8 ticks (Chatman Constant)
   */
  async validatePattern(
    split: string,
    join: string,
    modifiers?: string[]
  ): Promise<boolean> {
    const startTime = performance.now();

    return withSpan(
      'kernel.pattern.validate',
      async () => {
        const response = await fetchWithRetry<ApiResponse<{ valid: boolean }>>(
          getKernelApiUrl('/patterns/validate'),
          {
            method: 'POST',
            body: JSON.stringify({ split, join, modifiers }),
          }
        );

        const duration = performance.now() - startTime;

        // Check hot path budget (8 ticks = ~8ms)
        if (duration > knhkConfig.performance.hotPathBudget) {
          console.warn(
            `Pattern validation exceeded hot path budget: ${duration}ms > ${knhkConfig.performance.hotPathBudget}ms`
          );
        }

        if (!response.success || !response.data) {
          throw new Error(response.error?.message || 'Pattern validation failed');
        }

        return response.data.valid;
      },
      {
        'pattern.split': split,
        'pattern.join': join,
        'pattern.modifiers': modifiers?.join(',') || 'none',
      }
    );
  }

  /* ============================
   * Health Check
   * ============================ */

  /**
   * Check kernel health and connectivity
   */
  async healthCheck(): Promise<{
    status: 'healthy' | 'degraded' | 'unhealthy';
    version: string;
    uptime: number;
  }> {
    return withSpan('kernel.health', async () => {
      const response = await fetchWithRetry<
        ApiResponse<{
          status: 'healthy' | 'degraded' | 'unhealthy';
          version: string;
          uptime: number;
        }>
      >(getKernelApiUrl('/health'), { method: 'GET' });

      if (!response.success || !response.data) {
        throw new Error('Health check failed');
      }

      return response.data;
    });
  }
}

/* ============================================================================
 * Singleton Instance
 * ========================================================================== */

/**
 * Default client instance
 */
export const knhkClient = new KNHKClient();

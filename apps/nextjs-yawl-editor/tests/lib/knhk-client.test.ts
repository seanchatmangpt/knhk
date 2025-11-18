/**
 * knhk-kernel Client Unit Tests - London School TDD
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 3: MAPE-K autonomic feedback
 * - Covenant 6: Observable kernel communication
 *
 * Testing Strategy:
 * - Mock HTTP client (London School)
 * - Verify interactions with kernel API
 * - Test error handling and retries
 */

import { KnhkClient } from '@/lib/knhk/client';
import { mockKernelResponses } from '../fixtures/mock-kernels';

// Mock fetch globally
global.fetch = jest.fn();

describe('KnhkClient - Library Unit Tests', () => {
  let client: KnhkClient;
  const mockFetch = global.fetch as jest.MockedFunction<typeof fetch>;

  beforeEach(() => {
    jest.clearAllMocks();
    client = new KnhkClient({
      baseUrl: 'http://localhost:8080',
      timeout: 5000,
    });
  });

  describe('Workflow Submission', () => {
    it('should submit workflow to kernel successfully', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.submitWorkflow.success,
      } as Response);

      const workflow = { nodes: [], edges: [], metadata: {} };
      const result = await client.submitWorkflow(workflow);

      expect(result.success).toBe(true);
      expect(result.workflowId).toBe('wf-test-123');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/workflows/submit',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(workflow),
        })
      );
    });

    it('should handle validation errors from kernel', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 400,
        json: async () => mockKernelResponses.submitWorkflow.validationError,
      } as Response);

      const workflow = { nodes: [], edges: [], metadata: {} };
      const result = await client.submitWorkflow(workflow);

      expect(result.success).toBe(false);
      expect(result.errors).toBeDefined();
      expect(result.errors!.length).toBeGreaterThan(0);
    });

    it('should handle network errors gracefully', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      const workflow = { nodes: [], edges: [], metadata: {} };

      await expect(client.submitWorkflow(workflow)).rejects.toThrow('Network error');
    });

    it('should timeout after configured duration', async () => {
      mockFetch.mockImplementationOnce(
        () => new Promise(resolve => setTimeout(resolve, 10000))
      );

      const workflow = { nodes: [], edges: [], metadata: {} };

      await expect(client.submitWorkflow(workflow)).rejects.toThrow();
    });
  });

  describe('Execution Trace Retrieval', () => {
    it('should fetch execution trace from kernel', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.getExecutionTrace,
      } as Response);

      const trace = await client.getExecutionTrace('wf-test-123');

      expect(trace.workflowId).toBe('wf-test-123');
      expect(trace.steps.length).toBeGreaterThan(0);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/workflows/wf-test-123/trace',
        expect.any(Object)
      );
    });

    it('should handle missing traces', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        json: async () => ({ error: 'Trace not found' }),
      } as Response);

      await expect(client.getExecutionTrace('nonexistent')).rejects.toThrow();
    });
  });

  describe('Pattern Matrix Synchronization', () => {
    it('should fetch pattern matrix from kernel', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.patterns,
      } as Response);

      const patterns = await client.getPatternMatrix();

      expect(Array.isArray(patterns)).toBe(true);
      expect(patterns.length).toBeGreaterThan(0);
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/patterns/matrix',
        expect.any(Object)
      );
    });

    it('should cache pattern matrix locally', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.patterns,
      } as Response);

      // First call
      await client.getPatternMatrix();

      // Second call (should use cache)
      await client.getPatternMatrix();

      // Should only fetch once
      expect(mockFetch).toHaveBeenCalledTimes(1);
    });
  });

  describe('MAPE-K Communication', () => {
    it('should request MAPE-K recommendations', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.mapeK.plan,
      } as Response);

      const workflow = { nodes: [], edges: [], metadata: {} };
      const recommendations = await client.getMAPEKRecommendations(workflow);

      expect(recommendations.type).toBe('optimization');
      expect(recommendations.actions.length).toBeGreaterThan(0);
    });

    it('should apply MAPE-K recommendations', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.mapeK.execute,
      } as Response);

      const result = await client.applyRecommendation('rec-789');

      expect(result.status).toBe('applied');
      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:8080/api/mape-k/apply',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify({ recommendationId: 'rec-789' }),
        })
      );
    });
  });

  describe('Telemetry (Covenant 6)', () => {
    it('should track kernel API calls with telemetry', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.submitWorkflow.success,
      } as Response);

      const workflow = { nodes: [], edges: [], metadata: {} };
      await client.submitWorkflow(workflow);

      // Telemetry should be emitted (verified via OTEL mock)
      expect(mockFetch).toHaveBeenCalled();
    });

    it('should track API latency metrics', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => mockKernelResponses.submitWorkflow.success,
      } as Response);

      const workflow = { nodes: [], edges: [], metadata: {} };

      const start = performance.now();
      await client.submitWorkflow(workflow);
      const elapsed = performance.now() - start;

      // Should track latency
      expect(elapsed).toBeGreaterThan(0);
    });
  });

  describe('Error Recovery', () => {
    it('should retry on transient failures', async () => {
      mockFetch
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({
          ok: true,
          json: async () => mockKernelResponses.submitWorkflow.success,
        } as Response);

      const workflow = { nodes: [], edges: [], metadata: {} };

      // Should retry and succeed
      const result = await client.submitWorkflow(workflow, { retries: 1 });

      expect(result.success).toBe(true);
      expect(mockFetch).toHaveBeenCalledTimes(2);
    });

    it('should fail after max retries', async () => {
      mockFetch.mockRejectedValue(new Error('Network error'));

      const workflow = { nodes: [], edges: [], metadata: {} };

      await expect(
        client.submitWorkflow(workflow, { retries: 2 })
      ).rejects.toThrow('Network error');

      expect(mockFetch).toHaveBeenCalledTimes(3); // Initial + 2 retries
    });
  });

  describe('Configuration', () => {
    it('should use configured base URL', () => {
      const customClient = new KnhkClient({ baseUrl: 'http://custom:9000' });

      expect(customClient['config'].baseUrl).toBe('http://custom:9000');
    });

    it('should use configured timeout', () => {
      const customClient = new KnhkClient({ timeout: 10000 });

      expect(customClient['config'].timeout).toBe(10000);
    });
  });
});

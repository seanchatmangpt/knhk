/**
 * Chicago TDD Performance Tests - Validation Latency
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 5: Chatman Constant (≤8 ticks / ≤8ms hot path)
 * - Covenant 2: Q4 (Latency SLOs: hot path ≤8ms, warm path ≤100ms)
 *
 * Performance Testing Strategy:
 * - Measure actual latency, not mocked behavior
 * - Test under various workflow sizes
 * - Verify SLO compliance at p50, p95, p99
 */

import { validateWorkflow } from '@/lib/validation/pattern-validator';
import {
  simpleSequenceWorkflow,
  parallelWorkflow,
  exclusiveChoiceWorkflow,
  complexPerformanceWorkflow,
} from '../fixtures/sample-workflows';

/**
 * Run validation N times and collect latency statistics
 */
function benchmark(name: string, workflow: any, iterations: number = 100) {
  const latencies: number[] = [];

  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    validateWorkflow(workflow);
    const elapsed = performance.now() - start;
    latencies.push(elapsed);
  }

  latencies.sort((a, b) => a - b);

  return {
    name,
    iterations,
    p50: latencies[Math.floor(iterations * 0.5)],
    p95: latencies[Math.floor(iterations * 0.95)],
    p99: latencies[Math.floor(iterations * 0.99)],
    max: latencies[iterations - 1],
    avg: latencies.reduce((sum, lat) => sum + lat, 0) / iterations,
  };
}

describe('Chicago TDD - Validation Performance Tests', () => {
  describe('Covenant 5: Chatman Constant (≤8ms Hot Path)', () => {
    it('should validate simple workflow in ≤8ms (p99)', () => {
      const stats = benchmark('Simple Sequence', simpleSequenceWorkflow, 100);

      console.log(`
        Simple Workflow Validation:
        - p50: ${stats.p50.toFixed(2)}ms
        - p95: ${stats.p95.toFixed(2)}ms
        - p99: ${stats.p99.toFixed(2)}ms
        - max: ${stats.max.toFixed(2)}ms
        - avg: ${stats.avg.toFixed(2)}ms
      `);

      // Hot path MUST satisfy Chatman constant
      expect(stats.p99).toBeLessThan(8);
      expect(stats.avg).toBeLessThan(4); // Should be well under limit
    });

    it('should validate parallel workflow in ≤8ms (p99)', () => {
      const stats = benchmark('Parallel Split', parallelWorkflow, 100);

      console.log(`
        Parallel Workflow Validation:
        - p50: ${stats.p50.toFixed(2)}ms
        - p95: ${stats.p95.toFixed(2)}ms
        - p99: ${stats.p99.toFixed(2)}ms
        - max: ${stats.max.toFixed(2)}ms
        - avg: ${stats.avg.toFixed(2)}ms
      `);

      expect(stats.p99).toBeLessThan(8);
    });

    it('should validate exclusive choice in ≤8ms (p99)', () => {
      const stats = benchmark('Exclusive Choice', exclusiveChoiceWorkflow, 100);

      console.log(`
        XOR Workflow Validation:
        - p50: ${stats.p50.toFixed(2)}ms
        - p95: ${stats.p95.toFixed(2)}ms
        - p99: ${stats.p99.toFixed(2)}ms
        - max: ${stats.max.toFixed(2)}ms
        - avg: ${stats.avg.toFixed(2)}ms
      `);

      expect(stats.p99).toBeLessThan(8);
    });
  });

  describe('Covenant 2: Q4 Latency SLOs', () => {
    it('should validate 50-node workflow in ≤100ms (warm path)', () => {
      const stats = benchmark('Complex Workflow (50 nodes)', complexPerformanceWorkflow, 50);

      console.log(`
        Complex Workflow Validation (50 nodes):
        - p50: ${stats.p50.toFixed(2)}ms
        - p95: ${stats.p95.toFixed(2)}ms
        - p99: ${stats.p99.toFixed(2)}ms
        - max: ${stats.max.toFixed(2)}ms
        - avg: ${stats.avg.toFixed(2)}ms
      `);

      // Warm path SLO: ≤100ms
      expect(stats.p99).toBeLessThan(100);
    });

    it('should validate large workflow (200 nodes) in ≤500ms', () => {
      // Generate 200-node workflow
      const largeWorkflow = {
        nodes: Array.from({ length: 200 }, (_, i) => ({
          id: `node-${i}`,
          type: i === 0 ? 'start' : i === 199 ? 'end' : 'task',
          position: { x: i * 10, y: i * 10 },
          data: {},
        })),
        edges: Array.from({ length: 199 }, (_, i) => ({
          id: `e${i}`,
          source: `node-${i}`,
          target: `node-${i + 1}`,
        })),
        metadata: { name: 'Large Workflow' },
      };

      const stats = benchmark('Large Workflow (200 nodes)', largeWorkflow, 20);

      console.log(`
        Large Workflow Validation (200 nodes):
        - p50: ${stats.p50.toFixed(2)}ms
        - p95: ${stats.p95.toFixed(2)}ms
        - p99: ${stats.p99.toFixed(2)}ms
        - max: ${stats.max.toFixed(2)}ms
        - avg: ${stats.avg.toFixed(2)}ms
      `);

      // Cold path SLO: ≤500ms
      expect(stats.p99).toBeLessThan(500);
    });
  });

  describe('Worst-Case Scenarios', () => {
    it('should handle invalid workflows without performance degradation', () => {
      const invalidWorkflow = {
        nodes: [
          { id: 'start-1', type: 'start', position: { x: 0, y: 0 }, data: {} },
          { id: 'task-1', type: 'task', position: { x: 100, y: 0 }, data: {} },
          // Missing end node - invalid!
        ],
        edges: [{ id: 'e1', source: 'start-1', target: 'task-1' }],
        metadata: {},
      };

      const stats = benchmark('Invalid Workflow', invalidWorkflow, 100);

      console.log(`
        Invalid Workflow Validation:
        - p50: ${stats.p50.toFixed(2)}ms
        - p95: ${stats.p95.toFixed(2)}ms
        - p99: ${stats.p99.toFixed(2)}ms
      `);

      // Error detection should be just as fast
      expect(stats.p99).toBeLessThan(8);
    });

    it('should handle deeply nested patterns efficiently', () => {
      // Create nested splits/joins
      const nestedWorkflow = {
        nodes: [
          { id: 'start', type: 'start', position: { x: 0, y: 0 }, data: {} },
          { id: 'split-1', type: 'condition', position: { x: 100, y: 0 }, data: { splitType: 'and' } },
          { id: 'split-2', type: 'condition', position: { x: 200, y: 0 }, data: { splitType: 'and' } },
          { id: 'task-1', type: 'task', position: { x: 300, y: -50 }, data: {} },
          { id: 'task-2', type: 'task', position: { x: 300, y: 50 }, data: {} },
          { id: 'join-2', type: 'condition', position: { x: 400, y: 0 }, data: { joinType: 'and' } },
          { id: 'join-1', type: 'condition', position: { x: 500, y: 0 }, data: { joinType: 'and' } },
          { id: 'end', type: 'end', position: { x: 600, y: 0 }, data: {} },
        ],
        edges: [
          { id: 'e1', source: 'start', target: 'split-1' },
          { id: 'e2', source: 'split-1', target: 'split-2' },
          { id: 'e3', source: 'split-1', target: 'join-1' },
          { id: 'e4', source: 'split-2', target: 'task-1' },
          { id: 'e5', source: 'split-2', target: 'task-2' },
          { id: 'e6', source: 'task-1', target: 'join-2' },
          { id: 'e7', source: 'task-2', target: 'join-2' },
          { id: 'e8', source: 'join-2', target: 'join-1' },
          { id: 'e9', source: 'join-1', target: 'end' },
        ],
        metadata: { name: 'Nested Patterns' },
      };

      const stats = benchmark('Nested Patterns', nestedWorkflow, 100);

      console.log(`
        Nested Patterns Validation:
        - p50: ${stats.p50.toFixed(2)}ms
        - p95: ${stats.p95.toFixed(2)}ms
        - p99: ${stats.p99.toFixed(2)}ms
      `);

      expect(stats.p99).toBeLessThan(8);
    });
  });

  describe('Latency Consistency', () => {
    it('should have low latency variance (consistent performance)', () => {
      const stats = benchmark('Consistency Test', simpleSequenceWorkflow, 100);

      const variance = stats.max - stats.p50;

      console.log(`
        Latency Variance:
        - p50: ${stats.p50.toFixed(2)}ms
        - max: ${stats.max.toFixed(2)}ms
        - variance: ${variance.toFixed(2)}ms
      `);

      // Variance should be < 5ms for consistent performance
      expect(variance).toBeLessThan(5);
    });
  });
});

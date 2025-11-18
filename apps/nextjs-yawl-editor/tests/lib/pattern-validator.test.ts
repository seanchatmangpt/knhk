/**
 * Pattern Validator Unit Tests - London School TDD
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 2: Q invariants are enforced, not warned
 * - Covenant 5: ≤8ms validation latency (Chatman Constant)
 *
 * Testing Strategy:
 * - Behavior-focused: Does validator enforce patterns correctly?
 * - Performance: Chicago TDD latency constraints
 * - No mocks needed: validator is a pure function
 */

import { validateWorkflow } from '@/lib/validation/pattern-validator';
import { allValidationCases } from '../fixtures/validation-cases';
import { allTestWorkflows } from '../fixtures/sample-workflows';

describe('PatternValidator - Library Unit Tests', () => {
  describe('Covenant 2: Q Invariants Enforcement', () => {
    describe('Valid Patterns', () => {
      it('should accept valid sequence pattern (Pattern 1)', () => {
        const result = validateWorkflow(allTestWorkflows.simpleSequence);

        expect(result.valid).toBe(true);
        expect(result.errors).toHaveLength(0);
      });

      it('should accept valid parallel split pattern (Pattern 2)', () => {
        const result = validateWorkflow(allTestWorkflows.parallel);

        expect(result.valid).toBe(true);
        expect(result.errors).toHaveLength(0);
      });

      it('should accept valid exclusive choice pattern (Pattern 4)', () => {
        const result = validateWorkflow(allTestWorkflows.exclusiveChoice);

        expect(result.valid).toBe(true);
        expect(result.errors).toHaveLength(0);
      });
    });

    describe('Invalid Patterns - Hard Rejection', () => {
      it('should REJECT workflow missing end condition', () => {
        const result = validateWorkflow(allTestWorkflows.invalidMissingEnd);

        expect(result.valid).toBe(false);
        expect(result.errors.length).toBeGreaterThan(0);
        expect(result.errors.some(e => e.code.includes('END'))).toBe(true);
      });

      it('should REJECT workflow with unmatched split/join', () => {
        const result = validateWorkflow(allTestWorkflows.invalidUnmatchedSplit);

        expect(result.valid).toBe(false);
        expect(result.errors.length).toBeGreaterThan(0);
      });

      it('should REJECT empty workflow', () => {
        const result = validateWorkflow({ nodes: [], edges: [], metadata: {} });

        expect(result.valid).toBe(false);
        expect(result.errors.some(e => e.code === 'NO_START_NODE')).toBe(true);
      });
    });

    describe('Structural Soundness (Q1)', () => {
      it('should validate exactly one start node exists', () => {
        const workflow = {
          nodes: [
            { id: 'end-1', type: 'end', position: { x: 0, y: 0 }, data: {} },
          ],
          edges: [],
          metadata: {},
        };

        const result = validateWorkflow(workflow);

        expect(result.valid).toBe(false);
        expect(result.errors.some(e => e.code === 'NO_START_NODE')).toBe(true);
      });

      it('should reject multiple start nodes', () => {
        const workflow = {
          nodes: [
            { id: 'start-1', type: 'start', position: { x: 0, y: 0 }, data: {} },
            { id: 'start-2', type: 'start', position: { x: 100, y: 0 }, data: {} },
            { id: 'end-1', type: 'end', position: { x: 200, y: 0 }, data: {} },
          ],
          edges: [],
          metadata: {},
        };

        const result = validateWorkflow(workflow);

        expect(result.valid).toBe(false);
        expect(result.errors.some(e => e.code === 'MULTIPLE_START_NODES')).toBe(true);
      });

      it('should validate at least one end node exists', () => {
        const workflow = {
          nodes: [
            { id: 'start-1', type: 'start', position: { x: 0, y: 0 }, data: {} },
            { id: 'task-1', type: 'task', position: { x: 100, y: 0 }, data: {} },
          ],
          edges: [
            { id: 'e1', source: 'start-1', target: 'task-1' },
          ],
          metadata: {},
        };

        const result = validateWorkflow(workflow);

        expect(result.valid).toBe(false);
        expect(result.errors.some(e => e.code === 'NO_END_NODE')).toBe(true);
      });

      it('should detect unreachable nodes', () => {
        const workflow = {
          nodes: [
            { id: 'start-1', type: 'start', position: { x: 0, y: 0 }, data: {} },
            { id: 'task-1', type: 'task', position: { x: 100, y: 0 }, data: {} },
            { id: 'task-2', type: 'task', position: { x: 100, y: 100 }, data: {} }, // Unreachable!
            { id: 'end-1', type: 'end', position: { x: 200, y: 0 }, data: {} },
          ],
          edges: [
            { id: 'e1', source: 'start-1', target: 'task-1' },
            { id: 'e2', source: 'task-1', target: 'end-1' },
          ],
          metadata: {},
        };

        const result = validateWorkflow(workflow);

        expect(result.valid).toBe(false);
        expect(result.errors.some(e => e.code === 'UNREACHABLE_NODES')).toBe(true);
      });

      it('should detect dangling edges', () => {
        const workflow = {
          nodes: [
            { id: 'start-1', type: 'start', position: { x: 0, y: 0 }, data: {} },
            { id: 'end-1', type: 'end', position: { x: 100, y: 0 }, data: {} },
          ],
          edges: [
            { id: 'e1', source: 'start-1', target: 'nonexistent' }, // Dangling!
          ],
          metadata: {},
        };

        const result = validateWorkflow(workflow);

        expect(result.valid).toBe(false);
        expect(result.errors.some(e => e.code === 'DANGLING_EDGES')).toBe(true);
      });
    });
  });

  describe('Covenant 5: Chicago TDD - Performance Constraints', () => {
    it('should validate simple workflow in ≤8ms (Chatman Constant)', () => {
      const start = performance.now();

      validateWorkflow(allTestWorkflows.simpleSequence);

      const elapsed = performance.now() - start;

      // Hot path validation MUST be ≤8ms
      expect(elapsed).toBeLessThan(8);
      console.log(`✓ Simple validation: ${elapsed.toFixed(2)}ms (limit: 8ms)`);
    });

    it('should validate parallel workflow in ≤8ms', () => {
      const start = performance.now();

      validateWorkflow(allTestWorkflows.parallel);

      const elapsed = performance.now() - start;

      expect(elapsed).toBeLessThan(8);
      console.log(`✓ Parallel validation: ${elapsed.toFixed(2)}ms (limit: 8ms)`);
    });

    it('should validate complex workflow in ≤100ms (warm path)', () => {
      const start = performance.now();

      validateWorkflow(allTestWorkflows.complexPerformance);

      const elapsed = performance.now() - start;

      // Warm path SLO: ≤100ms
      expect(elapsed).toBeLessThan(100);
      console.log(`✓ Complex validation: ${elapsed.toFixed(2)}ms (limit: 100ms)`);
    });

    it('should handle validation errors without performance degradation', () => {
      const start = performance.now();

      validateWorkflow(allTestWorkflows.invalidMissingEnd);

      const elapsed = performance.now() - start;

      // Error detection must also be fast
      expect(elapsed).toBeLessThan(8);
      console.log(`✓ Error detection: ${elapsed.toFixed(2)}ms (limit: 8ms)`);
    });
  });

  describe('Pattern-Specific Validation', () => {
    it('should validate AND-split requires AND-join', () => {
      // Tested via invalidUnmatchedSplit workflow
      const result = validateWorkflow(allTestWorkflows.invalidUnmatchedSplit);

      expect(result.valid).toBe(false);
    });

    it('should validate XOR-split branches have conditions', () => {
      const workflow = allTestWorkflows.exclusiveChoice;

      const result = validateWorkflow(workflow);

      // Should be valid because conditions are present
      expect(result.valid).toBe(true);
    });
  });

  describe('Edge Cases', () => {
    it('should handle minimal valid workflow (start → end)', () => {
      const workflow = {
        nodes: [
          { id: 'start-1', type: 'start', position: { x: 0, y: 0 }, data: {} },
          { id: 'end-1', type: 'end', position: { x: 100, y: 0 }, data: {} },
        ],
        edges: [
          { id: 'e1', source: 'start-1', target: 'end-1' },
        ],
        metadata: {},
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should handle workflow with no edges', () => {
      const workflow = {
        nodes: [
          { id: 'start-1', type: 'start', position: { x: 0, y: 0 }, data: {} },
        ],
        edges: [],
        metadata: {},
      };

      const result = validateWorkflow(workflow);

      expect(result.valid).toBe(false);
    });
  });

  describe('Error Message Quality', () => {
    it('should provide actionable error messages', () => {
      const result = validateWorkflow(allTestWorkflows.invalidMissingEnd);

      result.errors.forEach(error => {
        expect(error.message).toBeTruthy();
        expect(error.code).toBeTruthy();
        expect(error.severity).toBeTruthy();
      });
    });

    it('should categorize errors by severity', () => {
      const result = validateWorkflow(allTestWorkflows.invalidUnmatchedSplit);

      result.errors.forEach(error => {
        expect(['error', 'warning']).toContain(error.severity);
      });
    });
  });
});

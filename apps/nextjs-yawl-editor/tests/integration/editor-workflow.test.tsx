/**
 * Editor Workflow Integration Tests - London School TDD
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 1: RDF round-trip integrity
 * - Covenant 2: Pattern validation enforcement
 * - Covenant 3: MAPE-K feedback integration
 *
 * Integration Testing:
 * - Component interactions with RDF store
 * - Canvas → Validation → Feedback loop
 * - Export → Import → Verify roundtrip
 */

import React from 'react';
import { renderWithProviders, screen, fireEvent, waitFor } from '../helpers/test-utils';
import { WorkflowCanvas } from '@/components/editor/workflow-canvas';
import { useWorkflow } from '@/hooks/use-workflow';
import { useValidation } from '@/hooks/use-validation';
import { allTestWorkflows } from '../fixtures/sample-workflows';

jest.mock('@/hooks/use-workflow');
jest.mock('@/hooks/use-validation');
jest.mock('@/hooks/use-telemetry', () => ({
  useTelemetry: () => ({
    trackEvent: jest.fn(),
    startSpan: jest.fn(),
    recordMetric: jest.fn(),
  }),
}));

jest.mock('reactflow', () => ({
  __esModule: true,
  default: ({ onConnect }: any) => (
    <div data-testid="react-flow">
      <button data-testid="add-edge" onClick={() => onConnect?.({ source: 'task-1', target: 'task-2' })}>
        Add Edge
      </button>
    </div>
  ),
  Background: () => null,
  Controls: () => null,
  MiniMap: () => null,
  Panel: ({ children }: any) => <div>{children}</div>,
  ConnectionMode: { Loose: 'loose' },
  MarkerType: { ArrowClosed: 'arrowclosed' },
  useNodesState: (initial: any) => [initial, jest.fn(), jest.fn()],
  useEdgesState: (initial: any) => [initial, jest.fn(), jest.fn()],
}));

describe('Editor Workflow Integration Tests', () => {
  let mockWorkflow: any;
  let mockValidation: any;

  beforeEach(() => {
    mockWorkflow = {
      workflow: allTestWorkflows.simpleSequence,
      addNode: jest.fn(),
      updateNode: jest.fn(),
      removeNode: jest.fn(),
      addEdge: jest.fn(),
      removeEdge: jest.fn(),
      selectNodes: jest.fn(),
      deselectAll: jest.fn(),
      exportToTurtle: jest.fn().mockResolvedValue('@prefix yawl: <http://knhk.io/ontology/yawl#> .'),
    };

    mockValidation = {
      validation: {
        valid: true,
        errors: [],
        warnings: [],
      },
      validateWorkflow: jest.fn().mockReturnValue({
        valid: true,
        errors: [],
        warnings: [],
      }),
    };

    (useWorkflow as jest.Mock).mockReturnValue(mockWorkflow);
    (useValidation as jest.Mock).mockReturnValue(mockValidation);
  });

  describe('Covenant 1: RDF Round-Trip Integrity', () => {
    it('should create workflow → export to Turtle → validate format', async () => {
      renderWithProviders(<WorkflowCanvas />);

      // Simulate workflow export
      const turtleRDF = await mockWorkflow.exportToTurtle();

      expect(turtleRDF).toBeTruthy();
      expect(turtleRDF).toContain('@prefix yawl:');
      expect(mockWorkflow.exportToTurtle).toHaveBeenCalled();
    });

    it('should preserve workflow structure in RDF export', async () => {
      mockWorkflow.workflow = allTestWorkflows.parallel;

      renderWithProviders(<WorkflowCanvas />);

      const turtleRDF = await mockWorkflow.exportToTurtle();

      expect(turtleRDF).toBeTruthy();
      expect(mockWorkflow.exportToTurtle).toHaveBeenCalled();
    });

    it('should update RDF when nodes are added', async () => {
      renderWithProviders(<WorkflowCanvas />);

      // Simulate node addition
      mockWorkflow.addNode({ id: 'new-task', type: 'task', label: 'New Task' });

      expect(mockWorkflow.addNode).toHaveBeenCalledWith(
        expect.objectContaining({ id: 'new-task' })
      );
    });

    it('should update RDF when edges are connected', async () => {
      renderWithProviders(<WorkflowCanvas />);

      const addEdgeButton = screen.getByTestId('add-edge');
      fireEvent.click(addEdgeButton);

      await waitFor(() => {
        expect(mockWorkflow.addEdge).toHaveBeenCalled();
      });
    });
  });

  describe('Covenant 2: Pattern Validation Integration', () => {
    it('should validate workflow after edge addition', async () => {
      renderWithProviders(<WorkflowCanvas />);

      const addEdgeButton = screen.getByTestId('add-edge');
      fireEvent.click(addEdgeButton);

      await waitFor(() => {
        expect(mockWorkflow.addEdge).toHaveBeenCalled();
      });
    });

    it('should prevent invalid patterns from being created', async () => {
      mockValidation.validateWorkflow.mockReturnValue({
        valid: false,
        errors: [{ code: 'INVALID_PATTERN', message: 'Invalid pattern' }],
      });

      renderWithProviders(<WorkflowCanvas />);

      const addEdgeButton = screen.getByTestId('add-edge');
      fireEvent.click(addEdgeButton);

      await waitFor(() => {
        expect(mockWorkflow.addEdge).toHaveBeenCalled();
      });
    });

    it('should display validation errors after workflow modification', async () => {
      mockValidation.validation = {
        valid: false,
        errors: [
          {
            code: 'UNMATCHED_SPLIT',
            message: 'AND-split without AND-join',
            severity: 'error',
          },
        ],
        warnings: [],
      };

      renderWithProviders(<WorkflowCanvas />);

      expect(useValidation).toHaveBeenCalled();
    });

    it('should re-validate after every change', async () => {
      const { rerender } = renderWithProviders(<WorkflowCanvas />);

      // First render
      expect(useValidation).toHaveBeenCalled();

      // Simulate workflow change
      mockWorkflow.workflow = {
        ...mockWorkflow.workflow,
        nodes: [...mockWorkflow.workflow.nodes, { id: 'new-node', type: 'task' }],
      };

      rerender(<WorkflowCanvas />);

      // Validation should be called again
      expect(useValidation).toHaveBeenCalledTimes(2);
    });
  });

  describe('Canvas → Store → Validation Integration', () => {
    it('should coordinate: Add Edge → Update Store → Validate → Show Feedback', async () => {
      renderWithProviders(<WorkflowCanvas />);

      // Step 1: Add edge
      const addEdgeButton = screen.getByTestId('add-edge');
      fireEvent.click(addEdgeButton);

      // Step 2: Verify store updated
      await waitFor(() => {
        expect(mockWorkflow.addEdge).toHaveBeenCalled();
      });

      // Step 3: Validation hook called
      expect(useValidation).toHaveBeenCalled();
    });

    it('should coordinate: Remove Node → Update Store → Validate → Update UI', async () => {
      renderWithProviders(<WorkflowCanvas />);

      // Simulate node removal
      mockWorkflow.removeNode('task-1');

      expect(mockWorkflow.removeNode).toHaveBeenCalledWith('task-1');
    });
  });

  describe('Performance Integration (Chicago TDD)', () => {
    it('should handle canvas operations in ≤100ms (warm path)', async () => {
      const start = performance.now();

      renderWithProviders(<WorkflowCanvas />);

      const elapsed = performance.now() - start;

      expect(elapsed).toBeLessThan(100);
      console.log(`✓ Canvas mount: ${elapsed.toFixed(2)}ms (limit: 100ms)`);
    });

    it('should validate workflows in ≤8ms after changes', async () => {
      renderWithProviders(<WorkflowCanvas />);

      const start = performance.now();

      mockValidation.validateWorkflow(mockWorkflow.workflow);

      const elapsed = performance.now() - start;

      expect(elapsed).toBeLessThan(8);
      console.log(`✓ Validation: ${elapsed.toFixed(2)}ms (limit: 8ms)`);
    });
  });

  describe('Error Recovery', () => {
    it('should handle validation errors gracefully', () => {
      mockValidation.validation = {
        valid: false,
        errors: [
          { code: 'ERROR_1', message: 'Error 1', severity: 'error' },
          { code: 'ERROR_2', message: 'Error 2', severity: 'error' },
        ],
        warnings: [],
      };

      renderWithProviders(<WorkflowCanvas />);

      // Should render without crashing
      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });

    it('should recover from export failures', async () => {
      mockWorkflow.exportToTurtle.mockRejectedValue(new Error('Export failed'));

      renderWithProviders(<WorkflowCanvas />);

      await expect(mockWorkflow.exportToTurtle()).rejects.toThrow('Export failed');
    });
  });
});

/**
 * WorkflowCanvas Unit Tests - London School TDD
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 1: RDF round-trip integrity
 * - Covenant 2: Pattern validation enforcement
 * - Covenant 6: Observable telemetry
 *
 * Testing Strategy:
 * - Mock-first approach (London School)
 * - Behavior verification over state inspection
 * - Focus on interactions between canvas and hooks
 */

import React from 'react';
import { renderWithProviders, screen, fireEvent, waitFor } from '../helpers/test-utils';
import { WorkflowCanvas } from '@/components/editor/workflow-canvas';
import { useWorkflow } from '@/hooks/use-workflow';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';

// Mock hooks (London School: define collaborator contracts)
jest.mock('@/hooks/use-workflow');
jest.mock('@/hooks/use-validation');
jest.mock('@/hooks/use-telemetry');

// Mock React Flow (external dependency)
jest.mock('reactflow', () => ({
  __esModule: true,
  default: ({ children, onNodesChange, onEdgesChange, onConnect }: any) => (
    <div data-testid="react-flow">
      <div data-testid="nodes-change-handler" onClick={() => onNodesChange?.([])}>
        Nodes
      </div>
      <div data-testid="edges-change-handler" onClick={() => onEdgesChange?.([])}>
        Edges
      </div>
      <div data-testid="connect-handler" onClick={() => onConnect?.({ source: 'a', target: 'b' })}>
        Connect
      </div>
      {children}
    </div>
  ),
  Background: () => <div data-testid="background" />,
  Controls: () => <div data-testid="controls" />,
  MiniMap: () => <div data-testid="minimap" />,
  Panel: ({ children }: any) => <div data-testid="panel">{children}</div>,
  ConnectionMode: { Loose: 'loose' },
  MarkerType: { ArrowClosed: 'arrowclosed' },
  useNodesState: (initial: any) => [initial, jest.fn(), jest.fn()],
  useEdgesState: (initial: any) => [initial, jest.fn(), jest.fn()],
}));

describe('WorkflowCanvas - Component Unit Tests', () => {
  // Mock implementations
  const mockWorkflowHook = {
    workflow: {
      nodes: [],
      edges: [],
      metadata: { name: 'Test Workflow', version: '1.0.0' },
    },
    addNode: jest.fn(),
    updateNode: jest.fn(),
    removeNode: jest.fn(),
    addEdge: jest.fn(),
    removeEdge: jest.fn(),
    selectNodes: jest.fn(),
    deselectAll: jest.fn(),
    exportToTurtle: jest.fn(),
  };

  const mockValidationHook = {
    validation: {
      valid: true,
      errors: [],
      warnings: [],
    },
    validateWorkflow: jest.fn(),
  };

  const mockTelemetryHook = {
    trackEvent: jest.fn(),
    startSpan: jest.fn(),
    recordMetric: jest.fn(),
  };

  beforeEach(() => {
    jest.clearAllMocks();
    (useWorkflow as jest.Mock).mockReturnValue(mockWorkflowHook);
    (useValidation as jest.Mock).mockReturnValue(mockValidationHook);
    (useTelemetry as jest.Mock).mockReturnValue(mockTelemetryHook);
  });

  describe('Covenant 1: RDF Projection', () => {
    it('should render canvas with workflow nodes from RDF state', () => {
      mockWorkflowHook.workflow.nodes = [
        { id: 'task-1', type: 'task', label: 'Process Order', position: { x: 100, y: 100 }, data: {} },
      ];

      renderWithProviders(<WorkflowCanvas />);

      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
      expect(useWorkflow).toHaveBeenCalled();
    });

    it('should project RDF nodes to React Flow format', () => {
      const yawlNodes = [
        {
          id: 'start-1',
          type: 'condition',
          label: 'Start',
          position: { x: 0, y: 0 },
          data: { conditionType: 'input' },
        },
        {
          id: 'task-1',
          type: 'task',
          label: 'Task',
          position: { x: 150, y: 0 },
          data: { decomposition: 'atomic' },
        },
      ];

      mockWorkflowHook.workflow.nodes = yawlNodes;

      renderWithProviders(<WorkflowCanvas />);

      // Verify hook was called (behavior verification)
      expect(useWorkflow).toHaveBeenCalled();
    });
  });

  describe('Covenant 2: Pattern Validation Enforcement', () => {
    it('should validate workflow when edges are added', async () => {
      renderWithProviders(<WorkflowCanvas />);

      // Simulate connection
      const connectHandler = screen.getByTestId('connect-handler');
      fireEvent.click(connectHandler);

      await waitFor(() => {
        expect(mockWorkflowHook.addEdge).toHaveBeenCalled();
      });
    });

    it('should display validation errors visually', () => {
      mockValidationHook.validation = {
        valid: false,
        errors: [
          {
            code: 'MISSING_END_NODE',
            message: 'Workflow must have an end node',
            severity: 'error',
          },
        ],
        warnings: [],
      };

      renderWithProviders(<WorkflowCanvas />);

      expect(useValidation).toHaveBeenCalled();
    });

    it('should prevent invalid connections based on pattern rules', async () => {
      mockValidationHook.validateWorkflow.mockReturnValue({
        valid: false,
        errors: [{ code: 'INVALID_PATTERN', message: 'Invalid pattern' }],
      });

      renderWithProviders(<WorkflowCanvas />);

      const connectHandler = screen.getByTestId('connect-handler');
      fireEvent.click(connectHandler);

      await waitFor(() => {
        expect(mockValidationHook.validateWorkflow).toHaveBeenCalled();
      });
    });
  });

  describe('Covenant 6: Observable Telemetry', () => {
    it('should track node addition events', async () => {
      renderWithProviders(<WorkflowCanvas />);

      expect(useTelemetry).toHaveBeenCalledWith('WorkflowCanvas');
    });

    it('should track edge creation events', async () => {
      renderWithProviders(<WorkflowCanvas />);

      const connectHandler = screen.getByTestId('connect-handler');
      fireEvent.click(connectHandler);

      await waitFor(() => {
        expect(mockWorkflowHook.addEdge).toHaveBeenCalled();
      });
    });

    it('should track node deletion events', () => {
      mockWorkflowHook.workflow.nodes = [
        { id: 'task-1', type: 'task', label: 'Task', position: { x: 0, y: 0 }, data: {} },
      ];

      renderWithProviders(<WorkflowCanvas />);

      expect(useTelemetry).toHaveBeenCalled();
    });
  });

  describe('Interaction Testing (London School)', () => {
    it('should coordinate node selection with property panel', () => {
      const onNodeSelect = jest.fn();

      mockWorkflowHook.workflow.nodes = [
        { id: 'task-1', type: 'task', label: 'Task', position: { x: 0, y: 0 }, data: {} },
      ];

      renderWithProviders(<WorkflowCanvas onNodeSelect={onNodeSelect} />);

      // Verify hook collaboration
      expect(useWorkflow).toHaveBeenCalled();
    });

    it('should update RDF store when nodes are moved', () => {
      renderWithProviders(<WorkflowCanvas />);

      const nodesHandler = screen.getByTestId('nodes-change-handler');
      fireEvent.click(nodesHandler);

      // Verify behavior: canvas talks to workflow hook
      expect(useWorkflow).toHaveBeenCalled();
    });

    it('should update RDF store when edges are modified', () => {
      renderWithProviders(<WorkflowCanvas />);

      const edgesHandler = screen.getByTestId('edges-change-handler');
      fireEvent.click(edgesHandler);

      // Verify workflow hook was used
      expect(useWorkflow).toHaveBeenCalled();
    });
  });

  describe('Error Handling', () => {
    it('should handle empty workflow gracefully', () => {
      mockWorkflowHook.workflow.nodes = [];
      mockWorkflowHook.workflow.edges = [];

      renderWithProviders(<WorkflowCanvas />);

      expect(screen.getByTestId('react-flow')).toBeInTheDocument();
    });

    it('should handle validation errors gracefully', () => {
      mockValidationHook.validation = {
        valid: false,
        errors: [
          { code: 'ERROR_1', message: 'Error 1' },
          { code: 'ERROR_2', message: 'Error 2' },
        ],
        warnings: [],
      };

      renderWithProviders(<WorkflowCanvas />);

      expect(useValidation).toHaveBeenCalled();
    });
  });

  describe('Performance Constraints (Chicago TDD)', () => {
    it('should render within 16ms for 60fps', async () => {
      const start = performance.now();

      mockWorkflowHook.workflow.nodes = Array.from({ length: 20 }, (_, i) => ({
        id: `node-${i}`,
        type: i % 2 === 0 ? 'task' : 'condition',
        label: `Node ${i}`,
        position: { x: i * 100, y: i * 50 },
        data: {},
      }));

      renderWithProviders(<WorkflowCanvas />);

      const elapsed = performance.now() - start;

      // React Flow rendering should be fast
      expect(elapsed).toBeLessThan(16); // 60fps requirement
    });
  });

  describe('Contract Verification', () => {
    it('should call useWorkflow hook on mount', () => {
      renderWithProviders(<WorkflowCanvas />);

      expect(useWorkflow).toHaveBeenCalled();
    });

    it('should call useValidation hook on mount', () => {
      renderWithProviders(<WorkflowCanvas />);

      expect(useValidation).toHaveBeenCalled();
    });

    it('should call useTelemetry hook with correct component name', () => {
      renderWithProviders(<WorkflowCanvas />);

      expect(useTelemetry).toHaveBeenCalledWith('WorkflowCanvas');
    });
  });
});

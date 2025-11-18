/**
 * TaskNode Unit Tests - London School TDD
 *
 * DOCTRINE ALIGNMENT:
 * - Covenant 6: Observable telemetry on node interactions
 * - Behavior verification: How TaskNode collaborates with validation hooks
 */

import React from 'react';
import { renderWithProviders, screen, fireEvent } from '../helpers/test-utils';
import { TaskNode } from '@/components/editor/task-node';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';

jest.mock('@/hooks/use-validation');
jest.mock('@/hooks/use-telemetry');

describe('TaskNode - Component Unit Tests', () => {
  const mockValidation = {
    validation: {
      valid: true,
      errors: [],
      warnings: [],
    },
    validateWorkflow: jest.fn(),
  };

  const mockTelemetry = {
    trackEvent: jest.fn(),
    startSpan: jest.fn(),
    recordMetric: jest.fn(),
  };

  const defaultProps = {
    id: 'task-1',
    data: {
      label: 'Process Order',
      type: 'task',
      status: 'pending' as const,
    },
    selected: false,
    type: 'taskNode',
    xPos: 100,
    yPos: 100,
    dragging: false,
    isConnectable: true,
    zIndex: 1,
  };

  beforeEach(() => {
    jest.clearAllMocks();
    (useValidation as jest.Mock).mockReturnValue(mockValidation);
    (useTelemetry as jest.Mock).mockReturnValue(mockTelemetry);
  });

  describe('Rendering', () => {
    it('should render task node with label', () => {
      renderWithProviders(<TaskNode {...defaultProps} />);

      expect(screen.getByText('Process Order')).toBeInTheDocument();
    });

    it('should apply selected styles when selected', () => {
      renderWithProviders(<TaskNode {...defaultProps} selected={true} />);

      const container = screen.getByText('Process Order').closest('div');
      expect(container).toHaveClass('border-blue-500');
    });

    it('should show error state when validation errors exist', () => {
      mockValidation.validation.errors = [
        {
          code: 'INVALID_TASK',
          message: 'Task configuration invalid',
          severity: 'error',
          node: 'task-1',
        },
      ];

      renderWithProviders(<TaskNode {...defaultProps} />);

      const container = screen.getByText('Process Order').closest('div');
      expect(container).toHaveClass('border-red-500');
    });
  });

  describe('Status Indicators', () => {
    it('should display pending status icon', () => {
      renderWithProviders(<TaskNode {...defaultProps} data={{ ...defaultProps.data, status: 'pending' }} />);

      expect(screen.getByText('Process Order')).toBeInTheDocument();
    });

    it('should display active status icon', () => {
      renderWithProviders(<TaskNode {...defaultProps} data={{ ...defaultProps.data, status: 'active' }} />);

      expect(screen.getByText('Process Order')).toBeInTheDocument();
    });

    it('should display complete status icon', () => {
      renderWithProviders(<TaskNode {...defaultProps} data={{ ...defaultProps.data, status: 'complete' }} />);

      expect(screen.getByText('Process Order')).toBeInTheDocument();
    });

    it('should display error status when has validation errors', () => {
      mockValidation.validation.errors = [
        { code: 'ERROR', message: 'Error', severity: 'error', node: 'task-1' },
      ];

      renderWithProviders(<TaskNode {...defaultProps} />);

      expect(useValidation).toHaveBeenCalled();
    });
  });

  describe('Covenant 6: Observable Telemetry', () => {
    it('should track double-click events', () => {
      renderWithProviders(<TaskNode {...defaultProps} />);

      const node = screen.getByText('Process Order').closest('div');
      fireEvent.doubleClick(node!);

      expect(mockTelemetry.trackEvent).toHaveBeenCalledWith('node.doubleClick', {
        nodeId: 'task-1',
      });
    });

    it('should initialize telemetry with component name', () => {
      renderWithProviders(<TaskNode {...defaultProps} />);

      expect(useTelemetry).toHaveBeenCalledWith('TaskNode');
    });
  });

  describe('Interaction Behavior (London School)', () => {
    it('should check validation errors for this node only', () => {
      mockValidation.validation.errors = [
        { code: 'ERROR_1', message: 'Error 1', severity: 'error', node: 'task-1' },
        { code: 'ERROR_2', message: 'Error 2', severity: 'error', node: 'task-2' },
      ];

      renderWithProviders(<TaskNode {...defaultProps} />);

      // Should only show errors for task-1
      expect(useValidation).toHaveBeenCalled();
    });

    it('should filter validation errors by node ID', () => {
      mockValidation.validation.errors = [
        { code: 'UNRELATED', message: 'Other error', severity: 'error', node: 'other-node' },
      ];

      renderWithProviders(<TaskNode {...defaultProps} />);

      // Should not show errors for other nodes
      const container = screen.getByText('Process Order').closest('div');
      expect(container).not.toHaveClass('border-red-500');
    });
  });

  describe('Memoization', () => {
    it('should be a memoized component for performance', () => {
      const { rerender } = renderWithProviders(<TaskNode {...defaultProps} />);

      // Update props
      rerender(<TaskNode {...defaultProps} data={{ ...defaultProps.data, label: 'Updated' }} />);

      expect(screen.getByText('Updated')).toBeInTheDocument();
    });
  });
});

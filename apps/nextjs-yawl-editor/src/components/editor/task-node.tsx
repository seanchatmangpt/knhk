/**
 * DOCTRINE ALIGNMENT: O (Observation)
 * TaskNode - Visual representation of a YAWL task
 */

'use client';

import { memo, useCallback } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { AlertCircle, CheckCircle2, Circle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';

interface TaskNodeData {
  label: string;
  type: string;
  status?: 'pending' | 'active' | 'complete' | 'error';
  description?: string;
}

/**
 * TaskNode - Visual representation of a workflow task
 *
 * Features:
 * - Display task name and type
 * - Show validation errors
 * - Customizable styling based on state
 * - Input/output flow ports
 *
 * @example
 * ```tsx
 * // Used internally by WorkflowCanvas via React Flow
 * const nodeTypes = {
 *   taskNode: TaskNode,
 * };
 * ```
 */
export const TaskNode = memo(({ id, data, selected }: NodeProps<TaskNodeData>) => {
  const { validation } = useValidation();
  const { trackEvent } = useTelemetry('TaskNode');

  // Check if this node has validation errors
  const nodeErrors = validation?.errors.filter((err) => err.node === id) || [];
  const hasErrors = nodeErrors.length > 0;

  // Determine node status
  const status = data.status || (hasErrors ? 'error' : 'pending');

  // Status icon
  const StatusIcon = {
    pending: Circle,
    active: AlertCircle,
    complete: CheckCircle2,
    error: AlertCircle,
  }[status];

  const statusColor = {
    pending: 'text-gray-400',
    active: 'text-blue-500',
    complete: 'text-green-500',
    error: 'text-red-500',
  }[status];

  const handleDoubleClick = useCallback(() => {
    trackEvent('node.doubleClick', { nodeId: id });
  }, [id, trackEvent]);

  return (
    <div
      className={cn(
        'min-w-[200px] rounded-lg border-2 bg-white shadow-md transition-all',
        selected
          ? 'border-blue-500 shadow-lg'
          : hasErrors
            ? 'border-red-500'
            : 'border-gray-300 hover:border-gray-400',
      )}
      onDoubleClick={handleDoubleClick}
    >
      {/* Input handle */}
      <Handle
        type="target"
        position={Position.Top}
        className="!bg-blue-500 !w-3 !h-3 !border-2 !border-white"
      />

      {/* Node content */}
      <div className="p-4">
        {/* Header */}
        <div className="flex items-center justify-between mb-2">
          <Badge variant="secondary" className="text-xs">
            {data.type}
          </Badge>
          <StatusIcon className={cn('h-4 w-4', statusColor)} />
        </div>

        {/* Label */}
        <div className="font-semibold text-sm text-gray-900 mb-1">{data.label}</div>

        {/* Description */}
        {data.description && (
          <div className="text-xs text-gray-500 line-clamp-2">{data.description}</div>
        )}

        {/* Validation errors */}
        {hasErrors && (
          <div className="mt-2 pt-2 border-t border-red-200">
            <div className="text-xs text-red-600 font-medium">
              {nodeErrors.length} error{nodeErrors.length > 1 ? 's' : ''}
            </div>
          </div>
        )}
      </div>

      {/* Output handle */}
      <Handle
        type="source"
        position={Position.Bottom}
        className="!bg-blue-500 !w-3 !h-3 !border-2 !border-white"
      />
    </div>
  );
});

TaskNode.displayName = 'TaskNode';

/**
 * DOCTRINE ALIGNMENT: O (Observation)
 * ConditionNode - Visual representation of a YAWL condition/decision
 */

'use client';

import { memo, useCallback } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { GitBranch, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/badge';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';

interface ConditionNodeData {
  label: string;
  type: string;
  condition?: string;
  description?: string;
}

/**
 * ConditionNode - Visual representation of a workflow condition/decision point
 *
 * Features:
 * - Display condition name
 * - Show XPath/XQuery expression
 * - Validation feedback
 * - Diamond visual styling
 * - Multiple output flow ports
 *
 * @example
 * ```tsx
 * // Used internally by WorkflowCanvas via React Flow
 * const nodeTypes = {
 *   conditionNode: ConditionNode,
 * };
 * ```
 */
export const ConditionNode = memo(({ id, data, selected }: NodeProps<ConditionNodeData>) => {
  const { validation } = useValidation();
  const { trackEvent } = useTelemetry('ConditionNode');

  // Check if this node has validation errors
  const nodeErrors = validation?.errors.filter((err) => err.node === id) || [];
  const hasErrors = nodeErrors.length > 0;

  const handleDoubleClick = useCallback(() => {
    trackEvent('node.doubleClick', { nodeId: id });
  }, [id, trackEvent]);

  return (
    <div
      className={cn(
        'min-w-[180px] rounded-lg border-2 bg-white shadow-md transition-all',
        selected
          ? 'border-purple-500 shadow-lg'
          : hasErrors
            ? 'border-red-500'
            : 'border-purple-300 hover:border-purple-400',
      )}
      onDoubleClick={handleDoubleClick}
    >
      {/* Input handle */}
      <Handle
        type="target"
        position={Position.Top}
        className="!bg-purple-500 !w-3 !h-3 !border-2 !border-white"
      />

      {/* Node content */}
      <div className="p-4 bg-gradient-to-br from-purple-50 to-white">
        {/* Header */}
        <div className="flex items-center justify-between mb-2">
          <Badge variant="secondary" className="text-xs bg-purple-100 text-purple-700">
            Condition
          </Badge>
          <GitBranch className="h-4 w-4 text-purple-500" />
        </div>

        {/* Label */}
        <div className="font-semibold text-sm text-gray-900 mb-1">{data.label}</div>

        {/* Condition expression */}
        {data.condition && (
          <div className="text-xs font-mono text-gray-600 bg-gray-50 p-2 rounded mt-2 line-clamp-2">
            {data.condition}
          </div>
        )}

        {/* Description */}
        {data.description && (
          <div className="text-xs text-gray-500 mt-2 line-clamp-2">{data.description}</div>
        )}

        {/* Validation errors */}
        {hasErrors && (
          <div className="mt-2 pt-2 border-t border-red-200">
            <div className="flex items-center gap-1 text-xs text-red-600 font-medium">
              <AlertCircle className="h-3 w-3" />
              {nodeErrors.length} error{nodeErrors.length > 1 ? 's' : ''}
            </div>
          </div>
        )}
      </div>

      {/* Output handles (true/false branches) */}
      <Handle
        type="source"
        position={Position.Bottom}
        id="true"
        className="!bg-green-500 !w-3 !h-3 !border-2 !border-white !-ml-6"
      />
      <Handle
        type="source"
        position={Position.Bottom}
        id="false"
        className="!bg-red-500 !w-3 !h-3 !border-2 !border-white !ml-6"
      />
    </div>
  );
});

ConditionNode.displayName = 'ConditionNode';

/**
 * Advanced Workflow Graph Visualization Component
 * Renders YAWL workflows with pattern visualization
 * Prepared for React Flow integration
 */

'use client'

import React, { useMemo } from 'react'
import type {
  YAWLSpecification,
  WorkflowNode,
  WorkflowEdge,
} from '@/types/yawl'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

interface WorkflowGraphProps {
  specification: YAWLSpecification
  onTaskSelect?: (taskId: string) => void
  onFlowSelect?: (flowId: string) => void
  interactive?: boolean
}

/**
 * Renders workflow as interactive graph
 * Future: Integrate React Flow for full drag-drop support
 */
export function WorkflowGraph({
  specification,
  onTaskSelect,
  onFlowSelect,
  interactive = true,
}: WorkflowGraphProps) {
  const { nodes, edges } = useMemo(() => {
    // Convert specification to graph representation
    const graphNodes: WorkflowNode[] = specification.tasks.map(
      (task, idx) => ({
        id: task.id,
        label: task.name,
        type: 'task',
        position: { x: 100 + idx * 150, y: 100 },
        data: task,
      })
    )

    const graphEdges: WorkflowEdge[] = specification.nets
      .flatMap((net) => net.flows || [])
      .map((flow) => ({
        id: flow.id,
        source: flow.source,
        target: flow.target,
        label: flow.predicate,
        pattern: flow.pattern,
      }))

    return { nodes: graphNodes, edges: graphEdges }
  }, [specification])

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle className="text-lg">
          Workflow Graph: {specification.name}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {/* Tasks visualization */}
          <div className="space-y-2">
            <h3 className="font-semibold text-sm">Tasks ({nodes.length})</h3>
            <div className="flex flex-wrap gap-2">
              {nodes.map((node) => (
                <button
                  key={node.id}
                  onClick={() => onTaskSelect?.(node.id)}
                  className="px-3 py-2 rounded-md bg-blue-100 hover:bg-blue-200 text-blue-900 text-sm font-medium transition-colors"
                >
                  {node.label}
                </button>
              ))}
            </div>
          </div>

          {/* Connections visualization */}
          {edges.length > 0 && (
            <div className="space-y-2">
              <h3 className="font-semibold text-sm">Control Flows ({edges.length})</h3>
              <div className="space-y-1 bg-gray-50 p-3 rounded-md">
                {edges.map((edge) => {
                  const sourceTask = nodes.find((n) => n.id === edge.source)
                  const targetTask = nodes.find((n) => n.id === edge.target)
                  return (
                    <div
                      key={edge.id}
                      onClick={() => onFlowSelect?.(edge.id)}
                      className="text-sm p-2 hover:bg-gray-200 rounded cursor-pointer transition-colors"
                    >
                      <span className="font-medium">{sourceTask?.label}</span>
                      <span className="mx-2 text-gray-500">→</span>
                      <span className="font-medium">{targetTask?.label}</span>
                      {edge.pattern && (
                        <span className="ml-2 text-xs bg-purple-100 text-purple-800 px-2 py-1 rounded">
                          {edge.pattern}
                        </span>
                      )}
                    </div>
                  )
                })}
              </div>
            </div>
          )}

          {/* Complexity metrics */}
          <div className="grid grid-cols-3 gap-2 bg-gray-50 p-3 rounded-md">
            <div className="text-center">
              <p className="text-xs text-gray-600">Tasks</p>
              <p className="text-lg font-bold">{nodes.length}</p>
            </div>
            <div className="text-center">
              <p className="text-xs text-gray-600">Flows</p>
              <p className="text-lg font-bold">{edges.length}</p>
            </div>
            <div className="text-center">
              <p className="text-xs text-gray-600">Complexity</p>
              <p className="text-lg font-bold">
                {((nodes.length * edges.length) / 10).toFixed(1)}
              </p>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

export default WorkflowGraph

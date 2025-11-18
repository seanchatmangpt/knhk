/**
 * DOCTRINE ALIGNMENT: O (Observation) + Σ (Ontology)
 * WorkflowCanvas - Main canvas component using React Flow
 * Visual projection of RDF graph with real-time validation
 */

'use client';

import { useCallback, useEffect, useMemo } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Node,
  Edge,
  Connection,
  NodeTypes,
  useNodesState,
  useEdgesState,
  Panel,
  ConnectionMode,
  MarkerType,
} from 'reactflow';
import 'reactflow/dist/style.css';

import { useWorkflow } from '@/hooks/use-workflow';
import { useValidation } from '@/hooks/use-validation';
import { useTelemetry } from '@/hooks/use-telemetry';
import { TaskNode } from './task-node';
import { ConditionNode } from './condition-node';
import type { YAWLNode, YAWLEdge } from '@/lib/types';

/**
 * Convert YAWL nodes to React Flow nodes
 */
function convertToFlowNodes(yawlNodes: YAWLNode[]): Node[] {
  return yawlNodes.map((node) => ({
    id: node.id,
    type: node.type === 'task' ? 'taskNode' : node.type === 'condition' ? 'conditionNode' : 'default',
    position: node.position,
    data: {
      label: node.label,
      type: node.type,
      ...node.data,
    },
  }));
}

/**
 * Convert YAWL edges to React Flow edges
 */
function convertToFlowEdges(yawlEdges: YAWLEdge[]): Edge[] {
  return yawlEdges.map((edge) => ({
    id: edge.id,
    source: edge.source,
    target: edge.target,
    label: edge.label,
    markerEnd: {
      type: MarkerType.ArrowClosed,
      width: 20,
      height: 20,
    },
    style: {
      strokeWidth: 2,
    },
  }));
}

export interface WorkflowCanvasProps {
  onNodeSelect?: (nodeId: string | null) => void;
  onEdgeSelect?: (edgeId: string | null) => void;
}

/**
 * WorkflowCanvas - Interactive canvas for YAWL workflow editing
 *
 * Features:
 * - Drag-drop nodes from palette
 * - Connect edges with validation
 * - Delete operations
 * - Zoom/pan controls
 * - Minimap
 * - Real-time validation feedback
 *
 * @example
 * ```tsx
 * <WorkflowCanvas
 *   onNodeSelect={(id) => console.log('Selected:', id)}
 * />
 * ```
 */
export function WorkflowCanvas({ onNodeSelect, onEdgeSelect }: WorkflowCanvasProps) {
  const {
    workflow,
    addNode,
    updateNode,
    removeNode,
    addEdge: addWorkflowEdge,
    removeEdge: removeWorkflowEdge,
    selectNodes,
    clearSelection,
  } = useWorkflow();

  const { validation } = useValidation(true, 500);
  const { trackEvent, trackMetric, withSpan } = useTelemetry('WorkflowCanvas');

  // React Flow state
  const [nodes, setNodes, onNodesChange] = useNodesState([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState([]);

  // Custom node types
  const nodeTypes: NodeTypes = useMemo(
    () => ({
      taskNode: TaskNode,
      conditionNode: ConditionNode,
    }),
    []
  );

  // Sync workflow state to React Flow
  useEffect(() => {
    if (!workflow) {
      setNodes([]);
      setEdges([]);
      return;
    }

    withSpan('syncWorkflowToFlow', async () => {
      const flowNodes = convertToFlowNodes(workflow.nodes);
      const flowEdges = convertToFlowEdges(workflow.edges);

      setNodes(flowNodes);
      setEdges(flowEdges);

      trackMetric('nodes.count', flowNodes.length);
      trackMetric('edges.count', flowEdges.length);
    });
  }, [workflow, setNodes, setEdges, withSpan, trackMetric]);

  // Handle edge connection
  const onConnect = useCallback(
    (connection: Connection) => {
      trackEvent('edge.connect', {
        source: connection.source || '',
        target: connection.target || '',
      });

      if (!connection.source || !connection.target) return;

      const newEdge: YAWLEdge = {
        id: `edge-${connection.source}-${connection.target}-${Date.now()}`,
        source: connection.source,
        target: connection.target,
      };

      addWorkflowEdge(newEdge);
    },
    [addWorkflowEdge, trackEvent]
  );

  // Handle node drag end (update position)
  const onNodeDragStop = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      trackEvent('node.drag', { nodeId: node.id });

      updateNode(node.id, {
        position: node.position,
      });
    },
    [updateNode, trackEvent]
  );

  // Handle node deletion
  const onNodesDelete = useCallback(
    (deletedNodes: Node[]) => {
      deletedNodes.forEach((node) => {
        trackEvent('node.delete', { nodeId: node.id });
        removeNode(node.id);
      });
    },
    [removeNode, trackEvent]
  );

  // Handle edge deletion
  const onEdgesDelete = useCallback(
    (deletedEdges: Edge[]) => {
      deletedEdges.forEach((edge) => {
        trackEvent('edge.delete', { edgeId: edge.id });
        removeWorkflowEdge(edge.id);
      });
    },
    [removeWorkflowEdge, trackEvent]
  );

  // Handle node selection
  const onNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      trackEvent('node.select', { nodeId: node.id });
      selectNodes([node.id]);
      onNodeSelect?.(node.id);
    },
    [selectNodes, onNodeSelect, trackEvent]
  );

  // Handle edge selection
  const onEdgeClick = useCallback(
    (_event: React.MouseEvent, edge: Edge) => {
      trackEvent('edge.select', { edgeId: edge.id });
      onEdgeSelect?.(edge.id);
    },
    [onEdgeSelect, trackEvent]
  );

  // Handle pane click (deselect)
  const onPaneClick = useCallback(() => {
    trackEvent('pane.click');
    clearSelection();
    onNodeSelect?.(null);
    onEdgeSelect?.(null);
  }, [clearSelection, onNodeSelect, onEdgeSelect, trackEvent]);

  // Handle drop from palette
  const onDrop = useCallback(
    (event: React.DragEvent<HTMLDivElement>) => {
      event.preventDefault();

      const type = event.dataTransfer.getData('application/reactflow');
      if (!type) return;

      const reactFlowBounds = event.currentTarget.getBoundingClientRect();
      const position = {
        x: event.clientX - reactFlowBounds.left,
        y: event.clientY - reactFlowBounds.top,
      };

      trackEvent('node.drop', { type, x: position.x, y: position.y });

      const newNode: YAWLNode = {
        id: `${type}-${Date.now()}`,
        type: type as YAWLNode['type'],
        label: `New ${type}`,
        position,
      };

      addNode(newNode);
    },
    [addNode, trackEvent]
  );

  const onDragOver = useCallback((event: React.DragEvent<HTMLDivElement>) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = 'move';
  }, []);

  // Validation status display
  const validationStatus = validation
    ? validation.valid
      ? 'Valid'
      : `${validation.errors.length} error(s)`
    : 'Validating...';

  const validationColor = validation?.valid ? 'text-green-600' : 'text-red-600';

  return (
    <div className="h-full w-full" onDrop={onDrop} onDragOver={onDragOver}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        onNodeDragStop={onNodeDragStop}
        onNodesDelete={onNodesDelete}
        onEdgesDelete={onEdgesDelete}
        onNodeClick={onNodeClick}
        onEdgeClick={onEdgeClick}
        onPaneClick={onPaneClick}
        nodeTypes={nodeTypes}
        connectionMode={ConnectionMode.Loose}
        fitView
        attributionPosition="bottom-left"
      >
        <Background />
        <Controls />
        <MiniMap
          nodeStrokeWidth={3}
          zoomable
          pannable
          className="bg-white border border-gray-300 rounded-lg"
        />
        <Panel position="top-right" className="bg-white p-3 rounded-lg shadow-md border">
          <div className="text-sm font-medium">
            Validation: <span className={validationColor}>{validationStatus}</span>
          </div>
        </Panel>
      </ReactFlow>
    </div>
  );
}

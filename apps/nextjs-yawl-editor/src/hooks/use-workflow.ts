/**
 * DOCTRINE ALIGNMENT: Σ (Sum of All Fears - System Integrity)
 * Custom hook for workflow operations with RDF/canvas synchronization
 */

'use client';

import { useCallback, useEffect, useState } from 'react';
import { useEditorStore } from '@/store/editor-store';
import type { YAWLWorkflow, YAWLNode, YAWLEdge } from '@/lib/types';
import { createSpan } from '@/lib/telemetry/setup';
import { SpanStatusCode } from '@opentelemetry/api';

export interface UseWorkflowReturn {
  workflow: YAWLWorkflow | null;
  selectedNodes: string[];
  selectedEdges: string[];
  mode: 'edit' | 'view' | 'validate';
  canUndo: boolean;
  canRedo: boolean;

  // Actions
  setWorkflow: (workflow: YAWLWorkflow) => void;
  addNode: (node: YAWLNode) => void;
  removeNode: (nodeId: string) => void;
  updateNode: (nodeId: string, updates: Partial<YAWLNode>) => void;
  addEdge: (edge: YAWLEdge) => void;
  removeEdge: (edgeId: string) => void;
  selectNodes: (nodeIds: string[]) => void;
  selectEdges: (edgeIds: string[]) => void;
  clearSelection: () => void;
  setMode: (mode: 'edit' | 'view' | 'validate') => void;
  undo: () => void;
  redo: () => void;
  copy: () => void;
  paste: () => void;
}

/**
 * Hook for managing workflow state with RDF synchronization
 *
 * @example
 * ```tsx
 * const { workflow, addNode, updateNode } = useWorkflow();
 *
 * // Add a task node
 * addNode({
 *   id: 'task-1',
 *   type: 'task',
 *   label: 'Process Order',
 *   position: { x: 100, y: 100 },
 * });
 * ```
 */
export function useWorkflow(): UseWorkflowReturn {
  const store = useEditorStore();
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
  }, []);

  const canUndo = isClient && store.history.past.length > 0;
  const canRedo = isClient && store.history.future.length > 0;

  // Wrap actions with telemetry
  const setWorkflow = useCallback((workflow: YAWLWorkflow) => {
    const span = createSpan('workflow.set', {
      'workflow.id': workflow.id,
      'workflow.nodeCount': workflow.nodes.length,
      'workflow.edgeCount': workflow.edges.length,
    });

    try {
      store.setWorkflow(workflow);
      span.setStatus({ code: SpanStatusCode.OK });
    } catch (error) {
      span.recordException(error as Error);
      span.setStatus({ code: SpanStatusCode.ERROR });
      throw error;
    } finally {
      span.end();
    }
  }, [store]);

  const addNode = useCallback((node: YAWLNode) => {
    const span = createSpan('workflow.addNode', {
      'node.type': node.type,
      'node.id': node.id,
    });

    try {
      store.addNode(node);
      span.setStatus({ code: SpanStatusCode.OK });
    } catch (error) {
      span.recordException(error as Error);
      span.setStatus({ code: SpanStatusCode.ERROR });
      throw error;
    } finally {
      span.end();
    }
  }, [store]);

  const updateNode = useCallback((nodeId: string, updates: Partial<YAWLNode>) => {
    const span = createSpan('workflow.updateNode', {
      'node.id': nodeId,
    });

    try {
      store.updateNode(nodeId, updates);
      span.setStatus({ code: SpanStatusCode.OK });
    } catch (error) {
      span.recordException(error as Error);
      span.setStatus({ code: SpanStatusCode.ERROR });
      throw error;
    } finally {
      span.end();
    }
  }, [store]);

  const removeNode = useCallback((nodeId: string) => {
    const span = createSpan('workflow.removeNode', {
      'node.id': nodeId,
    });

    try {
      store.removeNode(nodeId);
      span.setStatus({ code: SpanStatusCode.OK });
    } catch (error) {
      span.recordException(error as Error);
      span.setStatus({ code: SpanStatusCode.ERROR });
      throw error;
    } finally {
      span.end();
    }
  }, [store]);

  const addEdge = useCallback((edge: YAWLEdge) => {
    const span = createSpan('workflow.addEdge', {
      'edge.id': edge.id,
      'edge.source': edge.source,
      'edge.target': edge.target,
    });

    try {
      store.addEdge(edge);
      span.setStatus({ code: SpanStatusCode.OK });
    } catch (error) {
      span.recordException(error as Error);
      span.setStatus({ code: SpanStatusCode.ERROR });
      throw error;
    } finally {
      span.end();
    }
  }, [store]);

  const removeEdge = useCallback((edgeId: string) => {
    const span = createSpan('workflow.removeEdge', {
      'edge.id': edgeId,
    });

    try {
      store.removeEdge(edgeId);
      span.setStatus({ code: SpanStatusCode.OK });
    } catch (error) {
      span.recordException(error as Error);
      span.setStatus({ code: SpanStatusCode.ERROR });
      throw error;
    } finally {
      span.end();
    }
  }, [store]);

  return {
    workflow: store.workflow,
    selectedNodes: store.selectedNodes,
    selectedEdges: store.selectedEdges,
    mode: store.mode,
    canUndo,
    canRedo,

    setWorkflow,
    addNode,
    removeNode,
    updateNode,
    addEdge,
    removeEdge,
    selectNodes: store.selectNodes,
    selectEdges: store.selectEdges,
    clearSelection: store.clearSelection,
    setMode: store.setMode,
    undo: store.undo,
    redo: store.redo,
    copy: store.copy,
    paste: store.paste,
  };
}

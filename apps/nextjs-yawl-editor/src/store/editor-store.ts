/**
 * DOCTRINE ALIGNMENT: Σ (Sum of All Fears - System Integrity)
 * Zustand store for editor state management
 * Provides centralized state with undo/redo support
 */

import { create } from 'zustand';
import type { EditorState, YAWLWorkflow, YAWLNode, YAWLEdge } from '@/lib/types';
import { trackEditorOperation } from '@/lib/telemetry/setup';

interface EditorStore extends EditorState {
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
  setMode: (mode: EditorState['mode']) => void;
  undo: () => void;
  redo: () => void;
  copy: () => void;
  paste: () => void;
}

const initialState: EditorState = {
  workflow: null,
  selectedNodes: [],
  selectedEdges: [],
  clipboard: null,
  history: {
    past: [],
    future: [],
  },
  mode: 'edit',
};

export const useEditorStore = create<EditorStore>((set, get) => ({
  ...initialState,

  setWorkflow: (workflow) => {
    trackEditorOperation('set_workflow', { 'workflow.id': workflow.id });
    set({ workflow });
  },

  addNode: (node) => {
    const { workflow } = get();
    if (!workflow) return;

    trackEditorOperation('add_node', { 'node.type': node.type });

    const newWorkflow: YAWLWorkflow = {
      ...workflow,
      nodes: [...workflow.nodes, node],
      metadata: workflow.metadata ? {
        created: workflow.metadata.created,
        modified: new Date().toISOString(),
        author: workflow.metadata.author,
        description: workflow.metadata.description,
      } : {
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
      },
    };

    set((state) => ({
      workflow: newWorkflow,
      history: {
        past: workflow ? [...state.history.past, workflow] : state.history.past,
        future: [],
      },
    }));
  },

  removeNode: (nodeId) => {
    const { workflow } = get();
    if (!workflow) return;

    trackEditorOperation('remove_node', { 'node.id': nodeId });

    const newWorkflow: YAWLWorkflow = {
      ...workflow,
      nodes: workflow.nodes.filter((n: YAWLNode) => n.id !== nodeId),
      edges: workflow.edges.filter((e: YAWLEdge) => e.source !== nodeId && e.target !== nodeId),
      metadata: workflow.metadata ? {
        created: workflow.metadata.created,
        modified: new Date().toISOString(),
        author: workflow.metadata.author,
        description: workflow.metadata.description,
      } : {
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
      },
    };

    set((state) => ({
      workflow: newWorkflow,
      history: {
        past: [...state.history.past, workflow],
        future: [],
      },
    }));
  },

  updateNode: (nodeId, updates) => {
    const { workflow } = get();
    if (!workflow) return;

    trackEditorOperation('update_node', { 'node.id': nodeId });

    const newWorkflow: YAWLWorkflow = {
      ...workflow,
      nodes: workflow.nodes.map((n: YAWLNode) => (n.id === nodeId ? { ...n, ...updates } : n)),
      metadata: workflow.metadata ? {
        created: workflow.metadata.created,
        modified: new Date().toISOString(),
        author: workflow.metadata.author,
        description: workflow.metadata.description,
      } : {
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
      },
    };

    set((state) => ({
      workflow: newWorkflow,
      history: {
        past: [...state.history.past, workflow],
        future: [],
      },
    }));
  },

  addEdge: (edge) => {
    const { workflow } = get();
    if (!workflow) return;

    trackEditorOperation('add_edge');

    const newWorkflow: YAWLWorkflow = {
      ...workflow,
      edges: [...workflow.edges, edge],
      metadata: workflow.metadata ? {
        created: workflow.metadata.created,
        modified: new Date().toISOString(),
        author: workflow.metadata.author,
        description: workflow.metadata.description,
      } : {
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
      },
    };

    set((state) => ({
      workflow: newWorkflow,
      history: {
        past: [...state.history.past, workflow],
        future: [],
      },
    }));
  },

  removeEdge: (edgeId) => {
    const { workflow } = get();
    if (!workflow) return;

    trackEditorOperation('remove_edge', { 'edge.id': edgeId });

    const newWorkflow: YAWLWorkflow = {
      ...workflow,
      edges: workflow.edges.filter((e: YAWLEdge) => e.id !== edgeId),
      metadata: workflow.metadata ? {
        created: workflow.metadata.created,
        modified: new Date().toISOString(),
        author: workflow.metadata.author,
        description: workflow.metadata.description,
      } : {
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
      },
    };

    set((state) => ({
      workflow: newWorkflow,
      history: {
        past: [...state.history.past, workflow],
        future: [],
      },
    }));
  },

  selectNodes: (nodeIds) => {
    trackEditorOperation('select_nodes', { count: nodeIds.length });
    set({ selectedNodes: nodeIds });
  },

  selectEdges: (edgeIds) => {
    trackEditorOperation('select_edges', { count: edgeIds.length });
    set({ selectedEdges: edgeIds });
  },

  clearSelection: () => {
    trackEditorOperation('clear_selection');
    set({ selectedNodes: [], selectedEdges: [] });
  },

  setMode: (mode) => {
    trackEditorOperation('set_mode', { mode });
    set({ mode });
  },

  undo: () => {
    const { history, workflow } = get();
    if (history.past.length === 0 || !workflow) return;

    trackEditorOperation('undo');

    const previous = history.past[history.past.length - 1];
    const newPast = history.past.slice(0, -1);

    if (!previous) return; // Should never happen due to length check above

    set({
      workflow: previous,
      history: {
        past: newPast,
        future: [workflow, ...history.future],
      },
    });
  },

  redo: () => {
    const { history, workflow } = get();
    if (history.future.length === 0 || !workflow) return;

    trackEditorOperation('redo');

    const next = history.future[0];
    const newFuture = history.future.slice(1);

    if (!next) return; // Should never happen due to length check above

    set({
      workflow: next,
      history: {
        past: [...history.past, workflow],
        future: newFuture,
      },
    });
  },

  copy: () => {
    const { workflow, selectedNodes, selectedEdges } = get();
    if (!workflow) return;

    trackEditorOperation('copy', {
      nodes: selectedNodes.length,
      edges: selectedEdges.length,
    });

    const nodes = workflow.nodes.filter((n: YAWLNode) => selectedNodes.includes(n.id));
    const edges = workflow.edges.filter((e: YAWLEdge) => selectedEdges.includes(e.id));

    set({ clipboard: { nodes, edges } });
  },

  paste: () => {
    const { clipboard, workflow } = get();
    if (!clipboard || !workflow) return;

    trackEditorOperation('paste', {
      nodes: clipboard.nodes.length,
      edges: clipboard.edges.length,
    });

    // Generate new IDs for pasted nodes and edges
    const idMap = new Map<string, string>();
    const newNodes = clipboard.nodes.map((node: YAWLNode) => {
      const newId = `${node.id}-copy-${Date.now()}`;
      idMap.set(node.id, newId);
      return {
        ...node,
        id: newId,
        position: {
          x: node.position.x + 50,
          y: node.position.y + 50,
        },
      };
    });

    const newEdges = clipboard.edges.map((edge: YAWLEdge) => ({
      ...edge,
      id: `${edge.id}-copy-${Date.now()}`,
      source: idMap.get(edge.source) || edge.source,
      target: idMap.get(edge.target) || edge.target,
    }));

    const newWorkflow: YAWLWorkflow = {
      ...workflow,
      nodes: [...workflow.nodes, ...newNodes],
      edges: [...workflow.edges, ...newEdges],
      metadata: workflow.metadata ? {
        created: workflow.metadata.created,
        modified: new Date().toISOString(),
        author: workflow.metadata.author,
        description: workflow.metadata.description,
      } : {
        created: new Date().toISOString(),
        modified: new Date().toISOString(),
      },
    };

    set((state) => ({
      workflow: newWorkflow,
      history: {
        past: [...state.history.past, workflow],
        future: [],
      },
    }));
  },
}));

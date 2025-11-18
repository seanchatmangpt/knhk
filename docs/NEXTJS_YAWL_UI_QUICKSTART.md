# Next.js YAWL UI - Quick Start Implementation Guide

**Status**: DRAFT | **Version**: 1.0.0 | **Last Updated**: 2025-11-18

This guide provides step-by-step instructions for implementing the Next.js YAWL UI system.

---

## Prerequisites

- Node.js 20+
- npm/pnpm/yarn
- Git
- Basic understanding of Next.js, React, TypeScript, and RDF/Turtle

---

## Phase 1: Project Setup (Day 1-2)

### Step 1: Initialize Next.js Project

```bash
# Create Next.js project
npx create-next-app@latest nextjs-yawl-ui \
  --typescript \
  --tailwind \
  --app \
  --src-dir \
  --import-alias "@/*"

cd nextjs-yawl-ui
```

### Step 2: Install Core Dependencies

```bash
# UI Components
npx shadcn-ui@latest init

# RDF Processing
npm install n3 @comunica/query-sparql

# Workflow Visualization
npm install reactflow dagre

# State Management
npm install zustand immer @tanstack/react-query

# Forms & Validation
npm install zod react-hook-form @hookform/resolvers

# Icons
npm install lucide-react

# Development
npm install -D @types/node @types/react @types/react-dom
npm install -D vitest @testing-library/react @testing-library/jest-dom
npm install -D playwright @playwright/test
```

### Step 3: Install shadcn/ui Components

```bash
# Install required components
npx shadcn-ui@latest add button
npx shadcn-ui@latest add card
npx shadcn-ui@latest add dialog
npx shadcn-ui@latest add dropdown-menu
npx shadcn-ui@latest add input
npx shadcn-ui@latest add label
npx shadcn-ui@latest add select
npx shadcn-ui@latest add tabs
npx shadcn-ui@latest add toast
npx shadcn-ui@latest add alert
npx shadcn-ui@latest add badge
npx shadcn-ui@latest add separator
```

### Step 4: Create Directory Structure

```bash
# Create directory structure
mkdir -p src/app/api/workflows
mkdir -p src/app/api/sparql
mkdir -p src/app/api/validate
mkdir -p src/app/\(dashboard\)/workflows
mkdir -p src/components/workflow/{editor,nodes,renderer,validator,monitor}
mkdir -p src/components/patterns
mkdir -p src/components/providers
mkdir -p src/lib/{rdf,workflow,validation,mapek,performance,telemetry,api,utils}
mkdir -p src/hooks
mkdir -p src/stores
mkdir -p src/types
mkdir -p src/actions
mkdir -p tests/{unit,integration,e2e}

# Symlink ontology directory
ln -s /home/user/knhk/ontology ./ontology
```

### Step 5: Configure TypeScript

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2020",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "jsx": "preserve",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "allowJs": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "paths": {
      "@/*": ["./src/*"]
    },
    "incremental": true,
    "plugins": [
      {
        "name": "next"
      }
    ]
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules"]
}
```

---

## Phase 2: RDF Foundation (Day 3-5)

### Step 1: Create RDF Parser

```typescript
// src/lib/rdf/parser.ts
import { Parser, Store } from 'n3';

/**
 * Parse Turtle string into RDF store
 * COVENANT 1: Turtle is source of truth - no reconstruction logic
 */
export async function parseTurtle(turtle: string): Promise<Store> {
  const parser = new Parser();
  const store = new Store();

  return new Promise((resolve, reject) => {
    parser.parse(turtle, (error, quad, prefixes) => {
      if (error) {
        reject(new Error(`Turtle parsing failed: ${error.message}`));
      } else if (quad) {
        store.addQuad(quad);
      } else {
        // Parsing complete
        resolve(store);
      }
    });
  });
}

/**
 * Load Turtle file from ontology directory
 */
export async function loadTurtleFile(filename: string): Promise<string> {
  const fs = await import('fs/promises');
  const path = await import('path');

  const filePath = path.join(process.cwd(), 'ontology', filename);
  return fs.readFile(filePath, 'utf-8');
}
```

### Step 2: Create SPARQL Engine

```typescript
// src/lib/rdf/sparql.ts
import { Store, DataFactory } from 'n3';

const { namedNode, literal } = DataFactory;

/**
 * Execute SPARQL query on RDF store
 * COVENANT 1: Pure extraction, zero filtering logic
 */
export function executeSPARQL(store: Store, query: string): any[] {
  // For now, use N3's quad matching
  // TODO: Integrate Comunica for full SPARQL support

  // Example: Extract all workflows
  // SELECT ?workflow ?name WHERE {
  //   ?workflow a yawl:Workflow ;
  //             yawl:specName ?name .
  // }

  const results: any[] = [];
  const workflows = store.getSubjects(
    namedNode('http://www.w3.org/1999/02/22-rdf-syntax-ns#type'),
    namedNode('http://www.yawlfoundation.org/yawlschema#Workflow'),
    null
  );

  for (const workflow of workflows) {
    const name = store.getObjects(
      workflow,
      namedNode('http://www.yawlfoundation.org/yawlschema#specName'),
      null
    )[0];

    results.push({
      workflow: workflow.value,
      name: name?.value || 'Unnamed',
    });
  }

  return results;
}
```

### Step 3: Create RDF Prefixes

```typescript
// src/lib/rdf/prefixes.ts
export const PREFIXES = {
  rdf: 'http://www.w3.org/1999/02/22-rdf-syntax-ns#',
  rdfs: 'http://www.w3.org/2000/01/rdf-schema#',
  owl: 'http://www.w3.org/2002/07/owl#',
  xsd: 'http://www.w3.org/2001/XMLSchema#',
  yawl: 'http://www.yawlfoundation.org/yawlschema#',
  mapek: 'http://knhk.ai/ontology/mape-k#',
  pattern: 'http://knhk.ai/ontology/workflow-patterns#',
  knhk: 'urn:knhk:ontology#',
};

export function expandPrefix(prefix: string, localName: string): string {
  const namespace = PREFIXES[prefix as keyof typeof PREFIXES];
  if (!namespace) {
    throw new Error(`Unknown prefix: ${prefix}`);
  }
  return namespace + localName;
}

export function prefixedName(uri: string): string {
  for (const [prefix, namespace] of Object.entries(PREFIXES)) {
    if (uri.startsWith(namespace)) {
      return `${prefix}:${uri.substring(namespace.length)}`;
    }
  }
  return uri;
}
```

### Step 4: Create TypeScript Types

```typescript
// src/types/workflow.ts
export interface WorkflowRDF {
  uri: string;
  name: string;
  version: string;
  description?: string;

  // Core elements
  tasks: TaskRDF[];
  flows: FlowRDF[];
  inputCondition?: ConditionRDF;
  outputCondition?: ConditionRDF;

  // MAPE-K
  monitor?: string;
  analyzer?: string;
  planner?: string;
  executor?: string;
  knowledge?: string;

  // Metadata
  patterns: string[];
  created?: Date;
  updated?: Date;
}

export interface TaskRDF {
  uri: string;
  name: string;
  type: 'atomic' | 'composite';
  description?: string;

  // Control flow
  splitType: 'AND' | 'OR' | 'XOR' | null;
  joinType: 'AND' | 'OR' | 'XOR' | null;

  // Patterns
  patterns: string[];

  // Parameters
  inputParams: ParameterRDF[];
  outputParams: ParameterRDF[];

  // MAPE-K constraints
  constraints?: string[];
}

export interface FlowRDF {
  uri: string;
  source: string;
  target: string;
  predicate?: string;
  order?: number;
}

export interface ConditionRDF {
  uri: string;
  name: string;
  type: 'input' | 'output';
}

export interface ParameterRDF {
  name: string;
  type: string;
  direction: 'input' | 'output' | 'both';
  required: boolean;
}

export interface ValidationResult {
  isValid: boolean;
  violations: Violation[];
  patterns: DetectedPattern[];
}

export interface Violation {
  nodeId: string;
  message: string;
  severity: 'error' | 'warning';
}

export interface DetectedPattern {
  id: string;
  name: string;
  description: string;
}
```

---

## Phase 3: Workflow Parsing (Day 6-8)

### Step 1: Create Workflow Parser

```typescript
// src/lib/workflow/parser.ts
import { Store, DataFactory } from 'n3';
import { WorkflowRDF, TaskRDF, FlowRDF } from '@/types/workflow';
import { PREFIXES } from '@/lib/rdf/prefixes';

const { namedNode } = DataFactory;

/**
 * Parse RDF store into WorkflowRDF object
 * COVENANT 1: Pure extraction, zero reconstruction
 */
export function parseWorkflow(store: Store, workflowUri: string): WorkflowRDF {
  // Extract workflow metadata
  const name = getObjectValue(store, workflowUri, 'yawl:specName') || 'Unnamed';
  const version = getObjectValue(store, workflowUri, 'yawl:version') || '1.0';
  const description = getObjectValue(store, workflowUri, 'rdfs:comment');

  // Extract tasks
  const tasks = extractTasks(store, workflowUri);

  // Extract flows
  const flows = extractFlows(store, workflowUri);

  // Extract patterns
  const patterns = extractPatterns(store, tasks);

  return {
    uri: workflowUri,
    name,
    version,
    description,
    tasks,
    flows,
    patterns,
  };
}

function extractTasks(store: Store, workflowUri: string): TaskRDF[] {
  const tasks: TaskRDF[] = [];

  const taskUris = store.getObjects(
    namedNode(workflowUri),
    namedNode(PREFIXES.yawl + 'hasTask'),
    null
  );

  for (const taskUri of taskUris) {
    const name = getObjectValue(store, taskUri.value, 'yawl:taskName') || 'Unnamed Task';
    const splitType = getObjectValue(store, taskUri.value, 'yawl:split');
    const joinType = getObjectValue(store, taskUri.value, 'yawl:join');

    // Extract patterns
    const patternUris = store.getObjects(
      namedNode(taskUri.value),
      namedNode(PREFIXES.pattern + 'implementsPattern'),
      null
    );
    const patterns = patternUris.map(p => p.value);

    tasks.push({
      uri: taskUri.value,
      name,
      type: 'atomic', // TODO: detect composite
      splitType: splitType as any,
      joinType: joinType as any,
      patterns,
      inputParams: [],
      outputParams: [],
    });
  }

  return tasks;
}

function extractFlows(store: Store, workflowUri: string): FlowRDF[] {
  const flows: FlowRDF[] = [];

  const flowUris = store.getObjects(
    namedNode(workflowUri),
    namedNode(PREFIXES.yawl + 'hasFlow'),
    null
  );

  for (const flowUri of flowUris) {
    const source = getObjectValue(store, flowUri.value, 'yawl:flowsFrom');
    const target = getObjectValue(store, flowUri.value, 'yawl:flowsInto');

    if (source && target) {
      flows.push({
        uri: flowUri.value,
        source,
        target,
      });
    }
  }

  return flows;
}

function extractPatterns(store: Store, tasks: TaskRDF[]): string[] {
  const patterns = new Set<string>();

  for (const task of tasks) {
    for (const pattern of task.patterns) {
      patterns.add(pattern);
    }
  }

  return Array.from(patterns);
}

function getObjectValue(store: Store, subject: string, predicate: string): string | undefined {
  const [prefix, localName] = predicate.split(':');
  const predicateUri = PREFIXES[prefix as keyof typeof PREFIXES] + localName;

  const objects = store.getObjects(
    namedNode(subject),
    namedNode(predicateUri),
    null
  );

  return objects[0]?.value;
}
```

### Step 2: Create React Flow Transformer

```typescript
// src/lib/workflow/transformer.ts
import { Node, Edge } from 'reactflow';
import { WorkflowRDF } from '@/types/workflow';

/**
 * Transform WorkflowRDF to React Flow format
 * COVENANT 1: Pure transformation, no added semantics
 */
export function workflowToReactFlow(workflow: WorkflowRDF): {
  nodes: Node[];
  edges: Edge[];
} {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  // Create nodes from tasks
  for (const task of workflow.tasks) {
    nodes.push({
      id: task.uri,
      type: 'task',
      position: { x: 0, y: 0 }, // Will be auto-laid out
      data: {
        label: task.name,
        taskType: task.type,
        splitType: task.splitType,
        joinType: task.joinType,
        patterns: task.patterns,
        rdfUri: task.uri,
      },
    });
  }

  // Create edges from flows
  for (const flow of workflow.flows) {
    edges.push({
      id: flow.uri,
      source: flow.source,
      target: flow.target,
      type: 'smoothstep',
      animated: false,
    });
  }

  return { nodes, edges };
}
```

### Step 3: Create Auto-Layout

```typescript
// src/lib/workflow/layout.ts
import dagre from 'dagre';
import { Node, Edge } from 'reactflow';

/**
 * Apply Dagre layout algorithm to nodes
 */
export function applyDagreLayout(
  nodes: Node[],
  edges: Edge[],
  direction: 'TB' | 'LR' = 'TB'
): Node[] {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  dagreGraph.setGraph({ rankdir: direction });

  // Add nodes to dagre
  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: 200, height: 100 });
  });

  // Add edges to dagre
  edges.forEach((edge) => {
    dagreGraph.setEdge(edge.source, edge.target);
  });

  // Calculate layout
  dagre.layout(dagreGraph);

  // Update node positions
  return nodes.map((node) => {
    const nodeWithPosition = dagreGraph.node(node.id);
    return {
      ...node,
      position: {
        x: nodeWithPosition.x - 100,
        y: nodeWithPosition.y - 50,
      },
    };
  });
}
```

---

## Phase 4: UI Components (Day 9-12)

### Step 1: Create Task Node Component

```typescript
// src/components/workflow/nodes/task-node.tsx
import { memo } from 'react';
import { Handle, Position, NodeProps } from 'reactflow';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils/cn';

interface TaskNodeData {
  label: string;
  taskType: 'atomic' | 'composite';
  splitType: 'AND' | 'OR' | 'XOR' | null;
  joinType: 'AND' | 'OR' | 'XOR' | null;
  patterns: string[];
  rdfUri: string;
}

export const TaskNode = memo(({ data, selected }: NodeProps<TaskNodeData>) => {
  return (
    <Card
      className={cn(
        'min-w-[180px] transition-all cursor-pointer hover:shadow-lg',
        selected && 'ring-2 ring-primary shadow-lg',
        data.taskType === 'composite' && 'border-dashed'
      )}
    >
      <Handle
        type="target"
        position={Position.Top}
        className="!bg-primary !w-3 !h-3"
      />

      <div className="p-4 space-y-2">
        {/* Task name */}
        <div className="font-semibold text-sm leading-tight">
          {data.label}
        </div>

        {/* Split/Join indicators */}
        {(data.splitType || data.joinType) && (
          <div className="flex gap-1 flex-wrap">
            {data.splitType && (
              <Badge variant="secondary" className="text-xs">
                {data.splitType}-split
              </Badge>
            )}
            {data.joinType && (
              <Badge variant="secondary" className="text-xs">
                {data.joinType}-join
              </Badge>
            )}
          </div>
        )}

        {/* Pattern count */}
        {data.patterns.length > 0 && (
          <div className="text-xs text-muted-foreground">
            {data.patterns.length} pattern(s)
          </div>
        )}
      </div>

      <Handle
        type="source"
        position={Position.Bottom}
        className="!bg-primary !w-3 !h-3"
      />
    </Card>
  );
});

TaskNode.displayName = 'TaskNode';
```

### Step 2: Create Workflow Canvas

```typescript
// src/components/workflow/editor/canvas.tsx
'use client';

import { useCallback, useMemo } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  addEdge,
  Connection,
  Edge,
} from 'reactflow';
import 'reactflow/dist/style.css';

import { TaskNode } from '../nodes/task-node';

const nodeTypes = {
  task: TaskNode,
};

interface WorkflowCanvasProps {
  initialNodes: any[];
  initialEdges: any[];
  readonly?: boolean;
}

export function WorkflowCanvas({
  initialNodes,
  initialEdges,
  readonly = false,
}: WorkflowCanvasProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const onConnect = useCallback(
    (params: Connection) => {
      if (!readonly) {
        setEdges((eds) => addEdge(params, eds));
      }
    },
    [readonly, setEdges]
  );

  return (
    <div className="h-[600px] w-full border rounded-lg overflow-hidden">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onConnect={onConnect}
        nodeTypes={nodeTypes}
        fitView
        minZoom={0.1}
        maxZoom={2}
        nodesDraggable={!readonly}
        nodesConnectable={!readonly}
        elementsSelectable={!readonly}
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}
```

### Step 3: Create Workflow Page

```typescript
// src/app/(dashboard)/workflows/[id]/page.tsx
import { Suspense } from 'react';
import { notFound } from 'next/navigation';
import { WorkflowCanvas } from '@/components/workflow/editor/canvas';
import { loadTurtleFile } from '@/lib/rdf/parser';
import { parseTurtle } from '@/lib/rdf/parser';
import { parseWorkflow } from '@/lib/workflow/parser';
import { workflowToReactFlow } from '@/lib/workflow/transformer';
import { applyDagreLayout } from '@/lib/workflow/layout';

async function loadWorkflow(id: string) {
  try {
    // Load Turtle file
    const turtle = await loadTurtleFile(`workflows/${id}.ttl`);

    // Parse to RDF store
    const store = await parseTurtle(turtle);

    // Extract workflow object
    const workflowUri = `http://knhk.ai/workflows/${id}`;
    const workflow = parseWorkflow(store, workflowUri);

    // Transform to React Flow
    const { nodes, edges } = workflowToReactFlow(workflow);

    // Apply layout
    const layoutedNodes = applyDagreLayout(nodes, edges);

    return { workflow, nodes: layoutedNodes, edges };
  } catch (error) {
    console.error('Failed to load workflow:', error);
    return null;
  }
}

export default async function WorkflowPage({
  params,
}: {
  params: { id: string };
}) {
  const data = await loadWorkflow(params.id);

  if (!data) {
    notFound();
  }

  return (
    <div className="p-6 space-y-4">
      <div>
        <h1 className="text-3xl font-bold">{data.workflow.name}</h1>
        {data.workflow.description && (
          <p className="text-muted-foreground">{data.workflow.description}</p>
        )}
      </div>

      <WorkflowCanvas
        initialNodes={data.nodes}
        initialEdges={data.edges}
        readonly
      />
    </div>
  );
}
```

---

## Phase 5: Pattern Validation (Day 13-15)

### Step 1: Load Pattern Matrix

```typescript
// src/lib/validation/patterns.ts
import { Store } from 'n3';
import { loadTurtleFile, parseTurtle } from '@/lib/rdf/parser';
import { WorkflowRDF, ValidationResult, Violation } from '@/types/workflow';

let patternMatrixStore: Store | null = null;

/**
 * Load pattern permutation matrix
 */
async function loadPatternMatrix(): Promise<Store> {
  if (patternMatrixStore) {
    return patternMatrixStore;
  }

  const turtle = await loadTurtleFile('yawl-pattern-permutations.ttl');
  patternMatrixStore = await parseTurtle(turtle);
  return patternMatrixStore;
}

/**
 * Validate workflow against pattern matrix
 * COVENANT 2: Pattern matrix is law
 */
export async function validatePatterns(
  workflow: WorkflowRDF
): Promise<ValidationResult> {
  const matrix = await loadPatternMatrix();
  const violations: Violation[] = [];

  // Check each task's split/join combination
  for (const task of workflow.tasks) {
    if (!task.splitType && !task.joinType) {
      continue; // Simple task, no patterns
    }

    const isValid = await isValidPatternCombination(
      matrix,
      task.splitType,
      task.joinType
    );

    if (!isValid) {
      violations.push({
        nodeId: task.uri,
        message: `Invalid pattern combination: ${task.splitType}-split with ${task.joinType}-join`,
        severity: 'error',
      });
    }
  }

  return {
    isValid: violations.length === 0,
    violations,
    patterns: workflow.patterns.map(p => ({
      id: p,
      name: p.replace(/^Pattern\d+_/, ''),
      description: '',
    })),
  };
}

async function isValidPatternCombination(
  matrix: Store,
  split: string | null,
  join: string | null
): Promise<boolean> {
  // Query pattern matrix for valid combinations
  // TODO: Implement SPARQL query against matrix
  // For now, allow all combinations
  return true;
}
```

### Step 2: Create Validation UI Component

```typescript
// src/components/workflow/validator/pattern-matrix.tsx
'use client';

import { useEffect, useState } from 'react';
import { Card } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { CheckCircle2, XCircle, AlertTriangle } from 'lucide-react';
import { WorkflowRDF, ValidationResult } from '@/types/workflow';
import { validatePatterns } from '@/lib/validation/patterns';

interface PatternValidatorProps {
  workflow: WorkflowRDF;
}

export function PatternValidator({ workflow }: PatternValidatorProps) {
  const [result, setResult] = useState<ValidationResult | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    async function validate() {
      setLoading(true);
      const validationResult = await validatePatterns(workflow);
      setResult(validationResult);
      setLoading(false);
    }

    validate();
  }, [workflow]);

  if (loading) {
    return (
      <Card className="p-4">
        <div className="animate-pulse">Validating workflow...</div>
      </Card>
    );
  }

  if (!result) return null;

  return (
    <Card className="p-4 space-y-4">
      <h3 className="font-semibold flex items-center gap-2">
        {result.isValid ? (
          <CheckCircle2 className="h-5 w-5 text-green-500" />
        ) : (
          <XCircle className="h-5 w-5 text-red-500" />
        )}
        Pattern Validation
      </h3>

      {result.isValid ? (
        <Alert>
          <AlertDescription>
            ✓ Workflow is valid. Detected {result.patterns.length} pattern(s).
          </AlertDescription>
        </Alert>
      ) : (
        <Alert variant="destructive">
          <AlertDescription>
            ✗ {result.violations.length} violation(s) found.
          </AlertDescription>
        </Alert>
      )}

      {/* Detected patterns */}
      {result.patterns.length > 0 && (
        <div>
          <h4 className="text-sm font-medium mb-2">Detected Patterns:</h4>
          <div className="flex flex-wrap gap-2">
            {result.patterns.map((p) => (
              <Badge key={p.id} variant="outline">
                {p.name}
              </Badge>
            ))}
          </div>
        </div>
      )}

      {/* Violations */}
      {result.violations.length > 0 && (
        <div>
          <h4 className="text-sm font-medium mb-2 text-destructive">
            Violations:
          </h4>
          <ul className="space-y-2">
            {result.violations.map((v, idx) => (
              <li key={idx} className="flex items-start gap-2 text-sm">
                <AlertTriangle className="h-4 w-4 text-destructive mt-0.5" />
                <span className="text-muted-foreground">{v.message}</span>
              </li>
            ))}
          </ul>
        </div>
      )}
    </Card>
  );
}
```

---

## Phase 6: Testing (Day 16-18)

### Step 1: Setup Vitest

```typescript
// vitest.config.ts
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
});
```

```typescript
// tests/setup.ts
import '@testing-library/jest-dom';
```

### Step 2: Write Unit Tests

```typescript
// tests/unit/rdf/parser.test.ts
import { describe, it, expect } from 'vitest';
import { parseTurtle } from '@/lib/rdf/parser';

describe('RDF Parser', () => {
  it('should parse valid Turtle', async () => {
    const turtle = `
      @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .
      @prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

      <http://example.org/workflow1> a yawl:Workflow ;
        yawl:specName "Test Workflow" ;
        rdfs:comment "A test workflow" .
    `;

    const store = await parseTurtle(turtle);
    expect(store.size).toBeGreaterThan(0);
  });

  it('should throw on invalid Turtle', async () => {
    const invalidTurtle = '@prefix invalid syntax here';
    await expect(parseTurtle(invalidTurtle)).rejects.toThrow();
  });
});
```

### Step 3: Run Tests

```bash
# Run unit tests
npm run test

# Run with coverage
npm run test -- --coverage

# Run in watch mode
npm run test -- --watch
```

---

## Phase 7: Deployment (Day 19-20)

### Step 1: Configure Environment

```bash
# .env.local
NEXT_PUBLIC_API_URL=http://localhost:3000
RUST_BACKEND_URL=http://localhost:8080
WEAVER_ENDPOINT=http://localhost:9090
```

### Step 2: Deploy to Vercel

```bash
# Install Vercel CLI
npm install -g vercel

# Login to Vercel
vercel login

# Deploy
vercel

# Deploy to production
vercel --prod
```

### Step 3: Configure Production Environment

```bash
# Set environment variables in Vercel dashboard
vercel env add RUST_BACKEND_URL production
vercel env add WEAVER_ENDPOINT production
```

---

## Next Steps

1. **Implement Workflow Editor** (Phase 3 from roadmap)
2. **Add MAPE-K Integration** (Phase 5 from roadmap)
3. **Integrate Weaver Validation** (Phase 6 from roadmap)
4. **Build Pattern Library** (Phase 7 from roadmap)
5. **Add Export Features** (Phase 8 from roadmap)

---

## Troubleshooting

### Common Issues

**Issue**: N3 parser fails to parse Turtle
- **Solution**: Check Turtle syntax, ensure proper prefix declarations

**Issue**: React Flow nodes not rendering
- **Solution**: Ensure `nodeTypes` object includes all custom node types

**Issue**: SPARQL queries return empty results
- **Solution**: Verify RDF namespaces match exactly

**Issue**: Workflow validation fails
- **Solution**: Check pattern matrix file is loaded correctly

---

## Resources

- [Next.js Documentation](https://nextjs.org/docs)
- [shadcn/ui Components](https://ui.shadcn.com)
- [React Flow Documentation](https://reactflow.dev)
- [N3.js Documentation](https://github.com/rdfjs/N3.js)
- [YAWL Foundation](http://www.yawlfoundation.org)

---

**Document Status**: DRAFT
**Complements**: NEXTJS_YAWL_UI_ARCHITECTURE.md

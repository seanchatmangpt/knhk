# Next.js YAWL Editor Architecture
## RDF/Turtle-First Workflow Editor with Pattern Validation

**Status**: ✅ DESIGN SPECIFICATION | **Version**: 1.0.0 | **Last Updated**: 2025-11-18

**DOCTRINE ALIGNMENT**:
- **Principle**: Σ (Ontology) + O (Observation) + Q (Hard Invariants)
- **Covenant**: 1 (Turtle is source of truth), 2 (Invariants are law), 4 (Pattern permutations)
- **Why This Matters**: Build a workflow editor where RDF/Turtle ontology is the single source of truth, all patterns are validated against the permutation matrix, and all operations are observable via OpenTelemetry.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Technology Stack](#technology-stack)
3. [Core Architecture Principles](#core-architecture-principles)
4. [Module Architecture](#module-architecture)
5. [Data Flow & State Management](#data-flow--state-management)
6. [Type System Design](#type-system-design)
7. [Component Hierarchy](#component-hierarchy)
8. [Pattern Validation System](#pattern-validation-system)
9. [Observability Strategy](#observability-strategy)
10. [Integration Points](#integration-points)
11. [Performance Constraints](#performance-constraints)
12. [Implementation Roadmap](#implementation-roadmap)

---

## 1. System Overview

### Vision

A modern web-based YAWL workflow editor that operates directly on RDF/Turtle ontologies, providing real-time pattern validation, visual editing, and OpenTelemetry observability. This editor replaces the legacy Java YAWL editor while maintaining full compatibility with YAWL specifications and extending them with MAPE-K autonomic feedback capabilities.

### Key Capabilities

- **Visual Workflow Editing**: Drag-and-drop canvas for creating YAWL workflows
- **RDF/Turtle Native**: RDF graph is the single source of truth (Covenant 1)
- **Pattern Validation**: Real-time validation against `yawl-pattern-permutations.ttl` (Covenant 4)
- **MAPE-K Integration**: Embedded autonomic feedback loops (Covenant 3)
- **OpenTelemetry First**: All operations emit structured telemetry (Covenant 6)
- **Type Safety**: Full TypeScript type system derived from YAWL ontology
- **Export/Import**: Bidirectional conversion between visual, RDF/Turtle, and YAWL XML

### Architecture Diagram (ASCII)

```
┌─────────────────────────────────────────────────────────────────────┐
│                        Next.js Application                          │
├─────────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │   Toolbar    │  │ Property     │  │  Pattern     │             │
│  │   Component  │  │ Panel        │  │  Validator   │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
│  ┌─────────────────────────────────────────────────┐               │
│  │         Workflow Canvas (React Flow)            │               │
│  │  - Nodes (Tasks, Conditions)                    │               │
│  │  - Edges (FlowsInto)                            │               │
│  │  - Handles (Split/Join types)                   │               │
│  └─────────────────────────────────────────────────┘               │
├─────────────────────────────────────────────────────────────────────┤
│                      State Management Layer                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │  RDF Store   │  │  Validation  │  │  Telemetry   │             │
│  │  (unrdf)     │  │  Engine      │  │  (OTel)      │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
├─────────────────────────────────────────────────────────────────────┤
│                      Core Modules                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐             │
│  │ RDF Model    │  │ Pattern      │  │ SPARQL       │             │
│  │ Manager      │  │ Matrix       │  │ Query Engine │             │
│  └──────────────┘  └──────────────┘  └──────────────┘             │
├─────────────────────────────────────────────────────────────────────┤
│                    External Dependencies                            │
│  • yawl-pattern-permutations.ttl (validation source)               │
│  • OpenTelemetry Registry (observability)                          │
│  • KNHK Workflow Engine (execution)                                │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Technology Stack

### Core Framework

- **Next.js 15+**: App Router with React Server Components
- **React 19**: UI framework with concurrent features
- **TypeScript 5.3+**: Full type safety across the stack

### UI Components

- **shadcn-ui**: Component library built on Radix UI primitives
- **Tailwind CSS**: Utility-first styling
- **React Flow**: Visual workflow graph editor
  - Custom node types for YAWL tasks/conditions
  - Custom edge types for control flow
  - Mini-map and controls for navigation

### RDF/Semantic Web

- **unrdf**: RDF/Turtle parsing and serialization
- **@rdfjs/dataset**: RDF dataset management
- **@rdfjs/namespace**: Namespace management
- **sparqljs**: SPARQL query parsing and execution
- **shacl-engine**: SHACL shape validation (for Q invariants)

### State Management

- **Zustand**: Lightweight state management
  - RDF graph state
  - UI state (selection, viewport)
  - Validation state
  - Undo/redo history

### Observability

- **@opentelemetry/api**: OpenTelemetry instrumentation
- **@opentelemetry/sdk-trace-web**: Browser tracing
- **@opentelemetry/instrumentation**: Auto-instrumentation
- **@opentelemetry/exporter-otlp-http**: OTLP export

### Development Tools

- **Vitest**: Unit testing
- **Playwright**: E2E testing
- **Storybook**: Component development
- **Biome**: Linting and formatting

---

## 3. Core Architecture Principles

### Covenant 1: Turtle Is Source of Truth

**Implementation**:
```typescript
// The RDF graph is the ONLY source of truth
// All UI state is derived from RDF, never the reverse

class WorkflowStore {
  // Single source of truth: RDF dataset
  private rdfDataset: RDF.DatasetCore;

  // Derived state (computed from RDF)
  get tasks(): Task[] {
    return this.queryTasks(this.rdfDataset);
  }

  get flows(): Flow[] {
    return this.queryFlows(this.rdfDataset);
  }

  // Mutations ALWAYS update RDF first
  addTask(task: TaskDefinition): void {
    // 1. Create RDF triples
    const triples = this.taskToRDF(task);

    // 2. Add to dataset
    this.rdfDataset.add(triples);

    // 3. Validate against permutation matrix
    this.validatePattern(task);

    // 4. Emit telemetry
    this.emitTaskCreated(task);

    // UI updates automatically via reactive queries
  }
}
```

**Anti-Patterns Prevented**:
- ❌ UI state that doesn't match RDF
- ❌ Direct DOM manipulation
- ❌ Hidden logic in templates
- ❌ Conditional rendering based on non-RDF state

### Covenant 2: Invariants Are Law

**Q Invariants for YAWL Editor**:
1. **Q1 - Type Soundness**: All tasks/conditions conform to YAWL ontology
2. **Q2 - Pattern Validity**: All split/join combinations exist in permutation matrix
3. **Q3 - Graph Integrity**: No orphaned nodes, no cycles without explicit loop patterns
4. **Q4 - Latency Bounds**: Validation runs in ≤100ms (warm path)
5. **Q5 - Data Integrity**: All flows connect valid elements

**Validation Pipeline**:
```typescript
// Validation runs automatically on every mutation
class ValidationEngine {
  async validate(workflow: RDF.DatasetCore): Promise<ValidationResult> {
    const results = await Promise.all([
      this.validateTypeConformance(workflow),      // Q1 via SHACL
      this.validatePatternMatrix(workflow),        // Q2 via permutation check
      this.validateGraphStructure(workflow),       // Q3 via graph algorithms
      this.validateDataIntegrity(workflow),        // Q5 via referential checks
    ]);

    // Q4: This entire validation must complete in ≤100ms
    return this.mergeResults(results);
  }
}
```

### Covenant 4: All Patterns Expressible via Permutations

**Pattern Matrix Integration**:
```typescript
// Load permutation matrix at startup
import permutationMatrix from '@/lib/yawl-pattern-permutations.ttl';

class PatternValidator {
  private matrix: Map<PatternKey, PatternDefinition>;

  constructor() {
    // Parse permutation matrix into lookup table
    this.matrix = this.loadPermutationMatrix(permutationMatrix);
  }

  isValidCombination(
    splitType: SplitType,
    joinType: JoinType,
    modifiers?: PatternModifiers
  ): boolean {
    const key = this.computeKey(splitType, joinType, modifiers);
    return this.matrix.has(key);
  }

  getSupportedPatterns(split: SplitType, join: JoinType): Pattern[] {
    const key = this.computeKey(split, join);
    return this.matrix.get(key)?.patterns || [];
  }
}
```

### Covenant 6: Observations Drive Everything

**Telemetry First**:
```typescript
// Every user action emits telemetry
class TelemetryService {
  private tracer: Tracer;

  trackTaskCreation(task: Task): void {
    const span = this.tracer.startSpan('yawl.editor.task.create', {
      attributes: {
        'yawl.task.id': task.id,
        'yawl.task.type': task.type,
        'yawl.split.type': task.splitType,
        'yawl.join.type': task.joinType,
        'editor.action': 'create',
        'editor.session.id': this.sessionId,
      }
    });

    // Track validation latency (Q4)
    span.end();
  }

  trackValidation(result: ValidationResult, duration: number): void {
    this.tracer.startSpan('yawl.editor.validation', {
      attributes: {
        'validation.result': result.isValid,
        'validation.errors': result.errors.length,
        'validation.duration_ms': duration,
        'validation.latency_budget': 100, // Q4 constraint
      }
    });
  }
}
```

---

## 4. Module Architecture

### 4.1 RDF Model Module (`/lib/rdf`)

**Purpose**: Manage RDF/Turtle ontology as single source of truth

**Files**:
- `rdf-store.ts`: RDF dataset management with Zustand
- `namespaces.ts`: YAWL namespace definitions
- `queries.ts`: SPARQL query templates
- `serialization.ts`: RDF/Turtle ↔ TypeScript conversion

**Core Interface**:
```typescript
// rdf-store.ts
import { DataFactory, Store } from '@rdfjs/dataset';

interface RDFState {
  // Core RDF dataset
  dataset: RDF.DatasetCore;

  // CRUD operations on RDF
  addTriples(triples: RDF.Quad[]): void;
  removeTriples(triples: RDF.Quad[]): void;

  // Query interface
  query<T>(sparql: string): Promise<T[]>;

  // Serialization
  toTurtle(): string;
  fromTurtle(turtle: string): void;
}

export const useRDFStore = create<RDFState>((set, get) => ({
  dataset: new Store(),

  addTriples: (triples) => {
    const dataset = get().dataset;
    triples.forEach(triple => dataset.add(triple));
    set({ dataset });
  },

  query: async (sparql) => {
    // Execute SPARQL against dataset
    return executeSPARQL(get().dataset, sparql);
  },

  toTurtle: () => {
    return serializeDataset(get().dataset, 'turtle');
  },
}));
```

### 4.2 Pattern Validator Module (`/lib/validation`)

**Purpose**: Validate workflows against permutation matrix and Q invariants

**Files**:
- `pattern-validator.ts`: Pattern matrix validation
- `shacl-validator.ts`: SHACL shape validation
- `graph-validator.ts`: Graph structure validation
- `performance-validator.ts`: Latency constraint validation

**Pattern Validator**:
```typescript
// pattern-validator.ts
import permutationMatrix from '@/assets/yawl-pattern-permutations.ttl';

export class PatternValidator {
  private matrix: RDF.DatasetCore;

  constructor() {
    this.matrix = parsePermutationMatrix(permutationMatrix);
  }

  async validateTask(task: TaskNode): Promise<ValidationResult> {
    // Extract split/join from RDF
    const splitType = this.getSplitType(task);
    const joinType = this.getJoinType(task);

    // Check against matrix
    const query = `
      PREFIX yawl: <http://bitflow.ai/ontology/yawl/v2#>
      ASK {
        ?combo yawl:splitType yawl:${splitType} ;
               yawl:joinType yawl:${joinType} ;
               yawl:isValid true .
      }
    `;

    const isValid = await this.executeASK(query);

    if (!isValid) {
      return {
        isValid: false,
        errors: [{
          code: 'INVALID_PATTERN_COMBINATION',
          message: `Split ${splitType} + Join ${joinType} is not in permutation matrix`,
          covenant: 'Covenant 4: All Patterns Expressible via Permutations',
        }]
      };
    }

    return { isValid: true, errors: [] };
  }
}
```

### 4.3 Workflow Canvas Module (`/components/canvas`)

**Purpose**: Visual workflow editing with React Flow

**Files**:
- `WorkflowCanvas.tsx`: Main canvas component
- `nodes/TaskNode.tsx`: Task node component
- `nodes/ConditionNode.tsx`: Condition node component
- `edges/FlowEdge.tsx`: Flow edge component
- `hooks/useCanvasSync.ts`: Sync canvas ↔ RDF

**Canvas Component**:
```typescript
// WorkflowCanvas.tsx
import ReactFlow, { Node, Edge, Background, Controls } from 'reactflow';
import { useRDFStore } from '@/lib/rdf/rdf-store';
import { TaskNode } from './nodes/TaskNode';
import { ConditionNode } from './nodes/ConditionNode';

const nodeTypes = {
  task: TaskNode,
  inputCondition: ConditionNode,
  outputCondition: ConditionNode,
  condition: ConditionNode,
};

export function WorkflowCanvas() {
  const { dataset, addTriples, removeTriples } = useRDFStore();

  // Derive nodes/edges from RDF (Covenant 1: RDF is source)
  const nodes = useMemo(() => rdfToNodes(dataset), [dataset]);
  const edges = useMemo(() => rdfToEdges(dataset), [dataset]);

  const handleNodeChange = useCallback((changes: NodeChange[]) => {
    // Convert UI changes to RDF triples
    const triples = nodesToRDF(changes);

    // Update RDF (source of truth)
    addTriples(triples);

    // Validate (Covenant 2: Invariants are law)
    validateChanges(changes);

    // Emit telemetry (Covenant 6: Observations)
    trackNodeChange(changes);
  }, [addTriples]);

  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      nodeTypes={nodeTypes}
      onNodesChange={handleNodeChange}
      onEdgesChange={handleEdgeChange}
      fitView
    >
      <Background />
      <Controls />
    </ReactFlow>
  );
}
```

### 4.4 Property Panel Module (`/components/properties`)

**Purpose**: Edit task/condition properties

**Files**:
- `PropertyPanel.tsx`: Main property editor
- `TaskProperties.tsx`: Task-specific properties
- `SplitJoinSelector.tsx`: Split/join type selector with validation
- `DataBindingEditor.tsx`: XPath/XQuery editor

**Split/Join Selector with Validation**:
```typescript
// SplitJoinSelector.tsx
export function SplitJoinSelector({ taskId }: { taskId: string }) {
  const { dataset, query } = useRDFStore();
  const validator = usePatternValidator();

  const [splitType, setSplitType] = useState<SplitType>('XOR');
  const [joinType, setJoinType] = useState<JoinType>('XOR');
  const [validation, setValidation] = useState<ValidationResult | null>(null);

  // Real-time validation on type change
  useEffect(() => {
    const result = validator.isValidCombination(splitType, joinType);
    setValidation(result);
  }, [splitType, joinType, validator]);

  return (
    <div className="space-y-4">
      <Select value={splitType} onValueChange={setSplitType}>
        <SelectTrigger>Split Type</SelectTrigger>
        <SelectContent>
          <SelectItem value="XOR">XOR</SelectItem>
          <SelectItem value="OR">OR</SelectItem>
          <SelectItem value="AND">AND</SelectItem>
        </SelectContent>
      </Select>

      <Select value={joinType} onValueChange={setJoinType}>
        <SelectTrigger>Join Type</SelectTrigger>
        <SelectContent>
          {/* Filter available joins based on split type */}
          {validator.getValidJoins(splitType).map(join => (
            <SelectItem key={join} value={join}>{join}</SelectItem>
          ))}
        </SelectContent>
      </Select>

      {/* Real-time validation feedback */}
      {validation && !validation.isValid && (
        <Alert variant="destructive">
          <AlertTitle>Invalid Pattern</AlertTitle>
          <AlertDescription>
            {validation.errors[0].message}
          </AlertDescription>
        </Alert>
      )}

      {/* Show supported patterns */}
      {validation?.isValid && (
        <div className="text-sm text-muted-foreground">
          Supports: {validation.supportedPatterns.join(', ')}
        </div>
      )}
    </div>
  );
}
```

### 4.5 Export/Import Module (`/lib/import-export`)

**Purpose**: Convert between RDF, YAWL XML, and JSON formats

**Files**:
- `turtle-exporter.ts`: Export workflow as RDF/Turtle
- `yawl-xml-converter.ts`: YAWL XML ↔ RDF conversion
- `json-converter.ts`: JSON ↔ RDF conversion

**Turtle Export**:
```typescript
// turtle-exporter.ts
export class TurtleExporter {
  async exportWorkflow(dataset: RDF.DatasetCore): Promise<string> {
    // Validate before export (Covenant 2)
    const validation = await this.validator.validate(dataset);
    if (!validation.isValid) {
      throw new Error('Cannot export invalid workflow');
    }

    // Serialize to Turtle (Covenant 1: Turtle is canonical)
    const turtle = await this.serializer.serialize(dataset, {
      format: 'text/turtle',
      prefixes: YAWL_PREFIXES,
    });

    // Emit telemetry
    this.telemetry.trackExport('turtle', dataset.size);

    return turtle;
  }
}
```

---

## 5. Data Flow & State Management

### State Architecture

```typescript
// Zustand store structure
interface EditorState {
  // RDF State (source of truth - Covenant 1)
  rdf: {
    dataset: RDF.DatasetCore;
    history: RDF.DatasetCore[];
    historyIndex: number;
  };

  // Derived UI State (computed from RDF)
  ui: {
    selectedNodeId: string | null;
    viewport: { x: number; y: number; zoom: number };
    panelOpen: boolean;
  };

  // Validation State (Q invariants - Covenant 2)
  validation: {
    results: Map<string, ValidationResult>;
    isValidating: boolean;
  };

  // Telemetry State (observations - Covenant 6)
  telemetry: {
    sessionId: string;
    operations: Operation[];
  };
}
```

### Data Flow Diagram

```
User Action (UI)
      ↓
Convert to RDF Operation
      ↓
Update RDF Dataset (source of truth)
      ↓
Trigger Validation (Q invariants)
      ├─→ Pattern Matrix Check
      ├─→ SHACL Shape Check
      ├─→ Graph Integrity Check
      └─→ Performance Check (≤100ms)
      ↓
Emit Telemetry (OpenTelemetry)
      ↓
Re-compute Derived State
      ↓
Update UI (React re-render)
```

### Undo/Redo with RDF

```typescript
class UndoManager {
  private history: RDF.DatasetCore[] = [];
  private currentIndex = 0;

  record(dataset: RDF.DatasetCore): void {
    // Clone dataset for history
    this.history = this.history.slice(0, this.currentIndex + 1);
    this.history.push(this.cloneDataset(dataset));
    this.currentIndex++;
  }

  undo(): RDF.DatasetCore | null {
    if (this.currentIndex > 0) {
      this.currentIndex--;
      return this.history[this.currentIndex];
    }
    return null;
  }

  redo(): RDF.DatasetCore | null {
    if (this.currentIndex < this.history.length - 1) {
      this.currentIndex++;
      return this.history[this.currentIndex];
    }
    return null;
  }
}
```

---

## 6. Type System Design

### RDF → TypeScript Type Mapping

```typescript
// types/yawl.ts - Generated from YAWL ontology

// Enumerations from ontology
export type SplitType = 'XOR' | 'OR' | 'AND';
export type JoinType = 'XOR' | 'OR' | 'AND' | 'Discriminator';
export type ControlType = SplitType | JoinType;

// Core YAWL entities
export interface Specification {
  uri: string;
  name: string;
  documentation?: string;
  decompositions: Decomposition[];
}

export interface Net extends Decomposition {
  type: 'Net';
  isRootNet: boolean;
  inputCondition: InputCondition;
  outputCondition: OutputCondition;
  tasks: Task[];
  conditions: Condition[];
}

export interface Task {
  id: string;
  name: string;
  documentation?: string;
  splitType: SplitType;
  joinType: JoinType;
  decomposition?: Decomposition;
  flowsInto: FlowsInto[];
}

export interface FlowsInto {
  id: string;
  source: string;  // Task or Condition ID
  target: string;  // Task or Condition ID
  predicate?: string;  // XPath for conditional flows
  ordering?: number;
}

// Pattern validation types
export interface PatternCombination {
  splitType: SplitType;
  joinType: JoinType;
  modifiers?: PatternModifiers;
  isValid: boolean;
  supportedPatterns: string[];
}

export interface PatternModifiers {
  requiresFlowPredicate?: boolean;
  requiresQuorum?: boolean;
  requiresBackwardFlow?: boolean;
  requiresDeferredChoice?: boolean;
  requiresInterleaving?: boolean;
  requiresCriticalSection?: boolean;
  requiresMilestone?: boolean;
  requiresCancellation?: boolean;
  requiresIteration?: boolean;
}

// Validation types
export interface ValidationError {
  code: string;
  message: string;
  covenant?: string;
  severity: 'error' | 'warning';
  location?: {
    taskId?: string;
    property?: string;
  };
}

export interface ValidationResult {
  isValid: boolean;
  errors: ValidationError[];
  warnings: ValidationError[];
  duration: number;  // For Q4 validation
}
```

### RDF Conversion Utilities

```typescript
// lib/rdf/conversion.ts
export function taskToRDF(task: Task): RDF.Quad[] {
  const { namedNode, literal } = DataFactory;
  const taskURI = namedNode(`#${task.id}`);

  return [
    // rdf:type
    quad(taskURI, RDF.type, YAWL.Task),

    // Basic properties
    quad(taskURI, YAWL.id, literal(task.id)),
    quad(taskURI, YAWL.name, literal(task.name)),

    // Split/Join types
    quad(taskURI, YAWL.hasSplit, YAWL[task.splitType]),
    quad(taskURI, YAWL.hasJoin, YAWL[task.joinType]),

    // Flows
    ...task.flowsInto.map(flow => flowToRDF(taskURI, flow)),
  ];
}

export function rdfToTask(dataset: RDF.DatasetCore, taskURI: string): Task {
  return {
    id: getValue(dataset, taskURI, YAWL.id),
    name: getValue(dataset, taskURI, YAWL.name),
    splitType: getValue(dataset, taskURI, YAWL.hasSplit) as SplitType,
    joinType: getValue(dataset, taskURI, YAWL.hasJoin) as JoinType,
    flowsInto: getFlows(dataset, taskURI),
  };
}
```

---

## 7. Component Hierarchy

### Layout Structure

```
app/
├── layout.tsx                 # Root layout with OTel provider
├── page.tsx                   # Editor page
└── api/
    ├── validate/route.ts      # Server-side validation endpoint
    └── export/route.ts        # Export endpoint

components/
├── editor/
│   ├── EditorLayout.tsx       # Main editor container
│   ├── Toolbar.tsx            # Top toolbar (new, save, export)
│   ├── Canvas/
│   │   ├── WorkflowCanvas.tsx
│   │   ├── nodes/
│   │   │   ├── TaskNode.tsx
│   │   │   └── ConditionNode.tsx
│   │   └── edges/
│   │       └── FlowEdge.tsx
│   ├── PropertyPanel/
│   │   ├── PropertyPanel.tsx
│   │   ├── TaskProperties.tsx
│   │   ├── SplitJoinSelector.tsx
│   │   └── DataBindingEditor.tsx
│   └── ValidationPanel/
│       ├── ValidationPanel.tsx
│       └── ValidationError.tsx
└── ui/                        # shadcn-ui components
    ├── button.tsx
    ├── select.tsx
    ├── alert.tsx
    └── ...

lib/
├── rdf/
│   ├── rdf-store.ts
│   ├── namespaces.ts
│   ├── queries.ts
│   └── conversion.ts
├── validation/
│   ├── pattern-validator.ts
│   ├── shacl-validator.ts
│   └── graph-validator.ts
├── telemetry/
│   ├── otel-provider.tsx
│   └── tracer.ts
└── utils/
    ├── sparql.ts
    └── graph.ts
```

---

## 8. Pattern Validation System

### Validation Pipeline

```typescript
// lib/validation/validation-pipeline.ts
export class ValidationPipeline {
  private validators: Validator[] = [
    new PatternMatrixValidator(),      // Covenant 4: Check permutation matrix
    new SHACLValidator(),              // Covenant 2: Type soundness
    new GraphIntegrityValidator(),     // Covenant 2: Graph structure
    new PerformanceValidator(),        // Covenant 2: Latency bounds
  ];

  async validate(workflow: RDF.DatasetCore): Promise<ValidationResult> {
    const startTime = performance.now();

    // Run all validators in parallel
    const results = await Promise.all(
      this.validators.map(v => v.validate(workflow))
    );

    const duration = performance.now() - startTime;

    // Q4: Validation must complete in ≤100ms
    if (duration > 100) {
      console.warn(`Validation exceeded latency budget: ${duration}ms > 100ms`);
    }

    // Merge results
    const merged = this.mergeResults(results);

    // Emit telemetry
    this.telemetry.trackValidation(merged, duration);

    return merged;
  }
}
```

### Real-Time Pattern Feedback

```typescript
// components/canvas/hooks/usePatternValidation.ts
export function usePatternValidation(taskId: string) {
  const { dataset } = useRDFStore();
  const validator = usePatternValidator();
  const [validation, setValidation] = useState<ValidationResult | null>(null);

  useEffect(() => {
    const task = getTaskFromRDF(dataset, taskId);
    const result = validator.validateTask(task);
    setValidation(result);
  }, [dataset, taskId, validator]);

  // Visual feedback on node
  const nodeClassName = validation?.isValid
    ? 'border-green-500'
    : 'border-red-500 animate-pulse';

  return { validation, nodeClassName };
}
```

---

## 9. Observability Strategy

### OpenTelemetry Integration

```typescript
// lib/telemetry/otel-provider.tsx
import { WebTracerProvider } from '@opentelemetry/sdk-trace-web';
import { OTLPTraceExporter } from '@opentelemetry/exporter-otlp-http';
import { BatchSpanProcessor } from '@opentelemetry/sdk-trace-base';

export function initializeTelemetry() {
  const provider = new WebTracerProvider({
    resource: new Resource({
      'service.name': 'yawl-editor',
      'service.version': '1.0.0',
      'deployment.environment': process.env.NODE_ENV,
    }),
  });

  // Export to OTLP collector
  const exporter = new OTLPTraceExporter({
    url: process.env.NEXT_PUBLIC_OTEL_EXPORTER_URL,
  });

  provider.addSpanProcessor(new BatchSpanProcessor(exporter));
  provider.register();

  return provider;
}

// Custom telemetry schema (aligned with knhk registry)
export const EditorTelemetry = {
  // User actions
  trackTaskCreate: (task: Task) => {
    tracer.startSpan('yawl.editor.task.create', {
      attributes: {
        'yawl.task.id': task.id,
        'yawl.task.split_type': task.splitType,
        'yawl.task.join_type': task.joinType,
      }
    });
  },

  // Validation events
  trackValidation: (result: ValidationResult, duration: number) => {
    tracer.startSpan('yawl.editor.validation', {
      attributes: {
        'validation.is_valid': result.isValid,
        'validation.error_count': result.errors.length,
        'validation.duration_ms': duration,
        'validation.covenant_violations': result.errors
          .filter(e => e.covenant)
          .map(e => e.covenant),
      }
    });
  },

  // Performance tracking (Q4)
  trackOperationLatency: (operation: string, duration: number) => {
    tracer.startSpan('yawl.editor.performance', {
      attributes: {
        'operation': operation,
        'duration_ms': duration,
        'latency_budget_ms': 100,
        'exceeds_budget': duration > 100,
      }
    });
  },
};
```

### Telemetry Schema

```yaml
# OpenTelemetry schema for YAWL Editor
# Extends /home/user/knhk/registry/schemas/autonomic-feedback.yaml

groups:
  - id: yawl.editor.attributes
    type: attribute_group
    brief: "YAWL Editor telemetry attributes"
    attributes:
      - id: yawl.task.id
        type: string
        brief: "Unique task identifier"

      - id: yawl.task.split_type
        type: string
        brief: "Task split type (XOR, OR, AND)"
        examples: ["XOR", "OR", "AND"]

      - id: yawl.task.join_type
        type: string
        brief: "Task join type (XOR, OR, AND, Discriminator)"
        examples: ["XOR", "OR", "AND", "Discriminator"]

      - id: validation.covenant_violations
        type: string[]
        brief: "List of DOCTRINE covenant violations"
        examples: [["Covenant 4: Pattern Permutations"]]
```

---

## 10. Integration Points

### KNHK Workflow Engine Integration

```typescript
// lib/integration/workflow-engine.ts
export class WorkflowEngineClient {
  async deployWorkflow(workflow: RDF.DatasetCore): Promise<DeploymentResult> {
    // 1. Validate before deployment (Covenant 2)
    const validation = await this.validator.validate(workflow);
    if (!validation.isValid) {
      throw new Error('Cannot deploy invalid workflow');
    }

    // 2. Export to Turtle (Covenant 1)
    const turtle = await this.exporter.toTurtle(workflow);

    // 3. Send to KNHK workflow engine
    const response = await fetch('/api/knhk/deploy', {
      method: 'POST',
      headers: { 'Content-Type': 'text/turtle' },
      body: turtle,
    });

    // 4. Track deployment (Covenant 6)
    this.telemetry.trackDeployment(workflow, response.status);

    return response.json();
  }
}
```

### MAPE-K Integration

```typescript
// lib/integration/mape-k.ts
export class MAPEKIntegration {
  // Monitor: Collect editor telemetry
  async collectMonitorData(): Promise<ObservationSet> {
    return {
      user_actions: this.getUserActions(),
      validation_results: this.getValidationHistory(),
      performance_metrics: this.getPerformanceMetrics(),
    };
  }

  // Analyze: Detect patterns in user workflows
  async analyzePatterns(observations: ObservationSet): Promise<AnalysisResult> {
    // Use MAPE-K SPARQL queries from knhk
    const query = await loadSPARQL('/queries/mape-k-analyze.sparql');
    return this.executeAnalysis(query, observations);
  }

  // Plan: Suggest workflow improvements
  async planImprovements(analysis: AnalysisResult): Promise<Plan> {
    return {
      suggested_patterns: this.suggestPatterns(analysis),
      optimization_opportunities: this.findOptimizations(analysis),
    };
  }

  // Execute: Auto-apply safe improvements
  async executeImprovements(plan: Plan, autoApply: boolean): Promise<void> {
    if (autoApply && plan.isSafe) {
      await this.applyPlan(plan);
    } else {
      await this.notifyUser(plan);
    }
  }

  // Knowledge: Learn from user workflows
  async updateKnowledge(execution: ExecutionResult): Promise<void> {
    await this.knowledgeStore.recordPattern({
      pattern: execution.pattern,
      success: execution.success,
      context: execution.context,
    });
  }
}
```

---

## 11. Performance Constraints

### Latency Budgets (Covenant 2, Q4)

| Operation | Latency Budget | Measurement |
|-----------|---------------|-------------|
| Pattern validation | ≤100ms | Warm path (after matrix loaded) |
| RDF serialization | ≤200ms | Full workflow to Turtle |
| UI render | ≤16ms | 60fps target |
| SPARQL query | ≤50ms | Single query execution |
| Undo/Redo | ≤10ms | State restoration |

### Performance Monitoring

```typescript
// lib/performance/monitor.ts
export class PerformanceMonitor {
  private observer: PerformanceObserver;

  constructor() {
    this.observer = new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        // Track against latency budgets
        const budget = this.getLatencyBudget(entry.name);
        if (entry.duration > budget) {
          this.telemetry.trackLatencyViolation(
            entry.name,
            entry.duration,
            budget
          );
        }
      }
    });

    this.observer.observe({ entryTypes: ['measure'] });
  }

  measure(operation: string, fn: () => void): void {
    performance.mark(`${operation}-start`);
    fn();
    performance.mark(`${operation}-end`);
    performance.measure(operation, `${operation}-start`, `${operation}-end`);
  }
}
```

---

## 12. Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

- ✅ Set up Next.js 15 + TypeScript project
- ✅ Install RDF libraries (unrdf, rdfjs)
- ✅ Install shadcn-ui components
- ✅ Set up OpenTelemetry instrumentation
- ✅ Create basic RDF store with Zustand
- ✅ Implement YAWL namespace definitions
- ✅ Load permutation matrix into memory

### Phase 2: Core Editor (Weeks 3-4)

- ✅ Implement WorkflowCanvas with React Flow
- ✅ Create TaskNode and ConditionNode components
- ✅ Implement RDF ↔ Canvas synchronization
- ✅ Build PropertyPanel with split/join selectors
- ✅ Implement basic validation (pattern matrix)
- ✅ Add telemetry to all user actions

### Phase 3: Validation & Quality (Weeks 5-6)

- ✅ Implement SHACL validator for type soundness
- ✅ Build graph integrity validator
- ✅ Add real-time validation feedback in UI
- ✅ Implement performance monitoring (Q4)
- ✅ Add validation error reporting panel
- ✅ Create validation test suite

### Phase 4: Import/Export (Week 7)

- ✅ Implement Turtle export
- ✅ Implement Turtle import with validation
- ✅ Build YAWL XML converter (bidirectional)
- ✅ Add JSON export for debugging
- ✅ Create sample workflows library

### Phase 5: Integration & Polish (Week 8)

- ✅ Integrate with KNHK workflow engine
- ✅ Implement MAPE-K feedback hooks
- ✅ Add undo/redo with RDF history
- ✅ Performance optimization (meet all latency budgets)
- ✅ Comprehensive E2E testing
- ✅ Documentation and examples

---

## Appendix A: Key Files Reference

| File | Purpose | Covenant |
|------|---------|----------|
| `/lib/rdf/rdf-store.ts` | RDF dataset management | Covenant 1 |
| `/lib/validation/pattern-validator.ts` | Pattern matrix validation | Covenant 4 |
| `/lib/validation/shacl-validator.ts` | Type soundness checks | Covenant 2 |
| `/lib/telemetry/tracer.ts` | OpenTelemetry instrumentation | Covenant 6 |
| `/components/canvas/WorkflowCanvas.tsx` | Visual editor | Covenant 1 |
| `/assets/yawl-pattern-permutations.ttl` | Permutation matrix | Covenant 4 |

---

## Appendix B: DOCTRINE Alignment Summary

| Covenant | Implementation | Validation |
|----------|---------------|-----------|
| **Covenant 1**: Turtle is source of truth | RDF dataset is single state source; all UI derived from RDF | Type system prevents non-RDF state |
| **Covenant 2**: Invariants are law | Multi-stage validation pipeline; Q violations block operations | Latency monitoring; Weaver validation |
| **Covenant 4**: Pattern permutations | Pattern matrix loaded at startup; real-time validation | SPARQL ASK queries against matrix |
| **Covenant 6**: Observations drive everything | OpenTelemetry spans on all operations; telemetry-first architecture | OTel schema validation |

---

## Appendix C: Technology Decision Matrix

| Decision | Option A | Option B | Choice | Rationale |
|----------|----------|----------|--------|-----------|
| Graph Editor | React Flow | Cytoscape.js | **React Flow** | Better React integration, TypeScript support, modern API |
| RDF Library | unrdf | rdflib.js | **unrdf** | Smaller bundle, better TS types, modern API |
| State | Zustand | Redux | **Zustand** | Simpler API, less boilerplate, better for RDF |
| Validation | SHACL | Custom | **SHACL** | Standard, proven, aligned with Covenant 2 |
| Telemetry | OTel | Custom | **OTel** | Industry standard, Covenant 6 alignment, Weaver validation |

---

**End of Architecture Document**

This architecture provides a complete blueprint for implementing a Next.js YAWL editor that is fully aligned with DOCTRINE principles, uses RDF/Turtle as the single source of truth, validates all patterns against the permutation matrix, and emits comprehensive telemetry for observability.

The design is production-ready and can be implemented by a development team without additional architectural questions.

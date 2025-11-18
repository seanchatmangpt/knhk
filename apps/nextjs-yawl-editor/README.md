# Next.js YAWL Editor

A modern, web-based YAWL workflow editor built with Next.js, React Flow, and RDF/Turtle, fully aligned with DOCTRINE 2027 principles.

## Overview

This editor operates directly on RDF/Turtle ontologies, providing real-time pattern validation against the YAWL permutation matrix, visual drag-and-drop workflow editing, and comprehensive OpenTelemetry observability.

## Key Features

- **RDF/Turtle Native**: RDF graph is the single source of truth (Covenant 1)
- **Pattern Validation**: Real-time validation against `yawl-pattern-permutations.ttl` (Covenant 4)
- **Visual Editing**: Drag-and-drop workflow canvas with React Flow
- **Type Safety**: Full TypeScript type system derived from YAWL ontology
- **Observability**: OpenTelemetry instrumentation on all operations (Covenant 6)
- **MAPE-K Integration**: Embedded autonomic feedback loops
- **Export/Import**: Bidirectional RDF/Turtle, YAWL XML, and JSON support

## DOCTRINE Alignment

This project embodies:
- **Σ (Ontology)**: RDF/Turtle ontology as executable specification
- **O (Observation)**: OpenTelemetry telemetry streams for all operations
- **Q (Hard Invariants)**: Pattern matrix validation, type soundness, latency bounds
- **Π (Projections)**: Generated code and UI from RDF ontology
- **MAPE-K**: Autonomic feedback loops for continuous improvement

See [ARCHITECTURE.md](./ARCHITECTURE.md) for complete design specification.

## Quick Start

```bash
# Install dependencies
npm install

# Run development server
npm run dev

# Build for production
npm run build

# Run tests
npm test

# Run E2E tests
npm run test:e2e

# Type check
npm run typecheck

# Lint
npm run lint
```

## Technology Stack

- **Framework**: Next.js 15+ (App Router)
- **UI**: React 19, shadcn-ui, Tailwind CSS
- **Graph Editor**: React Flow
- **RDF**: unrdf, @rdfjs/dataset, sparqljs
- **Validation**: shacl-engine
- **State**: Zustand
- **Telemetry**: OpenTelemetry (browser SDK)
- **Testing**: Vitest, Playwright

## Project Structure

```
apps/nextjs-yawl-editor/
├── ARCHITECTURE.md              # Complete architecture specification
├── README.md                    # This file
├── package.json
├── tsconfig.json
├── next.config.js
├── app/
│   ├── layout.tsx              # Root layout with OTel provider
│   ├── page.tsx                # Editor page
│   └── api/
│       ├── validate/route.ts   # Server-side validation
│       └── export/route.ts     # Export endpoint
├── components/
│   ├── editor/
│   │   ├── EditorLayout.tsx
│   │   ├── Toolbar.tsx
│   │   ├── Canvas/
│   │   │   ├── WorkflowCanvas.tsx
│   │   │   ├── nodes/
│   │   │   └── edges/
│   │   ├── PropertyPanel/
│   │   └── ValidationPanel/
│   └── ui/                     # shadcn-ui components
├── lib/
│   ├── rdf/                    # RDF store and utilities
│   ├── validation/             # Pattern validation
│   ├── telemetry/              # OpenTelemetry setup
│   ├── integration/            # KNHK integration
│   └── utils/
├── types/
│   └── yawl.ts                 # TypeScript types from ontology
├── assets/
│   └── yawl-pattern-permutations.ttl
└── tests/
    ├── unit/
    └── e2e/
```

## Development Workflow

### 1. Create a New Workflow

1. Open the editor in browser (`http://localhost:3000`)
2. Drag tasks and conditions onto the canvas
3. Connect them with flows
4. Set split/join types in the property panel
5. Real-time validation ensures pattern correctness
6. Export to RDF/Turtle or YAWL XML

### 2. Validation

All workflows are validated in real-time against:
- **Pattern Matrix**: Split/join combinations must exist in permutation matrix
- **Type Soundness**: SHACL shape validation against YAWL ontology
- **Graph Integrity**: No orphaned nodes, valid flow connections
- **Performance**: Validation completes in ≤100ms (Q4 constraint)

### 3. Observability

All operations emit OpenTelemetry spans:
- Task creation/deletion
- Flow modifications
- Validation events
- Performance metrics
- Covenant violations

View telemetry in your OTel collector (e.g., Jaeger, Grafana Tempo).

## Integration with KNHK

This editor integrates with the KNHK workflow engine:

```typescript
import { WorkflowEngineClient } from '@/lib/integration/workflow-engine';

const client = new WorkflowEngineClient();

// Deploy workflow to KNHK engine
await client.deployWorkflow(rdfDataset);
```

## MAPE-K Autonomic Feedback

The editor includes MAPE-K integration:
- **Monitor**: Collects user actions and validation results
- **Analyze**: Detects patterns and optimization opportunities
- **Plan**: Suggests workflow improvements
- **Execute**: Auto-applies safe improvements (with user approval)
- **Knowledge**: Learns from user workflows to improve suggestions

## Performance Budgets

| Operation | Budget | Measurement |
|-----------|--------|-------------|
| Pattern validation | ≤100ms | Warm path |
| RDF serialization | ≤200ms | Full workflow |
| UI render | ≤16ms | 60fps |
| SPARQL query | ≤50ms | Single query |
| Undo/Redo | ≤10ms | State restoration |

## Testing

```bash
# Unit tests
npm test

# Watch mode
npm run test:watch

# E2E tests
npm run test:e2e

# E2E with UI
npm run test:e2e:ui

# Coverage
npm run test:coverage
```

## Contributing

See [ARCHITECTURE.md](./ARCHITECTURE.md) for:
- Module architecture
- Type system design
- Validation pipeline
- Telemetry schema
- Integration points

## DOCTRINE Covenant Compliance

This project satisfies all relevant DOCTRINE covenants:

| Covenant | Implementation | Validation |
|----------|---------------|-----------|
| **1: Turtle is source of truth** | RDF dataset is single state source | Type system prevents non-RDF state |
| **2: Invariants are law** | Multi-stage validation pipeline | Latency monitoring, Weaver validation |
| **4: Pattern permutations** | Pattern matrix validation | SPARQL ASK queries against matrix |
| **6: Observations drive everything** | OpenTelemetry spans on all operations | OTel schema validation |

## References

- [ARCHITECTURE.md](./ARCHITECTURE.md) - Complete architecture specification
- [DOCTRINE_2027.md](../../DOCTRINE_2027.md) - Foundational principles
- [DOCTRINE_COVENANT.md](../../DOCTRINE_COVENANT.md) - Binding covenants
- [yawl-pattern-permutations.ttl](../../ontology/yawl-pattern-permutations.ttl) - Permutation matrix

## License

See [LICENSE](../../LICENSE) in repository root.

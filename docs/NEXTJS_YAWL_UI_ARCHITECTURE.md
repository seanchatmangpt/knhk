# Next.js + shadcn/ui YAWL UI - System Architecture Design

**Status**: DRAFT | **Version**: 1.0.0 | **Last Updated**: 2025-11-18

**Canonical Reference**: Complete system architecture for modern YAWL workflow management UI

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Doctrine Alignment](#doctrine-alignment)
3. [Architecture Decision Records](#architecture-decision-records)
4. [Technology Stack](#technology-stack)
5. [System Architecture](#system-architecture)
6. [Directory Structure](#directory-structure)
7. [Component Architecture](#component-architecture)
8. [Data Flow](#data-flow)
9. [RDF Integration Strategy](#rdf-integration-strategy)
10. [API Design](#api-design)
11. [State Management](#state-management)
12. [Workflow Visualization](#workflow-visualization)
13. [Security & Performance](#security--performance)
14. [Testing Strategy](#testing-strategy)
15. [Implementation Roadmap](#implementation-roadmap)

---

## Executive Summary

The Next.js YAWL UI is a modern, web-based workflow management interface that provides:

- **Visual workflow editor** with drag-and-drop capabilities
- **RDF/Turtle native** workflow representation
- **Real-time MAPE-K integration** for autonomous workflow monitoring
- **Full YAWL pattern support** (all 43 W3C patterns + extensions)
- **Schema-first validation** via OpenTelemetry Weaver
- **Chatman Constant compliance** (≤8 ticks hot path operations)

### Key Differentiators

1. **Turtle-First Architecture**: RDF is not serialization, it's the source of truth
2. **Zero Reconstruction Logic**: UI renders exactly what's in the ontology
3. **Live MAPE-K Feedback**: Workflows self-monitor and adapt in real-time
4. **Pattern Matrix Validation**: All workflows validated against permutation matrix
5. **Doctrine Compliance**: Every component maps to O, Σ, Q, Π, or MAPE-K principles

---

## Doctrine Alignment

**CRITICAL**: Every architectural decision in this document traces back to DOCTRINE_2027.

### Covenant Mapping

| Component | Doctrine Principle | Covenant | Implementation |
|-----------|-------------------|----------|----------------|
| RDF Parser | O (Observation) | Covenant 1 | `lib/rdf/parser.ts` - Pure RDF consumption |
| Workflow Renderer | Σ (Ontology) | Covenant 1 | `components/workflow/renderer.tsx` - Zero template logic |
| Pattern Validator | Q (Invariants) | Covenant 2 | `lib/validation/patterns.ts` - Matrix validation |
| MAPE-K Monitor | MAPE-K Loop | Covenant 3 | `lib/mapek/monitor.ts` - Real-time telemetry |
| Performance Guard | Chatman Constant | Covenant 5 | `lib/performance/guards.ts` - 8-tick enforcement |
| Telemetry Collector | O (Observation) | Covenant 6 | `lib/telemetry/collector.ts` - Weaver integration |

### What Would Violate Doctrine

❌ **Template logic that filters/transforms RDF** → Violates Covenant 1
❌ **Client-side workflow validation without matrix** → Violates Covenant 2
❌ **Manual monitoring without MAPE-K** → Violates Covenant 3
❌ **Workflows exceeding 8 ticks hot path** → Violates Covenant 5
❌ **UI state not backed by telemetry** → Violates Covenant 6

---

## Architecture Decision Records

### ADR-001: Next.js App Router (Server Components)

**Decision**: Use Next.js 15+ App Router with React Server Components

**Context**: Need server-side RDF processing, zero-latency initial renders, and SEO-friendly workflow documentation.

**Rationale**:
- **Server Components**: Parse RDF/Turtle on server, send pure HTML to client
- **Streaming SSR**: Progressive workflow rendering (start → tasks → end)
- **API Routes**: Native SPARQL endpoint integration
- **Zero JS for Static Workflows**: Read-only workflows need zero client JS

**Alternatives Considered**:
- SPA (Create React App): ❌ No SSR, poor initial load, no SEO
- Remix: ✅ Viable, but weaker ecosystem for component libraries
- SvelteKit: ✅ Viable, but TypeScript/shadcn ecosystem less mature

**Covenant Alignment**: Covenant 1 (Turtle is definition) → Server parsing ensures no client-side reconstruction

---

### ADR-002: shadcn/ui Component Library

**Decision**: Use shadcn/ui (Radix UI + Tailwind) as base component system

**Context**: Need accessible, composable, customizable UI primitives that work with RDF data.

**Rationale**:
- **Copy-Paste Architecture**: Components live in codebase, not node_modules
- **Full Customization**: Modify for YAWL-specific patterns without forking
- **Accessibility**: Radix UI provides WCAG 2.1 AA compliance
- **TypeScript Native**: Type-safe props map naturally to RDF properties
- **Tailwind Integration**: Consistent styling with utility-first approach

**Alternatives Considered**:
- Material UI: ❌ Heavy bundle, opinionated theme, hard to customize
- Ant Design: ❌ Not accessible, Chinese-first docs, large bundle
- Chakra UI: ✅ Viable, but less flexible than shadcn approach

**Covenant Alignment**: Covenant 1 (Pure passthrough) → Components render RDF properties without interpretation

---

### ADR-003: unrdf for RDF/Turtle Processing

**Decision**: Use `unrdf` library for RDF parsing and SPARQL querying

**Context**: Need pure JavaScript RDF processing that works in browser and Node.js.

**Rationale**:
- **Universal**: Runs in browser, Node.js, and Edge runtime
- **SPARQL Support**: Query RDF directly without conversion
- **Turtle Native**: First-class Turtle serialization/deserialization
- **Type-Safe**: TypeScript definitions for RDF terms
- **Lightweight**: ~50KB minified, tree-shakeable

**Alternatives Considered**:
- rdflib.js: ❌ Large bundle (200KB+), slow SPARQL engine
- graphy: ✅ Viable, but less TypeScript support
- oxigraph: ❌ WASM binary, not browser-optimized
- Custom parser: ❌ Would violate Covenant 1 (introduces custom logic)

**Covenant Alignment**: Covenant 1 (Turtle is cause) → Direct RDF consumption, zero transformation

---

### ADR-004: React Flow for Workflow Visualization

**Decision**: Use React Flow for interactive workflow graph rendering

**Context**: Need performant, accessible, customizable workflow visualization with 1000+ node support.

**Rationale**:
- **Performance**: Canvas-based rendering, handles 10,000+ nodes
- **Customization**: Custom node types map to YAWL task types
- **Layout Algorithms**: Dagre, ELK for automatic layout
- **Interactions**: Pan, zoom, node selection, edge routing
- **Accessibility**: Keyboard navigation, ARIA labels
- **TypeScript**: Full type safety for nodes/edges

**Alternatives Considered**:
- D3.js: ❌ Too low-level, would require extensive custom code
- Cytoscape.js: ✅ Viable, but less React-native
- mxGraph: ❌ Abandoned, no React support
- Custom Canvas: ❌ Would violate performance constraints (Covenant 5)

**Covenant Alignment**: Covenant 2 (Q validates) → Layout must respect pattern matrix constraints

---

### ADR-005: Zustand for Client State Management

**Decision**: Use Zustand for client-side state (workflow editor state only)

**Context**: Need minimal state management for transient UI state (not workflow data).

**Rationale**:
- **Minimal API**: ~1KB, no boilerplate
- **No Context Hell**: Direct store access, no provider wrapping
- **Immer Integration**: Immutable updates (aligns with Covenant 1 - no retrocausation)
- **DevTools**: Redux DevTools compatible
- **TypeScript**: Full type inference

**Alternatives Considered**:
- Redux Toolkit: ❌ Too heavy, unnecessary complexity
- Jotai: ✅ Viable, but atomic model less suited to workflow editor
- Recoil: ❌ Meta-owned, uncertain future
- Context API: ❌ Re-render performance issues at scale

**Critical Constraint**: Workflow data (RDF) NEVER stored in Zustand, only in RDF store or server.

**Covenant Alignment**: Covenant 1 (Turtle is source of truth) → Client state is cache, not truth

---

### ADR-006: Server Actions for Mutations

**Decision**: Use Next.js Server Actions for all workflow mutations

**Context**: Need type-safe mutations that preserve RDF as source of truth.

**Rationale**:
- **Type Safety**: End-to-end TypeScript from UI to RDF store
- **Server Validation**: SPARQL validation happens server-side
- **Progressive Enhancement**: Works without JavaScript
- **Optimistic Updates**: Client state syncs after server confirmation
- **Covenant Compliance**: Mutations go through Weaver validation

**Alternatives Considered**:
- REST API: ❌ Requires separate type definitions, loses type safety
- GraphQL: ❌ Impedance mismatch with SPARQL, adds complexity
- tRPC: ✅ Viable, but Next.js Server Actions are native

**Covenant Alignment**: Covenant 2 (Q validates) → All mutations validated server-side before persisting

---

### ADR-007: Weaver Schema as API Contract

**Decision**: OpenTelemetry Weaver schema defines UI → Backend contract

**Context**: Need canonical contract between frontend and Rust workflow engine.

**Rationale**:
- **Schema-First**: Weaver schema is source of truth for telemetry
- **Live Validation**: `weaver registry live-check` validates runtime behavior
- **No Drift**: UI can't display metrics not declared in schema
- **Documentation**: Schema is executable specification

**Covenant Alignment**: Covenant 6 (O drives everything) → All UI metrics backed by schema

---

## Technology Stack

### Core Framework

```json
{
  "framework": "Next.js 15.x",
  "react": "19.x",
  "typescript": "5.x",
  "node": ">=20.x"
}
```

### UI Components

```json
{
  "@radix-ui/react-*": "Latest", // Accessible primitives
  "class-variance-authority": "^0.7.0", // Component variants
  "tailwindcss": "^3.4.0", // Utility-first CSS
  "tailwind-merge": "^2.0.0", // Class merging
  "lucide-react": "^0.300.0" // Icons
}
```

### RDF/SPARQL

```json
{
  "unrdf": "^0.5.0", // RDF parsing/serialization
  "@comunica/query-sparql": "^3.0.0", // SPARQL engine (optional)
  "n3": "^1.17.0" // Alternative Turtle parser
}
```

### Workflow Visualization

```json
{
  "reactflow": "^11.10.0", // Flow diagram library
  "dagre": "^0.8.5", // Auto-layout algorithm
  "elkjs": "^0.9.0" // Advanced layout (optional)
}
```

### State & Data Fetching

```json
{
  "zustand": "^4.4.0", // Client state
  "@tanstack/react-query": "^5.0.0", // Server state cache
  "immer": "^10.0.0" // Immutable updates
}
```

### Validation & Forms

```json
{
  "zod": "^3.22.0", // Schema validation
  "react-hook-form": "^7.49.0", // Form state
  "@hookform/resolvers": "^3.3.0" // Zod integration
}
```

### Performance & Monitoring

```json
{
  "@opentelemetry/api": "^1.7.0", // OTEL client
  "@vercel/speed-insights": "^1.0.0", // Performance tracking
  "web-vitals": "^3.5.0" // Core Web Vitals
}
```

### Development

```json
{
  "eslint": "^8.56.0",
  "prettier": "^3.1.0",
  "vitest": "^1.0.0", // Unit tests
  "playwright": "@latest" // E2E tests
}
```

---

## System Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                     Next.js Application                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────────────────────────────────────────────┐    │
│  │           Browser (Client Components)                  │    │
│  ├───────────────────────────────────────────────────────┤    │
│  │                                                         │    │
│  │  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐ │    │
│  │  │  Workflow   │  │   Pattern    │  │    MAPE-K    │ │    │
│  │  │   Editor    │  │  Validator   │  │   Monitor    │ │    │
│  │  │  (React     │  │  (Client     │  │  (Real-time  │ │    │
│  │  │   Flow)     │  │   Rules)     │  │  Dashboard)  │ │    │
│  │  └─────────────┘  └──────────────┘  └──────────────┘ │    │
│  │         ↓                 ↓                 ↓          │    │
│  │  ┌──────────────────────────────────────────────────┐ │    │
│  │  │         Zustand Store (Transient UI State)       │ │    │
│  │  │   - Editor selection                             │ │    │
│  │  │   - UI mode (edit/view)                          │ │    │
│  │  │   - Temporary node positions                     │ │    │
│  │  └──────────────────────────────────────────────────┘ │    │
│  └───────────────────────────────────────────────────────┘    │
│                           ↕ Server Actions                     │
│  ┌───────────────────────────────────────────────────────┐    │
│  │        Next.js Server (Server Components/Actions)     │    │
│  ├───────────────────────────────────────────────────────┤    │
│  │                                                         │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌─────────────┐ │    │
│  │  │  RDF Parser  │  │   SPARQL     │  │   Weaver    │ │    │
│  │  │   (unrdf)    │  │   Engine     │  │  Validator  │ │    │
│  │  └──────────────┘  └──────────────┘  └─────────────┘ │    │
│  │         ↓                 ↓                 ↓          │    │
│  │  ┌──────────────────────────────────────────────────┐ │    │
│  │  │            RDF Graph Store (In-Memory)           │ │    │
│  │  │  - Parsed Turtle workflows                       │ │    │
│  │  │  - SPARQL query cache                            │ │    │
│  │  │  - Pattern matrix index                          │ │    │
│  │  └──────────────────────────────────────────────────┘ │    │
│  └───────────────────────────────────────────────────────┘    │
│                           ↕ HTTP/gRPC                          │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│              Rust KNHK Workflow Engine (Backend)                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │  Workflow    │  │   Pattern    │  │   MAPE-K     │         │
│  │  Executor    │  │  Validator   │  │   Engine     │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│         ↓                 ↓                 ↓                   │
│  ┌──────────────────────────────────────────────────┐          │
│  │         RDF Triple Store (Persistent)            │          │
│  │  - Workflow definitions (.ttl)                   │          │
│  │  - Execution receipts (immutable log)            │          │
│  │  - MAPE-K knowledge base                         │          │
│  └──────────────────────────────────────────────────┘          │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Component Interaction Flow

```
┌──────────┐     1. Load      ┌────────────┐    2. Parse     ┌─────────┐
│  User    │ ───Workflow──→   │  Next.js   │ ───Turtle───→  │   RDF   │
│ Browser  │                  │   Server   │                 │  Parser │
└──────────┘                  └────────────┘                 └─────────┘
     ↑                              │                              │
     │                              │ 3. SPARQL Extract            │
     │                              ↓                              │
     │                        ┌─────────────┐                     │
     │                        │   SPARQL    │ ←────────────────────┘
     │                        │   Engine    │
     │                        └─────────────┘
     │                              │
     │                              │ 4. Serialize to JSON
     │                              ↓
     │                        ┌─────────────┐
     │     5. Render HTML     │  Workflow   │
     │    ←──────────────     │   React     │
     │                        │ Components  │
     └────────────────────────└─────────────┘
                                    │
                                    │ 6. User Interaction
                                    ↓
                              ┌─────────────┐
                              │   Server    │
                              │   Action    │
                              └─────────────┘
                                    │
                                    │ 7. Validate + Save
                                    ↓
                              ┌─────────────┐
                              │    Rust     │
                              │   Backend   │
                              └─────────────┘
```

---

## Directory Structure

```
nextjs-yawl-ui/
├── app/                              # Next.js App Router
│   ├── (auth)/                       # Auth routes (grouped)
│   │   ├── login/
│   │   └── register/
│   ├── (dashboard)/                  # Main app routes
│   │   ├── workflows/                # Workflow management
│   │   │   ├── page.tsx              # List all workflows
│   │   │   ├── [id]/                 # Single workflow
│   │   │   │   ├── page.tsx          # View workflow
│   │   │   │   ├── edit/page.tsx     # Edit workflow
│   │   │   │   └── monitor/page.tsx  # MAPE-K dashboard
│   │   │   └── new/page.tsx          # Create workflow
│   │   ├── patterns/                 # Pattern library
│   │   │   ├── page.tsx              # All 43 patterns
│   │   │   └── [id]/page.tsx         # Pattern details
│   │   ├── templates/                # Workflow templates
│   │   └── settings/                 # App settings
│   ├── api/                          # API routes
│   │   ├── workflows/                # Workflow CRUD
│   │   │   ├── route.ts              # GET /api/workflows
│   │   │   └── [id]/route.ts         # GET/PUT/DELETE
│   │   ├── sparql/                   # SPARQL endpoint
│   │   │   └── route.ts              # POST /api/sparql
│   │   ├── validate/                 # Weaver validation
│   │   │   └── route.ts              # POST /api/validate
│   │   └── telemetry/                # OTEL collector
│   │       └── route.ts              # POST /api/telemetry
│   ├── layout.tsx                    # Root layout
│   ├── page.tsx                      # Home page
│   └── globals.css                   # Global styles
│
├── components/                       # React components
│   ├── ui/                           # shadcn/ui components
│   │   ├── button.tsx
│   │   ├── card.tsx
│   │   ├── dialog.tsx
│   │   ├── dropdown-menu.tsx
│   │   ├── input.tsx
│   │   ├── label.tsx
│   │   ├── select.tsx
│   │   ├── tabs.tsx
│   │   ├── toast.tsx
│   │   └── ...                       # Other shadcn components
│   │
│   ├── workflow/                     # Workflow-specific components
│   │   ├── editor/                   # Workflow editor
│   │   │   ├── canvas.tsx            # React Flow canvas
│   │   │   ├── toolbar.tsx           # Editor toolbar
│   │   │   ├── properties-panel.tsx  # Task properties
│   │   │   ├── minimap.tsx           # Workflow minimap
│   │   │   └── controls.tsx          # Zoom/pan controls
│   │   │
│   │   ├── nodes/                    # Custom node types
│   │   │   ├── task-node.tsx         # Atomic task
│   │   │   ├── composite-node.tsx    # Composite task
│   │   │   ├── condition-node.tsx    # Input/output condition
│   │   │   ├── and-split-node.tsx    # AND split
│   │   │   ├── or-split-node.tsx     # OR split
│   │   │   ├── xor-split-node.tsx    # XOR split
│   │   │   └── ...                   # Other pattern nodes
│   │   │
│   │   ├── renderer/                 # Read-only workflow display
│   │   │   ├── workflow-graph.tsx    # Render workflow
│   │   │   ├── task-card.tsx         # Task display
│   │   │   └── flow-legend.tsx       # Legend/key
│   │   │
│   │   ├── validator/                # Validation UI
│   │   │   ├── pattern-matrix.tsx    # Show valid patterns
│   │   │   ├── violation-list.tsx    # Show violations
│   │   │   └── soundness-check.tsx   # SHACL results
│   │   │
│   │   └── monitor/                  # MAPE-K monitoring
│   │       ├── metrics-dashboard.tsx # Real-time metrics
│   │       ├── latency-chart.tsx     # Chatman compliance
│   │       ├── throughput-chart.tsx  # Workflow throughput
│   │       └── anomaly-alerts.tsx    # MAPE-K alerts
│   │
│   ├── patterns/                     # Pattern components
│   │   ├── pattern-card.tsx          # Pattern display card
│   │   ├── pattern-search.tsx        # Search patterns
│   │   └── pattern-diagram.tsx       # Pattern visualization
│   │
│   ├── templates/                    # Template components
│   │   ├── template-gallery.tsx      # Browse templates
│   │   └── template-preview.tsx      # Preview template
│   │
│   └── providers/                    # React context providers
│       ├── theme-provider.tsx        # Dark/light mode
│       ├── toast-provider.tsx        # Toast notifications
│       └── auth-provider.tsx         # Auth context
│
├── lib/                              # Core libraries
│   ├── rdf/                          # RDF processing
│   │   ├── parser.ts                 # Turtle → RDF graph
│   │   ├── serializer.ts             # RDF graph → Turtle
│   │   ├── sparql.ts                 # SPARQL query execution
│   │   ├── store.ts                  # In-memory RDF store
│   │   └── prefixes.ts               # Common RDF prefixes
│   │
│   ├── workflow/                     # Workflow logic
│   │   ├── parser.ts                 # RDF → Workflow object
│   │   ├── serializer.ts             # Workflow object → RDF
│   │   ├── transformer.ts            # Workflow → React Flow
│   │   ├── layout.ts                 # Auto-layout algorithms
│   │   └── exporter.ts               # Export formats (PNG, SVG, PDF)
│   │
│   ├── validation/                   # Validation logic
│   │   ├── patterns.ts               # Pattern matrix validation
│   │   ├── shacl.ts                  # SHACL shape validation
│   │   ├── soundness.ts              # Workflow soundness checks
│   │   └── weaver.ts                 # Weaver schema validation
│   │
│   ├── mapek/                        # MAPE-K integration
│   │   ├── monitor.ts                # Telemetry collection
│   │   ├── analyzer.ts               # Anomaly detection
│   │   ├── planner.ts                # Adaptation planning
│   │   ├── executor.ts               # Execute adaptations
│   │   └── knowledge.ts              # Knowledge base sync
│   │
│   ├── performance/                  # Performance utilities
│   │   ├── guards.ts                 # Chatman constant checks
│   │   ├── profiler.ts               # Performance profiling
│   │   └── metrics.ts                # Custom metrics
│   │
│   ├── telemetry/                    # OpenTelemetry
│   │   ├── collector.ts              # OTEL collector client
│   │   ├── spans.ts                  # Span creation helpers
│   │   └── metrics.ts                # Metric recording
│   │
│   ├── api/                          # API client
│   │   ├── client.ts                 # HTTP client
│   │   ├── workflows.ts              # Workflow API calls
│   │   └── patterns.ts               # Pattern API calls
│   │
│   └── utils/                        # Utility functions
│       ├── cn.ts                     # Class name merger
│       ├── format.ts                 # Date/time formatting
│       └── debounce.ts               # Debounce helper
│
├── hooks/                            # Custom React hooks
│   ├── use-workflow.ts               # Load/save workflow
│   ├── use-patterns.ts               # Load pattern library
│   ├── use-validation.ts             # Validate workflow
│   ├── use-telemetry.ts              # Send telemetry
│   ├── use-toast.ts                  # Toast notifications
│   └── use-theme.ts                  # Theme switching
│
├── stores/                           # Zustand stores
│   ├── editor-store.ts               # Workflow editor state
│   ├── ui-store.ts                   # UI state (sidebar, etc)
│   └── settings-store.ts             # User settings
│
├── types/                            # TypeScript types
│   ├── workflow.ts                   # Workflow data types
│   ├── pattern.ts                    # Pattern types
│   ├── rdf.ts                        # RDF term types
│   ├── mapek.ts                      # MAPE-K types
│   └── telemetry.ts                  # Telemetry types
│
├── actions/                          # Server actions
│   ├── workflows.ts                  # Workflow CRUD actions
│   ├── validate.ts                   # Validation actions
│   └── export.ts                     # Export actions
│
├── ontology/                         # YAWL ontologies (symlink to /home/user/knhk/ontology)
│   ├── yawl.ttl
│   ├── yawl-extended.ttl
│   ├── yawl-pattern-permutations.ttl
│   ├── mape-k-autonomic.ttl
│   └── workflows/
│
├── public/                           # Static assets
│   ├── patterns/                     # Pattern diagrams (SVG)
│   └── icons/                        # App icons
│
├── tests/                            # Test files
│   ├── unit/                         # Unit tests (Vitest)
│   │   ├── rdf/
│   │   ├── workflow/
│   │   └── validation/
│   ├── integration/                  # Integration tests
│   │   ├── workflow-crud.test.ts
│   │   └── validation.test.ts
│   └── e2e/                          # E2E tests (Playwright)
│       ├── workflow-editor.spec.ts
│       └── pattern-library.spec.ts
│
├── scripts/                          # Build/utility scripts
│   ├── generate-pattern-diagrams.ts  # Generate SVG diagrams
│   ├── validate-ontologies.ts        # Validate TTL files
│   └── seed-workflows.ts             # Seed example workflows
│
├── docs/                             # Documentation
│   ├── ARCHITECTURE.md               # This file
│   ├── COMPONENTS.md                 # Component documentation
│   ├── API.md                        # API documentation
│   └── DEPLOYMENT.md                 # Deployment guide
│
├── .env.local                        # Environment variables
├── .env.example                      # Example env vars
├── .eslintrc.json                    # ESLint config
├── .prettierrc                       # Prettier config
├── next.config.js                    # Next.js config
├── tailwind.config.ts                # Tailwind config
├── tsconfig.json                     # TypeScript config
├── package.json                      # Dependencies
└── README.md                         # Project README
```

---

## Component Architecture

### Component Hierarchy

```
App
├── RootLayout
│   ├── ThemeProvider
│   ├── ToastProvider
│   └── AuthProvider
│       └── DashboardLayout
│           ├── Sidebar
│           │   ├── NavItem (Workflows)
│           │   ├── NavItem (Patterns)
│           │   └── NavItem (Templates)
│           └── MainContent
│               └── WorkflowsPage
│                   ├── WorkflowList
│                   │   ├── WorkflowCard[]
│                   │   └── CreateWorkflowButton
│                   └── WorkflowEditor (when editing)
│                       ├── EditorToolbar
│                       │   ├── SaveButton
│                       │   ├── ValidateButton
│                       │   └── ExportDropdown
│                       ├── ReactFlowCanvas
│                       │   ├── TaskNode[]
│                       │   ├── ConditionNode[]
│                       │   ├── SplitNode[]
│                       │   └── Edge[]
│                       ├── PropertiesPanel
│                       │   ├── NodeProperties
│                       │   └── EdgeProperties
│                       └── ValidationPanel
│                           ├── PatternMatrix
│                           └── ViolationList
```

### Core Components Design

#### 1. WorkflowEditor Component

**Purpose**: Main workflow editor with drag-and-drop node editing

**Props**:
```typescript
interface WorkflowEditorProps {
  workflowId?: string;              // Existing workflow ID (undefined for new)
  initialWorkflow?: WorkflowRDF;    // Initial RDF data
  onSave?: (workflow: WorkflowRDF) => Promise<void>;
  readonly?: boolean;               // Read-only mode
}
```

**State** (Zustand):
```typescript
interface EditorStore {
  // React Flow state
  nodes: Node[];
  edges: Edge[];
  selectedNodeId: string | null;
  selectedEdgeId: string | null;

  // Editor mode
  mode: 'view' | 'edit';
  isDirty: boolean;

  // Actions
  addNode: (node: Node) => void;
  updateNode: (id: string, data: Partial<Node>) => void;
  deleteNode: (id: string) => void;
  addEdge: (edge: Edge) => void;
  deleteEdge: (id: string) => void;
  setSelection: (nodeId: string | null, edgeId: string | null) => void;
  reset: () => void;
}
```

**Implementation**:
```typescript
// components/workflow/editor/canvas.tsx
'use client';

import { useCallback, useEffect } from 'react';
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
} from 'reactflow';
import { useEditorStore } from '@/stores/editor-store';
import { TaskNode } from '../nodes/task-node';
import { ConditionNode } from '../nodes/condition-node';

const nodeTypes = {
  task: TaskNode,
  condition: ConditionNode,
  // ... other node types
};

export function WorkflowCanvas() {
  const { nodes, edges, addEdge, setSelection } = useEditorStore();

  const onConnect = useCallback((params) => {
    addEdge(params);
  }, [addEdge]);

  const onNodeClick = useCallback((event, node) => {
    setSelection(node.id, null);
  }, [setSelection]);

  return (
    <div className="h-full w-full">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onConnect={onConnect}
        onNodeClick={onNodeClick}
        nodeTypes={nodeTypes}
        fitView
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}
```

#### 2. TaskNode Component

**Purpose**: Custom React Flow node for YAWL tasks

**Props**:
```typescript
interface TaskNodeProps {
  id: string;
  data: {
    label: string;
    taskType: 'atomic' | 'composite';
    splitType: 'AND' | 'OR' | 'XOR' | null;
    joinType: 'AND' | 'OR' | 'XOR' | null;
    patterns: string[];           // Pattern IDs (e.g., "Pattern1_Sequence")
    rdfUri: string;                // RDF URI for this task
  };
  selected: boolean;
}
```

**Implementation**:
```typescript
// components/workflow/nodes/task-node.tsx
import { memo } from 'react';
import { Handle, Position } from 'reactflow';
import { Card } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

export const TaskNode = memo(({ id, data, selected }: TaskNodeProps) => {
  return (
    <Card className={selected ? 'ring-2 ring-primary' : ''}>
      <Handle type="target" position={Position.Top} />

      <div className="p-4 min-w-[150px]">
        <div className="font-semibold">{data.label}</div>

        {/* Split/Join indicators */}
        {data.splitType && (
          <Badge variant="secondary" className="mt-2">
            {data.splitType}-split
          </Badge>
        )}
        {data.joinType && (
          <Badge variant="secondary" className="mt-2 ml-1">
            {data.joinType}-join
          </Badge>
        )}

        {/* Pattern badges */}
        <div className="mt-2 flex flex-wrap gap-1">
          {data.patterns.map(pattern => (
            <Badge key={pattern} variant="outline" className="text-xs">
              {pattern.replace('Pattern', 'P')}
            </Badge>
          ))}
        </div>
      </div>

      <Handle type="source" position={Position.Bottom} />
    </Card>
  );
});

TaskNode.displayName = 'TaskNode';
```

#### 3. PatternValidator Component

**Purpose**: Real-time pattern matrix validation display

**Props**:
```typescript
interface PatternValidatorProps {
  workflow: WorkflowRDF;
  onValidationComplete?: (result: ValidationResult) => void;
}

interface ValidationResult {
  isValid: boolean;
  violations: Violation[];
  patterns: DetectedPattern[];
}
```

**Implementation**:
```typescript
// components/workflow/validator/pattern-matrix.tsx
'use client';

import { useEffect, useState } from 'react';
import { Card } from '@/components/ui/card';
import { Alert } from '@/components/ui/alert';
import { validatePatterns } from '@/lib/validation/patterns';

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

  if (loading) return <div>Validating...</div>;
  if (!result) return null;

  return (
    <Card className="p-4">
      <h3 className="font-semibold mb-4">Pattern Validation</h3>

      {result.isValid ? (
        <Alert variant="success">
          ✓ Workflow is valid. Detected {result.patterns.length} patterns.
        </Alert>
      ) : (
        <Alert variant="destructive">
          ✗ {result.violations.length} violation(s) found.
        </Alert>
      )}

      {/* Detected patterns */}
      <div className="mt-4">
        <h4 className="text-sm font-medium">Detected Patterns:</h4>
        <ul className="mt-2 space-y-1">
          {result.patterns.map(p => (
            <li key={p.id} className="text-sm">
              {p.name} ({p.id})
            </li>
          ))}
        </ul>
      </div>

      {/* Violations */}
      {result.violations.length > 0 && (
        <div className="mt-4">
          <h4 className="text-sm font-medium text-destructive">Violations:</h4>
          <ul className="mt-2 space-y-2">
            {result.violations.map((v, idx) => (
              <li key={idx} className="text-sm text-muted-foreground">
                {v.message} (Node: {v.nodeId})
              </li>
            ))}
          </ul>
        </div>
      )}
    </Card>
  );
}
```

#### 4. MAPEKMonitor Component

**Purpose**: Real-time MAPE-K metrics dashboard

**Props**:
```typescript
interface MAPEKMonitorProps {
  workflowId: string;
  refreshInterval?: number;  // Default: 1000ms
}
```

**Implementation**:
```typescript
// components/workflow/monitor/metrics-dashboard.tsx
'use client';

import { useEffect, useState } from 'react';
import { Card } from '@/components/ui/card';
import { LatencyChart } from './latency-chart';
import { ThroughputChart } from './throughput-chart';
import { AnomalyAlerts } from './anomaly-alerts';
import { useTelemetry } from '@/hooks/use-telemetry';

export function MAPEKMonitor({ workflowId, refreshInterval = 1000 }: MAPEKMonitorProps) {
  const { metrics, loading } = useTelemetry(workflowId, refreshInterval);

  if (loading) return <div>Loading metrics...</div>;

  return (
    <div className="grid gap-4 md:grid-cols-2">
      {/* Latency (Chatman Constant compliance) */}
      <Card className="p-4">
        <h3 className="font-semibold mb-4">Latency (Chatman Constant)</h3>
        <LatencyChart data={metrics.latency} threshold={8} />
        {metrics.latency.current > 8 && (
          <Alert variant="destructive" className="mt-2">
            ⚠ Latency exceeds 8 ticks (Chatman Constant violated)
          </Alert>
        )}
      </Card>

      {/* Throughput */}
      <Card className="p-4">
        <h3 className="font-semibold mb-4">Throughput</h3>
        <ThroughputChart data={metrics.throughput} />
      </Card>

      {/* MAPE-K Anomalies */}
      <Card className="p-4 md:col-span-2">
        <h3 className="font-semibold mb-4">MAPE-K Anomalies</h3>
        <AnomalyAlerts anomalies={metrics.anomalies} />
      </Card>
    </div>
  );
}
```

---

## Data Flow

### 1. Workflow Load Flow

```
User navigates to /workflows/[id]
         ↓
Next.js Server Component loads
         ↓
Server Action: loadWorkflow(id)
         ↓
┌─────────────────────────────┐
│ Read .ttl file from ontology│
│ or fetch from Rust backend  │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Parse Turtle → RDF Graph    │
│ (unrdf library)             │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Execute SPARQL query:       │
│ - Extract tasks             │
│ - Extract flows             │
│ - Extract patterns          │
│ - Extract MAPE-K config     │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Transform to Workflow object│
│ (WorkflowRDF type)          │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Transform to React Flow     │
│ nodes/edges format          │
└─────────────────────────────┘
         ↓
Render WorkflowEditor component
         ↓
User sees workflow in UI
```

### 2. Workflow Save Flow

```
User clicks "Save" in editor
         ↓
Client validates local state
         ↓
Call Server Action: saveWorkflow(data)
         ↓
┌─────────────────────────────┐
│ Transform React Flow data   │
│ → Workflow object           │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Validate against pattern    │
│ matrix (SPARQL query)       │
└─────────────────────────────┘
         ↓
     Valid?
    /      \
  Yes       No
   ↓         ↓
   │    Return violations
   │    to client → Show errors
   ↓
┌─────────────────────────────┐
│ Transform Workflow object   │
│ → RDF triples               │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Serialize to Turtle format  │
│ (pure passthrough template) │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Validate with Weaver        │
│ weaver registry check       │
└─────────────────────────────┘
         ↓
     Valid?
    /      \
  Yes       No
   ↓         ↓
   │    Return schema violations
   │    to client → Show errors
   ↓
┌─────────────────────────────┐
│ Save .ttl file to ontology/ │
│ or persist to Rust backend  │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Return success to client    │
└─────────────────────────────┘
         ↓
Show success toast
         ↓
Revalidate workflow list
```

### 3. Real-time MAPE-K Flow

```
Rust Workflow Engine executes workflow
         ↓
Emits OpenTelemetry spans/metrics
         ↓
┌─────────────────────────────┐
│ OTEL Collector receives     │
│ telemetry data              │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Next.js API route receives  │
│ POST /api/telemetry         │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Validate against Weaver     │
│ schema (live-check)         │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Store in MAPE-K knowledge   │
│ base (RDF graph)            │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Execute MAPE-K Analyze      │
│ (SPARQL query for anomalies)│
└─────────────────────────────┘
         ↓
   Anomaly detected?
    /      \
  Yes       No
   ↓         ↓
   │    Store metric
   │    Continue
   ↓
┌─────────────────────────────┐
│ Execute MAPE-K Plan         │
│ (generate adaptation plan)  │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Send plan to Rust engine    │
│ (Execute phase)             │
└─────────────────────────────┘
         ↓
┌─────────────────────────────┐
│ Send real-time update to    │
│ client via Server-Sent      │
│ Events or WebSocket         │
└─────────────────────────────┘
         ↓
Client MAPEKMonitor updates
         ↓
User sees live metrics
```

---

## RDF Integration Strategy

### Covenant 1 Compliance: Turtle as Source of Truth

**CRITICAL**: All workflow data MUST be stored in RDF/Turtle format. The UI is a projection, not the source.

### RDF Parsing Pipeline

```typescript
// lib/rdf/parser.ts
import { Parser, Store } from 'n3';

export async function parseTurtle(turtle: string): Promise<Store> {
  const parser = new Parser();
  const store = new Store();

  return new Promise((resolve, reject) => {
    parser.parse(turtle, (error, quad, prefixes) => {
      if (error) {
        reject(error);
      } else if (quad) {
        store.addQuad(quad);
      } else {
        // Parsing complete
        resolve(store);
      }
    });
  });
}
```

### SPARQL Query Execution

```typescript
// lib/rdf/sparql.ts
import { Store } from 'n3';

export function executeSPARQL(store: Store, query: string): any[] {
  // Use N3's SPARQL engine or integrate Comunica
  // For now, manual quad matching

  // Example: Extract all tasks
  const tasks = [];
  for (const quad of store.match(null, RDF('type'), YAWL('AtomicTask'))) {
    tasks.push({
      uri: quad.subject.value,
      // ... extract properties
    });
  }
  return tasks;
}
```

### Workflow → RDF Transformation

```typescript
// lib/workflow/serializer.ts
import { Writer } from 'n3';
import { WorkflowRDF } from '@/types/workflow';

export function serializeWorkflow(workflow: WorkflowRDF): string {
  const writer = new Writer({
    prefixes: {
      yawl: 'http://www.yawlfoundation.org/yawlschema#',
      mapek: 'http://knhk.ai/ontology/mape-k#',
      pattern: 'http://knhk.ai/ontology/workflow-patterns#',
    }
  });

  // Add workflow metadata
  writer.addQuad(
    workflow.uri,
    RDF('type'),
    MAPEK('SelfExecutingWorkflow')
  );
  writer.addQuad(
    workflow.uri,
    YAWL('specName'),
    `"${workflow.name}"`
  );

  // Add tasks
  for (const task of workflow.tasks) {
    writer.addQuad(
      task.uri,
      RDF('type'),
      YAWL('AtomicTask')
    );
    writer.addQuad(
      task.uri,
      YAWL('taskName'),
      `"${task.name}"`
    );
    // ... add other properties
  }

  // Add flows
  for (const flow of workflow.flows) {
    writer.addQuad(
      flow.uri,
      RDF('type'),
      YAWL('Flow')
    );
    writer.addQuad(
      flow.uri,
      YAWL('flowsFrom'),
      flow.source
    );
    writer.addQuad(
      flow.uri,
      YAWL('flowsInto'),
      flow.target
    );
  }

  return new Promise((resolve, reject) => {
    writer.end((error, result) => {
      if (error) reject(error);
      else resolve(result);
    });
  });
}
```

### RDF → React Flow Transformation

```typescript
// lib/workflow/transformer.ts
import { Node, Edge } from 'reactflow';
import { WorkflowRDF } from '@/types/workflow';

export function workflowToReactFlow(workflow: WorkflowRDF): {
  nodes: Node[];
  edges: Edge[];
} {
  const nodes: Node[] = [];
  const edges: Edge[] = [];

  // Transform tasks to nodes
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

  // Transform flows to edges
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

### Auto-Layout Algorithm

```typescript
// lib/workflow/layout.ts
import dagre from 'dagre';
import { Node, Edge } from 'reactflow';

export function applyDagreLayout(
  nodes: Node[],
  edges: Edge[],
  direction: 'TB' | 'LR' = 'TB'
): Node[] {
  const dagreGraph = new dagre.graphlib.Graph();
  dagreGraph.setDefaultEdgeLabel(() => ({}));
  dagreGraph.setGraph({ rankdir: direction });

  // Add nodes
  nodes.forEach((node) => {
    dagreGraph.setNode(node.id, { width: 200, height: 100 });
  });

  // Add edges
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
        x: nodeWithPosition.x - 100, // Center node
        y: nodeWithPosition.y - 50,
      },
    };
  });
}
```

---

## API Design

### REST API Endpoints

```typescript
// API Routes (Next.js App Router)

// GET /api/workflows
// List all workflows
export async function GET(request: Request) {
  const workflows = await loadAllWorkflows();
  return Response.json(workflows);
}

// POST /api/workflows
// Create new workflow
export async function POST(request: Request) {
  const body = await request.json();
  const workflow = await createWorkflow(body);
  return Response.json(workflow, { status: 201 });
}

// GET /api/workflows/[id]
// Get single workflow
export async function GET(
  request: Request,
  { params }: { params: { id: string } }
) {
  const workflow = await loadWorkflow(params.id);
  if (!workflow) {
    return Response.json({ error: 'Not found' }, { status: 404 });
  }
  return Response.json(workflow);
}

// PUT /api/workflows/[id]
// Update workflow
export async function PUT(
  request: Request,
  { params }: { params: { id: string } }
) {
  const body = await request.json();

  // Validate against pattern matrix
  const validation = await validatePatterns(body);
  if (!validation.isValid) {
    return Response.json(
      { errors: validation.violations },
      { status: 400 }
    );
  }

  // Validate with Weaver
  const weaverResult = await validateWithWeaver(body);
  if (!weaverResult.isValid) {
    return Response.json(
      { errors: weaverResult.errors },
      { status: 400 }
    );
  }

  const workflow = await updateWorkflow(params.id, body);
  return Response.json(workflow);
}

// DELETE /api/workflows/[id]
// Delete workflow
export async function DELETE(
  request: Request,
  { params }: { params: { id: string } }
) {
  await deleteWorkflow(params.id);
  return Response.json({ success: true });
}

// POST /api/sparql
// Execute SPARQL query
export async function POST(request: Request) {
  const { query } = await request.json();
  const results = await executeSPARQL(query);
  return Response.json({ results });
}

// POST /api/validate
// Validate workflow
export async function POST(request: Request) {
  const { workflow } = await request.json();

  const patternResult = await validatePatterns(workflow);
  const weaverResult = await validateWithWeaver(workflow);
  const shaclResult = await validateSHACL(workflow);

  return Response.json({
    isValid: patternResult.isValid && weaverResult.isValid && shaclResult.isValid,
    patterns: patternResult,
    weaver: weaverResult,
    shacl: shaclResult,
  });
}

// POST /api/telemetry
// Receive telemetry data from OTEL collector
export async function POST(request: Request) {
  const telemetry = await request.json();

  // Validate against Weaver schema
  const validation = await validateTelemetry(telemetry);
  if (!validation.isValid) {
    return Response.json(
      { error: 'Invalid telemetry' },
      { status: 400 }
    );
  }

  // Store in MAPE-K knowledge base
  await storeTelemetry(telemetry);

  // Trigger MAPE-K analysis
  await analyzeTelemetry(telemetry);

  return Response.json({ success: true });
}
```

### Server Actions

```typescript
// actions/workflows.ts
'use server';

import { revalidatePath } from 'next/cache';
import { parseTurtle } from '@/lib/rdf/parser';
import { serializeWorkflow } from '@/lib/workflow/serializer';
import { validatePatterns } from '@/lib/validation/patterns';
import { validateWithWeaver } from '@/lib/validation/weaver';

export async function saveWorkflowAction(
  id: string,
  workflow: WorkflowRDF
): Promise<{ success: boolean; errors?: string[] }> {
  // Validate patterns
  const patternValidation = await validatePatterns(workflow);
  if (!patternValidation.isValid) {
    return {
      success: false,
      errors: patternValidation.violations.map(v => v.message),
    };
  }

  // Validate with Weaver
  const weaverValidation = await validateWithWeaver(workflow);
  if (!weaverValidation.isValid) {
    return {
      success: false,
      errors: weaverValidation.errors,
    };
  }

  // Serialize to Turtle
  const turtle = await serializeWorkflow(workflow);

  // Save to file system
  await fs.writeFile(
    `/home/user/knhk/ontology/workflows/${id}.ttl`,
    turtle,
    'utf-8'
  );

  // Revalidate Next.js cache
  revalidatePath(`/workflows/${id}`);
  revalidatePath('/workflows');

  return { success: true };
}

export async function deleteWorkflowAction(
  id: string
): Promise<{ success: boolean }> {
  await fs.unlink(`/home/user/knhk/ontology/workflows/${id}.ttl`);

  revalidatePath('/workflows');

  return { success: true };
}
```

---

## State Management

### Zustand Store Design

```typescript
// stores/editor-store.ts
import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import { Node, Edge } from 'reactflow';

interface EditorState {
  // React Flow state
  nodes: Node[];
  edges: Edge[];
  selectedNodeId: string | null;
  selectedEdgeId: string | null;

  // Editor metadata
  workflowId: string | null;
  isDirty: boolean;
  mode: 'view' | 'edit';

  // Actions
  setNodes: (nodes: Node[]) => void;
  setEdges: (edges: Edge[]) => void;
  addNode: (node: Node) => void;
  updateNode: (id: string, data: Partial<Node['data']>) => void;
  deleteNode: (id: string) => void;
  addEdge: (edge: Edge) => void;
  updateEdge: (id: string, data: Partial<Edge>) => void;
  deleteEdge: (id: string) => void;
  setSelection: (nodeId: string | null, edgeId: string | null) => void;
  setMode: (mode: 'view' | 'edit') => void;
  reset: () => void;
}

export const useEditorStore = create<EditorState>()(
  immer((set) => ({
    // Initial state
    nodes: [],
    edges: [],
    selectedNodeId: null,
    selectedEdgeId: null,
    workflowId: null,
    isDirty: false,
    mode: 'view',

    // Actions
    setNodes: (nodes) =>
      set((state) => {
        state.nodes = nodes;
      }),

    setEdges: (edges) =>
      set((state) => {
        state.edges = edges;
      }),

    addNode: (node) =>
      set((state) => {
        state.nodes.push(node);
        state.isDirty = true;
      }),

    updateNode: (id, data) =>
      set((state) => {
        const node = state.nodes.find((n) => n.id === id);
        if (node) {
          node.data = { ...node.data, ...data };
          state.isDirty = true;
        }
      }),

    deleteNode: (id) =>
      set((state) => {
        state.nodes = state.nodes.filter((n) => n.id !== id);
        state.edges = state.edges.filter(
          (e) => e.source !== id && e.target !== id
        );
        state.isDirty = true;
      }),

    addEdge: (edge) =>
      set((state) => {
        state.edges.push(edge);
        state.isDirty = true;
      }),

    updateEdge: (id, data) =>
      set((state) => {
        const edge = state.edges.find((e) => e.id === id);
        if (edge) {
          Object.assign(edge, data);
          state.isDirty = true;
        }
      }),

    deleteEdge: (id) =>
      set((state) => {
        state.edges = state.edges.filter((e) => e.id !== id);
        state.isDirty = true;
      }),

    setSelection: (nodeId, edgeId) =>
      set((state) => {
        state.selectedNodeId = nodeId;
        state.selectedEdgeId = edgeId;
      }),

    setMode: (mode) =>
      set((state) => {
        state.mode = mode;
      }),

    reset: () =>
      set((state) => {
        state.nodes = [];
        state.edges = [];
        state.selectedNodeId = null;
        state.selectedEdgeId = null;
        state.isDirty = false;
      }),
  }))
);
```

### TanStack Query for Server State

```typescript
// hooks/use-workflow.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { WorkflowRDF } from '@/types/workflow';
import { saveWorkflowAction } from '@/actions/workflows';

export function useWorkflow(id: string) {
  return useQuery({
    queryKey: ['workflow', id],
    queryFn: async () => {
      const res = await fetch(`/api/workflows/${id}`);
      if (!res.ok) throw new Error('Failed to load workflow');
      return res.json() as Promise<WorkflowRDF>;
    },
  });
}

export function useWorkflows() {
  return useQuery({
    queryKey: ['workflows'],
    queryFn: async () => {
      const res = await fetch('/api/workflows');
      if (!res.ok) throw new Error('Failed to load workflows');
      return res.json() as Promise<WorkflowRDF[]>;
    },
  });
}

export function useSaveWorkflow() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      id,
      workflow,
    }: {
      id: string;
      workflow: WorkflowRDF;
    }) => {
      return saveWorkflowAction(id, workflow);
    },
    onSuccess: (_, { id }) => {
      // Invalidate queries
      queryClient.invalidateQueries({ queryKey: ['workflow', id] });
      queryClient.invalidateQueries({ queryKey: ['workflows'] });
    },
  });
}
```

---

## Workflow Visualization

### React Flow Configuration

```typescript
// components/workflow/editor/canvas.tsx
import ReactFlow, {
  Background,
  Controls,
  MiniMap,
  Panel,
  ReactFlowProvider,
} from 'reactflow';
import 'reactflow/dist/style.css';

const nodeTypes = {
  task: TaskNode,
  composite: CompositeNode,
  condition: ConditionNode,
  andSplit: AndSplitNode,
  orSplit: OrSplitNode,
  xorSplit: XorSplitNode,
};

const edgeTypes = {
  default: DefaultEdge,
  conditional: ConditionalEdge,
};

export function WorkflowCanvas() {
  const { nodes, edges, addEdge } = useEditorStore();

  const onConnect = useCallback(
    (params) => addEdge({ ...params, type: 'default' }),
    [addEdge]
  );

  return (
    <ReactFlowProvider>
      <div className="h-[600px] w-full">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onConnect={onConnect}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          fitView
          minZoom={0.1}
          maxZoom={2}
          defaultViewport={{ x: 0, y: 0, zoom: 1 }}
        >
          <Background />
          <Controls />
          <MiniMap />

          <Panel position="top-left">
            <EditorToolbar />
          </Panel>
        </ReactFlow>
      </div>
    </ReactFlowProvider>
  );
}
```

### Custom Node Styling

```typescript
// components/workflow/nodes/task-node.tsx
import { Handle, Position } from 'reactflow';
import { Card } from '@/components/ui/card';
import { cn } from '@/lib/utils/cn';

export function TaskNode({ data, selected }: NodeProps) {
  return (
    <Card
      className={cn(
        'min-w-[180px] transition-all',
        selected && 'ring-2 ring-primary shadow-lg',
        data.taskType === 'composite' && 'border-dashed'
      )}
    >
      <Handle
        type="target"
        position={Position.Top}
        className="!bg-primary"
      />

      <div className="p-4 space-y-2">
        {/* Task name */}
        <div className="font-semibold text-sm">{data.label}</div>

        {/* Split/Join badges */}
        <div className="flex gap-1">
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

        {/* Pattern indicators */}
        {data.patterns.length > 0 && (
          <div className="text-xs text-muted-foreground">
            {data.patterns.length} pattern(s)
          </div>
        )}
      </div>

      <Handle
        type="source"
        position={Position.Bottom}
        className="!bg-primary"
      />
    </Card>
  );
}
```

---

## Security & Performance

### Security Considerations

1. **Input Validation**: All RDF/Turtle input validated before parsing
2. **SPARQL Injection**: Parameterized queries only, no string concatenation
3. **File System Access**: Restricted to `/ontology` directory only
4. **Authentication**: NextAuth.js for user authentication
5. **Authorization**: Role-based access control (RBAC)
6. **CSRF Protection**: Built-in Next.js CSRF tokens
7. **Content Security Policy**: Strict CSP headers

```typescript
// next.config.js
const securityHeaders = [
  {
    key: 'X-DNS-Prefetch-Control',
    value: 'on'
  },
  {
    key: 'Strict-Transport-Security',
    value: 'max-age=63072000; includeSubDomains; preload'
  },
  {
    key: 'X-Frame-Options',
    value: 'SAMEORIGIN'
  },
  {
    key: 'X-Content-Type-Options',
    value: 'nosniff'
  },
  {
    key: 'Content-Security-Policy',
    value: "default-src 'self'; script-src 'self' 'unsafe-eval'; style-src 'self' 'unsafe-inline';"
  }
];

module.exports = {
  async headers() {
    return [
      {
        source: '/:path*',
        headers: securityHeaders,
      },
    ];
  },
};
```

### Performance Optimizations

#### 1. Chatman Constant Enforcement (≤8 ticks)

```typescript
// lib/performance/guards.ts
export class ChatmanGuard {
  private static readonly MAX_TICKS = 8;

  static async measure<T>(
    fn: () => Promise<T>,
    label: string
  ): Promise<T> {
    const start = performance.now();
    const result = await fn();
    const end = performance.now();

    const ticks = Math.ceil((end - start) * 1000000); // Convert to nanoseconds

    if (ticks > ChatmanGuard.MAX_TICKS) {
      console.warn(
        `⚠ Chatman Constant violated: ${label} took ${ticks} ticks (max: ${ChatmanGuard.MAX_TICKS})`
      );
      // Send telemetry
      recordMetric('chatman_violation', { label, ticks });
    }

    return result;
  }
}

// Usage
const workflow = await ChatmanGuard.measure(
  () => loadWorkflow(id),
  'workflow_load'
);
```

#### 2. RDF Caching

```typescript
// lib/rdf/store.ts
import { LRUCache } from 'lru-cache';

const rdfCache = new LRUCache<string, Store>({
  max: 100,
  ttl: 1000 * 60 * 5, // 5 minutes
  updateAgeOnGet: true,
});

export async function getCachedRDFStore(
  workflowId: string
): Promise<Store> {
  const cached = rdfCache.get(workflowId);
  if (cached) return cached;

  const turtle = await loadTurtleFile(workflowId);
  const store = await parseTurtle(turtle);

  rdfCache.set(workflowId, store);
  return store;
}
```

#### 3. React Flow Performance

```typescript
// components/workflow/editor/canvas.tsx
import { memo } from 'react';

// Memoize node components
export const TaskNode = memo(({ data, selected }: NodeProps) => {
  // ... implementation
});

// Virtualize large workflows
export function WorkflowCanvas() {
  const nodesDraggable = useStore((s) => s.nodesDraggable);
  const nodesConnectable = useStore((s) => s.nodesConnectable);

  return (
    <ReactFlow
      // Only render visible nodes
      onlyRenderVisibleElements
      // Disable interactivity in view mode
      nodesDraggable={nodesDraggable}
      nodesConnectable={nodesConnectable}
      // Optimize rendering
      minZoom={0.1}
      maxZoom={2}
    >
      {/* ... */}
    </ReactFlow>
  );
}
```

#### 4. Server Component Optimization

```typescript
// app/workflows/[id]/page.tsx
import { Suspense } from 'react';

export default async function WorkflowPage({ params }: { params: { id: string } }) {
  // Load workflow on server
  const workflow = await loadWorkflow(params.id);

  return (
    <div>
      <Suspense fallback={<WorkflowSkeleton />}>
        <WorkflowRenderer workflow={workflow} />
      </Suspense>
    </div>
  );
}
```

---

## Testing Strategy

### Unit Tests (Vitest)

```typescript
// tests/unit/rdf/parser.test.ts
import { describe, it, expect } from 'vitest';
import { parseTurtle } from '@/lib/rdf/parser';

describe('RDF Parser', () => {
  it('should parse valid Turtle', async () => {
    const turtle = `
      @prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .

      <http://example.org/workflow1> a yawl:Workflow ;
        yawl:specName "Test Workflow" .
    `;

    const store = await parseTurtle(turtle);
    expect(store.size).toBe(2); // 2 triples
  });

  it('should throw on invalid Turtle', async () => {
    const invalidTurtle = '@prefix invalid syntax';
    await expect(parseTurtle(invalidTurtle)).rejects.toThrow();
  });
});
```

```typescript
// tests/unit/validation/patterns.test.ts
import { describe, it, expect } from 'vitest';
import { validatePatterns } from '@/lib/validation/patterns';

describe('Pattern Validation', () => {
  it('should validate Sequence pattern (P1)', async () => {
    const workflow = createTestWorkflow({
      tasks: [
        { id: 'A', split: 'XOR', join: 'XOR' },
        { id: 'B', split: 'XOR', join: 'XOR' },
      ],
      flows: [
        { from: 'A', to: 'B' },
      ],
    });

    const result = await validatePatterns(workflow);
    expect(result.isValid).toBe(true);
    expect(result.patterns).toContainEqual({
      id: 'Pattern1_Sequence',
      name: 'Sequence',
    });
  });

  it('should reject invalid pattern combination', async () => {
    const workflow = createTestWorkflow({
      tasks: [
        { id: 'A', split: 'AND', join: 'XOR' }, // Invalid: AND-split with XOR-join
      ],
    });

    const result = await validatePatterns(workflow);
    expect(result.isValid).toBe(false);
    expect(result.violations.length).toBeGreaterThan(0);
  });
});
```

### Integration Tests

```typescript
// tests/integration/workflow-crud.test.ts
import { describe, it, expect } from 'vitest';

describe('Workflow CRUD', () => {
  it('should create, read, update, delete workflow', async () => {
    // Create
    const createRes = await fetch('/api/workflows', {
      method: 'POST',
      body: JSON.stringify({
        name: 'Test Workflow',
        tasks: [...],
      }),
    });
    expect(createRes.status).toBe(201);
    const created = await createRes.json();

    // Read
    const readRes = await fetch(`/api/workflows/${created.id}`);
    expect(readRes.status).toBe(200);
    const workflow = await readRes.json();
    expect(workflow.name).toBe('Test Workflow');

    // Update
    const updateRes = await fetch(`/api/workflows/${created.id}`, {
      method: 'PUT',
      body: JSON.stringify({
        ...workflow,
        name: 'Updated Workflow',
      }),
    });
    expect(updateRes.status).toBe(200);

    // Delete
    const deleteRes = await fetch(`/api/workflows/${created.id}`, {
      method: 'DELETE',
    });
    expect(deleteRes.status).toBe(200);
  });
});
```

### E2E Tests (Playwright)

```typescript
// tests/e2e/workflow-editor.spec.ts
import { test, expect } from '@playwright/test';

test('create workflow via editor', async ({ page }) => {
  await page.goto('/workflows/new');

  // Add task A
  await page.click('[data-testid="add-task-button"]');
  await page.fill('[data-testid="task-name"]', 'Task A');
  await page.click('[data-testid="save-task"]');

  // Add task B
  await page.click('[data-testid="add-task-button"]');
  await page.fill('[data-testid="task-name"]', 'Task B');
  await page.click('[data-testid="save-task"]');

  // Connect A → B
  await page.dragAndDrop(
    '[data-testid="task-a-handle"]',
    '[data-testid="task-b-handle"]'
  );

  // Validate
  await page.click('[data-testid="validate-button"]');
  await expect(page.locator('[data-testid="validation-status"]')).toHaveText(
    'Valid'
  );

  // Save
  await page.click('[data-testid="save-workflow"]');
  await expect(page.locator('[data-testid="save-success"]')).toBeVisible();
});
```

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Goal**: Basic Next.js app with RDF parsing

- [ ] Initialize Next.js 15 project with TypeScript
- [ ] Setup shadcn/ui components
- [ ] Implement RDF parser (unrdf/n3)
- [ ] Create basic workflow list page
- [ ] Implement workflow detail page (read-only)
- [ ] Setup TailwindCSS theme
- [ ] Create basic layout components

**Deliverables**:
- Working Next.js app
- Can load and display .ttl workflows
- shadcn/ui component library integrated

---

### Phase 2: Workflow Visualization (Weeks 3-4)

**Goal**: React Flow workflow renderer

- [ ] Integrate React Flow
- [ ] Create custom node components (Task, Condition, Split/Join)
- [ ] Implement RDF → React Flow transformer
- [ ] Implement auto-layout (Dagre)
- [ ] Add minimap and controls
- [ ] Style nodes based on YAWL patterns
- [ ] Add pattern badges to nodes

**Deliverables**:
- Workflows rendered as interactive graphs
- Auto-layout working
- Visual pattern indicators

---

### Phase 3: Workflow Editor (Weeks 5-7)

**Goal**: Drag-and-drop workflow editor

- [ ] Implement editable React Flow canvas
- [ ] Add node creation toolbar
- [ ] Implement node/edge deletion
- [ ] Create properties panel
- [ ] Implement Zustand editor store
- [ ] Add save/cancel functionality
- [ ] Implement React Flow → RDF serializer
- [ ] Add unsaved changes warning

**Deliverables**:
- Fully functional workflow editor
- Can create/edit/save workflows
- Properties panel for task configuration

---

### Phase 4: Pattern Validation (Weeks 8-9)

**Goal**: Real-time pattern matrix validation

- [ ] Load pattern permutation matrix (RDF)
- [ ] Implement pattern detection algorithm
- [ ] Create validation UI component
- [ ] Add real-time validation (debounced)
- [ ] Display violations with suggestions
- [ ] Integrate SHACL validation
- [ ] Show pattern compliance badges

**Deliverables**:
- Real-time pattern validation
- Clear violation messages
- Pattern compliance indicators

---

### Phase 5: MAPE-K Integration (Weeks 10-12)

**Goal**: Live MAPE-K monitoring dashboard

- [ ] Implement OpenTelemetry client
- [ ] Create telemetry API endpoint
- [ ] Implement MAPE-K monitor component
- [ ] Add real-time metrics charts
- [ ] Implement Chatman Constant indicator
- [ ] Add anomaly alert system
- [ ] Integrate with Rust backend (WebSocket/SSE)
- [ ] Create MAPE-K knowledge base viewer

**Deliverables**:
- Real-time MAPE-K dashboard
- Chatman Constant monitoring
- Anomaly detection UI

---

### Phase 6: Weaver Integration (Weeks 13-14)

**Goal**: OpenTelemetry Weaver schema validation

- [ ] Implement Weaver validation client
- [ ] Add schema validation to save flow
- [ ] Create validation error UI
- [ ] Integrate live-check for runtime validation
- [ ] Add telemetry schema viewer
- [ ] Document schema compliance

**Deliverables**:
- Weaver validation integrated
- Schema compliance verified
- Clear validation error messages

---

### Phase 7: Pattern Library (Weeks 15-16)

**Goal**: Browseable pattern library

- [ ] Create pattern list page
- [ ] Implement pattern detail pages
- [ ] Add pattern diagrams (SVG)
- [ ] Create pattern search/filter
- [ ] Add "use pattern" quick-create
- [ ] Document all 43 W3C patterns
- [ ] Add pattern comparison tool

**Deliverables**:
- Complete pattern library
- Pattern documentation
- Pattern quick-create templates

---

### Phase 8: Templates & Export (Weeks 17-18)

**Goal**: Workflow templates and export

- [ ] Create template gallery
- [ ] Implement template preview
- [ ] Add "create from template"
- [ ] Implement PNG export
- [ ] Implement SVG export
- [ ] Implement PDF export (with metadata)
- [ ] Add Turtle download

**Deliverables**:
- Template system working
- Multiple export formats
- Export includes metadata

---

### Phase 9: Testing & Documentation (Weeks 19-20)

**Goal**: Comprehensive testing and docs

- [ ] Write unit tests (80%+ coverage)
- [ ] Write integration tests
- [ ] Write E2E tests (critical flows)
- [ ] Performance testing (Chatman Constant)
- [ ] Write component documentation
- [ ] Write API documentation
- [ ] Write user guide
- [ ] Create video tutorials

**Deliverables**:
- 80%+ test coverage
- Complete documentation
- User guide and tutorials

---

### Phase 10: Production Deployment (Weeks 21-22)

**Goal**: Production-ready deployment

- [ ] Setup Vercel deployment
- [ ] Configure production environment
- [ ] Setup monitoring (Sentry, etc)
- [ ] Configure CDN for static assets
- [ ] Setup CI/CD pipeline
- [ ] Performance optimization
- [ ] Security audit
- [ ] Load testing

**Deliverables**:
- Production deployment
- Monitoring configured
- Performance optimized

---

## Appendix: Key Type Definitions

### WorkflowRDF Type

```typescript
// types/workflow.ts
export interface WorkflowRDF {
  uri: string;                    // Workflow URI
  name: string;                   // Workflow name
  version: string;                // Version

  // Core elements
  tasks: TaskRDF[];
  flows: FlowRDF[];
  conditions: ConditionRDF[];

  // MAPE-K
  monitor?: MonitorRDF;
  analyzer?: AnalyzerRDF;
  planner?: PlannerRDF;
  executor?: ExecutorRDF;
  knowledge?: KnowledgeRDF;

  // Metadata
  patterns: string[];             // Detected patterns
  created: Date;
  updated: Date;
}

export interface TaskRDF {
  uri: string;
  name: string;
  type: 'atomic' | 'composite';

  // Control flow
  splitType: 'AND' | 'OR' | 'XOR' | null;
  joinType: 'AND' | 'OR' | 'XOR' | null;

  // Patterns
  patterns: string[];

  // Parameters
  inputParams: ParameterRDF[];
  outputParams: ParameterRDF[];

  // MAPE-K
  constraints?: string[];

  // Metadata
  description?: string;
  documentation?: string;
}

export interface FlowRDF {
  uri: string;
  source: string;                 // Source task/condition URI
  target: string;                 // Target task/condition URI
  predicate?: string;             // Conditional flow predicate
  order?: number;                 // Flow ordering
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
```

---

## Conclusion

This architecture design provides a comprehensive blueprint for building a modern, Next.js-based YAWL UI that:

1. **Honors DOCTRINE_2027**: Every component maps to O, Σ, Q, Π, or MAPE-K principles
2. **Maintains Turtle as Source of Truth**: Zero reconstruction logic, pure RDF projection
3. **Validates via Pattern Matrix**: All workflows validated against permutation matrix
4. **Enforces Chatman Constant**: ≤8 ticks for hot path operations
5. **Integrates MAPE-K**: Real-time autonomous monitoring and adaptation
6. **Uses Modern Stack**: Next.js 15, React 19, TypeScript, shadcn/ui, React Flow

The implementation roadmap provides a 22-week plan to deliver a production-ready workflow management UI that rivals or exceeds commercial workflow systems while maintaining strict adherence to the KNHK doctrine principles.

**Next Steps**:
1. Review this architecture with stakeholders
2. Validate technical approach with Rust backend team
3. Begin Phase 1 implementation
4. Iterate based on learnings

---

**Document Status**: DRAFT
**Review Required**: Yes
**Approval Required**: System Architect, Backend Team Lead, Product Owner

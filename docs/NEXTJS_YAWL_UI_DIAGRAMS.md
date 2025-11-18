# Next.js YAWL UI - Architecture Diagrams

**Status**: DRAFT | **Version**: 1.0.0 | **Last Updated**: 2025-11-18

This document contains visual architecture diagrams for the Next.js YAWL UI system using text-based diagram formats.

---

## Table of Contents

1. [C4 Model Diagrams](#c4-model-diagrams)
2. [Sequence Diagrams](#sequence-diagrams)
3. [Component Interaction Diagrams](#component-interaction-diagrams)
4. [Data Flow Diagrams](#data-flow-diagrams)
5. [Deployment Architecture](#deployment-architecture)

---

## C4 Model Diagrams

### Level 1: System Context Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                         System Context                             │
└────────────────────────────────────────────────────────────────────┘

    ┌──────────────┐
    │   Workflow   │
    │   Designer   │──────┐
    │   (Human)    │      │
    └──────────────┘      │
                          │ Creates/Edits
                          │ Workflows
                          ↓
    ┌──────────────┐    ┌────────────────────────────┐
    │   System     │    │                            │
    │  Administrator│───→│    Next.js YAWL UI        │
    │   (Human)    │    │   (Web Application)        │
    └──────────────┘    │                            │
          │             │  - Workflow Editor         │
          │             │  - Pattern Validator       │
          │             │  - MAPE-K Monitor          │
          │             └────────────────────────────┘
          │                        ↕
          │                        │ RDF/Turtle
          │                        │ gRPC/HTTP
          │                        ↓
    Configures          ┌────────────────────────────┐
    Settings            │   Rust KNHK Workflow       │
                        │   Engine (Backend)         │
                        │                            │
                        │  - Workflow Executor       │
    ┌──────────────┐   │  - MAPE-K Engine           │
    │ OpenTelemetry│←──│  - Pattern Validator       │
    │  Collector   │   │  - RDF Triple Store        │
    │  (External)  │   └────────────────────────────┘
    └──────────────┘              ↕
          ↑                       │ Telemetry
          │                       │ OTLP
          │                       ↓
          │            ┌────────────────────────────┐
          └────────────│   Weaver Schema            │
                       │   Validator                │
                       │   (Validation Service)     │
                       └────────────────────────────┘
```

---

### Level 2: Container Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                Next.js YAWL UI (Container View)                    │
└────────────────────────────────────────────────────────────────────┘

    ┌──────────────┐
    │   Browser    │
    │   (User)     │
    └──────────────┘
          ↓ HTTPS
    ┌──────────────────────────────────────────────────────────┐
    │              Next.js Application (Node.js)               │
    ├──────────────────────────────────────────────────────────┤
    │                                                          │
    │  ┌────────────────────────────────────────────────┐    │
    │  │        Client Components (React)               │    │
    │  ├────────────────────────────────────────────────┤    │
    │  │  - WorkflowEditor (React Flow)                 │    │
    │  │  - PatternValidator (Real-time validation)     │    │
    │  │  - MAPEKMonitor (Live metrics)                 │    │
    │  │  - Zustand Store (UI state)                    │    │
    │  └────────────────────────────────────────────────┘    │
    │                     ↕ Server Actions                    │
    │  ┌────────────────────────────────────────────────┐    │
    │  │       Server Components/Actions (Node.js)      │    │
    │  ├────────────────────────────────────────────────┤    │
    │  │  - RDF Parser (unrdf)                          │    │
    │  │  - SPARQL Engine (Comunica)                    │    │
    │  │  - Pattern Matrix Validator                    │    │
    │  │  - Workflow Serializer (Turtle)                │    │
    │  │  - API Routes (REST)                           │    │
    │  └────────────────────────────────────────────────┘    │
    │                     ↕                                   │
    │  ┌────────────────────────────────────────────────┐    │
    │  │       In-Memory RDF Store (N3.js)              │    │
    │  ├────────────────────────────────────────────────┤    │
    │  │  - Parsed Turtle workflows                     │    │
    │  │  - Pattern permutation matrix                  │    │
    │  │  - SPARQL query cache                          │    │
    │  └────────────────────────────────────────────────┘    │
    │                                                          │
    └──────────────────────────────────────────────────────────┘
                            ↓ HTTP/gRPC
    ┌──────────────────────────────────────────────────────────┐
    │         Rust KNHK Workflow Engine (Rust)                 │
    ├──────────────────────────────────────────────────────────┤
    │  - RDF Triple Store (Oxigraph)                           │
    │  - Workflow Executor                                     │
    │  - MAPE-K Engine                                         │
    │  - Weaver Client                                         │
    └──────────────────────────────────────────────────────────┘
```

---

### Level 3: Component Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│            Next.js Server Components (Component View)              │
└────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                      RDF Processing Layer                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────────┐   │
│  │  RDF Parser   │  │    SPARQL     │  │  RDF Serializer  │   │
│  │   (unrdf)     │  │    Engine     │  │    (N3.js)       │   │
│  └───────────────┘  └───────────────┘  └──────────────────┘   │
│         ↓                  ↓                    ↑               │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Workflow Processing Layer                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────────┐   │
│  │   Workflow    │  │   Workflow    │  │   React Flow     │   │
│  │    Parser     │  │  Transformer  │  │  Transformer     │   │
│  └───────────────┘  └───────────────┘  └──────────────────┘   │
│         ↓                  ↓                    ↓               │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                    Validation Layer                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────────┐   │
│  │   Pattern     │  │     SHACL     │  │     Weaver       │   │
│  │  Validator    │  │   Validator   │  │    Validator     │   │
│  └───────────────┘  └───────────────┘  └──────────────────┘   │
│         ↓                  ↓                    ↓               │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                       MAPE-K Layer                              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────────┐   │
│  │   Telemetry   │  │   Anomaly     │  │   Knowledge      │   │
│  │  Collector    │  │   Detector    │  │   Base Sync      │   │
│  └───────────────┘  └───────────────┘  └──────────────────┘   │
│         ↓                  ↓                    ↓               │
└─────────────────────────────────────────────────────────────────┘
```

---

## Sequence Diagrams

### Workflow Load Sequence

```
User          Next.js       RDF           SPARQL        React Flow
             Server        Parser        Engine        Transformer
  │             │            │              │               │
  │─────────────→            │              │               │
  │ GET /workflows/123       │              │               │
  │             │            │              │               │
  │             │────────────→              │               │
  │             │ loadWorkflow(123)         │               │
  │             │            │              │               │
  │             │            │─────────────→│               │
  │             │            │ parseTurtle  │               │
  │             │            │              │               │
  │             │            │←─────────────│               │
  │             │            │ RDF Store    │               │
  │             │            │              │               │
  │             │────────────────────────────→              │
  │             │        executeSPARQL       │              │
  │             │    (extract tasks/flows)   │              │
  │             │            │              │               │
  │             │←────────────────────────────              │
  │             │        Workflow object     │              │
  │             │            │              │               │
  │             │──────────────────────────────────────────→│
  │             │             workflowToReactFlow()         │
  │             │            │              │               │
  │             │←──────────────────────────────────────────│
  │             │        { nodes, edges }    │              │
  │             │            │              │               │
  │←─────────────            │              │               │
  │ HTML (Server Component)  │              │               │
  │   with nodes/edges       │              │               │
  │             │            │              │               │
```

---

### Workflow Save Sequence

```
User      WorkflowEditor  Server Action  Pattern      Weaver      RDF Store
                                         Validator    Validator
  │             │              │            │            │            │
  │─────────────→              │            │            │            │
  │ Click "Save"               │            │            │            │
  │             │              │            │            │            │
  │             │──────────────→            │            │            │
  │             │ saveWorkflow(data)        │            │            │
  │             │              │            │            │            │
  │             │              │───────────→│            │            │
  │             │              │ validatePatterns(data)  │            │
  │             │              │            │            │            │
  │             │              │←───────────│            │            │
  │             │              │ { isValid: true }       │            │
  │             │              │            │            │            │
  │             │              │────────────────────────→│            │
  │             │              │   validateWithWeaver(data)           │
  │             │              │            │            │            │
  │             │              │←────────────────────────│            │
  │             │              │   { isValid: true }     │            │
  │             │              │            │            │            │
  │             │              │─────────────────────────────────────→│
  │             │              │   serializeWorkflow(data)            │
  │             │              │            │            │            │
  │             │              │←─────────────────────────────────────│
  │             │              │   Turtle string         │            │
  │             │              │            │            │            │
  │             │              │─────────────────────────────────────→│
  │             │              │   saveTurtleFile()      │            │
  │             │              │            │            │            │
  │             │              │←─────────────────────────────────────│
  │             │              │   { success: true }     │            │
  │             │              │            │            │            │
  │             │←──────────────            │            │            │
  │             │ { success: true }         │            │            │
  │             │              │            │            │            │
  │←─────────────              │            │            │            │
  │ Success Toast              │            │            │            │
  │             │              │            │            │            │
```

---

### MAPE-K Real-time Monitoring Sequence

```
Rust Engine   OTEL        Next.js       MAPE-K        Client
             Collector    API Route     Analyzer      Monitor
     │           │            │            │             │
     │──────────→│            │            │             │
     │ Emit span │            │            │             │
     │ (OTLP)    │            │            │             │
     │           │            │            │             │
     │           │───────────→│            │             │
     │           │ POST /api/telemetry     │             │
     │           │            │            │             │
     │           │            │───────────→│             │
     │           │            │ analyzeTelemetry()       │
     │           │            │            │             │
     │           │            │←───────────│             │
     │           │            │ { anomaly: true }        │
     │           │            │            │             │
     │           │            │────────────────────────→ │
     │           │            │   Send SSE event         │
     │           │            │   (anomaly detected)     │
     │           │            │            │             │
     │           │            │            │             │←─────┐
     │           │            │            │             │      │
     │           │            │            │             │ Update UI
     │           │            │            │             │      │
     │           │            │            │             │←─────┘
     │           │            │            │             │
```

---

## Component Interaction Diagrams

### Workflow Editor Component Interaction

```
┌─────────────────────────────────────────────────────────────────┐
│                    WorkflowEditor Page                          │
└─────────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ↓                   ↓                   ↓
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  EditorCanvas│    │ PropertiesPanel    │ValidationPanel
│ (React Flow) │    │               │    │              │
└──────────────┘    └──────────────┘    └──────────────┘
        │                   │                   │
        │                   │                   │
        │            Shares State               │
        └───────────────────┼───────────────────┘
                            ↓
                    ┌──────────────┐
                    │ EditorStore  │
                    │  (Zustand)   │
                    └──────────────┘
                            │
                            │ reads/writes
                            ↓
                    ┌──────────────┐
                    │ { nodes,     │
                    │   edges,     │
                    │   selection }│
                    └──────────────┘
```

---

### Pattern Validation Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                  Pattern Validation Flow                        │
└─────────────────────────────────────────────────────────────────┘

    Workflow Object
         │
         ↓
    ┌─────────────────┐
    │  Extract Nodes  │
    │  & Edges        │
    └─────────────────┘
         │
         ↓
    ┌─────────────────┐
    │ Load Pattern    │
    │ Permutation     │ ←── yawl-pattern-permutations.ttl
    │ Matrix (RDF)    │
    └─────────────────┘
         │
         ↓
    ┌─────────────────┐
    │ For each node:  │
    │ - Get split/join│
    │ - Check against │
    │   matrix        │
    └─────────────────┘
         │
    ┌────┴────┐
    │         │
    ↓         ↓
Valid?     Invalid?
    │         │
    │         ↓
    │    ┌─────────────────┐
    │    │ Collect         │
    │    │ Violations      │
    │    └─────────────────┘
    │         │
    └────┬────┘
         ↓
    ┌─────────────────┐
    │ Return Result:  │
    │ - isValid       │
    │ - patterns[]    │
    │ - violations[]  │
    └─────────────────┘
```

---

## Data Flow Diagrams

### RDF Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                       RDF Data Flow                             │
└─────────────────────────────────────────────────────────────────┘

   Turtle File (.ttl)
         │
         │ Read
         ↓
   ┌─────────────┐
   │ N3 Parser   │
   └─────────────┘
         │
         │ Parse
         ↓
   ┌─────────────┐
   │ RDF Quads   │ ← Subject-Predicate-Object-Graph
   └─────────────┘
         │
         │ Store
         ↓
   ┌─────────────┐
   │  N3 Store   │ ← In-memory triple store
   └─────────────┘
         │
         │ Query
         ↓
   ┌─────────────┐
   │   SPARQL    │ ← Extract structured data
   │   Engine    │
   └─────────────┘
         │
         │ Map
         ↓
   ┌─────────────┐
   │  Workflow   │ ← TypeScript object
   │   Object    │
   └─────────────┘
         │
         ├──────→ Transform ──→ React Flow { nodes, edges }
         │                             │
         │                             ↓
         │                       Render in UI
         │
         └──────→ Serialize ──→ Turtle String
                                       │
                                       ↓
                                  Save to File
```

---

### State Management Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                   State Management Flow                         │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────────┐
│   Server State       │ ← Managed by TanStack Query
├──────────────────────┤
│ - Workflow RDF data  │
│ - Pattern library    │
│ - Validation results │
│ - Telemetry metrics  │
└──────────────────────┘
         ↓
   Fetched via API
         ↓
┌──────────────────────┐
│  React Query Cache   │ ← Cached, auto-revalidated
└──────────────────────┘
         ↓
   Provided to components
         ↓
┌──────────────────────┐
│  Client State        │ ← Managed by Zustand
├──────────────────────┤
│ - Editor selection   │
│ - UI mode            │
│ - Temporary positions│
│ - Panel open/closed  │
└──────────────────────┘
         ↓
   Used by UI components
         ↓
┌──────────────────────┐
│  React Components    │
└──────────────────────┘
```

---

## Deployment Architecture

### Vercel Deployment Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Vercel Edge Network                          │
└─────────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ↓                   ↓                   ↓
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   CDN Edge   │    │   CDN Edge   │    │   CDN Edge   │
│  (us-east-1) │    │  (eu-west-1) │    │ (ap-south-1) │
└──────────────┘    └──────────────┘    └──────────────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
                            ↓
                    ┌──────────────┐
                    │  Next.js     │
                    │  Serverless  │
                    │  Functions   │
                    └──────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ↓                   ↓                   ↓
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Static      │    │  API Routes  │    │  Server      │
│  Assets      │    │  (REST)      │    │  Actions     │
│  (S3/CDN)    │    │              │    │              │
└──────────────┘    └──────────────┘    └──────────────┘
                            │
                            ↓
                    ┌──────────────┐
                    │  Rust KNHK   │
                    │  Backend     │
                    │  (Cloud Run  │
                    │   or EC2)    │
                    └──────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ↓                   ↓                   ↓
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  RDF Triple  │    │  OTEL        │    │  Weaver      │
│  Store       │    │  Collector   │    │  Validator   │
│  (Postgres)  │    │  (Service)   │    │  (Service)   │
└──────────────┘    └──────────────┘    └──────────────┘
```

---

### Production Infrastructure

```
┌─────────────────────────────────────────────────────────────────┐
│                  Production Infrastructure                      │
└─────────────────────────────────────────────────────────────────┘

    ┌─────────────────────────────────────────────────────────┐
    │                    Load Balancer                        │
    │                  (Cloudflare / AWS ALB)                 │
    └─────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ↓                   ↓                   ↓
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Next.js     │    │  Next.js     │    │  Next.js     │
│  Instance 1  │    │  Instance 2  │    │  Instance 3  │
│  (us-east)   │    │  (us-west)   │    │  (eu-west)   │
└──────────────┘    └──────────────┘    └──────────────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
                            ↓
                    ┌──────────────┐
                    │  API Gateway │
                    └──────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        │                   │                   │
        ↓                   ↓                   ↓
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Workflow    │    │  Validation  │    │  Telemetry   │
│  Service     │    │  Service     │    │  Service     │
│  (Rust)      │    │  (Weaver)    │    │  (OTEL)      │
└──────────────┘    └──────────────┘    └──────────────┘
        │                   │                   │
        └───────────────────┼───────────────────┘
                            │
                            ↓
                    ┌──────────────┐
                    │  Database    │
                    │  Cluster     │
                    │  (Postgres + │
                    │   Oxigraph)  │
                    └──────────────┘
```

---

## Technology Integration Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│              Technology Stack Integration                       │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    Frontend Layer                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  React 19           shadcn/ui          React Flow               │
│  (UI Framework)     (Components)       (Visualization)          │
│       │                 │                   │                   │
│       └─────────────────┼───────────────────┘                   │
│                         │                                       │
│                    TypeScript                                   │
│                   (Type Safety)                                 │
│                         │                                       │
│                         ↓                                       │
│  ┌─────────────────────────────────────────────────┐           │
│  │         State Management Layer                  │           │
│  ├─────────────────────────────────────────────────┤           │
│  │  Zustand           TanStack Query               │           │
│  │  (Client State)    (Server State Cache)         │           │
│  └─────────────────────────────────────────────────┘           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                   Next.js Layer                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  App Router         Server Actions     API Routes               │
│  (Routing)          (Mutations)        (REST)                   │
│       │                 │                   │                   │
│       └─────────────────┼───────────────────┘                   │
│                         │                                       │
│                    Node.js Runtime                              │
│                         │                                       │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                  RDF Processing Layer                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  unrdf / N3.js      Comunica          SPARQL                    │
│  (RDF Parser)       (Query Engine)    (Query Language)          │
│       │                 │                   │                   │
│       └─────────────────┼───────────────────┘                   │
│                         │                                       │
│                   In-Memory Store                               │
│                   (RDF Quads)                                   │
│                         │                                       │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                 Validation Layer                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Pattern Matrix     SHACL             Weaver                    │
│  (Permutations)     (Shapes)          (OTEL Schema)             │
│       │                 │                   │                   │
│       └─────────────────┼───────────────────┘                   │
│                         │                                       │
│                 Covenant Enforcement                            │
│                (Q Invariants)                                   │
│                         │                                       │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                   Backend Layer (Rust)                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Workflow Engine    MAPE-K Engine     RDF Store                 │
│  (Executor)         (Autonomic)       (Oxigraph)                │
│       │                 │                   │                   │
│       └─────────────────┼───────────────────┘                   │
│                         │                                       │
│                 OpenTelemetry                                   │
│                 (Telemetry)                                     │
│                         │                                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## MAPE-K Loop Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      MAPE-K Feedback Loop                       │
└─────────────────────────────────────────────────────────────────┘

                    ┌──────────────┐
                    │   Monitor    │
                    │   (Collect   │
                    │  Telemetry)  │
                    └──────────────┘
                           │
                           │ Observations
                           ↓
                    ┌──────────────┐
                    │   Analyze    │
                    │   (Detect    │
                    │  Anomalies)  │
                    └──────────────┘
                           │
                           │ Symptoms
                           ↓
                    ┌──────────────┐
                    │     Plan     │
                    │  (Generate   │
                    │ Adaptations) │
                    └──────────────┘
                           │
                           │ Actions
                           ↓
                    ┌──────────────┐
                    │   Execute    │
                    │  (Apply      │
                    │   Changes)   │
                    └──────────────┘
                           │
                           │ Effects
                           ↓
                    ┌──────────────┐
                    │  Knowledge   │ ←─────┐
                    │   (Store     │       │
                    │  Learnings)  │       │
                    └──────────────┘       │
                           │               │
                           │ Feedback      │
                           └───────────────┘

    All components read/write from Knowledge base
    Knowledge base contains:
    - Historical metrics
    - Learned patterns
    - Adaptation policies
    - Baseline performance
```

---

## Doctrine Alignment Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│              DOCTRINE_2027 → Architecture Mapping               │
└─────────────────────────────────────────────────────────────────┘

┌─────────────┐           ┌─────────────────────────┐
│     O       │  ────→    │  RDF Parser             │
│(Observation)│           │  SPARQL Engine          │
│             │           │  Telemetry Collector    │
└─────────────┘           └─────────────────────────┘

┌─────────────┐           ┌─────────────────────────┐
│     Σ       │  ────→    │  YAWL Ontology (.ttl)   │
│ (Ontology)  │           │  Pattern Matrix         │
│             │           │  MAPE-K Ontology        │
└─────────────┘           └─────────────────────────┘

┌─────────────┐           ┌─────────────────────────┐
│     Q       │  ────→    │  Pattern Validator      │
│(Invariants) │           │  SHACL Validator        │
│             │           │  Chatman Guard          │
└─────────────┘           └─────────────────────────┘

┌─────────────┐           ┌─────────────────────────┐
│     Π       │  ────→    │  React Components       │
│(Projections)│           │  Turtle Serializer      │
│             │           │  React Flow Transformer │
└─────────────┘           └─────────────────────────┘

┌─────────────┐           ┌─────────────────────────┐
│  MAPE-K     │  ────→    │  MAPEKMonitor Component │
│   Loop      │           │  Anomaly Detector       │
│             │           │  Knowledge Base Sync    │
└─────────────┘           └─────────────────────────┘

┌─────────────┐           ┌─────────────────────────┐
│  Chatman    │  ────→    │  Performance Guards     │
│  Constant   │           │  Latency Metrics        │
│   (≤8 ticks)│           │  Hot Path Optimization  │
└─────────────┘           └─────────────────────────┘
```

---

## Security Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                   Security Architecture                         │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                  Edge Layer (Cloudflare)                        │
├─────────────────────────────────────────────────────────────────┤
│  - DDoS Protection                                              │
│  - WAF Rules                                                    │
│  - Rate Limiting                                                │
│  - SSL/TLS Termination                                          │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                Application Layer (Next.js)                      │
├─────────────────────────────────────────────────────────────────┤
│  - NextAuth.js (Authentication)                                 │
│  - RBAC (Authorization)                                         │
│  - CSRF Tokens                                                  │
│  - Content Security Policy                                      │
│  - Input Validation (Zod)                                       │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                  API Layer (Server Actions)                     │
├─────────────────────────────────────────────────────────────────┤
│  - Request Validation                                           │
│  - Parameterized SPARQL (no injection)                          │
│  - File System Sandbox (/ontology only)                         │
│  - API Rate Limiting                                            │
└─────────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                  Data Layer (Rust Backend)                      │
├─────────────────────────────────────────────────────────────────┤
│  - Encrypted at Rest                                            │
│  - Encrypted in Transit (TLS)                                   │
│  - Immutable Audit Log                                          │
│  - Row-level Security (RLS)                                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Conclusion

These diagrams provide visual representations of:

1. **System Architecture** (C4 Model) - From high-level context to detailed components
2. **Sequence Flows** - How data moves through the system
3. **Component Interactions** - How different parts work together
4. **Deployment Architecture** - Production infrastructure layout
5. **Technology Integration** - How the tech stack connects
6. **MAPE-K Feedback** - Autonomous monitoring and adaptation
7. **Doctrine Alignment** - How architecture honors DOCTRINE_2027
8. **Security** - Multi-layer security approach

Use these diagrams in conjunction with the main architecture document (`NEXTJS_YAWL_UI_ARCHITECTURE.md`) for complete system understanding.

---

**Document Status**: DRAFT
**Complements**: NEXTJS_YAWL_UI_ARCHITECTURE.md

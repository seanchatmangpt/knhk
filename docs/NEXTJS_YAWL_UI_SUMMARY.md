# Next.js YAWL UI - Executive Summary

**Status**: DRAFT | **Version**: 1.0.0 | **Last Updated**: 2025-11-18

**Quick Links**:
- [Full Architecture](./NEXTJS_YAWL_UI_ARCHITECTURE.md)
- [Architecture Diagrams](./NEXTJS_YAWL_UI_DIAGRAMS.md)
- [Quick Start Guide](./NEXTJS_YAWL_UI_QUICKSTART.md)

---

## Overview

The Next.js YAWL UI is a modern, web-based workflow management interface that brings YAWL (Yet Another Workflow Language) to the browser with a focus on **Doctrine compliance**, **RDF-first architecture**, and **autonomous MAPE-K monitoring**.

### Key Features

✅ **Visual Workflow Editor** - Drag-and-drop workflow design with React Flow
✅ **RDF/Turtle Native** - Workflows stored as RDF, not JSON
✅ **Pattern Validation** - Real-time validation against W3C workflow patterns
✅ **MAPE-K Integration** - Autonomous monitoring and adaptation
✅ **Weaver Validation** - OpenTelemetry schema compliance
✅ **Modern Stack** - Next.js 15, React 19, TypeScript, shadcn/ui

---

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Framework** | Next.js 15 (App Router) | Server-side rendering, API routes, server actions |
| **UI Library** | React 19 + shadcn/ui | Modern, accessible component library |
| **Language** | TypeScript 5 | Type safety across frontend and backend |
| **RDF Processing** | unrdf / N3.js | Parse and serialize Turtle workflows |
| **Workflow Viz** | React Flow | Interactive workflow graph rendering |
| **State** | Zustand + TanStack Query | Client state + server state cache |
| **Validation** | Zod + SHACL + Weaver | Schema validation at all layers |
| **Styling** | TailwindCSS | Utility-first CSS framework |

---

## Doctrine Alignment

**CRITICAL**: Every architectural decision honors DOCTRINE_2027.

### The Six Covenants

| Covenant | Principle | Implementation |
|----------|-----------|----------------|
| **1. Turtle is Source of Truth** | O ⊨ Σ | RDF parser with zero reconstruction logic |
| **2. Invariants Are Law** | Q validates | Pattern matrix + SHACL + Weaver validation |
| **3. MAPE-K Runs at Machine Speed** | Feedback loops | Real-time telemetry dashboard |
| **4. All Patterns Expressible** | Σ completeness | W3C pattern permutation matrix |
| **5. Chatman Constant Guards Complexity** | ≤8 ticks | Performance guards on hot paths |
| **6. Observations Drive Everything** | O → Analysis | OpenTelemetry integration |

**No code violates these covenants. Violations are build failures.**

---

## Architecture at a Glance

```
┌────────────────────────────────────────────────────┐
│              Browser (React)                       │
│  - Workflow Editor (React Flow)                    │
│  - Pattern Validator (Real-time)                   │
│  - MAPE-K Monitor (Live metrics)                   │
└────────────────────────────────────────────────────┘
                      ↕ Server Actions
┌────────────────────────────────────────────────────┐
│         Next.js Server (Node.js)                   │
│  - RDF Parser (unrdf)                              │
│  - SPARQL Engine (Comunica)                        │
│  - Pattern Validation (Matrix)                     │
│  - Weaver Integration                              │
└────────────────────────────────────────────────────┘
                      ↕ HTTP/gRPC
┌────────────────────────────────────────────────────┐
│       Rust KNHK Workflow Engine                    │
│  - Workflow Executor                               │
│  - MAPE-K Engine                                   │
│  - RDF Triple Store                                │
└────────────────────────────────────────────────────┘
```

---

## Key Design Decisions

### ADR-001: Next.js App Router

**Why**: Server Components enable zero-latency RDF parsing, SEO-friendly workflow documentation, and streaming SSR.

**Trade-off**: More complex than SPA, but aligns with Covenant 1 (server-side Turtle parsing).

---

### ADR-002: shadcn/ui

**Why**: Copy-paste components live in codebase, not node_modules. Full customization without forking.

**Trade-off**: More setup than pre-built libraries, but total control over YAWL-specific patterns.

---

### ADR-003: unrdf for RDF Processing

**Why**: Universal (browser + Node.js), lightweight (~50KB), TypeScript-native.

**Trade-off**: Smaller ecosystem than rdflib.js, but no WASM binary requirement.

---

### ADR-004: React Flow for Visualization

**Why**: Handles 10,000+ nodes, custom node types, keyboard navigation, accessibility.

**Trade-off**: Canvas-based (not SVG), but performance is critical.

---

### ADR-005: Zustand for Client State

**Why**: Minimal API (~1KB), no Context hell, Immer integration for immutable updates.

**Critical Constraint**: Workflow data NEVER stored in Zustand. Only RDF store or server.

---

### ADR-006: Server Actions for Mutations

**Why**: Type-safe mutations from UI to RDF store, progressive enhancement, Covenant 2 alignment.

**Trade-off**: Requires Next.js, but native integration is worth it.

---

### ADR-007: Weaver Schema as Contract

**Why**: OpenTelemetry Weaver schema is source of truth for UI ↔ Backend contract.

**Covenant Alignment**: Covenant 6 (all UI metrics backed by schema).

---

## Data Flow

### Loading a Workflow

```
User → Next.js Server → Load .ttl → Parse Turtle → SPARQL Extract
  → Transform to React Flow → Render → User sees workflow
```

### Saving a Workflow

```
User → Click Save → Validate Patterns → Validate Weaver
  → Serialize to Turtle → Save .ttl → Success
```

### Real-time Monitoring

```
Rust Engine → Emit OTLP → OTEL Collector → Next.js API
  → Validate Weaver → Store in MAPE-K → Analyze Anomalies
  → Send SSE → Client Dashboard → User sees metrics
```

---

## Implementation Roadmap

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| **1. Foundation** | 2 weeks | Next.js app, RDF parsing, workflow list |
| **2. Visualization** | 2 weeks | React Flow renderer, auto-layout |
| **3. Editor** | 3 weeks | Drag-and-drop editor, properties panel |
| **4. Validation** | 2 weeks | Pattern matrix validation UI |
| **5. MAPE-K** | 3 weeks | Real-time monitoring dashboard |
| **6. Weaver** | 2 weeks | Schema validation integration |
| **7. Patterns** | 2 weeks | Pattern library, documentation |
| **8. Templates** | 2 weeks | Template gallery, export (PNG/SVG/PDF) |
| **9. Testing** | 2 weeks | 80%+ coverage, E2E tests |
| **10. Production** | 2 weeks | Vercel deployment, monitoring |
| **Total** | **22 weeks** | Production-ready YAWL UI |

---

## Success Metrics

### Functional Requirements

- [ ] Load and display any YAWL .ttl workflow
- [ ] Create workflows via drag-and-drop editor
- [ ] Validate workflows against pattern matrix
- [ ] Real-time MAPE-K metrics dashboard
- [ ] Export workflows as PNG/SVG/PDF/Turtle
- [ ] Pattern library with all 43 W3C patterns

### Non-Functional Requirements

- [ ] **Performance**: Chatman Constant compliance (≤8 ticks hot path)
- [ ] **Validation**: 100% Weaver schema compliance
- [ ] **Quality**: Zero Covenant violations
- [ ] **Testing**: 80%+ code coverage
- [ ] **Accessibility**: WCAG 2.1 AA compliance
- [ ] **Documentation**: Complete user guide + API docs

### Doctrine Compliance

- [ ] Covenant 1: All workflows in RDF/Turtle (zero JSON)
- [ ] Covenant 2: All workflows pass pattern matrix validation
- [ ] Covenant 3: MAPE-K feedback loop ≤ 1 second latency
- [ ] Covenant 4: All 43 W3C patterns expressible
- [ ] Covenant 5: Hot path operations ≤ 8 ticks
- [ ] Covenant 6: All UI metrics backed by Weaver schema

---

## Risks and Mitigations

### Risk: RDF Parsing Performance

**Impact**: High
**Likelihood**: Medium
**Mitigation**: Server-side parsing with LRU cache (5-min TTL)

---

### Risk: React Flow Scalability

**Impact**: High
**Likelihood**: Low
**Mitigation**: Virtualization, memoization, only render visible nodes

---

### Risk: Pattern Matrix Complexity

**Impact**: Medium
**Likelihood**: Medium
**Mitigation**: Pre-compute valid combinations, cache SPARQL results

---

### Risk: Weaver Integration Latency

**Impact**: Medium
**Likelihood**: Low
**Mitigation**: Async validation, optimistic UI updates

---

## Getting Started

### For Developers

1. Read [Quick Start Guide](./NEXTJS_YAWL_UI_QUICKSTART.md)
2. Clone repo and run `npm install`
3. Start with Phase 1: Foundation
4. Follow 22-week roadmap

### For Architects

1. Review [Full Architecture](./NEXTJS_YAWL_UI_ARCHITECTURE.md)
2. Review [Architecture Diagrams](./NEXTJS_YAWL_UI_DIAGRAMS.md)
3. Validate ADRs against your requirements
4. Approve or request changes

### For Product Owners

1. Review this summary
2. Confirm functional requirements
3. Review 22-week timeline
4. Approve Phase 1 kickoff

---

## Questions and Answers

### Q: Why not use existing YAWL UI?

**A**: Legacy YAWL UI is Java-based, not web-native, no RDF-first architecture, no MAPE-K integration.

---

### Q: Why RDF/Turtle instead of JSON?

**A**: Doctrine Covenant 1 - Turtle is source of truth. JSON would be reconstruction, violating Covenant 1.

---

### Q: Can this integrate with existing Rust backend?

**A**: Yes, designed for it. Uses HTTP/gRPC APIs, OpenTelemetry, and Weaver validation.

---

### Q: What about offline support?

**A**: Phase 11+ (future). Requires service worker + IndexedDB RDF store.

---

### Q: How does this differ from BPMN.io?

**A**: BPMN.io is BPMN-focused, JSON-based, no RDF, no MAPE-K, no Weaver validation, no Doctrine compliance.

---

## Conclusion

The Next.js YAWL UI provides a **modern, web-based workflow management interface** that honors every Doctrine principle while delivering a **production-grade user experience**.

**Key Differentiators**:
1. **RDF-First**: Not serialization, but source of truth
2. **Pattern Matrix Validation**: Every workflow validated against W3C patterns
3. **MAPE-K Native**: Autonomous monitoring built-in
4. **Weaver Validated**: OpenTelemetry schema compliance
5. **Modern Stack**: Next.js 15, React 19, TypeScript 5

**Timeline**: 22 weeks to production-ready deployment
**Risk**: Low (proven technologies, clear roadmap)
**Impact**: High (enables web-based YAWL workflow management)

**Recommendation**: Approve Phase 1 kickoff.

---

**Document Status**: DRAFT
**Review Required**: System Architect, Backend Team Lead, Product Owner
**Approval Required**: Yes

---

## Document Index

1. **Executive Summary** (this document) - High-level overview
2. [**Full Architecture**](./NEXTJS_YAWL_UI_ARCHITECTURE.md) - Complete system design
3. [**Architecture Diagrams**](./NEXTJS_YAWL_UI_DIAGRAMS.md) - Visual diagrams (C4, sequence, etc.)
4. [**Quick Start Guide**](./NEXTJS_YAWL_UI_QUICKSTART.md) - Developer implementation guide

**Start here, drill down as needed.**

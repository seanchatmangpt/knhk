# Agent Selection Guide (Skills Matching)

**Principle:** Match task requirements to agent capabilities

---

## Decision Matrix

| Task Type | Use This Agent | Why |
|-----------|----------------|-----|
| Production validation | `production-validator` | Specialized in DoD/deployment checks |
| Code quality review | `code-analyzer` | Deep analysis of architecture/debt |
| System design | `system-architect` | Architectural decisions |
| Performance analysis | `performance-benchmarker` | Profiling/optimization |
| Backend/infra | `backend-dev` | Docker/OTLP/databases |
| Complex workflows | `task-orchestrator` | Multi-phase coordination |
| TDD implementation | `tdd-london-swarm` | Mock-driven tests |
| CI/CD pipelines | `cicd-engineer` | GitHub Actions/workflows |

---

## Anti-Patterns (Waste)

❌ **Using `researcher` for analysis**
  → Use `code-analyzer` or `system-architect` instead
  → Waste: 2x time (researcher lacks domain expertise)

❌ **Using `coder` for architecture**
  → Use `system-architect` instead
  → Waste: Poor design decisions, 4x rework

❌ **Using `tester` for TDD**
  → Use `tdd-london-swarm` instead
  → Waste: Missing mock-driven approach, 2x effort

---

## Validation

Before spawning agents, ask:
1. Does this task require specialized expertise? → Use advanced agent
2. Is this a simple, well-defined task? → Basic agent OK
3. Multiple phases/coordination needed? → Use `task-orchestrator`

**Waste eliminated:** ~10 hours/week (wrong agent selection → rework)

# Single-Piece Flow Implementation

**Anti-Pattern (Batching):**
```
Write 10 features → Write 10 tests → Review 10 features → Fix 10 bugs
└─ Long feedback loops, late defect discovery
```

**Pattern (Single-Piece Flow):**
```
Write 1 feature → Write 1 test → Review 1 feature → Fix 1 bug → DONE
└─ Immediate feedback, early defect discovery
```

---

## Implementation Rules

1. **Complete one thing at a time**
   - Don't start feature #2 until feature #1 is DONE
   - DONE = coded + tested + reviewed + merged

2. **Immediate validation**
   - Run tests after every change
   - Review code within 1 hour of writing
   - Fix defects within 1 hour of discovery

3. **Small batch sizes**
   - PRs: <100 lines changed
   - Features: Completable in <4 hours
   - Tests: Runnable in <1 minute

---

## Waste Eliminated

- **Batching delay:** 4-8 hours → <1 hour
- **Rework cost:** 4x (late discovery) → 1x (immediate)
- **Context switching:** 30% overhead → 5% overhead

---

**Metric:** Cycle time (code → production)
- Before: 2-5 days
- Target: <4 hours

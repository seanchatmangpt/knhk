# Agent Swarm Instructions: Fix knhk-etl Compilation Errors

## Mission

**Fix 10 compilation errors in knhk-etl to unblock the entire critical path.**

**Time Estimate**: 30-45 minutes
**Priority**: CRITICAL - Blocks knhk-kernel, knhk-patterns, knhk-workflow-engine

---

## Agent Assignments

### Agent 1: Code Analyzer (PRIMARY)
**Type**: `code-analyzer`
**Task**: Analyze knhk-etl structure and identify exact fixes needed

**Instructions**:
```
1. Read knhk-etl source files:
   - rust/knhk-etl/src/ingest.rs
   - rust/knhk-etl/src/transform.rs
   - rust/knhk-etl/src/load.rs
   - rust/knhk-etl/src/lib.rs

2. Identify:
   - RawTriple actual struct definition (what fields exist?)
   - IngestStage current implementation (what's missing?)
   - Transform/Load error patterns

3. Create detailed fix plan with:
   - Exact line numbers to change
   - Before/after code snippets
   - Rationale for each fix

4. Store analysis in memory:
   npx claude-flow@alpha hooks post-edit \
     --file "rust/knhk-etl/analysis.md" \
     --memory-key "swarm/etl-fix/analysis"
```

---

### Agent 2: Backend Developer (EXECUTOR)
**Type**: `backend-dev`
**Task**: Implement the fixes identified by Agent 1

**Instructions**:
```
BEFORE WORK:
npx claude-flow@alpha hooks pre-task --description "Fix knhk-etl compilation errors"
npx claude-flow@alpha hooks session-restore --session-id "swarm-etl-fix"

WORK SEQUENCE:

1. Fix IngestStage API (rust/knhk-etl/src/ingest.rs):
   - Add IngestStage::new(validator: Arc<dyn Validator>) -> Self
   - Add IngestStage::ingest(&self, source: &str) -> Result<Vec<RawTriple>, PipelineError>
   - Implement basic logic (can be minimal MVP)

2. Fix RawTriple field access (load.rs, transform.rs):
   - Find all references to `triple.graph`
   - Replace with correct RDF fields: subject, predicate, object
   - Verify RawTriple struct definition in ingest.rs

3. Fix Result unwrapping (transform.rs):
   - Line ~47: Change `input.triples` to `input?.triples` or proper unwrap
   - Line ~57: Same pattern
   - Ensure Result is unwrapped before field access

4. Add type annotations where compiler requests:
   - Check error[E0282] locations
   - Add explicit type annotations

AFTER EACH FILE:
npx claude-flow@alpha hooks post-edit \
  --file "rust/knhk-etl/src/{filename}" \
  --memory-key "swarm/etl-fix/{filename}"

AFTER WORK:
npx claude-flow@alpha hooks post-task --task-id "fix-knhk-etl"
npx claude-flow@alpha hooks notify --message "knhk-etl fixes completed"
```

---

### Agent 3: Test Engineer (VALIDATOR)
**Type**: `tester`
**Task**: Verify fixes work and critical path unblocks

**Instructions**:
```
VALIDATION SEQUENCE:

1. Compile knhk-etl:
   cd rust/knhk-etl
   cargo check 2>&1 | tee etl-check.log

   EXPECTED: 0 errors
   IF ERRORS: Report back to Agent 2 for fixes

2. Compile critical path packages:
   cd rust/knhk-kernel && cargo check
   cd rust/knhk-patterns && cargo check
   cd rust/knhk-workflow-engine && cargo check

   EXPECTED: All should compile
   IF ERRORS: Investigate if still blocked by knhk-etl

3. Run basic tests (if any exist):
   cd rust/knhk-etl
   cargo test

   EXPECTED: Tests pass or no tests defined
   IF FAILURES: Document but don't block (can fix later)

4. Document results:
   Create verification report in memory:
   npx claude-flow@alpha hooks post-edit \
     --file "verification-report.md" \
     --memory-key "swarm/etl-fix/verification"
```

---

### Agent 4: System Architect (REVIEWER)
**Type**: `system-architect`
**Task**: Ensure fixes don't violate DOCTRINE covenants

**Instructions**:
```
DOCTRINE ALIGNMENT CHECK:

1. Verify fixes align with DOCTRINE_2027.md principles:
   - Q (Hard Invariants): Are quality gates maintained?
   - O (Observability): Is telemetry preserved?
   - Π (Permutation Matrix): Do changes respect pattern definitions?

2. Check for anti-patterns:
   - No .unwrap() in production paths (use ? operator)
   - No placeholder implementations with unimplemented!()
   - No fake Ok(()) returns

3. Verify Chicago TDD compatibility:
   - Do fixes maintain ≤8 tick hot path guarantee?
   - Are changes measurable via OTEL?

4. Review code quality:
   - Proper error handling (Result types)
   - No println! in production code
   - Trait objects remain dyn-compatible

CRITICAL GATE:
If any DOCTRINE violation found, STOP and report as blocking.

AFTER REVIEW:
npx claude-flow@alpha hooks post-task --task-id "etl-fix-review"
npx claude-flow@alpha hooks notify --message "Architecture review completed"
```

---

## Success Criteria

**ALL must be true:**

- [ ] `cargo check` on knhk-etl passes with 0 errors
- [ ] `cargo check` on knhk-kernel passes
- [ ] `cargo check` on knhk-patterns passes
- [ ] `cargo check` on knhk-workflow-engine passes
- [ ] No DOCTRINE covenant violations
- [ ] No new warnings introduced
- [ ] Proper error handling (no .unwrap() in production paths)

---

## Error Reference

**Current knhk-etl errors (10 total):**

```
error[E0609]: no field `graph` on type `&ingest::RawTriple` (3 instances)
  → Fix: Use subject/predicate/object fields instead

error[E0609]: no field `triples` on type `Result<Vec<RawTriple>, ...>` (2 instances)
  → Fix: Unwrap Result before accessing fields

error[E0560]: struct `ingest::RawTriple` has no field named `graph`
  → Fix: Remove graph field references

error[E0599]: no function or associated item named `new` found for struct `IngestStage`
  → Fix: Implement IngestStage::new()

error[E0599]: no method named `ingest` found for struct `IngestStage` (2 instances)
  → Fix: Implement IngestStage::ingest()

error[E0282]: type annotations needed
  → Fix: Add explicit type annotations where compiler requests
```

---

## Files to Modify

**Primary Files** (where errors are):
- `rust/knhk-etl/src/ingest.rs` - Add missing IngestStage methods
- `rust/knhk-etl/src/transform.rs` - Fix Result unwrapping, field names
- `rust/knhk-etl/src/load.rs` - Fix RawTriple field access

**Reference Files** (read-only):
- `rust/knhk-etl/src/lib.rs` - Understand public API
- `rust/knhk-etl/src/error.rs` - Understand PipelineError
- `DOCTRINE_2027.md` - Covenant alignment
- `DOCTRINE_COVENANT.md` - Enforcement rules

---

## Coordination Protocol

### Agent Execution Order:
1. **Code Analyzer** (Agent 1) - Runs FIRST, creates fix plan
2. **Backend Developer** (Agent 2) - Implements fixes from plan
3. **Test Engineer** (Agent 3) - Validates fixes
4. **System Architect** (Agent 4) - Reviews for DOCTRINE compliance

### Memory Keys:
- `swarm/etl-fix/analysis` - Code analysis and fix plan
- `swarm/etl-fix/ingest.rs` - IngestStage fixes
- `swarm/etl-fix/transform.rs` - Transform fixes
- `swarm/etl-fix/load.rs` - Load fixes
- `swarm/etl-fix/verification` - Test results
- `swarm/etl-fix/review` - Architecture review

### Session ID:
- `swarm-etl-fix` - Use for all hooks session-restore

---

## Deployment

**To execute this swarm:**

```javascript
// Use Claude Code's Task tool to spawn all agents concurrently
[Single Message - Parallel Agent Execution]:
  Task("Code Analyzer", "Analyze knhk-etl errors and create fix plan. Store in memory: swarm/etl-fix/analysis", "code-analyzer")
  Task("Backend Developer", "Implement knhk-etl fixes from analysis. Coordinate via hooks.", "backend-dev")
  Task("Test Engineer", "Validate knhk-etl compiles and unblocks critical path.", "tester")
  Task("System Architect", "Review fixes for DOCTRINE compliance and quality gates.", "system-architect")

  // Track all work
  TodoWrite { todos: [
    {content: "Analyze knhk-etl errors", status: "in_progress", activeForm: "Analyzing knhk-etl errors"},
    {content: "Implement IngestStage methods", status: "pending", activeForm: "Implementing IngestStage methods"},
    {content: "Fix RawTriple field access", status: "pending", activeForm: "Fixing RawTriple field access"},
    {content: "Fix Result unwrapping", status: "pending", activeForm: "Fixing Result unwrapping"},
    {content: "Add type annotations", status: "pending", activeForm: "Adding type annotations"},
    {content: "Verify knhk-etl compiles", status: "pending", activeForm: "Verifying knhk-etl compiles"},
    {content: "Verify critical path unblocks", status: "pending", activeForm: "Verifying critical path unblocks"},
    {content: "DOCTRINE compliance review", status: "pending", activeForm: "Reviewing DOCTRINE compliance"},
    {content: "Update documentation", status: "pending", activeForm: "Updating documentation"}
  ]}
```

---

## Expected Outcome

**After 30-45 minutes:**
- ✅ knhk-etl compiles with 0 errors
- ✅ knhk-kernel, knhk-patterns, knhk-workflow-engine all compile
- ✅ Critical path unblocked for hot path development
- ✅ DOCTRINE covenants maintained
- ✅ Ready to proceed with Chicago TDD validation

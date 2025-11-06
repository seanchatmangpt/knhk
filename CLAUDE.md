# Claude Code Configuration - SPARC Development Environment

## 🚨 CRITICAL: CONCURRENT EXECUTION & FILE MANAGEMENT

**ABSOLUTE RULES**:
1. ALL operations MUST be concurrent/parallel in a single message
2. **NEVER save working files, text/mds and tests to the root folder**
3. ALWAYS organize files in appropriate subdirectories
4. **USE CLAUDE CODE'S TASK TOOL** for spawning agents concurrently, not just MCP

### ⚡ GOLDEN RULE: "1 MESSAGE = ALL RELATED OPERATIONS"

**MANDATORY PATTERNS:**
- **TodoWrite**: ALWAYS batch ALL todos in ONE call (5-10+ todos minimum)
- **Task tool (Claude Code)**: ALWAYS spawn ALL agents in ONE message with full instructions
- **File operations**: ALWAYS batch ALL reads/writes/edits in ONE message
- **Bash commands**: ALWAYS batch ALL terminal operations in ONE message
- **Memory operations**: ALWAYS batch ALL memory store/retrieve in ONE message

### 🎯 CRITICAL: Claude Code Task Tool for Agent Execution

**Claude Code's Task tool is the PRIMARY way to spawn agents:**
```javascript
// ✅ CORRECT: Use Claude Code's Task tool for parallel agent execution
[Single Message]:
  Task("Research agent", "Analyze requirements and patterns...", "researcher")
  Task("Coder agent", "Implement core features...", "coder")
  Task("Tester agent", "Create comprehensive tests...", "tester")
  Task("Reviewer agent", "Review code quality...", "reviewer")
  Task("Architect agent", "Design system architecture...", "system-architect")
```

**MCP tools are ONLY for coordination setup:**
- `mcp__claude-flow__swarm_init` - Initialize coordination topology
- `mcp__claude-flow__agent_spawn` - Define agent types for coordination
- `mcp__claude-flow__task_orchestrate` - Orchestrate high-level workflows

### 📁 File Organization Rules

**NEVER save to root folder. Use these directories:**
- `/src` - Source code files
- `/tests` - Test files
- `/docs` - Documentation and markdown files
- `/config` - Configuration files
- `/scripts` - Utility scripts
- `/examples` - Example code

## Project Overview

This project uses SPARC (Specification, Pseudocode, Architecture, Refinement, Completion) methodology with Claude-Flow orchestration for systematic Test-Driven Development.

## SPARC Commands

### Core Commands
- `npx claude-flow sparc modes` - List available modes
- `npx claude-flow sparc run <mode> "<task>"` - Execute specific mode
- `npx claude-flow sparc tdd "<feature>"` - Run complete TDD workflow
- `npx claude-flow sparc info <mode>` - Get mode details

### Batchtools Commands
- `npx claude-flow sparc batch <modes> "<task>"` - Parallel execution
- `npx claude-flow sparc pipeline "<task>"` - Full pipeline processing
- `npx claude-flow sparc concurrent <mode> "<tasks-file>"` - Multi-task processing

### Build Commands

**KNHK-Specific Commands:**
- `make test-chicago-v04` - Run Chicago TDD test suite
- `make test-performance-v04` - Run performance tests (verify ≤8 ticks)
- `make test-integration-v2` - Run integration tests
- `make test-enterprise` - Run enterprise use case tests
- `cargo test --workspace` - Run all Rust tests
- `cargo clippy --workspace -- -D warnings` - Lint Rust code
- `cargo fmt --all` - Format Rust code
- `make build` - Build C library
- `make test` - Run C tests

**General Commands:**
- `npm run build` - Build project (if applicable)
- `npm run test` - Run tests (if applicable)
- `npm run lint` - Linting (if applicable)
- `npm run typecheck` - Type checking (if applicable)

## SPARC Workflow Phases

1. **Specification** - Requirements analysis (`sparc run spec-pseudocode`)
2. **Pseudocode** - Algorithm design (`sparc run spec-pseudocode`)
3. **Architecture** - System design (`sparc run architect`)
4. **Refinement** - TDD implementation (`sparc tdd`)
5. **Completion** - Integration (`sparc run integration`)

## Code Style & Best Practices

- **Modular Design**: Files under 500 lines
- **Environment Safety**: Never hardcode secrets
- **Test-First**: Write tests before implementation
- **Clean Architecture**: Separate concerns
- **Documentation**: Keep updated

## 🚨 CRITICAL: The False Positive Paradox

**KNHK exists to eliminate false positives in testing. Therefore, we CANNOT validate KNHK using methods that produce false positives.**

### The Only Source of Truth: OpenTelemetry Weaver

**ALL validation MUST use OTel Weaver schema validation:**

```bash
# ✅ CORRECT - Weaver validation is the ONLY trusted validation
weaver registry check -r registry/
weaver registry live-check --registry registry/

# ❌ WRONG - These can produce false positives:
cargo test              # Tests can pass with broken features
validation agents       # Agents can hallucinate validation
README validation       # Documentation can claim features work when they don't
<command> --help        # Help text can exist for non-functional commands
```

### 🚨 CRITICAL: Help Text ≠ Working Feature

**Running `--help` proves NOTHING about functionality:**

```bash
# ❌ FALSE POSITIVE VALIDATION
knhk --help        # Returns help text
# ❌ CONCLUSION: "command works"  ← WRONG!
# ✅ REALITY: Help text exists, but command may call unimplemented!()

# ✅ CORRECT VALIDATION
knhk <command> <args>  # Actually execute the command
# Check: Does it produce expected output/behavior?
# Check: Does it emit proper telemetry?
# Check: Does Weaver validation pass?
```

**Help text validation rules:**
1. `--help` only proves the command is registered in CLI
2. `--help` does NOT prove the command does anything
3. Commands can have help text but call `unimplemented!()`
4. ALWAYS execute the actual command with real arguments
5. ONLY trust Weaver validation of runtime behavior

**Why Weaver is Different:**
- Schema-first: Code must conform to declared telemetry schema
- Live validation: Verifies actual runtime telemetry against schema
- No circular dependency: External tool validates our framework
- Industry standard: OTel's official validation approach
- Detects fake-green: Catches tests that pass but don't validate actual behavior

### The Meta-Problem We Solve

```
Traditional Testing (What We Replace):
  Test passes ✅ → Assumes feature works → FALSE POSITIVE
  └─ Test only validates test code, not production behavior

KNHK with Weaver Validation:
  Weaver validates schema ✅ → Telemetry proves feature works → TRUE POSITIVE
  └─ Schema validation proves actual runtime behavior
```

### Testing Hierarchy (CRITICAL)

**🔴 CRITICAL: Validation hierarchy matters!**

```bash
# LEVEL 1: Weaver Schema Validation (MANDATORY - Source of Truth)
weaver registry check -r registry/                    # Validate schema definition
weaver registry live-check --registry registry/       # Validate runtime telemetry

# LEVEL 2: Compilation & Code Quality (Baseline)
cargo build --release                                 # Must compile
cargo clippy --workspace -- -D warnings               # Zero warnings
make build                                            # C library compiles

# LEVEL 3: Traditional Tests (Supporting Evidence - Can Have False Positives)
cargo test --workspace                                # Rust unit tests
make test-chicago-v04                                 # C Chicago TDD tests
make test-performance-v04                             # Performance tests
make test-integration-v2                              # Integration tests
```

**⚠️ Test Passes ≠ Feature Works:**
- Tests can pass even when features don't work (false positives)
- Only Weaver validation proves runtime behavior matches schema
- Traditional tests provide supporting evidence, not proof

**If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.**

## 🚀 CRITICAL: USE ADVANCED AGENTS FIRST

**ALWAYS use specialized advanced agents instead of basic agents when the task matches their expertise.**

### ⚡ Advanced Agents (PRIORITY - Use These First!)

| Agent | Use Case | When to Use |
|-------|----------|-------------|
| **`production-validator`** | Production readiness validation | Validating deployments, infrastructure, dependencies, release readiness, final certification |
| **`code-analyzer`** | Advanced code quality analysis | Deep code review, technical debt analysis, architecture assessment, instrumentation |
| **`system-architect`** | System architecture design | Designing systems, integration patterns, architectural decisions, infrastructure design |
| **`performance-benchmarker`** | Performance measurement & optimization | Benchmarking, performance analysis, bottleneck identification, profiling |
| **`backend-dev`** | Backend implementation | Docker, containers, APIs, databases, infrastructure code, OTLP setup |
| **`task-orchestrator`** | Complex workflow orchestration | Multi-phase workflows, coordination, dependency management |
| **`code-review-swarm`** | Comprehensive code reviews | Multi-agent code review, validation, quality assessment |
| **`tdd-london-swarm`** | Test-driven development | Mock-driven development, comprehensive test suites |
| **`cicd-engineer`** | CI/CD pipeline creation | GitHub Actions, workflow automation, deployment pipelines |
| **`security-manager`** | Security analysis | Security audits, vulnerability assessment, compliance checks |

### 🔴 Basic Agents (Use Only for Simple Tasks)

| Agent | Use Case | When to Use |
|-------|----------|-------------|
| `coder` | Simple implementation | ONLY when task is straightforward and doesn't require specialized expertise |
| `reviewer` | Basic code review | ONLY for simple, localized reviews |
| `tester` | Basic testing | ONLY for simple test cases |
| `planner` | Simple planning | ONLY for basic task breakdowns |
| `researcher` | Basic research | ONLY for simple information gathering |

### 🎯 Decision Matrix: Which Agent to Use?

**Task: Production Validation** → ✅ Use `production-validator` (NOT `tester`)
**Task: Code Quality Review** → ✅ Use `code-analyzer` (NOT `reviewer`)
**Task: Architecture Design** → ✅ Use `system-architect` (NOT `planner`)
**Task: Docker/OTLP Setup** → ✅ Use `backend-dev` (NOT `coder`)
**Task: Performance Analysis** → ✅ Use `performance-benchmarker` (NOT `researcher`)
**Task: Complex Workflow** → ✅ Use `task-orchestrator` (NOT `planner`)
**Task: TDD Implementation** → ✅ Use `tdd-london-swarm` (NOT `tester`)
**Task: CI/CD Pipeline** → ✅ Use `cicd-engineer` (NOT `coder`)

### ❌ Common Mistakes to Avoid

```yaml
# ❌ WRONG - Using basic agents for specialized work
Task("Research patterns", "...", "researcher")  # TOO BASIC
Task("Write code", "...", "coder")              # TOO BASIC
Task("Run tests", "...", "tester")              # TOO BASIC

# ✅ CORRECT - Using specialized agents
Task("Analyze architecture patterns", "...", "system-architect")
Task("Implement backend infrastructure", "...", "backend-dev")
Task("Validate production readiness", "...", "production-validator")
```

**Why Advanced Agents Are Better:**
- ✅ **5x more comprehensive output** (178KB vs 20KB from basic agents)
- ✅ **Domain-specific expertise** and best practices
- ✅ **Production-grade deliverables** (FAANG-level quality)
- ✅ **Automated workflows** and coordination
- ✅ **Better architecture** and design decisions

### 🚫 Agents That Don't Exist (Common Mistakes)

**These agent types do NOT exist** - use the correct alternatives:

| ❌ Wrong Agent | ✅ Correct Alternative | Why |
|---------------|----------------------|-----|
| `analyst` | `code-analyzer` or `system-architect` | Analysis requires specialized agent |
| `validator` | `production-validator` | Use full name |
| `architect` | `system-architect` | Use full name |
| `developer` | `backend-dev` or `coder` | Be specific about type |
| `engineer` | `cicd-engineer` or `backend-dev` | Be specific about domain |
| `tdd` | `tdd-london-swarm` | Use full name |
| `benchmark` | `performance-benchmarker` | Use full name |

## 🚀 Available Agents (54 Total)

### Core Development
`coder`, `reviewer`, `tester`, `planner`, `researcher`

### Swarm Coordination
`hierarchical-coordinator`, `mesh-coordinator`, `adaptive-coordinator`, `collective-intelligence-coordinator`, `swarm-memory-manager`

### Consensus & Distributed
`byzantine-coordinator`, `raft-manager`, `gossip-coordinator`, `consensus-builder`, `crdt-synchronizer`, `quorum-manager`, `security-manager`

### Performance & Optimization
`perf-analyzer`, `performance-benchmarker`, `task-orchestrator`, `memory-coordinator`, `smart-agent`

### GitHub & Repository
`github-modes`, `pr-manager`, `code-review-swarm`, `issue-tracker`, `release-manager`, `workflow-automation`, `project-board-sync`, `repo-architect`, `multi-repo-swarm`

### SPARC Methodology
`sparc-coord`, `sparc-coder`, `specification`, `pseudocode`, `architecture`, `refinement`

### Specialized Development
`backend-dev`, `mobile-dev`, `ml-developer`, `cicd-engineer`, `api-docs`, `system-architect`, `code-analyzer`, `base-template-generator`

### Testing & Validation
`tdd-london-swarm`, `production-validator`

### Migration & Planning
`migration-planner`, `swarm-init`

## 🎯 Claude Code vs MCP Tools

### Claude Code Handles ALL EXECUTION:
- **Task tool**: Spawn and run agents concurrently for actual work
- File operations (Read, Write, Edit, MultiEdit, Glob, Grep)
- Code generation and programming
- Bash commands and system operations
- Implementation work
- Project navigation and analysis
- TodoWrite and task management
- Git operations
- Package management
- Testing and debugging

### MCP Tools ONLY COORDINATE:
- Swarm initialization (topology setup)
- Agent type definitions (coordination patterns)
- Task orchestration (high-level planning)
- Memory management
- Neural features
- Performance tracking
- GitHub integration

**KEY**: MCP coordinates the strategy, Claude Code's Task tool executes with real agents.

## 🚀 Quick Setup

```bash
# Add MCP servers (Claude Flow required, others optional)
claude mcp add claude-flow npx claude-flow@alpha mcp start
claude mcp add ruv-swarm npx ruv-swarm mcp start  # Optional: Enhanced coordination
claude mcp add flow-nexus npx flow-nexus@latest mcp start  # Optional: Cloud features
```

## MCP Tool Categories

### Coordination
`swarm_init`, `agent_spawn`, `task_orchestrate`

### Monitoring
`swarm_status`, `agent_list`, `agent_metrics`, `task_status`, `task_results`

### Memory & Neural
`memory_usage`, `neural_status`, `neural_train`, `neural_patterns`

### GitHub Integration
`github_swarm`, `repo_analyze`, `pr_enhance`, `issue_triage`, `code_review`

### System
`benchmark_run`, `features_detect`, `swarm_monitor`

### Flow-Nexus MCP Tools (Optional Advanced Features)
Flow-Nexus extends MCP capabilities with 70+ cloud-based orchestration tools:

**Key MCP Tool Categories:**
- **Swarm & Agents**: `swarm_init`, `swarm_scale`, `agent_spawn`, `task_orchestrate`
- **Sandboxes**: `sandbox_create`, `sandbox_execute`, `sandbox_upload` (cloud execution)
- **Templates**: `template_list`, `template_deploy` (pre-built project templates)
- **Neural AI**: `neural_train`, `neural_patterns`, `seraphina_chat` (AI assistant)
- **GitHub**: `github_repo_analyze`, `github_pr_manage` (repository management)
- **Real-time**: `execution_stream_subscribe`, `realtime_subscribe` (live monitoring)
- **Storage**: `storage_upload`, `storage_list` (cloud file management)

**Authentication Required:**
- Register: `mcp__flow-nexus__user_register` or `npx flow-nexus@latest register`
- Login: `mcp__flow-nexus__user_login` or `npx flow-nexus@latest login`
- Access 70+ specialized MCP tools for advanced orchestration

## 🚀 Agent Execution Flow with Claude Code

### The Correct Pattern:

1. **Optional**: Use MCP tools to set up coordination topology
2. **REQUIRED**: Use Claude Code's Task tool to spawn agents that do actual work
3. **REQUIRED**: Each agent runs hooks for coordination
4. **REQUIRED**: Batch all operations in single messages

### Example Full-Stack Development:

```javascript
// Single message with all agent spawning via Claude Code's Task tool
[Parallel Agent Execution]:
  Task("Backend Developer", "Build REST API with Express. Use hooks for coordination.", "backend-dev")
  Task("Frontend Developer", "Create React UI. Coordinate with backend via memory.", "coder")
  Task("Database Architect", "Design PostgreSQL schema. Store schema in memory.", "code-analyzer")
  Task("Test Engineer", "Write Jest tests. Check memory for API contracts.", "tester")
  Task("DevOps Engineer", "Setup Docker and CI/CD. Document in memory.", "cicd-engineer")
  Task("Security Auditor", "Review authentication. Report findings via hooks.", "reviewer")
  
  // All todos batched together
  TodoWrite { todos: [...8-10 todos...] }
  
  // All file operations together
  Write "backend/server.js"
  Write "frontend/App.jsx"
  Write "database/schema.sql"
```

## 📋 Agent Coordination Protocol

### Every Agent Spawned via Task Tool MUST:

**1️⃣ BEFORE Work:**
```bash
npx claude-flow@alpha hooks pre-task --description "[task]"
npx claude-flow@alpha hooks session-restore --session-id "swarm-[id]"
```

**2️⃣ DURING Work:**
```bash
npx claude-flow@alpha hooks post-edit --file "[file]" --memory-key "swarm/[agent]/[step]"
npx claude-flow@alpha hooks notify --message "[what was done]"
```

**3️⃣ AFTER Work:**
```bash
npx claude-flow@alpha hooks post-task --task-id "[task]"
npx claude-flow@alpha hooks session-end --export-metrics true
```

## 🎯 Concurrent Execution Examples

### ✅ CORRECT WORKFLOW: MCP Coordinates, Claude Code Executes

```javascript
// Step 1: MCP tools set up coordination (optional, for complex tasks)
[Single Message - Coordination Setup]:
  mcp__claude-flow__swarm_init { topology: "mesh", maxAgents: 6 }
  mcp__claude-flow__agent_spawn { type: "researcher" }
  mcp__claude-flow__agent_spawn { type: "coder" }
  mcp__claude-flow__agent_spawn { type: "tester" }

// Step 2: Claude Code Task tool spawns ACTUAL agents that do the work
[Single Message - Parallel Agent Execution]:
  // Claude Code's Task tool spawns real agents concurrently
  Task("Research agent", "Analyze API requirements and best practices. Check memory for prior decisions.", "researcher")
  Task("Coder agent", "Implement REST endpoints with authentication. Coordinate via hooks.", "coder")
  Task("Database agent", "Design and implement database schema. Store decisions in memory.", "code-analyzer")
  Task("Tester agent", "Create comprehensive test suite with 90% coverage.", "tester")
  Task("Reviewer agent", "Review code quality and security. Document findings.", "reviewer")
  
  // Batch ALL todos in ONE call
  TodoWrite { todos: [
    {id: "1", content: "Research API patterns", status: "in_progress", priority: "high"},
    {id: "2", content: "Design database schema", status: "in_progress", priority: "high"},
    {id: "3", content: "Implement authentication", status: "pending", priority: "high"},
    {id: "4", content: "Build REST endpoints", status: "pending", priority: "high"},
    {id: "5", content: "Write unit tests", status: "pending", priority: "medium"},
    {id: "6", content: "Integration tests", status: "pending", priority: "medium"},
    {id: "7", content: "API documentation", status: "pending", priority: "low"},
    {id: "8", content: "Performance optimization", status: "pending", priority: "low"}
  ]}
  
  // Parallel file operations
  Bash "mkdir -p app/{src,tests,docs,config}"
  Write "app/package.json"
  Write "app/src/server.js"
  Write "app/tests/server.test.js"
  Write "app/docs/API.md"
```

### ❌ WRONG (Multiple Messages):
```javascript
Message 1: mcp__claude-flow__swarm_init
Message 2: Task("agent 1")
Message 3: TodoWrite { todos: [single todo] }
Message 4: Write "file.js"
// This breaks parallel coordination!
```

## Performance Benefits

- **84.8% SWE-Bench solve rate**
- **32.3% token reduction**
- **2.8-4.4x speed improvement**
- **27+ neural models**

## Hooks Integration

### Pre-Operation
- Auto-assign agents by file type
- Validate commands for safety
- Prepare resources automatically
- Optimize topology by complexity
- Cache searches

### Post-Operation
- Auto-format code
- Train neural patterns
- Update memory
- Analyze performance
- Track token usage

### Session Management
- Generate summaries
- Persist state
- Track metrics
- Restore context
- Export workflows

## Advanced Features (v2.0.0)

- 🚀 Automatic Topology Selection
- ⚡ Parallel Execution (2.8-4.4x speed)
- 🧠 Neural Training
- 📊 Bottleneck Analysis
- 🤖 Smart Auto-Spawning
- 🛡️ Self-Healing Workflows
- 💾 Cross-Session Memory
- 🔗 GitHub Integration

## Integration Tips

1. Start with basic swarm init
2. Scale agents gradually
3. Use memory for context
4. Monitor progress regularly
5. Train patterns from success
6. Enable hooks automation
7. Use GitHub tools first

## KNHK Project-Specific Guidelines

### Definition of Done (CRITICAL: Weaver Validation Required)

Before ANY code is production-ready, ALL must be true:

#### Build & Code Quality (Baseline)
- [ ] `cargo build --workspace` succeeds with zero warnings
- [ ] `cargo clippy --workspace -- -D warnings` shows zero issues
- [ ] `make build` succeeds (C library)
- [ ] No `.unwrap()` or `.expect()` in production code paths
- [ ] All traits remain `dyn` compatible (no async trait methods)
- [ ] Proper `Result<T, E>` error handling
- [ ] No `println!` in production code (use `tracing` macros)
- [ ] No fake `Ok(())` returns from incomplete implementations

#### Weaver Validation (MANDATORY - Source of Truth)
- [ ] **`weaver registry check -r registry/` passes** (schema is valid)
- [ ] **`weaver registry live-check --registry registry/` passes** (runtime telemetry conforms to schema)
- [ ] All claimed OTEL spans/metrics/logs defined in schema
- [ ] Schema documents exact telemetry behavior
- [ ] Live telemetry matches schema declarations

#### Functional Validation (MANDATORY - Must Actually Execute)
- [ ] **Command executed with REAL arguments** (not just `--help`)
- [ ] **Command produces expected output/behavior**
- [ ] **Command emits proper telemetry** (validated by Weaver)
- [ ] **End-to-end workflow tested** (not just unit tests)
- [ ] **Performance constraints met** (≤8 ticks for hot path)

#### Traditional Testing (Supporting Evidence Only)
- [ ] `cargo test --workspace` passes completely
- [ ] `make test-chicago-v04` passes
- [ ] `make test-performance-v04` passes (verifies ≤8 ticks)
- [ ] `make test-integration-v2` passes
- [ ] Tests follow AAA pattern with descriptive names

**⚠️ CRITICAL HIERARCHY:**
1. **Weaver validation** = Source of truth (proves feature works)
2. **Compilation + Clippy** = Code quality baseline (proves code is valid)
3. **Traditional tests** = Supporting evidence (can have false positives)

### Key Principles

1. **Schema-First Validation**: OTel Weaver validation is the ONLY source of truth
2. **No False Positives**: Tests can lie; telemetry schemas don't
3. **Performance Compliance**: Hot path operations ≤8 ticks (Chatman Constant)
4. **80/20 Focus**: Critical path implementations first, no placeholders
5. **Never Trust the Text**: Only trust test results and OTEL validation
6. **No Fake Implementations**: Incomplete features must call `unimplemented!()`
7. **Trait Design**: Never use async trait methods (breaks dyn compatibility)
8. **Behavior-Focused Testing**: Test what code does, not how it does it

### The Meta-Principle: Don't Trust Tests, Trust Schemas

**Problem KNHK Solves:**
```
Traditional Testing:
  assert(result == expected) ✅  ← Can pass even when feature is broken
  └─ Tests validate test logic, not production behavior

KNHK Solution:
  Schema defines behavior → Weaver validates runtime telemetry ✅
  └─ Schema validation proves actual runtime behavior matches specification
```

**Why This Matters:**
- A test can pass because it tests the wrong thing
- A test can pass because it's mocked incorrectly
- A test can pass because it doesn't test the actual feature
- **A Weaver schema validation can only pass if the actual runtime telemetry matches the declared schema**

This is why KNHK uses Weaver validation as the source of truth.

## Support

- Documentation: https://github.com/ruvnet/claude-flow
- Issues: https://github.com/ruvnet/claude-flow/issues
- Flow-Nexus Platform: https://flow-nexus.ruv.io (registration required for cloud features)

---

Remember: **Claude Flow coordinates, Claude Code creates!**

# important-instruction-reminders
Do what has been asked; nothing more, nothing less.
NEVER create files unless they're absolutely necessary for achieving your goal.
ALWAYS prefer editing an existing file to creating a new one.
NEVER proactively create documentation files (*.md) or README files. Only create documentation files if explicitly requested by the User.
Never save working files, text/mds and tests to the root folder.

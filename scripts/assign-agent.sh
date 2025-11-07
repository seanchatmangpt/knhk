#!/bin/bash
# Automatically suggest best agent for task
# Usage: ./scripts/assign-agent.sh <task-type>

set -euo pipefail

task_type="${1:-}"

if [ -z "$task_type" ]; then
  echo "Usage: $0 <task-type>"
  echo ""
  echo "Available task types:"
  echo "  compilation, code-quality    - Code quality and compilation issues"
  echo "  performance, benchmarks      - Performance optimization"
  echo "  weaver, otel, telemetry      - OpenTelemetry and schema validation"
  echo "  tests, tdd                   - Test-driven development"
  echo "  architecture, design         - System architecture design"
  echo "  security, vulnerabilities    - Security audits"
  echo "  documentation, api-docs      - Documentation writing"
  echo "  cicd, github-actions         - CI/CD pipelines"
  echo "  production, deployment       - Production readiness"
  echo "  ffi, c-integration          - FFI and C integration"
  echo "  ring-buffer                 - Ring buffer implementation"
  echo "  etl, pipeline               - ETL pipeline architecture"
  echo ""
  echo "See docs/AGENT_SELECTION_MATRIX.md for full details"
  exit 1
fi

# Normalize input (lowercase, trim)
task_type=$(echo "$task_type" | tr '[:upper:]' '[:lower:]' | xargs)

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🤖 Agent Assignment Recommendation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

case "$task_type" in
  "compilation" | "code-quality" | "clippy" | "traits")
    echo "✅ Best Agent: code-analyzer"
    echo "📋 Second Choice: backend-dev"
    echo ""
    echo "📖 Reason:"
    echo "  - Specialized in code quality analysis"
    echo "  - Expert in Clippy warnings and compilation issues"
    echo "  - Understands trait compatibility and Rust patterns"
    echo ""
    echo "❌ Avoid: production-validator, coder"
    ;;

  "performance" | "benchmarks" | "8-ticks" | "chatman")
    echo "✅ Best Agent: performance-benchmarker"
    echo "📋 Second Choice: system-architect"
    echo ""
    echo "📖 Reason:"
    echo "  - PMU expertise and performance analysis"
    echo "  - Validates Chatman Constant (≤8 ticks)"
    echo "  - Cache optimization and lockless algorithms"
    echo ""
    echo "❌ Avoid: coder, tdd-london-swarm"
    ;;

  "weaver" | "otel" | "telemetry" | "otlp")
    echo "✅ Best Agent: backend-dev"
    echo "📋 Second Choice: production-validator"
    echo ""
    echo "📖 Reason:"
    echo "  - OTLP and schema validation expert"
    echo "  - Telemetry infrastructure specialist"
    echo "  - Understands Weaver registry validation"
    echo ""
    echo "❌ Avoid: api-docs, coder"
    ;;

  "tests" | "tdd" | "chicago" | "testing")
    echo "✅ Best Agent: tdd-london-swarm"
    echo "📋 Second Choice: coder"
    echo ""
    echo "📖 Reason:"
    echo "  - TDD methodology expertise (Chicago-style)"
    echo "  - Comprehensive test design and mocking"
    echo "  - Test organization and AAA pattern"
    echo ""
    echo "❌ Avoid: performance-benchmarker, system-architect"
    ;;

  "architecture" | "design" | "system-design" | "integration")
    echo "✅ Best Agent: system-architect"
    echo "📋 Second Choice: backend-dev"
    echo ""
    echo "📖 Reason:"
    echo "  - High-level design and architecture expertise"
    echo "  - System integration patterns"
    echo "  - 8-beat orchestration knowledge"
    echo ""
    echo "❌ Avoid: coder, code-analyzer"
    ;;

  "security" | "vulnerabilities" | "audit" | "byzantine")
    echo "✅ Best Agent: security-manager"
    echo "📋 Second Choice: code-analyzer"
    echo ""
    echo "📖 Reason:"
    echo "  - Security audit and vulnerability detection"
    echo "  - Threat modeling and Byzantine fault tolerance"
    echo "  - Consensus security expertise"
    echo ""
    echo "❌ Avoid: performance-benchmarker, api-docs"
    ;;

  "documentation" | "api-docs" | "docs" | "readme")
    echo "✅ Best Agent: api-docs"
    echo "📋 Second Choice: coder"
    echo ""
    echo "📖 Reason:"
    echo "  - Technical writing expertise"
    echo "  - API documentation standards"
    echo "  - Clear communication skills"
    echo ""
    echo "❌ Avoid: backend-dev, performance-benchmarker"
    ;;

  "cicd" | "github-actions" | "ci" | "cd" | "automation")
    echo "✅ Best Agent: cicd-engineer"
    echo "📋 Second Choice: backend-dev"
    echo ""
    echo "📖 Reason:"
    echo "  - CI/CD pipeline expertise"
    echo "  - GitHub Actions workflow automation"
    echo "  - Deployment automation"
    echo ""
    echo "❌ Avoid: api-docs, code-analyzer"
    ;;

  "production" | "deployment" | "dod" | "readiness")
    echo "✅ Best Agent: production-validator"
    echo "📋 Second Choice: system-architect"
    echo ""
    echo "📖 Reason:"
    echo "  - Production readiness validation"
    echo "  - Definition of Done compliance"
    echo "  - Deployment expertise"
    echo ""
    echo "❌ Avoid: coder, tdd-london-swarm"
    ;;

  "ffi" | "c-integration" | "abi" | "cbindgen")
    echo "✅ Best Agent: backend-dev"
    echo "📋 Second Choice: code-analyzer"
    echo ""
    echo "📖 Reason:"
    echo "  - FFI safety expertise"
    echo "  - ABI compatibility knowledge"
    echo "  - C integration patterns"
    echo ""
    echo "❌ Avoid: api-docs, tdd-london-swarm"
    ;;

  "ring-buffer" | "lockless" | "hot-path")
    echo "✅ Best Agent: performance-benchmarker"
    echo "📋 Second Choice: backend-dev"
    echo ""
    echo "📖 Reason:"
    echo "  - Lockless algorithm expertise"
    echo "  - Cache optimization knowledge"
    echo "  - Ring buffer performance tuning"
    echo ""
    echo "❌ Avoid: api-docs, tdd-london-swarm"
    ;;

  "etl" | "pipeline" | "8-beat" | "data-flow")
    echo "✅ Best Agent: system-architect"
    echo "📋 Second Choice: backend-dev"
    echo ""
    echo "📖 Reason:"
    echo "  - Data flow architecture expertise"
    echo "  - 8-beat orchestration knowledge"
    echo "  - Pipeline design patterns"
    echo ""
    echo "❌ Avoid: api-docs, performance-benchmarker"
    ;;

  *)
    echo "❌ Unknown task type: $task_type"
    echo ""
    echo "Available task types:"
    echo "  compilation, performance, weaver, tests, architecture"
    echo "  security, documentation, cicd, production, ffi"
    echo "  ring-buffer, etl"
    echo ""
    echo "See docs/AGENT_SELECTION_MATRIX.md for full list"
    exit 1
    ;;
esac

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💡 Next Steps:"
echo "  1. Use recommended agent in Task() call"
echo "  2. Check AGENT_SELECTION_MATRIX.md for detailed guidance"
echo "  3. Avoid anti-patterns listed above"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

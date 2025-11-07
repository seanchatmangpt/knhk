# KNHK v1.0 CI/CD & Release Automation Plan

**Document Version**: 1.0
**Target Release**: KNHK v1.0
**Last Updated**: 2025-01-06
**Owner**: CI/CD Engineer (Agent #8)
**Status**: Production Ready

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Current State Assessment](#current-state-assessment)
3. [Release Pipeline Design](#release-pipeline-design)
4. [GitHub Actions Automation](#github-actions-automation)
5. [Release Checklist](#release-checklist)
6. [Deployment Strategy](#deployment-strategy)
7. [Rollback Procedures](#rollback-procedures)
8. [Appendices](#appendices)

---

## 1. Executive Summary

### Purpose
This document defines the automated CI/CD pipeline and release process for KNHK v1.0, establishing repeatable, reliable deployment automation that enforces the project's quality gates.

### Key Principles
- **Weaver Validation First**: Schema validation is the source of truth
- **Zero Manual Steps**: Complete automation from commit to release
- **Quality Gates**: Multi-level validation (Weaver → Build → Tests)
- **Fail Fast**: Stop pipeline on first critical failure
- **Rollback Ready**: Automated rollback on production issues

### Release Readiness Status
- ✅ **GitHub Actions Configured**: `ci.yml` implements 3-level validation
- ✅ **Validation Scripts Ready**: Production-ready validation automation
- ✅ **Makefile Targets**: Comprehensive build and test targets
- ⚠️ **Release Workflow**: Needs v1.0-specific configuration
- ⚠️ **Artifact Publishing**: Package registry configuration needed

---

## 2. Current State Assessment

### 2.1 Existing CI/CD Infrastructure

#### GitHub Actions Workflows

**`.github/workflows/ci.yml`** (269 lines)
- ✅ 3-level validation hierarchy (Weaver → Build → Tests)
- ✅ Matrix builds (ubuntu-latest, macos-latest)
- ✅ Dependency caching
- ✅ Security audit integration
- ✅ PR comment automation
- Status: **Production Ready**

**`.github/workflows/v0.4.0-release.yml`** (118 lines)
- ✅ Release validation
- ✅ Artifact upload
- ✅ GitHub release creation
- ⚠️ Version-specific (needs update for v1.0)
- Status: **Needs Adaptation**

**`.github/workflows/mdbook.yml`**
- ✅ Documentation deployment
- Status: **Production Ready**

#### Build System

**C Library** (`c/Makefile`)
- ✅ Static library build (`make lib`)
- ✅ 30+ test targets
- ✅ Architecture-specific optimizations (ARM64, x86_64)
- ✅ Comprehensive test coverage
- Status: **Production Ready**

**Rust Workspace**
- ✅ 10+ crates with independent builds
- ✅ Feature flags for conditional compilation
- ✅ Cargo workspace structure
- Status: **Production Ready**

#### Validation Scripts

**`scripts/validate-production-ready.sh`** (201 lines)
- ✅ 3-level validation (Weaver → Build → Tests)
- ✅ Unsafe code detection
- ✅ Comprehensive reporting
- Status: **Production Ready**

**`scripts/release_checklist.sh`** (269 lines)
- ✅ Interactive release validation
- ✅ Report generation
- Status: **Production Ready**

**`scripts/verify-weaver.sh`** (102 lines)
- ✅ Weaver installation verification
- ✅ Health check automation
- Status: **Production Ready**

**`scripts/run-all-tests.sh`**
- ✅ Comprehensive test orchestration
- Status: **Production Ready**

### 2.2 Gaps Analysis

| Area | Current State | Required for v1.0 | Gap |
|------|---------------|-------------------|-----|
| **Weaver Validation** | Schema check implemented | Live-check during CI | Runtime validation needed |
| **Build Automation** | Per-crate builds | Workspace build | Missing workspace-level coordination |
| **Artifact Publishing** | Local builds only | Package registry | No registry integration |
| **Version Management** | Manual version bumps | Automated versioning | No version automation |
| **Release Notes** | Manual generation | Automated changelog | No changelog automation |
| **Docker Images** | No container builds | Container registry | Missing containerization |
| **Binary Distribution** | No pre-built binaries | GitHub releases | No binary artifacts |

---

## 3. Release Pipeline Design

### 3.1 Pipeline Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      KNHK v1.0 Release Pipeline              │
└─────────────────────────────────────────────────────────────┘

┌────────────┐   ┌────────────┐   ┌────────────┐   ┌────────────┐
│   Commit   │──▶│  PR Check  │──▶│  Release   │──▶│  Deploy    │
│   /Push    │   │  (CI.yml)  │   │  Pipeline  │   │  Pipeline  │
└────────────┘   └────────────┘   └────────────┘   └────────────┘
                        │                 │                 │
                        ▼                 ▼                 ▼
                 ┌──────────────┐  ┌──────────────┐  ┌──────────────┐
                 │ Weaver Check │  │ Build Assets │  │   Staging    │
                 │ Build & Lint │  │ Run Tests    │  │  Validation  │
                 │ Unit Tests   │  │ Tag Version  │  │  Production  │
                 └──────────────┘  └──────────────┘  └──────────────┘
```

### 3.2 Quality Gates

#### Gate 1: Weaver Registry Validation (MANDATORY)
```yaml
Status: BLOCKING
Validation: weaver registry check -r registry/
Success Criteria: 0 schema violations
On Failure: STOP - Block PR/Release
```

#### Gate 2: Build & Code Quality (BASELINE)
```yaml
Status: BLOCKING
Validations:
  - cargo build --release (all crates)
  - cargo clippy -- -D warnings (0 warnings)
  - cargo fmt --check (formatted)
  - make build (C library)
  - No .unwrap() / .expect() in production code
Success Criteria: All checks pass
On Failure: STOP - Block PR/Release
```

#### Gate 3: Test Suites (SUPPORTING EVIDENCE)
```yaml
Status: BLOCKING
Validations:
  - cargo test --workspace (100% pass)
  - make test-chicago-v04 (Chicago TDD tests)
  - make test-performance-v04 (≤8 ticks hot path)
  - make test-integration-v2 (Integration tests)
Success Criteria: All tests pass
On Failure: STOP - Block PR/Release
```

#### Gate 4: Security Audit (ADVISORY)
```yaml
Status: NON-BLOCKING
Validations:
  - cargo audit (known vulnerabilities)
  - cargo deny (license compliance)
Success Criteria: No critical vulnerabilities
On Failure: WARN - Log and continue
```

### 3.3 Pipeline Stages

#### Stage 1: Pre-Release Validation (Automated)
```bash
# Triggered on: Push to main/develop, PRs
# Timeout: 30 minutes

1. Checkout code
2. Setup environment (Rust, dependencies, Weaver)
3. Cache dependencies
4. GATE 1: Weaver registry check
5. GATE 2: Build & lint (parallel matrix)
6. GATE 3: Test suites (parallel matrix)
7. GATE 4: Security audit
8. Generate PR comment
```

#### Stage 2: Release Build (Automated)
```bash
# Triggered on: Tag push (v1.0.*)
# Timeout: 45 minutes

1. Run all Stage 1 validations
2. Build release artifacts:
   - C library (libknhk.a)
   - Rust binaries (knhk CLI)
   - Documentation (mdBook)
3. Package artifacts:
   - Tarball (knhk-v1.0.0-{os}-{arch}.tar.gz)
   - Checksums (SHA256)
4. Generate changelog
5. Create GitHub release
6. Upload artifacts
```

#### Stage 3: Deployment (Semi-Automated)
```bash
# Triggered on: Manual approval after release
# Timeout: 60 minutes

1. Deploy to staging
2. Run smoke tests
3. Weaver live-check validation
4. Performance benchmarks
5. Manual QA approval
6. Deploy to production (blue-green)
7. Health checks
8. Monitor for 24h
```

---

## 4. GitHub Actions Automation

### 4.1 Enhanced CI Workflow (`ci.yml`)

**Current Status**: Production Ready
**Enhancements Needed**: Runtime Weaver validation

```yaml
# ENHANCEMENT: Add Weaver live-check to CI
jobs:
  weaver-runtime-validation:
    name: Weaver Live-Check (Runtime)
    runs-on: ubuntu-latest
    needs: build-and-lint
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Weaver
        run: |
          curl -sSL https://github.com/open-telemetry/weaver/releases/download/v0.9.0/weaver-linux-x86_64 -o weaver
          chmod +x weaver
          sudo mv weaver /usr/local/bin/

      - name: Start KNHK services (background)
        run: |
          # Start KNHK sidecar or CLI with OTEL export
          ./target/release/knhk boot --config test-config.toml &
          KNHK_PID=$!
          echo "KNHK_PID=$KNHK_PID" >> $GITHUB_ENV
          sleep 5

      - name: Run Weaver live-check
        run: |
          weaver registry live-check \
            --registry registry/ \
            --otlp-grpc-port 4317 \
            --admin-port 18080 \
            --inactivity-timeout 30 \
            --format json \
            > weaver-live-check.json

          # Check for violations
          VIOLATIONS=$(jq '.violations | length' weaver-live-check.json)
          if [ "$VIOLATIONS" -gt 0 ]; then
            echo "❌ Weaver live-check found $VIOLATIONS violations"
            jq '.violations' weaver-live-check.json
            exit 1
          fi
          echo "✅ Weaver live-check passed (0 violations)"

      - name: Stop KNHK services
        if: always()
        run: kill $KNHK_PID || true

      - name: Upload live-check report
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: weaver-live-check-report
          path: weaver-live-check.json
```

### 4.2 New Release Workflow (`release-v1.yml`)

**File**: `.github/workflows/release-v1.yml`

```yaml
name: v1.0 Release Pipeline

on:
  push:
    tags:
      - 'v1.0.*'
  workflow_dispatch:
    inputs:
      version:
        description: 'Version to release (e.g., 1.0.0)'
        required: true
      pre_release:
        description: 'Mark as pre-release'
        type: boolean
        default: false

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  # ============================================
  # Stage 1: Pre-Release Validation
  # ============================================
  validate:
    name: Pre-Release Validation
    uses: ./.github/workflows/ci.yml
    # Reuse existing CI workflow for validation

  # ============================================
  # Stage 2: Build Release Artifacts
  # ============================================
  build-artifacts:
    name: Build Release Artifacts
    needs: validate
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            archive: tar.gz
          - os: macos-latest
            target: x86_64-apple-darwin
            archive: tar.gz
          - os: macos-latest
            target: aarch64-apple-darwin
            archive: tar.gz

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install dependencies
        run: |
          if [ "$RUNNER_OS" == "macOS" ]; then
            brew install raptor2 || true
          else
            sudo apt-get update
            sudo apt-get install -y libraptor2-dev pkg-config || true
          fi

      - name: Build C library
        working-directory: c
        run: make lib

      - name: Build Rust workspace
        run: cargo build --release --target ${{ matrix.target }} --workspace

      - name: Create release archive
        run: |
          VERSION="${{ github.ref_name }}"
          ARCHIVE_NAME="knhk-${VERSION}-${{ matrix.target }}"

          mkdir -p "$ARCHIVE_NAME"/{bin,lib,include,docs}

          # Copy binaries
          cp target/${{ matrix.target }}/release/knhk "$ARCHIVE_NAME/bin/"

          # Copy C library
          cp c/libknhk.a "$ARCHIVE_NAME/lib/"
          cp c/include/*.h "$ARCHIVE_NAME/include/"

          # Copy documentation
          cp README.md LICENSE "$ARCHIVE_NAME/docs/"

          # Create tarball
          tar czf "${ARCHIVE_NAME}.tar.gz" "$ARCHIVE_NAME"

          # Generate checksum
          sha256sum "${ARCHIVE_NAME}.tar.gz" > "${ARCHIVE_NAME}.tar.gz.sha256"

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: knhk-${{ matrix.target }}
          path: |
            knhk-*.tar.gz
            knhk-*.tar.gz.sha256

  # ============================================
  # Stage 3: Create GitHub Release
  # ============================================
  create-release:
    name: Create GitHub Release
    needs: build-artifacts
    runs-on: ubuntu-latest

    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Download all artifacts
        uses: actions/download-artifact@v3

      - name: Generate release notes
        id: release_notes
        run: |
          VERSION="${{ github.ref_name }}"
          cat > release_notes.md <<EOF
          # KNHK ${VERSION} Release

          ## Overview
          KNHK (Kinetically Non-Halting Kernel) is a zero-copy, 8-tick bounded data transformation framework with integrated observability.

          ## Key Features
          - ✅ Hot path operations ≤8 ticks (1-6 ticks typical)
          - ✅ Warm path operations ≤500ms
          - ✅ Zero-copy data handling
          - ✅ OpenTelemetry integration
          - ✅ ETL pipeline with Kafka/Salesforce connectors
          - ✅ Guard-enforced validation
          - ✅ CLI tool with 25 commands

          ## Components
          - **C Library**: \`libknhk.a\` (hot path core)
          - **Rust Workspace**: 10+ crates
          - **CLI Tool**: \`knhk\` binary
          - **Documentation**: mdBook documentation

          ## Installation

          ### From Binary (Recommended)
          \`\`\`bash
          # Download for your platform
          curl -LO https://github.com/yourusername/knhk/releases/download/${VERSION}/knhk-${VERSION}-x86_64-unknown-linux-gnu.tar.gz

          # Verify checksum
          sha256sum -c knhk-${VERSION}-x86_64-unknown-linux-gnu.tar.gz.sha256

          # Extract and install
          tar xzf knhk-${VERSION}-x86_64-unknown-linux-gnu.tar.gz
          sudo cp knhk-${VERSION}-x86_64-unknown-linux-gnu/bin/knhk /usr/local/bin/
          sudo cp knhk-${VERSION}-x86_64-unknown-linux-gnu/lib/libknhk.a /usr/local/lib/
          sudo cp knhk-${VERSION}-x86_64-unknown-linux-gnu/include/*.h /usr/local/include/
          \`\`\`

          ### From Source
          \`\`\`bash
          git clone https://github.com/yourusername/knhk.git
          cd knhk
          git checkout ${VERSION}

          # Build C library
          cd c && make lib

          # Build Rust workspace
          cd .. && cargo build --release --workspace
          \`\`\`

          ## Verification

          ### Validate Installation
          \`\`\`bash
          knhk --version
          # Expected: knhk ${VERSION}

          # Verify Weaver schema
          weaver registry check -r registry/
          \`\`\`

          ### Run Tests
          \`\`\`bash
          # Rust tests
          cargo test --workspace

          # C tests
          cd c && make test-chicago-v04

          # Performance validation
          make test-performance-v04
          \`\`\`

          ## Deployment

          ### Sidecar Pattern (Recommended)
          \`\`\`yaml
          # docker-compose.yml
          services:
            knhk-sidecar:
              image: knhk:${VERSION}
              volumes:
                - ./config.toml:/etc/knhk/config.toml
              environment:
                OTEL_EXPORTER_OTLP_ENDPOINT: http://otel-collector:4317
          \`\`\`

          ## Breaking Changes
          None (first major release)

          ## Known Limitations
          - CONSTRUCT8 operates in warm path (≤500ms) due to SIMD requirements
          - Configuration management in progress (v0.5.0)
          - Windows builds not yet available

          ## Support
          - Documentation: https://yourusername.github.io/knhk
          - Issues: https://github.com/yourusername/knhk/issues
          - Discussions: https://github.com/yourusername/knhk/discussions

          ## Credits
          Built with Claude Code using SPARC methodology and Chicago TDD.

          ---

          ## Checksums
          \`\`\`
          $(cat knhk-*/knhk-*.tar.gz.sha256)
          \`\`\`
          EOF

          cat release_notes.md

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          name: KNHK ${{ github.ref_name }}
          body_path: release_notes.md
          draft: false
          prerelease: ${{ github.event.inputs.pre_release == 'true' }}
          files: |
            knhk-*/knhk-*.tar.gz
            knhk-*/knhk-*.tar.gz.sha256
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### 4.3 Automated Version Bumping

**Script**: `scripts/bump-version.sh`

```bash
#!/bin/bash
# Automated version bumping for KNHK
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version> (e.g., 1.0.0)"
    exit 1
fi

echo "Bumping version to $VERSION..."

# Update Rust crate versions
for crate in rust/knhk-*/Cargo.toml; do
    sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" "$crate"
    rm "${crate}.bak"
done

# Update C library version
sed -i.bak "s/KNHK_VERSION \".*\"/KNHK_VERSION \"$VERSION\"/" c/include/knhk.h
rm c/include/knhk.h.bak

# Update documentation
sed -i.bak "s/version = \".*\"/version = \"$VERSION\"/" book/book.toml
rm book/book.toml.bak

echo "✅ Version bumped to $VERSION"
echo "Next steps:"
echo "  1. Review changes: git diff"
echo "  2. Commit: git commit -am 'chore: bump version to $VERSION'"
echo "  3. Tag: git tag -a v$VERSION -m 'Release v$VERSION'"
echo "  4. Push: git push origin main --tags"
```

---

## 5. Release Checklist

### 5.1 Pre-Release Checklist

**Timeline**: T-7 days before release

```markdown
## Code Freeze (T-7 days)
- [ ] All features merged to main
- [ ] All PRs reviewed and approved
- [ ] No open P0/P1 bugs
- [ ] Release branch created (release/v1.0)

## Validation (T-5 days)
- [ ] All CI checks passing
- [ ] Weaver registry check passes
- [ ] Weaver live-check passes (0 violations)
- [ ] All test suites pass (100%)
- [ ] Performance benchmarks meet SLO (≤8 ticks)
- [ ] Security audit complete (no critical vulns)
- [ ] Dependency audit complete

## Documentation (T-3 days)
- [ ] README updated
- [ ] CHANGELOG generated
- [ ] API documentation current
- [ ] Migration guide written (if breaking changes)
- [ ] Installation guide updated
- [ ] Release notes drafted

## Build & Package (T-2 days)
- [ ] Version bumped (Cargo.toml, headers)
- [ ] Build artifacts generated (all platforms)
- [ ] Checksums generated (SHA256)
- [ ] Archives created (.tar.gz)
- [ ] Documentation built (mdBook)

## Testing (T-1 day)
- [ ] Staging deployment successful
- [ ] Smoke tests pass
- [ ] End-to-end workflows tested
- [ ] Performance validation on staging
- [ ] Integration tests with downstream systems

## Release Day (T-0)
- [ ] Tag version (v1.0.0)
- [ ] Trigger release workflow
- [ ] Monitor CI/CD pipeline
- [ ] GitHub release created
- [ ] Artifacts uploaded
- [ ] Release notes published
```

### 5.2 Release Day Checklist

**Automated Steps** (via GitHub Actions):
1. ✅ Tag pushed (triggers release workflow)
2. ✅ Pre-release validation runs
3. ✅ Artifacts built (Linux, macOS)
4. ✅ Checksums generated
5. ✅ GitHub release created
6. ✅ Assets uploaded

**Manual Steps**:
1. **Verify Release**
   ```bash
   # Download and verify artifacts
   curl -LO https://github.com/yourusername/knhk/releases/download/v1.0.0/knhk-v1.0.0-x86_64-unknown-linux-gnu.tar.gz
   sha256sum -c knhk-v1.0.0-x86_64-unknown-linux-gnu.tar.gz.sha256
   ```

2. **Announce Release**
   - Post to project discussions
   - Update documentation site
   - Notify downstream projects

3. **Monitor for 24h**
   - Watch for issue reports
   - Monitor telemetry/metrics
   - Track adoption

### 5.3 Post-Release Checklist

```markdown
## Immediate (T+0 to T+24h)
- [ ] Release announcement published
- [ ] Documentation site updated
- [ ] Package registries updated (if applicable)
- [ ] Social media announcement
- [ ] Monitor issues/discussions
- [ ] Track download metrics

## Short-term (T+1 to T+7 days)
- [ ] Collect user feedback
- [ ] Triage bug reports
- [ ] Plan patch releases (if needed)
- [ ] Update roadmap

## Long-term (T+1 month)
- [ ] Release retrospective
- [ ] Process improvements documented
- [ ] Lessons learned captured
- [ ] Next release planning
```

---

## 6. Deployment Strategy

### 6.1 Deployment Topology

KNHK supports **sidecar pattern** deployment:

```
┌─────────────────────────────────────────────────────────────┐
│                      Production Deployment                   │
└─────────────────────────────────────────────────────────────┘

┌────────────────┐        ┌────────────────┐        ┌────────────────┐
│  Application   │◀──────▶│  KNHK Sidecar  │◀──────▶│ OTEL Collector │
│   Container    │  IPC   │   Container    │  gRPC  │   Container    │
└────────────────┘        └────────────────┘        └────────────────┘
                                   │
                                   ▼
                          ┌────────────────┐
                          │  Observability │
                          │   Backend      │
                          │ (Jaeger/Prom)  │
                          └────────────────┘
```

### 6.2 Deployment Requirements

#### Infrastructure Requirements
- **OS**: Linux (Ubuntu 20.04+, RHEL 8+), macOS 11+
- **Architecture**: x86_64, ARM64 (Apple Silicon)
- **Memory**: 512MB minimum (2GB recommended)
- **CPU**: 2 cores minimum (4 cores recommended)
- **Storage**: 100MB for binaries, 10GB for data

#### Software Dependencies
- **Required**:
  - libraptor2 (RDF parsing)
  - OpenSSL 1.1+
  - glibc 2.31+ (Linux)
- **Optional**:
  - Docker/Podman (containerized deployment)
  - Kubernetes 1.20+ (orchestration)
  - OTEL Collector (telemetry)

#### Network Requirements
- **Inbound**: None (sidecar listens on localhost only)
- **Outbound**:
  - OTEL Collector: 4317 (gRPC), 4318 (HTTP)
  - Kafka: 9092 (configurable)
  - Salesforce: 443 (HTTPS)

### 6.3 Configuration Management

#### Configuration File
**Location**: `/etc/knhk/config.toml` or `~/.knhk/config.toml`

```toml
[system]
log_level = "info"
telemetry_enabled = true

[otel]
endpoint = "http://localhost:4317"
service_name = "knhk-sidecar"
trace_enabled = true
metrics_enabled = true

[connectors.kafka]
brokers = ["localhost:9092"]
topic = "knhk-events"
security_protocol = "SASL_SSL"

[connectors.salesforce]
instance_url = "https://yourinstance.salesforce.com"
api_version = "v57.0"

[performance]
hot_path_budget_ticks = 8
warm_path_budget_ms = 500
batch_size = 1000
```

#### Environment Variables
```bash
# Override configuration via environment
export KNHK_LOG_LEVEL=debug
export KNHK_OTEL_ENDPOINT=http://otel-collector:4317
export KNHK_KAFKA_BROKERS=kafka1:9092,kafka2:9092
```

### 6.4 Deployment Methods

#### Method 1: Docker Compose (Recommended)

**File**: `docker-compose.yml`

```yaml
version: '3.8'

services:
  knhk-sidecar:
    image: knhk:v1.0.0
    container_name: knhk-sidecar
    restart: unless-stopped

    volumes:
      - ./config.toml:/etc/knhk/config.toml:ro
      - knhk-data:/var/lib/knhk

    environment:
      - KNHK_LOG_LEVEL=info
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317

    networks:
      - knhk-net

    depends_on:
      - otel-collector

    healthcheck:
      test: ["CMD", "knhk", "health"]
      interval: 30s
      timeout: 10s
      retries: 3

  otel-collector:
    image: otel/opentelemetry-collector:latest
    container_name: otel-collector
    restart: unless-stopped

    command: ["--config=/etc/otel-collector-config.yaml"]

    volumes:
      - ./otel-collector-config.yaml:/etc/otel-collector-config.yaml:ro

    ports:
      - "4317:4317"  # OTLP gRPC
      - "4318:4318"  # OTLP HTTP

    networks:
      - knhk-net

volumes:
  knhk-data:

networks:
  knhk-net:
    driver: bridge
```

**Deployment**:
```bash
docker-compose up -d
docker-compose logs -f knhk-sidecar
```

#### Method 2: Kubernetes

**File**: `k8s/knhk-deployment.yaml`

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: knhk-sidecar
  namespace: knhk
spec:
  replicas: 3
  selector:
    matchLabels:
      app: knhk-sidecar
  template:
    metadata:
      labels:
        app: knhk-sidecar
    spec:
      containers:
      - name: knhk
        image: knhk:v1.0.0
        ports:
        - containerPort: 8080

        env:
        - name: KNHK_LOG_LEVEL
          value: "info"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://otel-collector:4317"

        volumeMounts:
        - name: config
          mountPath: /etc/knhk
          readOnly: true

        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"

        livenessProbe:
          exec:
            command:
            - knhk
            - health
          initialDelaySeconds: 30
          periodSeconds: 30

        readinessProbe:
          exec:
            command:
            - knhk
            - health
          initialDelaySeconds: 10
          periodSeconds: 10

      volumes:
      - name: config
        configMap:
          name: knhk-config
```

**Deployment**:
```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml
kubectl apply -f k8s/knhk-deployment.yaml
kubectl rollout status deployment/knhk-sidecar -n knhk
```

#### Method 3: Systemd Service

**File**: `/etc/systemd/system/knhk.service`

```ini
[Unit]
Description=KNHK Sidecar Service
After=network.target

[Service]
Type=simple
User=knhk
Group=knhk
WorkingDirectory=/opt/knhk
ExecStart=/usr/local/bin/knhk boot --config /etc/knhk/config.toml
Restart=on-failure
RestartSec=10

Environment=KNHK_LOG_LEVEL=info
Environment=OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

StandardOutput=journal
StandardError=journal
SyslogIdentifier=knhk

[Install]
WantedBy=multi-user.target
```

**Deployment**:
```bash
sudo systemctl daemon-reload
sudo systemctl enable knhk
sudo systemctl start knhk
sudo systemctl status knhk
```

### 6.5 Deployment Validation

**Smoke Tests** (run after deployment):

```bash
#!/bin/bash
# smoke-test.sh
set -e

echo "=== KNHK Deployment Smoke Tests ==="

# Test 1: CLI responds
echo "Test 1: CLI availability"
knhk --version || exit 1

# Test 2: Health check
echo "Test 2: Health check"
knhk health || exit 1

# Test 3: Configuration loads
echo "Test 3: Configuration"
knhk config validate || exit 1

# Test 4: Weaver schema valid
echo "Test 4: Weaver validation"
weaver registry check -r registry/ || exit 1

# Test 5: Simple query
echo "Test 5: Hot path query"
echo "SELECT * WHERE { ?s ?p ?o } LIMIT 1" | knhk query || exit 1

# Test 6: Telemetry export
echo "Test 6: Telemetry"
curl -f http://localhost:4317/health || exit 1

echo "✅ All smoke tests passed"
```

**Monitoring Checks**:
```bash
# CPU usage < 50%
top -bn1 | grep knhk | awk '{print $9}'

# Memory usage < 1GB
ps -o rss= -p $(pgrep knhk) | awk '{print $1/1024 " MB"}'

# Log errors (should be 0)
journalctl -u knhk --since "5 minutes ago" | grep -i error | wc -l

# Telemetry spans exported
curl -s http://localhost:18080/metrics | grep spans_exported
```

---

## 7. Rollback Procedures

### 7.1 Rollback Triggers

**Automatic Rollback** (when detected):
- Health check failures (3 consecutive)
- CPU usage > 80% for 5 minutes
- Memory usage > 90%
- Error rate > 5%
- Zero telemetry export for 2 minutes

**Manual Rollback** (on observation):
- Production incidents
- Data corruption
- Security vulnerability
- Breaking changes affecting downstream

### 7.2 Rollback Process

#### Docker Compose Rollback

```bash
# Step 1: Stop current version
docker-compose down

# Step 2: Restore previous version
docker tag knhk:v1.0.0 knhk:v1.0.0-backup
docker tag knhk:v0.4.0 knhk:v1.0.0

# Step 3: Start previous version
docker-compose up -d

# Step 4: Verify
docker-compose logs -f knhk-sidecar
./smoke-test.sh
```

#### Kubernetes Rollback

```bash
# Automatic rollback (preferred)
kubectl rollout undo deployment/knhk-sidecar -n knhk

# Manual rollback to specific revision
kubectl rollout history deployment/knhk-sidecar -n knhk
kubectl rollout undo deployment/knhk-sidecar --to-revision=2 -n knhk

# Verify rollback
kubectl rollout status deployment/knhk-sidecar -n knhk
kubectl get pods -n knhk -l app=knhk-sidecar
```

#### Systemd Rollback

```bash
# Step 1: Stop service
sudo systemctl stop knhk

# Step 2: Restore previous binary
sudo cp /usr/local/bin/knhk /usr/local/bin/knhk.v1.0.0.backup
sudo cp /usr/local/bin/knhk.v0.4.0 /usr/local/bin/knhk

# Step 3: Restart service
sudo systemctl start knhk

# Step 4: Verify
sudo systemctl status knhk
./smoke-test.sh
```

### 7.3 Rollback Validation

**Post-Rollback Checklist**:
```markdown
- [ ] Service started successfully
- [ ] Health checks passing
- [ ] CPU/memory usage normal
- [ ] Error rate < 1%
- [ ] Telemetry exporting
- [ ] Downstream systems functional
- [ ] Performance within SLO
- [ ] No data loss
```

### 7.4 Incident Response

**Severity Levels**:

**P0 (Critical)** - Immediate rollback
- Production outage
- Data corruption
- Security breach
- Response: Rollback within 5 minutes

**P1 (High)** - Planned rollback
- Performance degradation > 50%
- Error rate > 5%
- Feature regression
- Response: Rollback within 30 minutes

**P2 (Medium)** - Evaluate rollback
- Non-critical bugs
- Minor performance issues
- Response: Fix forward or rollback within 4 hours

**P3 (Low)** - No rollback
- Cosmetic issues
- Documentation errors
- Response: Fix in next release

---

## 8. Appendices

### Appendix A: Release Timeline (90-Day Rollout)

```
D0: v1.0 Release
├─ Day 0: Tag and release
├─ Day 1-7: Monitor adoption
├─ Day 7: First weekly retrospective
└─ Day 30: Monthly retrospective

D0-30: Early Adopter Phase
├─ Limited rollout to pilot users
├─ Daily monitoring
├─ Hotfix releases if needed (v1.0.1, v1.0.2)
└─ Feedback collection

D30-60: Staged Rollout
├─ Expand to 25% of users
├─ Performance validation
├─ Integration testing
└─ Weekly retrospectives

D60-90: General Availability
├─ Full production rollout
├─ Monitor for regressions
├─ Plan v1.1 features
└─ Final retrospective
```

### Appendix B: Automation Scripts

#### Pre-Commit Hook

**File**: `.git/hooks/pre-commit`

```bash
#!/bin/bash
# KNHK pre-commit hook
set -e

echo "Running pre-commit validation..."

# Run format check
cargo fmt --all -- --check

# Run clippy
cargo clippy --all -- -D warnings

# Run Weaver check
weaver registry check -r registry/

echo "✅ Pre-commit checks passed"
```

#### Release Script

**File**: `scripts/release.sh`

```bash
#!/bin/bash
# Complete release automation
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

echo "=== KNHK v${VERSION} Release ==="

# Step 1: Validate
./scripts/validate-production-ready.sh

# Step 2: Bump version
./scripts/bump-version.sh "$VERSION"

# Step 3: Build
cargo build --release --workspace
cd c && make lib && cd ..

# Step 4: Test
cargo test --workspace
cd c && make test-chicago-v04 && cd ..

# Step 5: Tag
git add -A
git commit -m "chore: release v${VERSION}"
git tag -a "v${VERSION}" -m "Release v${VERSION}"

echo "✅ Release prepared"
echo "Next: git push origin main --tags"
```

### Appendix C: Metrics & Monitoring

**Key Performance Indicators (KPIs)**:

| Metric | Target | Alert Threshold |
|--------|--------|----------------|
| Hot path latency (p95) | ≤8 ticks | >10 ticks |
| Warm path latency (p95) | ≤500ms | >1000ms |
| Memory usage | <1GB | >1.5GB |
| CPU usage | <50% | >80% |
| Error rate | <0.1% | >1% |
| Span export rate | >1000/sec | <100/sec |
| Build success rate | 100% | <95% |
| Test pass rate | 100% | <99% |

**Monitoring Dashboards**:
- CI/CD Pipeline Status (GitHub Actions)
- Deployment Health (Grafana)
- Performance Metrics (Prometheus)
- Error Tracking (Sentry/Jaeger)

### Appendix D: Contact & Escalation

**Release Team**:
- **Release Manager**: Coordinates overall release
- **CI/CD Engineer**: Pipeline automation and deployment
- **QA Lead**: Test validation and sign-off
- **Security Lead**: Security audit and approval

**Escalation Path**:
1. CI/CD Engineer → Release Manager → VP Engineering
2. Security issues → Security Lead → CISO

---

## Document History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-06 | CI/CD Engineer (Agent #8) | Initial release plan |

---

**Next Steps**:
1. Review and approve this document
2. Create `.github/workflows/release-v1.yml`
3. Test release pipeline on staging
4. Schedule v1.0 release
5. Execute release and monitor

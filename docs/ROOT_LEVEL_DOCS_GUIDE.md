# Root-Level Documentation Guide

This document explains the organization of documentation files in the repository root and how to navigate them.

## 📍 Root-Level Essential Documents

These files are kept at the repository root for critical project information:

### **Essential (Core)**
Keep at `/home/user/knhk/`:

| File | Purpose | When to Read |
|------|---------|-------------|
| **`README.md`** | Project overview, features, quick start | First thing you read |
| **`REPOSITORY_OVERVIEW.md`** | Complete system features and capabilities | Understanding full scope |
| **`CLAUDE.md`** | Project configuration and methodology | Claude Code users, SPARC methodology |

### **Reference (High Priority)**
Keep at `/home/user/knhk/`:

| File | Purpose | When to Read |
|------|---------|-------------|
| **`BREAKING_POINTS.md`** | Known limitations and breaking points | Before production deployment |
| **`FALSE_POSITIVES_REPORT.md`** | False positives in testing framework | Understanding test reliability |
| **`PROCESS_MINING_INSIGHTS.md`** | Process mining and workflow analysis | Understanding workflow behavior |

### **Advanced Topics (Methodology)**
Keep at `/home/user/knhk/`:

| File | Purpose | When to Read |
|------|---------|-------------|
| **`VAN_DER_AALST_CLI_COMMANDS.md`** | CLI commands for Van der Aalst validation | Workflow validation |
| **`aa-code-spec-alignment.md`** | Code-specification alignment (DMEDI) | Implementation phase |
| **`aa-dmedi-define.md`** | DMEDI Define phase documentation | Project planning |
| **`aa-dmedi-explore.md`** | DMEDI Explore phase documentation | Requirements analysis |
| **`aa-dmedi-measure.md`** | DMEDI Measure phase documentation | Performance measurement |

## 📚 Related Documentation Location

For comprehensive documentation, refer to [`docs/SITE_MAP.md`](/home/user/knhk/docs/SITE_MAP.md) which provides:

- Navigation by role (Developers, Architects, Researchers, etc.)
- Navigation by topic (Workflow, Performance, Testing, etc.)
- Complete directory structure overview
- Finding documentation by keyword

## 🗂️ File Organization Strategy

### Current Structure
```
/home/user/knhk/
├── README.md                         ⭐ START HERE
├── REPOSITORY_OVERVIEW.md            System overview
├── CLAUDE.md                         Project config
├── BREAKING_POINTS.md                Known limitations
├── FALSE_POSITIVES_REPORT.md         Testing framework limitations
├── PROCESS_MINING_INSIGHTS.md        Workflow analysis
├── VAN_DER_AALST_CLI_COMMANDS.md     Workflow validation CLI
├── aa-code-spec-alignment.md         DMEDI alignment
├── aa-dmedi-define.md                DMEDI Define phase
├── aa-dmedi-explore.md               DMEDI Explore phase
├── aa-dmedi-measure.md               DMEDI Measure phase
│
├── docs/                             PRIMARY DOCUMENTATION (192MB)
│   ├── SITE_MAP.md                   ⭐ DOCUMENTATION HUB (this directory)
│   ├── QUICK_START.md                5-minute setup
│   ├── ARCHITECTURE.md               System architecture
│   ├── WORKFLOW_ENGINE.md            Workflow reference
│   ├── [90+ other documentation]
│   └── [26 subdirectories]
│
├── book/                             PUBLISHABLE DOCUMENTATION
└── [other directories]
```

## 🎯 Navigation Guide

### If you want to understand the project:
1. Start: **`README.md`**
2. Then: **`docs/SITE_MAP.md`** (choose your role)
3. Then: Role-specific documentation in `docs/` folder

### If you want to implement something:
1. Read: **`docs/SITE_MAP.md`** → Developer path
2. Check: **`docs/API.md`** and **`docs/INTEGRATION.md`**
3. Review: **`examples/`** directory
4. Refer: Project-specific README in **`rust/`**, **`c/`**, or **`erlang/`**

### If you're doing research:
1. Read: **`docs/SITE_MAP.md`** → Researcher path
2. Study: **`docs/papers/chatman-equation/`**
3. Deep dive: **`docs/formal-foundations.md`**

### If you're deploying to production:
1. Review: **`BREAKING_POINTS.md`**
2. Read: **`docs/PRODUCTION.md`**
3. Check: **`DEFINITION_OF_DONE_V1_FORTUNE5.md`**
4. Validate: **`docs/evidence/`** for validation results

---

## 🔄 Adding New Documentation

When creating new documentation:

1. **Ask**: Is this essential root-level information (project config, overview, methodology)?
   - **YES** → Keep at `/home/user/knhk/*.md` (but keep count under control)
   - **NO** → Place in appropriate `docs/` subdirectory

2. **Determine scope**:
   - **Quick Start/Getting Started** → `docs/QUICK_START.md` or `docs/getting-started/`
   - **Technical Reference** → Topic-specific in `docs/` (e.g., `docs/API.md`)
   - **Architecture Decisions** → `docs/architecture/ADR/`
   - **Research/Papers** → `docs/papers/`
   - **Examples/Code** → `/examples/` directory
   - **Test Results/Evidence** → `docs/evidence/`
   - **Operations/Deployment** → `docs/runbooks/` or `docs/deployment.md`

3. **Update navigation**:
   - Add entry to **`docs/SITE_MAP.md`** under appropriate sections
   - Update **`docs/DOCUMENTATION_ORGANIZATION.md`** if organization changes
   - Update main **`README.md`** if it's a critical discovery path

---

## 📋 Documentation Maintenance Checklist

Periodically review documentation:

- [ ] **Root-level files** count is ≤ 15 (keep essential only)
- [ ] **`docs/SITE_MAP.md`** is current and links work
- [ ] **New documentation** is findable from site map
- [ ] **Broken links** are fixed
- [ ] **Outdated information** is updated or archived
- [ ] **Duplicate content** is consolidated
- [ ] **Project-specific READMEs** exist in `/rust/*/`, `/c/`, `/erlang/`

---

## 🔗 Quick Reference

| Need | Location | File |
|------|----------|------|
| Getting started | Root | `README.md` |
| Full system overview | Root | `REPOSITORY_OVERVIEW.md` |
| Find any documentation | Docs | `docs/SITE_MAP.md` |
| Quick setup (5 min) | Docs | `docs/QUICK_START.md` |
| How something works | Docs | See site map by topic |
| Testing/Validation evidence | Docs | `docs/evidence/` |
| Production readiness | Docs | `docs/PRODUCTION.md` |
| Research papers | Docs | `docs/papers/chatman-equation/` |
| Code examples | Root | `/examples/` |
| Build & deploy | Docs | `docs/deployment.md` |

---

**Last Updated**: 2025-11-15
**Related**: [`docs/SITE_MAP.md`](docs/SITE_MAP.md)

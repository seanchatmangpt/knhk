# Documentation Maintenance Guide

A comprehensive guide for maintaining KNHK's extensive documentation system. This document is for documentation maintainers and contributors who want to keep the documentation organized, discoverable, and up-to-date.

## 🎯 Purpose

KNHK has 150+ markdown files, 12+ research papers, 200+ diagrams, and 18+ schema definitions. This guide ensures documentation remains:

- **Organized** - Logical directory structure
- **Discoverable** - Easy to find what you need
- **Current** - Accurate and up-to-date
- **Linked** - Cross-references work correctly
- **Complete** - Nothing is orphaned or missing

## 📊 Documentation Statistics

Current state as of 2025-11-15:

| Category | Count | Location |
|----------|-------|----------|
| Markdown files | 150+ | `/docs/` + root + others |
| Root-level files | 11 | `/home/user/knhk/` |
| Subdirectories in `/docs/` | 26 | Organized by topic |
| Archive directories | 18 | `/docs/archived/` |
| Research papers | 12+ | `/docs/papers/chatman-equation/` |
| Diagrams | 200+ | `/docs/papers/mermaid/` |
| Schema files | 18 | `/registry/` + `/ontology/` |
| Examples | 5 | `/examples/` |
| Rust crates | 20 | `/rust/` |

## 🗂️ Directory Organization

### Primary Documentation Hub
**Location**: `/home/user/knhk/docs/`

```
docs/
├── SITE_MAP.md                        # ⭐ CENTRAL NAVIGATION HUB
├── ROOT_LEVEL_DOCS_GUIDE.md          # Explains root-level files
├── PROJECTS_DOCUMENTATION_INDEX.md   # All project documentation
├── DOCUMENTATION_MAINTENANCE_GUIDE.md # This file
│
├── [Core Guides - Direct children of /docs/]
├── QUICK_START.md                    # 5-minute setup
├── ARCHITECTURE.md                   # System architecture
├── WORKFLOW_ENGINE.md                # Workflow reference
├── API.md                            # API documentation
├── CLI.md                            # CLI reference
├── TESTING.md                        # Testing guide
├── PRODUCTION.md                     # Production readiness
├── PERFORMANCE.md                    # Performance guide
├── INTEGRATION.md                    # Integration patterns
│
├── [Topic-Specific Subdirectories - 26 total]
├── architecture/                     # 45 ADRs and arch docs
├── papers/                           # Research papers and whitepapers
├── evidence/                         # Validation test results
├── v1/                               # v1 release documentation
├── archived/                         # Historical documentation
├── schemas/                          # Schema documentation hub
├── telemetry/                        # OTEL telemetry docs
├── performance/                      # Performance analysis
├── testing/                          # Testing methodologies
├── diagrams/                         # Architecture diagrams
├── examples/                         # Code examples
│
├── [and 16 more subdirectories]
└── [See SITE_MAP.md for complete listing]
```

### Root-Level Documentation
**Location**: `/home/user/knhk/`

**Essential (Must Keep)**:
- `README.md` - Project overview
- `REPOSITORY_OVERVIEW.md` - System features
- `CLAUDE.md` - Project configuration

**High Priority (Keep)**:
- `BREAKING_POINTS.md` - Limitations
- `FALSE_POSITIVES_REPORT.md` - Testing framework notes
- `PROCESS_MINING_INSIGHTS.md` - Analysis insights
- `VAN_DER_AALST_CLI_COMMANDS.md` - Validation CLI

**Methodology (Keep)**:
- `aa-dmedi-*.md` (4 files) - DMEDI methodology

## 📝 Maintenance Responsibilities

### Daily/Weekly Tasks

1. **Check Link Validity** (Weekly)
   ```bash
   # Find broken links in markdown
   markdown-link-check docs/**/*.md
   # Or manually spot-check when updating docs
   ```

2. **Update When Code Changes** (Daily)
   - If code changes, update related documentation
   - Sync API documentation with implementation
   - Update examples if behavior changes

3. **Monitor for Drift** (Weekly)
   - Check if documentation matches implementation
   - Look for TODO or FIXME in docs that should be resolved
   - Verify all referenced files still exist

### Monthly Tasks

1. **Review Structure** (Monthly)
   - Are new topics organized in correct places?
   - Do any directories have too many files (>20)?
   - Are old archived docs still relevant?

2. **Check SITE_MAP.md** (Monthly)
   - All major documents listed?
   - Links still valid?
   - Navigation still clear?

3. **Archive Old Content** (Monthly)
   - Move outdated docs to `archived/`
   - Update archive organization
   - Document why content was archived

4. **Update Statistics**
   - Update file counts in guides
   - Document major organizational changes
   - Update "Last Updated" dates

### Quarterly Tasks

1. **Major Review** (Quarterly)
   - Read through SITE_MAP.md for completeness
   - Check if major features have documentation
   - Validate that examples still work
   - Update the documentation maintenance guide

2. **Clean Up Archives** (Quarterly)
   - Review `docs/archived/` (18 subdirectories!)
   - Consider consolidating similar archived docs
   - Document retention policy
   - Move very old docs to long-term archive

3. **Validate Schemas** (Quarterly)
   ```bash
   weaver registry check -r registry/
   rapper -c ontology/*.ttl
   ```

4. **Test Examples** (Quarterly)
   - Run all 5 examples to ensure they still work
   - Update example documentation if needed
   - Update output in example READMEs

## 🔄 Adding New Documentation

### Step-by-Step Process

#### 1. Determine Documentation Type

| Type | Location | When |
|------|----------|------|
| **Quick Start** | `docs/QUICK_START.md` | New feature users need immediately |
| **Core Reference** | `docs/*.md` (root of docs) | Essential knowledge (API, CLI, etc.) |
| **Architecture** | `docs/architecture/` | Design decisions and system structure |
| **Guide** | `docs/[topic]/` | How-to and tutorial content |
| **Research** | `docs/papers/` | Academic papers and whitepapers |
| **Examples** | `/examples/` | Reference implementations |
| **Test Results** | `docs/evidence/` | Validation and test results |
| **Schema** | `docs/schemas/` or `/registry/` | Schema definitions |
| **Product** | `docs/product-planning/` | Product requirements |
| **Deployment** | `docs/deployment.md` or `docs/runbooks/` | Operations guides |
| **Archive** | `docs/archived/` | Deprecated or historical content |

#### 2. Choose Appropriate Subdirectory

```
✅ CORRECT:
/docs/INTEGRATION.md             → Core integration guide
/docs/architecture/ADR-123.md    → Architecture decision
/docs/evidence/validation-report.txt → Test results
/docs/papers/research-paper.md   → Research

❌ WRONG:
/docs/integration.md             → Would clutter root
/home/user/knhk/INTEGRATION.md   → Clutter root level
/docs/my-guide.md                → No clear category
```

#### 3. Create the Documentation File

**Template for New Guide**:
```markdown
# [Topic Title]

**Purpose**: Brief explanation of what this document covers

**Audience**: Who should read this (developers, architects, etc.)

**Quick Links**:
- Related: [Link to related docs]
- See Also: [Link to related content]

---

## Overview
[Introduction to topic]

## Key Concepts
[Important concepts]

## How To...
[Step-by-step instructions]

## Examples
[Code/configuration examples]

## Related Documentation
- [Related link]
- [Related link]

---

**Last Updated**: [Date]
**Related**: [`docs/SITE_MAP.md`](/docs/SITE_MAP.md)
```

#### 4. Update Navigation Documents

Update these files to include your new documentation:

1. **`docs/SITE_MAP.md`**
   - Add to appropriate section
   - Update statistics if applicable
   - Ensure link path is correct

2. **`docs/PROJECTS_DOCUMENTATION_INDEX.md`**
   - Add if project-specific documentation
   - Link to relevant projects

3. **Project-specific sections**:
   - If architecture documentation, update `docs/architecture/`
   - If schema documentation, update `docs/schemas/README.md`
   - If deployment documentation, update `docs/deployment.md`

#### 5. Validate Links

Ensure all links work:
```bash
# Manual check - try opening in editor
# Or use markdown link checker (if available)
```

#### 6. Commit Documentation

```bash
git add docs/[your-file].md
git add docs/SITE_MAP.md    # Updated navigation
git commit -m "docs: add [description]"
```

## 🔗 Documentation Links Best Practices

### Link Formats

**Absolute paths** (preferred for reliability):
```markdown
[SITE_MAP](/home/user/knhk/docs/SITE_MAP.md)
[Architecture](/home/user/knhk/docs/ARCHITECTURE.md)
```

**Relative paths** (works in current context):
```markdown
[SITE_MAP](SITE_MAP.md)           # From /docs/
[API](../API.md)                  # From subdirectory
```

**Anchor links**:
```markdown
[Jump to section](#key-concepts)
```

### When to Update Links

✅ **DO update links when:**
- File is moved or renamed
- Major documentation structure changes
- Testing links and finding broken ones

❌ **DON'T update links just for:**
- Minor formatting changes
- Adding new cross-references (use SITE_MAP instead)
- Stylistic changes

## 🏷️ Tagging & Categorization

### Metadata Headers

Include at top of documentation:

```markdown
# [Topic]

**Purpose**: Clear explanation of document purpose
**Audience**: Who should read this
**Difficulty**: Beginner/Intermediate/Advanced (if applicable)
**Related**: Links to related documentation
**Updated**: YYYY-MM-DD
```

### Using Metadata Wisely

```markdown
# Workflow Engine Integration

**Purpose**: Guide to integrating KNHK workflow engine into your application
**Audience**: Developers implementing workflow functionality
**Difficulty**: Intermediate
**Prerequisites**:
- Basic understanding of KNHK architecture
- Knowledge of async/await in Rust
**Related**:
- [WORKFLOW_ENGINE.md](WORKFLOW_ENGINE.md)
- [INTEGRATION.md](INTEGRATION.md)
- [API.md](API.md)
```

## 📋 Checklist for New Documentation

Before finalizing new documentation:

- [ ] **Content**
  - [ ] Purpose is clear in introduction
  - [ ] Audience is identified
  - [ ] Information is accurate
  - [ ] Examples are tested and work
  - [ ] Step-by-step instructions are complete

- [ ] **Organization**
  - [ ] File is in correct subdirectory
  - [ ] Filename describes content clearly
  - [ ] Headings are hierarchical and logical
  - [ ] Sections have clear purpose

- [ ] **Navigation**
  - [ ] SITE_MAP.md is updated
  - [ ] Related docs are linked
  - [ ] All internal links are correct
  - [ ] Metadata headers are complete

- [ ] **Quality**
  - [ ] Grammar and spelling checked
  - [ ] Markdown formatting is correct
  - [ ] Code examples are properly formatted
  - [ ] No broken links

- [ ] **Maintenance**
  - [ ] "Last Updated" date is current
  - [ ] No TODOs left in content
  - [ ] Related documents list is current

## 🗑️ Archiving Old Documentation

### When to Archive

- Documentation for deprecated features
- Historical information no longer relevant
- Old release notes (keep current, archive old)
- Superseded approaches or patterns

### How to Archive

1. **Move to `docs/archived/`**
   ```bash
   mv docs/old-guide.md docs/archived/old-guide-YYYY-MM-DD.md
   ```

2. **Create forwarding notice** (if still linked):
   ```markdown
   # [OLD] [Topic]

   **⚠️ ARCHIVED**: This documentation is archived and no longer maintained.
   See [Current Alternative](link-to-current.md) for up-to-date information.
   ```

3. **Update links to point to archive**
   - Change SITE_MAP.md to point to archived version
   - Update any direct links to old documentation
   - Document why it was archived

### Archive Organization

Current: `/docs/archived/` (18 subdirectories - needs reorganization!)

**Recommended new structure**:
```
archived/
├── 2024-q4/           # Organized by date
├── 2025-q1/
├── legacy/            # Pre-2024 items
├── deprecated/        # Explicitly deprecated features
└── ARCHIVE_README.md  # Retention policy and index
```

## 🔍 Finding Documentation Issues

### Common Issues to Look For

1. **Broken Links**
   - Run markdown link checker
   - Manually test critical paths
   - Check relative paths work from all locations

2. **Outdated Information**
   - Dates are current
   - Code examples match implementation
   - Feature descriptions are accurate
   - Version numbers match latest release

3. **Orphaned Files**
   - Files not linked from SITE_MAP
   - Subdirectories with no clear purpose
   - README files missing from major directories

4. **Duplication**
   - Same content in multiple places
   - Overlapping topics
   - Inconsistent information

### How to Report Issues

1. Search for file in SITE_MAP.md
2. Check if it should exist
3. If broken: file an issue or PR to fix
4. If duplicate: consolidate into single location
5. If orphaned: either promote or archive

## 🚀 Documentation Evolution

### Phases of Documentation

**Phase 1: Creation**
- New feature gets documented
- Added to appropriate subdirectory
- Linked from SITE_MAP.md
- One maintainer assigned

**Phase 2: Stabilization**
- Accuracy verified
- Examples tested
- Links validated
- Community feedback integrated

**Phase 3: Maintenance**
- Kept current with code changes
- Updated as feature evolves
- Reviewed quarterly
- Well-established location

**Phase 4: Archival**
- Feature deprecated
- Documentation frozen
- Moved to archived/
- Kept for historical reference

## 📊 Documentation Metrics to Track

### What to Monitor

| Metric | Target | Check Frequency |
|--------|--------|-----------------|
| **Broken Links** | 0 | Monthly |
| **Outdated Docs** | < 5 % | Quarterly |
| **Orphaned Files** | 0 | Monthly |
| **SITE_MAP Coverage** | 100% | Weekly |
| **Examples Working** | 100% | Quarterly |
| **Schema Validation** | Passing | Every release |
| **Last Updated Dates** | Current | Monthly |

## 🎓 Documentation Best Practices

### Writing Guidelines

**DO:**
- Use clear, active language
- Break content into logical sections
- Include examples with explanations
- Link to related documentation
- Keep sections focused on one topic
- Use consistent formatting

**DON'T:**
- Create multi-topic documents
- Write in passive voice
- Skip examples or make them too abstract
- Assume reader knowledge
- Let documents get too long (>1000 lines → split)
- Forget to update when code changes

### Structure Guidelines

**Good Structure**:
```markdown
# Title
## Quick Summary
## Prerequisites
## Step-by-Step Guide
## Examples
## Troubleshooting
## Related Documentation
```

**Bad Structure**:
```markdown
# Everything About X
[500 lines of mixed content]
```

### Example Guidelines

**Good Examples**:
- Complete, runnable code
- Show expected output
- Include error cases
- Reference related examples
- Are tested and working

**Bad Examples**:
- Incomplete code (requires modification to work)
- No expected output shown
- No error handling
- Syntax errors or outdated APIs

## 🔐 Maintenance Automation

### Potential Tools

| Tool | Purpose | Command |
|------|---------|---------|
| **markdown-link-check** | Validate links | `mlc docs/**/*.md` |
| **markdown-linter** | Check formatting | `markdownlint docs/**/*.md` |
| **prettier** | Format markdown | `prettier --write docs/**/*.md` |
| **weaver** | Validate schemas | `weaver registry check -r registry/` |
| **rapper** | Validate RDF | `rapper -c ontology/*.ttl` |

## 🔄 Version Control & Documentation

### Commit Messages for Documentation

```bash
# Feature documentation
git commit -m "docs: add [feature] documentation"

# Updates to existing docs
git commit -m "docs: update [topic] for clarity"

# Fixing broken docs
git commit -m "docs: fix broken links and examples"

# Major reorganization
git commit -m "docs: reorganize [category] structure"

# Archiving old docs
git commit -m "docs: archive [topic] as deprecated"
```

### Keep Documentation in Sync with Code

- PR that adds feature → PR that documents feature
- PR that fixes bug → PR that updates doc if needed
- PR that refactors → Update architecture docs
- PR that deprecates → Archive related docs

## 📚 Quick Reference

### Common Tasks

**Add new documentation:**
1. Create file in appropriate location
2. Update SITE_MAP.md
3. Update related parent docs
4. Commit with descriptive message

**Update existing documentation:**
1. Find file (use SITE_MAP.md)
2. Update content
3. Update "Last Updated" date
4. Validate links
5. Commit

**Archive documentation:**
1. Move to `docs/archived/[category]/`
2. Add archived notice at top
3. Update or remove SITE_MAP link
4. Commit with explanation

**Fix documentation issues:**
1. Identify issue type (link, content, structure)
2. Locate file using SITE_MAP.md
3. Make correction
4. Update "Last Updated" date
5. Validate fix
6. Commit

---

## 📞 Getting Help

### Documentation Maintenance Team

For questions about:
- **Organization**: See SITE_MAP.md
- **Project docs**: See PROJECTS_DOCUMENTATION_INDEX.md
- **Schemas**: See docs/schemas/README.md
- **Archived content**: See docs/archived/
- **Root-level files**: See docs/ROOT_LEVEL_DOCS_GUIDE.md

### Common Questions

**Q: Where should I put my new documentation?**
A: See "Adding New Documentation" section above, or check SITE_MAP.md for examples of similar documentation.

**Q: How do I find documentation on a topic?**
A: Start with SITE_MAP.md, search by role or topic, or use Ctrl+F to search this guide.

**Q: What if documentation seems outdated?**
A: Check the "Last Updated" date. If it's more than 3 months old and relates to active features, consider updating or archiving it.

**Q: How do I report documentation issues?**
A: Use GitHub issues or submit a PR with corrections.

---

**Last Updated**: 2025-11-15
**Related**: [`docs/SITE_MAP.md`](/docs/SITE_MAP.md), [`ROOT_LEVEL_DOCS_GUIDE.md`](/docs/ROOT_LEVEL_DOCS_GUIDE.md)

---

## 📋 Documentation Maintenance Checklist Template

Use this checklist monthly to maintain documentation quality:

```markdown
# Documentation Maintenance - [Month/Year]

## Validation
- [ ] All links in SITE_MAP.md checked
- [ ] No orphaned documentation files
- [ ] Schema validation passes (weaver, rapper)
- [ ] Examples still work

## Updates
- [ ] Documentation matches current implementation
- [ ] "Last Updated" dates are current
- [ ] Archive outdated content as needed
- [ ] Statistics updated in guides

## Organization
- [ ] Directory structure still logical
- [ ] No directory exceeds 20 files
- [ ] Archive is properly organized
- [ ] Naming conventions consistent

## Coverage
- [ ] New features documented
- [ ] Deprecated features archived
- [ ] Examples are complete
- [ ] All projects listed in index

## Completion
- [ ] All items above completed
- [ ] No outstanding issues
- [ ] SITE_MAP.md is current
- [ ] Last maintenance date recorded
```

---

**Remember**: Good documentation is an ongoing commitment. Regular maintenance ensures KNHK remains accessible and understandable for all users.

# Diátaxis Migration Documentation

**Migration Date**: 2025-11-15
**Framework**: [Diátaxis](https://diataxis.fr/)

## What Changed

The `docs/papers/` directory has been reorganized using the Diátaxis documentation framework, which divides documentation into four distinct categories based on user needs.

## Migration Summary

### Before (Traditional Structure)
```
papers/
├── README.md
├── CLEANUP_PLAN.md
├── the_chatman_equation_fortune5_v1.2.0.pdf
├── kgc-manifestation-fortune5.pdf
├── kgc-manifestation-fortune5.tex
├── chatman-equation/
│   ├── *.tex files
│   └── mermaid/
└── mermaid/
```

### After (Diátaxis Structure)
```
papers/
├── README.md                      # Navigation hub with Diátaxis explanation
├── CLEANUP_PLAN.md                # Historical record
├── DIATAXIS_MIGRATION.md          # This file
│
├── tutorials/                     # Learning-oriented
│   └── README.md                  # Coming soon: step-by-step guides
│
├── how-to-guides/                 # Task-oriented
│   └── README.md                  # Coming soon: problem-solving guides
│
├── reference/                     # Information-oriented
│   ├── README.md                  # Technical specifications index
│   ├── the_chatman_equation_fortune5_v1.2.0.pdf
│   ├── kgc-manifestation-fortune5.pdf
│   ├── kgc-manifestation-fortune5.tex
│   ├── chatman-equation/          # LaTeX sources
│   └── mermaid/                   # Diagrams
│
└── explanation/                   # Understanding-oriented
    ├── README.md                  # Conceptual documentation index
    ├── the_chatman_equation_fortune5.md → ../../the_chatman_equation_fortune5.md
    ├── kgs_whitepaper_v2_0_sean_chatman.md → ../../kgs_whitepaper_v2_0_sean_chatman.md
    ├── formal-foundations.md → ../../formal-foundations.md
    └── spr_kgs_gaps_filled.md → ../../spr_kgs_gaps_filled.md
```

## The Four Categories

### 1. Tutorials (Learning-oriented)
**Location**: `tutorials/`
**Status**: Coming soon
**Purpose**: Help users learn through hands-on, guided practice

**Planned Content**:
- Your first KNHK workflow
- Understanding telemetry
- Chicago TDD basics
- Building production-ready features

### 2. How-to Guides (Task-oriented)
**Location**: `how-to-guides/`
**Status**: Coming soon
**Purpose**: Show how to solve specific problems

**Planned Content**:
- How to set up development environment
- How to run tests efficiently
- How to debug failing tests
- How to create OTel schemas
- How to fix Weaver validation errors
- How to optimize performance

### 3. Reference (Information-oriented)
**Location**: `reference/`
**Status**: ✅ Complete
**Purpose**: Provide accurate technical specifications

**Current Content**:
- ✅ Chatman Equation paper (PDF, v1.2.0)
- ✅ KGC Manifestation paper (PDF)
- ✅ LaTeX sources (chatman-equation/)
- ✅ Diagrams (mermaid/, 90+ files)
- ✅ Comprehensive README with build instructions

### 4. Explanation (Understanding-oriented)
**Location**: `explanation/`
**Status**: ✅ Complete
**Purpose**: Help users understand concepts and rationale

**Current Content**:
- ✅ Chatman Equation conceptual overview (markdown)
- ✅ KGS Whitepaper (conceptual explanation)
- ✅ Formal foundations (mathematical background)
- ✅ SPR KGS gaps filled (gap analysis)
- ✅ Comprehensive README with learning paths

## Migration Details

### Files Moved
- `chatman-equation/` → `reference/chatman-equation/`
- `mermaid/` → `reference/mermaid/`
- `*.pdf` files → `reference/*.pdf`
- `*.tex` files → `reference/*.tex`

### Symlinks Created
Created symlinks in `explanation/` pointing to markdown docs in parent `docs/` directory:
- `the_chatman_equation_fortune5.md`
- `kgs_whitepaper_v2_0_sean_chatman.md`
- `formal-foundations.md`
- `spr_kgs_gaps_filled.md`

### New Files Created
- `README.md` - Complete Diátaxis navigation hub
- `reference/README.md` - Reference documentation index
- `explanation/README.md` - Explanation documentation index
- `tutorials/README.md` - Tutorials placeholder
- `how-to-guides/README.md` - How-to guides placeholder
- `DIATAXIS_MIGRATION.md` - This file

### Files Preserved
- `CLEANUP_PLAN.md` - Historical record of previous cleanup (2025-11-15)

## Benefits of Diátaxis Structure

### For Users
1. **Clear Navigation**: Know exactly where to find what you need
2. **Better Discoverability**: Documentation organized by user intent
3. **Reduced Confusion**: Each category serves a specific purpose
4. **Efficient Learning**: Clear learning paths for different goals

### For Maintainers
1. **Purposeful Organization**: Each document has a clear role
2. **Gap Identification**: Easy to see what's missing
3. **Consistency**: Framework provides structure
4. **Scalability**: Easy to add new content in right place

## User Journeys

### New User Path
1. **Start**: Read [Explanation](explanation/) to understand concepts
2. **Learn**: Follow [Tutorials](tutorials/) (coming soon) for hands-on practice
3. **Build**: Use [How-to Guides](how-to-guides/) (coming soon) to solve problems
4. **Reference**: Check [Reference](reference/) for technical details

### Experienced User Path
1. **Problem**: Need to solve specific issue
2. **Solution**: Go directly to [How-to Guides](how-to-guides/) (coming soon)
3. **Details**: Check [Reference](reference/) for specifications
4. **Understanding**: Read [Explanation](explanation/) for context if needed

### Researcher Path
1. **Papers**: Access [Reference](reference/) for formal papers
2. **Theory**: Read [Explanation](explanation/) for conceptual background
3. **Citations**: Use BibTeX from [Reference README](reference/README.md)
4. **Sources**: Access LaTeX sources in [reference/chatman-equation/](reference/chatman-equation/)

## Cross-References Preserved

All cross-references have been updated to reflect the new structure:
- Main README points to all four categories
- Each category README cross-links to others
- Original markdown files remain in `/docs` (symlinked from explanation/)
- Git history preserved for all moved files

## Backwards Compatibility

### Breaking Changes
❌ Direct file paths changed:
- OLD: `docs/papers/the_chatman_equation_fortune5_v1.2.0.pdf`
- NEW: `docs/papers/reference/the_chatman_equation_fortune5_v1.2.0.pdf`

### Preserved Access
✅ Content remains accessible:
- All PDFs in `reference/` directory
- All markdown files accessible via symlinks in `explanation/`
- Original files in `/docs` unchanged
- Git history shows file moves

### Update Required
If you have bookmarks or scripts referencing old paths:
```bash
# Update paths like this:
# OLD: docs/papers/file.pdf
# NEW: docs/papers/reference/file.pdf

# Example updates:
docs/papers/the_chatman_equation_fortune5_v1.2.0.pdf
→ docs/papers/reference/the_chatman_equation_fortune5_v1.2.0.pdf

docs/papers/kgc-manifestation-fortune5.pdf
→ docs/papers/reference/kgc-manifestation-fortune5.pdf

docs/papers/chatman-equation/
→ docs/papers/reference/chatman-equation/
```

## Next Steps

### Immediate (Done)
- ✅ Create Diátaxis directory structure
- ✅ Move reference materials
- ✅ Create symlinks for explanation docs
- ✅ Write comprehensive READMEs
- ✅ Update main papers README with navigation

### Short-term (Next Sprint)
- [ ] Create "Getting Started" tutorial
- [ ] Write "How to Set Up Development" guide
- [ ] Add "How to Run Tests" guide
- [ ] Create "How to Fix Common Errors" guide

### Medium-term (Next Month)
- [ ] Complete tutorials section (5-10 tutorials)
- [ ] Complete how-to guides section (10-15 guides)
- [ ] Add code examples to tutorials
- [ ] Create video tutorials (optional)

### Long-term (Ongoing)
- [ ] Expand tutorials as features are added
- [ ] Keep how-to guides updated with best practices
- [ ] Maintain reference docs with each release
- [ ] Grow explanation docs as architecture evolves

## Validation

### Structure Check
```bash
# Verify directory structure
find docs/papers -type d | sort

# Expected output:
# docs/papers
# docs/papers/explanation
# docs/papers/how-to-guides
# docs/papers/reference
# docs/papers/reference/chatman-equation
# docs/papers/reference/chatman-equation/mermaid
# docs/papers/reference/mermaid
# docs/papers/tutorials
```

### Content Check
```bash
# Verify all PDFs present
find docs/papers/reference -name "*.pdf"

# Expected output:
# docs/papers/reference/kgc-manifestation-fortune5.pdf
# docs/papers/reference/the_chatman_equation_fortune5_v1.2.0.pdf

# Verify symlinks
ls -la docs/papers/explanation/*.md

# Expected: 4 symlinks to ../../*.md files
```

### README Check
```bash
# Verify all READMEs created
find docs/papers -name "README.md"

# Expected output:
# docs/papers/README.md
# docs/papers/explanation/README.md
# docs/papers/how-to-guides/README.md
# docs/papers/reference/README.md
# docs/papers/tutorials/README.md
```

## Resources

**Diátaxis Framework**:
- Website: https://diataxis.fr/
- Documentation: https://diataxis.fr/
- GitHub: https://github.com/evildmp/diataxis-documentation-framework

**KNHK Documentation**:
- Main README: [`/README.md`](/README.md)
- Development Guide: [`/CLAUDE.md`](/CLAUDE.md)
- Site Map: [`/docs/SITE_MAP.md`](/docs/SITE_MAP.md)

## Feedback

Found an issue with the organization? Have suggestions for tutorials or how-to guides?

**Contact**:
- Open an issue: [GitHub Issues](https://github.com/seanchatmangpt/knhk/issues)
- Submit a PR: [Pull Requests](https://github.com/seanchatmangpt/knhk/pulls)

---

**Migration By**: KNHK Documentation Team
**Framework**: Diátaxis (https://diataxis.fr/)
**Date**: 2025-11-15
**Status**: ✅ Complete (Reference & Explanation), 🔄 In Progress (Tutorials & How-to)

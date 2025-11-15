# Papers Directory Cleanup Plan

## Strategy: Keep Latest Versions Only

### /docs/papers/ (Root Level)
**Purpose**: Public-facing, final versions

**KEEP** (Latest versions):
- `the_chatman_equation_fortune5_v1.2.0.pdf` (670KB - Latest)
- `kgc-manifestation-fortune5.pdf` (315KB)
- `kgc-manifestation-fortune5.tex` (22KB)
- `mermaid/` directory (diagrams)

**REMOVE** (Old versions):
- `the_chatman_equation_fortune5_v1.0.pdf` (657KB - OLD)
- `the_chatman_equation_fortune5_v1.1.0.pdf` (656KB - OLD)
- `the_chatman_equation_fortune5.pdf` (496KB - UNVERSIONED, old)

**REMOVE** (Old TEX versions):
- `the_chatman_equation_fortune5_v1.0.tex` (14MB - OLD)
- `the_chatman_equation_fortune5_v1.1.0.tex` (30MB - OLD)
- `the_chatman_equation_fortune5.tex` (78KB - UNVERSIONED, old)
- `the_chatman_equation_fortune5_complete.tex` (103KB - OLD variant)

**REMOVE** (Build Artifacts):
- `the_chatman_equation_fortune5_v1.0.aux` (35KB)
- `the_chatman_equation_fortune5_v1.1.0.aux` (35KB)
- `the_chatman_equation_fortune5_v1.2.0.aux` (31KB)
- `the_chatman_equation_fortune5.aux` (14KB)
- `the_chatman_equation_fortune5_v1.0.out` (41KB)
- `the_chatman_equation_fortune5_v1.1.0.out` (41KB)
- `the_chatman_equation_fortune5_v1.2.0.out` (36KB)
- `the_chatman_equation_fortune5.out` (20KB)
- `kgc-manifestation-fortune5.aux` (3.7KB)
- `kgc-manifestation-fortune5.out` (4.2KB)

### /docs/papers/chatman-equation/ (Source & Working Directory)
**Purpose**: Source materials and chapter files

**KEEP** (Source chapters - essential for building):
- `00-header.tex` (5.2KB)
- `01-introduction.tex` (4.2KB)
- `02-chatman-equation.tex` (3.9KB)
- `03-knowledge-hooks.tex` (4.7KB)
- `04-43-workflow-patterns.tex` (7.0KB)
- `05-architecture.tex` (8.9KB)
- `06-zero-human-governance.tex` (4.9KB)
- `07-implementation.tex` (6.7KB)
- `10-related-work.tex` (11KB)
- `11-artifacts.tex` (4.3KB)
- `12-conclusion.tex` (5.1KB)
- `13-appendix.tex` (9.5KB)
- `mermaid/` directory (diagrams)

**REMOVE** (Keep v1.2.0 in root, not here):
- `the_chatman_equation_fortune5_v1.0.pdf` (658KB - OLD)
- `the_chatman_equation_fortune5.pdf` (516KB - UNVERSIONED, old)

**REMOVE** (Old TEX versions - v1.2.0 already in root):
- `the_chatman_equation_fortune5_v1.0.tex` (14MB - OLD)
- `the_chatman_equation_fortune5_v1.1.0.tex` (30MB - OLD)
- `the_chatman_equation_fortune5.tex` (1.3MB - UNVERSIONED, old)

**REMOVE** (Build Artifacts):
- `the_chatman_equation_fortune5_v1.0.aux` (35KB)
- `the_chatman_equation_fortune5_v1.0.out` (41KB)
- `the_chatman_equation_fortune5.aux` (15KB)
- `the_chatman_equation_fortune5.out` (20KB)
- `merged.aux` (14KB)
- `merged.out` (20KB)

## Space Savings

**Before**: ~180MB in papers directories
**After**: ~1MB (chapters + latest PDF)

**Removed**: ~179MB
- 3x large TEX files (v1.0, v1.1.0): ~74MB
- 1x large TEX v1.2.0: 44MB (needed, keep)
- Build artifacts: ~310KB
- Old PDFs: ~2MB
- Unversioned/old files: ~2MB

## Note
- v1.2.0 is the latest, timestamped Nov 14 05:34
- All files have same timestamp, so v1.2.0 in filename indicates latest
- Build artifacts (.aux, .out) are intermediate files and can always be regenerated

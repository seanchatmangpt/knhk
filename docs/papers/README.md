# Research Papers & Publications

This directory contains all KNHK research papers, whitepapers, and related materials.

## 📚 Papers Included

### Chatman Equation - Fortune 500 Optimization
**Latest Version**: v1.2.0 (2025-11-14)

- **PDF**: `the_chatman_equation_fortune5_v1.2.0.pdf` (670 KB)
  - For reading and citation
  - Published/final version

- **Source Materials**: `chatman-equation/` subdirectory
  - Source TEX file with complete content (44 MB)
  - Individual chapter files (00-header.tex through 13-appendix.tex)
  - Mermaid diagrams and visualizations (90+ diagrams)
  - Use this directory for:
    - Building/compiling the paper
    - Modifying content
    - Referencing individual sections
    - Understanding paper structure

**Related Documentation**:
- Main guide: [`/docs/the_chatman_equation_fortune5.md`](/docs/the_chatman_equation_fortune5.md) (50 KB markdown version)
- Whitepaper: [`/docs/kgs_whitepaper_v2_0_sean_chatman.md`](/docs/kgs_whitepaper_v2_0_sean_chatman.md)

### KGS Manifestation - Knowledge Graph Complexities
**Status**: Latest Version

- **PDF**: `kgc-manifestation-fortune5.pdf` (315 KB)
  - Complete published version

- **Source**: `kgc-manifestation-fortune5.tex` (22 KB)
  - LaTeX source for modifications

**Related Documentation**:
- Whitepaper: [`/docs/kgs_whitepaper_v2_0_sean_chatman.md`](/docs/kgs_whitepaper_v2_0_sean_chatman.md)

---

## 📂 Directory Structure

```
papers/
├── README.md                                    # This file
├── CLEANUP_PLAN.md                              # History of cleanup (2025-11-15)
├── the_chatman_equation_fortune5_v1.2.0.pdf    # ✅ Latest PDF (use this)
├── kgc-manifestation-fortune5.pdf              # ✅ KGC paper (final)
├── kgc-manifestation-fortune5.tex              # KGC source (optional)
│
├── chatman-equation/                            # Source materials
│   ├── the_chatman_equation_fortune5_v1.2.0.tex # ✅ Latest source (authoritative)
│   ├── 00-header.tex                           # Chapter: Header
│   ├── 01-introduction.tex                     # Chapter: Introduction
│   ├── 02-chatman-equation.tex                 # Chapter: Equation definition
│   ├── 03-knowledge-hooks.tex                  # Chapter: Knowledge hooks
│   ├── 04-43-workflow-patterns.tex             # Chapter: 43 workflow patterns
│   ├── 05-architecture.tex                     # Chapter: Architecture
│   ├── 06-zero-human-governance.tex            # Chapter: Zero human governance
│   ├── 07-implementation.tex                   # Chapter: Implementation
│   ├── 10-related-work.tex                     # Chapter: Related work
│   ├── 11-artifacts.tex                        # Chapter: Artifacts
│   ├── 12-conclusion.tex                       # Chapter: Conclusion
│   ├── 13-appendix.tex                         # Chapter: Appendix
│   │
│   └── mermaid/                                # Visualizations (90+ diagrams)
│       ├── *.mmd                               # Mermaid source files
│       ├── *.svg                               # SVG versions
│       └── *.png                               # PNG versions
│
└── mermaid/                                     # Diagram copies (for convenience)
    ├── *.mmd
    ├── *.svg
    └── *.png
```

---

## 🚀 Using These Papers

### For Reading
- Use **PDF files** in this directory
- Latest versions only (no old versions kept)

### For Citation
```bibtex
@article{Chatman2025,
  title={The Chatman Equation: Fortune 500 Optimization},
  author={Chatman, Sean},
  year={2025},
  version={1.2.0},
  url={https://github.com/seanchatmangpt/knhk/docs/papers/}
}

@article{KGC2025,
  title={KGS Manifestation: Knowledge Graph Complexities},
  author={Chatman, Sean},
  year={2025},
  url={https://github.com/seanchatmangpt/knhk/docs/papers/}
}
```

### For Modification/Compilation
1. Navigate to `chatman-equation/` subdirectory
2. Use `the_chatman_equation_fortune5_v1.2.0.tex` as the master file
3. Modify individual chapter files (00-header.tex, etc.)
4. Compile with: `pdflatex the_chatman_equation_fortune5_v1.2.0.tex`
5. Output PDF will be generated in same directory

### For Diagrams
- Mermaid diagrams available in `mermaid/` subdirectory
- Multiple formats: `.mmd` (source), `.svg` (scalable), `.png` (raster)
- Use `.svg` for web, `.png` for documents

---

## 📊 Versioning

**Latest Version**: v1.2.0 (2025-11-14)

**Version History** (archived):
- v1.2.0 - Latest release
- v1.1.0 - Previous (removed, use v1.2.0)
- v1.0 - Original (removed, use v1.2.0)

**Note**: Only latest version maintained. Old versions were removed 2025-11-15 to reduce storage (saved ~179 MB).

For historical versions, check git history:
```bash
git log --oneline -- docs/papers/
```

---

## 📈 File Statistics

| Item | Size | Count |
|------|------|-------|
| PDFs (final versions) | 985 KB | 2 |
| TEX source files | 44 MB | 1 |
| Chapter files | ~90 KB | 12 |
| Diagrams (Mermaid) | ~5 MB | 90+ |
| Total | ~49 MB | - |

**Historical Context**:
- Before cleanup: ~180 MB (3 versions of each)
- After cleanup: ~49 MB (latest only)
- Space saved: 131 MB

---

## 🔗 Related Documentation

| Document | Purpose | Location |
|----------|---------|----------|
| **Chatman Equation (Markdown)** | Readable version in markdown | [`/docs/the_chatman_equation_fortune5.md`](/docs/the_chatman_equation_fortune5.md) |
| **KGS Whitepaper** | Knowledge Graph Structures | [`/docs/kgs_whitepaper_v2_0_sean_chatman.md`](/docs/kgs_whitepaper_v2_0_sean_chatman.md) |
| **Formal Foundations** | Mathematical theory | [`/docs/formal-foundations.md`](/docs/formal-foundations.md) |
| **Papers Site Map** | All papers organized | [`/docs/SITE_MAP.md`](/docs/SITE_MAP.md) (Research section) |
| **Documentation Hub** | All KNHK documentation | [`/docs/SITE_MAP.md`](/docs/SITE_MAP.md) |

---

## 🔄 Maintenance

**Last Cleanup**: 2025-11-15
- Removed v1.0 and v1.1.0 versions
- Removed build artifacts (.aux, .out files)
- Kept only latest v1.2.0 versions
- Centralized source in chatman-equation/ subdirectory
- See [`CLEANUP_PLAN.md`](CLEANUP_PLAN.md) for details

**Next Review**: 2025-12-15

---

## ❓ Questions?

- **How do I cite these papers?** → See "Citation" section above
- **Can I modify these?** → Use files in `chatman-equation/` subdirectory
- **Where's version X?** → Only latest (v1.2.0) maintained. Old versions removed 2025-11-15
- **Need other formats?** → See markdown versions in `/docs/`
- **More papers?** → Check `/docs/papers.md` or SITE_MAP documentation

---

**Last Updated**: 2025-11-15
**Related**: [`CLEANUP_PLAN.md`](CLEANUP_PLAN.md), [`/docs/SITE_MAP.md`](/docs/SITE_MAP.md)

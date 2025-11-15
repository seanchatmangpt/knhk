# Reference Documentation

**Category**: Reference (Information-oriented)

This section contains technical specifications, formal papers, source materials, and authoritative reference materials for KNHK.

## 📚 What's Here

Reference documentation provides **accurate, detailed technical information** about KNHK. Use this when you need:
- Exact specifications and formulas
- Technical paper citations
- LaTeX source for modifications
- Diagram assets
- Formal mathematical foundations

---

## 📄 Research Papers

### The Chatman Equation: Fortune 500 Optimization
**Version**: v1.2.0 (2025-11-14)

**PDF**: [`the_chatman_equation_fortune5_v1.2.0.pdf`](the_chatman_equation_fortune5_v1.2.0.pdf) (670 KB)
- Published/final version for reading and citation
- Complete technical specification
- 90+ diagrams and visualizations

**Source Materials**: [`chatman-equation/`](chatman-equation/) subdirectory
- Master TEX file: `the_chatman_equation_fortune5_v1.2.0.tex` (44 MB)
- Individual chapters: `00-header.tex` through `13-appendix.tex`
- Mermaid diagrams: [`chatman-equation/mermaid/`](chatman-equation/mermaid/)
- Use for building, modifying, or referencing individual sections

**Citation**:
```bibtex
@article{Chatman2025Equation,
  title={The Chatman Equation: Fortune 500 Optimization},
  author={Chatman, Sean},
  year={2025},
  version={1.2.0},
  url={https://github.com/seanchatmangpt/knhk/docs/papers/reference/}
}
```

### KGS Manifestation: Knowledge Graph Complexities
**Status**: Latest Version

**PDF**: [`kgc-manifestation-fortune5.pdf`](kgc-manifestation-fortune5.pdf) (315 KB)
- Complete published version
- Technical specification for knowledge graph structures

**Source**: [`kgc-manifestation-fortune5.tex`](kgc-manifestation-fortune5.tex) (22 KB)
- LaTeX source for modifications

**Citation**:
```bibtex
@article{Chatman2025KGC,
  title={KGS Manifestation: Knowledge Graph Complexities},
  author={Chatman, Sean},
  year={2025},
  url={https://github.com/seanchatmangpt/knhk/docs/papers/reference/}
}
```

---

## 📊 Diagrams and Visualizations

### Mermaid Diagrams
**Location**: [`mermaid/`](mermaid/) directory (90+ diagrams)

**Available Formats**:
- `.mmd` - Mermaid source files (editable)
- `.svg` - Scalable Vector Graphics (web use)
- `.png` - Raster images (documents)

**Usage**:
- Use `.svg` for web pages and scalable presentations
- Use `.png` for embedding in documents
- Edit `.mmd` files to create custom variations

---

## 📂 Directory Structure

```
reference/
├── README.md                                    # This file
├── the_chatman_equation_fortune5_v1.2.0.pdf    # Latest Chatman Equation paper
├── kgc-manifestation-fortune5.pdf              # KGC paper
├── kgc-manifestation-fortune5.tex              # KGC source
│
├── chatman-equation/                            # Source materials
│   ├── the_chatman_equation_fortune5_v1.2.0.tex # Master TEX file
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
│   └── mermaid/                                # Diagrams directory
│       ├── *.mmd                               # Mermaid source files
│       ├── *.svg                               # SVG versions
│       └── *.png                               # PNG versions
│
└── mermaid/                                     # Top-level diagrams
    ├── *.mmd
    ├── *.svg
    └── *.png
```

---

## 🔧 Building from Source

### Prerequisites
```bash
# LaTeX distribution required
sudo apt-get install texlive-full  # Ubuntu/Debian
brew install --cask mactex          # macOS
```

### Compile PDF
```bash
cd chatman-equation/
pdflatex the_chatman_equation_fortune5_v1.2.0.tex
# Run twice for references
pdflatex the_chatman_equation_fortune5_v1.2.0.tex
```

### Generate Diagrams
```bash
cd chatman-equation/mermaid/
# Convert .mmd to .svg
mmdc -i diagram.mmd -o diagram.svg
# Convert .mmd to .png
mmdc -i diagram.mmd -o diagram.png
```

---

## 📈 Quick Reference

| Need | File | Location |
|------|------|----------|
| **Read the paper** | `the_chatman_equation_fortune5_v1.2.0.pdf` | This directory |
| **Cite the paper** | See BibTeX above | - |
| **Modify content** | `chatman-equation/*.tex` | Subdirectory |
| **View diagrams** | `mermaid/*.svg` or `mermaid/*.png` | Subdirectory |
| **Understand concepts** | See [Explanation](../explanation/) docs | Parent directory |
| **Learn by doing** | See [Tutorials](../tutorials/) (coming soon) | Parent directory |
| **Solve specific problems** | See [How-to Guides](../how-to-guides/) (coming soon) | Parent directory |

---

## 🔗 Related Documentation

**Other Diátaxis Categories**:
- [**Explanation**](../explanation/) - Conceptual understanding (whitepapers, markdown versions)
- [**Tutorials**](../tutorials/) - Learning-oriented guides (coming soon)
- [**How-to Guides**](../how-to-guides/) - Task-oriented solutions (coming soon)

**Cross-References**:
- Markdown version: [`explanation/the_chatman_equation_fortune5.md`](../explanation/the_chatman_equation_fortune5.md)
- KGS Whitepaper: [`explanation/kgs_whitepaper_v2_0_sean_chatman.md`](../explanation/kgs_whitepaper_v2_0_sean_chatman.md)
- Formal foundations: [`explanation/formal-foundations.md`](../explanation/formal-foundations.md)

---

## 📊 File Statistics

| Item | Size | Count |
|------|------|-------|
| PDFs (final versions) | 985 KB | 2 |
| TEX source files | 44 MB | 1 |
| Chapter files | ~90 KB | 12 |
| Diagrams (Mermaid) | ~5 MB | 90+ |
| **Total** | **~49 MB** | - |

---

**Last Updated**: 2025-11-15
**Framework**: Diátaxis (Information-oriented Reference)

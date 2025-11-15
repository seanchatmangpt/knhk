# KNHK Research & Documentation

Welcome to the KNHK documentation hub, organized using the **Diátaxis framework** for clear, purposeful documentation.

## 📚 What is Diátaxis?

[Diátaxis](https://diataxis.fr/) is a systematic approach to documentation that divides content into four categories based on user needs:

| Category | Orientation | Purpose | When to Use |
|----------|-------------|---------|-------------|
| **[Tutorials](tutorials/)** | Learning-oriented | Learn by doing | You're new and want to learn fundamentals |
| **[How-to Guides](how-to-guides/)** | Task-oriented | Solve specific problems | You know basics, need to accomplish something |
| **[Reference](reference/)** | Information-oriented | Look up technical details | You need precise specifications or citations |
| **[Explanation](explanation/)** | Understanding-oriented | Understand concepts | You want context, rationale, or big picture |

---

## 🚀 Quick Start

### New to KNHK?
1. **Start here**: [Explanation](explanation/) - Understand the concepts
   - Read [The Industrial Revolution of Knowledge](explanation/industrial-revolution-of-knowledge.md) - Why this matters
   - Read [The Chatman Equation overview](explanation/the_chatman_equation_fortune5.md)
   - Explore [KGS Whitepaper](explanation/kgs_whitepaper_v2_0_sean_chatman.md)

2. **Then**: [Tutorials](tutorials/) (coming soon) - Learn by doing
   - Your first KNHK workflow
   - Understanding telemetry

3. **Next**: [How-to Guides](how-to-guides/) (coming soon) - Build real features
   - Set up development environment
   - Run tests efficiently

### Need Technical Details?
**Go to**: [Reference](reference/) - Technical specifications
- [Chatman Equation paper (PDF)](reference/the_chatman_equation_fortune5_v1.2.0.pdf)
- [KGC Manifestation paper (PDF)](reference/kgc-manifestation-fortune5.pdf)
- [LaTeX sources](reference/chatman-equation/)
- [Diagrams](reference/mermaid/)

### Implementing Features?
**Go to**: [How-to Guides](how-to-guides/) (coming soon)
- Development workflows
- Telemetry validation
- Performance optimization

### Understanding Concepts?
**Go to**: [Explanation](explanation/)
- [Chatman Equation philosophy](explanation/the_chatman_equation_fortune5.md)
- [Knowledge graphs explained](explanation/kgs_whitepaper_v2_0_sean_chatman.md)
- [Formal foundations](explanation/formal-foundations.md)

---

## 📂 Directory Structure

```
papers/
├── README.md                      # This file (navigation hub)
│
├── tutorials/                     # Learning-oriented (coming soon)
│   ├── README.md
│   └── [Step-by-step learning guides]
│
├── how-to-guides/                 # Task-oriented (coming soon)
│   ├── README.md
│   └── [Problem-solving guides]
│
├── reference/                     # Information-oriented
│   ├── README.md
│   ├── the_chatman_equation_fortune5_v1.2.0.pdf    # Latest paper
│   ├── kgc-manifestation-fortune5.pdf              # KGC paper
│   ├── kgc-manifestation-fortune5.tex              # KGC source
│   ├── chatman-equation/                           # Source materials
│   │   ├── the_chatman_equation_fortune5_v1.2.0.tex
│   │   ├── 00-header.tex ... 13-appendix.tex       # Individual chapters
│   │   └── mermaid/                                # 90+ diagrams
│   └── mermaid/                                    # Top-level diagrams
│
└── explanation/                   # Understanding-oriented
    ├── README.md
    ├── the_chatman_equation_fortune5.md            # Conceptual overview
    ├── kgs_whitepaper_v2_0_sean_chatman.md         # KGS explained
    ├── formal-foundations.md                       # Mathematical background
    └── spr_kgs_gaps_filled.md                      # Gap analysis

Historical:
├── CLEANUP_PLAN.md               # Documentation of 2025-11-15 cleanup
```

---

## 🎯 Find What You Need

### By Goal

**"I want to learn KNHK from scratch"**
→ [Tutorials](tutorials/) (coming soon) + [Explanation](explanation/)

**"I need to solve a specific problem"**
→ [How-to Guides](how-to-guides/) (coming soon)

**"I need exact formulas or specifications"**
→ [Reference](reference/)

**"I want to understand why things work this way"**
→ [Explanation](explanation/)

### By Document Type

**Research Papers** (formal, peer-review ready):
- [Chatman Equation v1.2.0 (PDF)](reference/the_chatman_equation_fortune5_v1.2.0.pdf)
- [KGC Manifestation (PDF)](reference/kgc-manifestation-fortune5.pdf)

**Conceptual Explanations** (accessible, markdown):
- [Chatman Equation explained](explanation/the_chatman_equation_fortune5.md)
- [KGS Whitepaper](explanation/kgs_whitepaper_v2_0_sean_chatman.md)
- [Formal foundations](explanation/formal-foundations.md)

**Source Materials** (for modification):
- [LaTeX sources](reference/chatman-equation/)
- [Mermaid diagrams](reference/mermaid/)

### By Topic

**The Chatman Equation**:
- 📖 Concept: [explanation/the_chatman_equation_fortune5.md](explanation/the_chatman_equation_fortune5.md)
- 📄 Paper: [reference/the_chatman_equation_fortune5_v1.2.0.pdf](reference/the_chatman_equation_fortune5_v1.2.0.pdf)
- 🔧 Source: [reference/chatman-equation/](reference/chatman-equation/)

**Knowledge Graph Structures**:
- 📖 Concept: [explanation/kgs_whitepaper_v2_0_sean_chatman.md](explanation/kgs_whitepaper_v2_0_sean_chatman.md)
- 📄 Paper: [reference/kgc-manifestation-fortune5.pdf](reference/kgc-manifestation-fortune5.pdf)

**Mathematical Foundations**:
- 📖 Theory: [explanation/formal-foundations.md](explanation/formal-foundations.md)
- 📖 Gaps: [explanation/spr_kgs_gaps_filled.md](explanation/spr_kgs_gaps_filled.md)

**Visualizations**:
- 🎨 Diagrams: [reference/mermaid/](reference/mermaid/) (90+ diagrams in .mmd, .svg, .png)

---

## 📊 Content Overview

### Research Papers

| Paper | Version | Size | Location | Purpose |
|-------|---------|------|----------|---------|
| **Chatman Equation** | v1.2.0 | 670 KB | [PDF](reference/the_chatman_equation_fortune5_v1.2.0.pdf) | Fortune 500 optimization |
| **KGC Manifestation** | Latest | 315 KB | [PDF](reference/kgc-manifestation-fortune5.pdf) | Knowledge graph complexities |

### Source Materials

| Material | Count/Size | Location | Purpose |
|----------|------------|----------|---------|
| **LaTeX Source** | 44 MB | [chatman-equation/](reference/chatman-equation/) | Build/modify papers |
| **Chapter Files** | 12 files | [chatman-equation/*.tex](reference/chatman-equation/) | Individual sections |
| **Diagrams** | 90+ files | [mermaid/](reference/mermaid/) | Visualizations (.mmd, .svg, .png) |

### Explanatory Docs

| Document | Size | Location | Purpose |
|----------|------|----------|---------|
| **Chatman Equation (MD)** | 50 KB | [explanation/](explanation/the_chatman_equation_fortune5.md) | Conceptual overview |
| **KGS Whitepaper** | - | [explanation/](explanation/kgs_whitepaper_v2_0_sean_chatman.md) | KGS explained |
| **Formal Foundations** | - | [explanation/](explanation/formal-foundations.md) | Math background |
| **SPR KGS Gaps** | - | [explanation/](explanation/spr_kgs_gaps_filled.md) | Gap analysis |

---

## 📖 Citations

### BibTeX

**Chatman Equation**:
```bibtex
@article{Chatman2025Equation,
  title={The Chatman Equation: Fortune 500 Optimization},
  author={Chatman, Sean},
  year={2025},
  version={1.2.0},
  url={https://github.com/seanchatmangpt/knhk/docs/papers/reference/}
}
```

**KGC Manifestation**:
```bibtex
@article{Chatman2025KGC,
  title={KGS Manifestation: Knowledge Graph Complexities},
  author={Chatman, Sean},
  year={2025},
  url={https://github.com/seanchatmangpt/knhk/docs/papers/reference/}
}
```

---

## 🔗 Related Documentation

**Project Documentation**:
- [`/README.md`](/README.md) - Project overview
- [`/CLAUDE.md`](/CLAUDE.md) - Development guidelines
- [`/docs/SITE_MAP.md`](/docs/SITE_MAP.md) - Full documentation index

**External Resources**:
- [Diátaxis Framework](https://diataxis.fr/) - Documentation methodology
- [KNHK Repository](https://github.com/seanchatmangpt/knhk) - Source code

---

## 🔄 Maintenance & History

**Latest Update**: 2025-11-15
- Reorganized using Diátaxis framework
- Four categories: Tutorials, How-to Guides, Reference, Explanation
- Improved navigation and discoverability

**Previous Cleanup**: 2025-11-15
- Removed old versions (v1.0, v1.1.0)
- Removed build artifacts
- Saved ~131 MB storage
- See [CLEANUP_PLAN.md](CLEANUP_PLAN.md) for details

**Next Review**: 2025-12-15

---

## ❓ Common Questions

**Q: Where do I start?**
A: New to KNHK? Start with [Explanation](explanation/) to understand concepts.

**Q: I need the formal paper**
A: Go to [Reference](reference/) for PDFs and LaTeX sources.

**Q: How do I cite these papers?**
A: See the Citations section above for BibTeX entries.

**Q: Can I modify the papers?**
A: Yes! LaTeX sources are in [reference/chatman-equation/](reference/chatman-equation/).

**Q: Where are the diagrams?**
A: [reference/mermaid/](reference/mermaid/) has 90+ diagrams in multiple formats.

**Q: Why is this organized differently now?**
A: We adopted the Diátaxis framework for clearer, more purposeful documentation.

**Q: Where did version X go?**
A: Only latest versions maintained. Check git history for old versions.

---

## 📚 Understanding Diátaxis

The Diátaxis framework recognizes that documentation serves different purposes:

### Learning (Tutorials)
**What**: Guided learning experiences
**When**: You're new, want to learn by doing
**Example**: "Your first KNHK workflow" (coming soon)

### Tasks (How-to Guides)
**What**: Steps to solve specific problems
**When**: You know basics, need to accomplish something
**Example**: "How to optimize performance" (coming soon)

### Information (Reference)
**What**: Technical specifications and details
**When**: You need exact formulas, APIs, specifications
**Example**: [Chatman Equation PDF](reference/the_chatman_equation_fortune5_v1.2.0.pdf)

### Understanding (Explanation)
**What**: Conceptual clarity and context
**When**: You want to understand "why" and "how it fits"
**Example**: [Chatman Equation explained](explanation/the_chatman_equation_fortune5.md)

**Learn more**: [diataxis.fr](https://diataxis.fr/)

---

**Framework**: Diátaxis
**Last Updated**: 2025-11-15
**Maintained by**: KNHK Project

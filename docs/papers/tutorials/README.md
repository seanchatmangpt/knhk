# Tutorials

**Category**: Tutorials (Learning-oriented)

This section contains **hands-on, learning-oriented tutorials** that guide you through practical experiences with KNHK.

## 📚 What's Here

Tutorials are designed for **learning by doing**. They:
- Guide you step-by-step through practical tasks
- Help you gain competence and confidence
- Focus on learning outcomes, not problem-solving
- Are safe to follow without breaking things

---

## 🚀 Coming Soon

We're working on comprehensive tutorials including:

### Getting Started Series
- **Your First KNHK Workflow** - Set up and run your first telemetry-validated workflow
- **Understanding Telemetry** - Hands-on exploration of OpenTelemetry integration
- **Chicago TDD Basics** - Learn Test-Driven Development with real examples

### Advanced Tutorials
- **Building Production-Ready Features** - End-to-end feature development
- **Optimizing Performance** - Meet the ≤8 tick Chatman Constant
- **Schema-First Development** - OTel Weaver validation workflow

### Tool-Specific Tutorials
- **Working with Knowledge Hooks** - Practical guide to K-hooks
- **Implementing Workflow Patterns** - The 43 patterns in practice
- **Telemetry Schema Design** - Create effective OTel schemas

---

## 📖 Tutorial Structure

Each tutorial will follow this structure:

1. **Learning Objectives** - What you'll accomplish
2. **Prerequisites** - What you need before starting
3. **Setup** - Environment preparation
4. **Step-by-Step Guide** - Detailed walkthrough
5. **Verification** - Confirm it works
6. **What You Learned** - Recap and next steps

---

## 🎯 For Now

While we build comprehensive tutorials, you can:

### Learn the Concepts
Start with [**Explanation**](../explanation/) documentation:
- Read [`the_chatman_equation_fortune5.md`](../explanation/the_chatman_equation_fortune5.md)
- Understand [`kgs_whitepaper_v2_0_sean_chatman.md`](../explanation/kgs_whitepaper_v2_0_sean_chatman.md)

### Explore the Code
Check the main repository:
```bash
# Clone and explore
git clone https://github.com/seanchatmangpt/knhk.git
cd knhk

# Run basic tests
cargo test --workspace
make test-chicago-v04
```

### Review Examples
Look at existing implementations:
- Browse `/src` for Rust implementations
- Check `/tests` for test patterns
- Review `/examples` (if available)

---

## 🔗 Related Documentation

**Other Diátaxis Categories**:
- [**Explanation**](../explanation/) - Conceptual understanding (start here if new)
- [**Reference**](../reference/) - Technical specifications and papers
- [**How-to Guides**](../how-to-guides/) - Task-oriented problem solving (coming soon)

**External Resources**:
- [Diátaxis Framework](https://diataxis.fr/tutorials/) - Understanding tutorials
- [KNHK Repository](https://github.com/seanchatmangpt/knhk) - Source code and examples

---

## 💡 Tutorials vs. How-to Guides

**Tutorials** (this section):
- **Goal**: Help you learn through guided practice
- **Approach**: Step-by-step learning journey
- **Example**: "Your First KNHK Workflow" - Learn fundamentals
- **When**: You're new and want to learn

**How-to Guides** (task-focused):
- **Goal**: Solve a specific problem
- **Approach**: Direct steps to achieve outcome
- **Example**: "How to Fix Failing Tests" - Solve a problem
- **When**: You know basics, need to do something specific

---

## 📚 Diátaxis Framework

This documentation follows the [Diátaxis framework](https://diataxis.fr/):

| Type | Orientation | What it does | Example |
|------|-------------|--------------|---------|
| **Tutorials** | Learning | Guides through learning experience | "Your first KNHK workflow" (this section) |
| **How-to** | Tasks | Shows how to solve problems | "How to optimize telemetry" |
| **Reference** | Information | Describes technical details | "Chatman Equation specification" |
| **Explanation** | Understanding | Clarifies and discusses | "Why knowledge hooks matter" |

**You are here**: Tutorials (Learning-oriented)

---

## 🤝 Contributing

Want to help create tutorials? We'd love your help!

### Tutorial Ideas Welcome
- Share what you struggled to learn
- Suggest topics that would help newcomers
- Contribute your own learning experiences

### Quality Standards
- Clear learning objectives
- Tested step-by-step instructions
- Safe for beginners to follow
- Verified working examples

**Contact**: Open an issue or PR on the [KNHK repository](https://github.com/seanchatmangpt/knhk)

---

**Last Updated**: 2025-11-15
**Status**: Coming Soon
**Framework**: Diátaxis (Learning-oriented Tutorials)

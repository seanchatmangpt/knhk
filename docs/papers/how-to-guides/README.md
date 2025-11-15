# How-to Guides

**Category**: How-to Guides (Task-oriented)

This section contains **practical, task-oriented guides** that show you how to solve specific problems with KNHK.

## 📚 What's Here

How-to guides are designed for **getting things done**. They:
- Provide direct steps to achieve specific outcomes
- Assume you know the basics
- Focus on practical solutions, not learning
- Help you solve real problems efficiently

---

## 🚀 Coming Soon

We're working on practical how-to guides including:

### Development Workflows
- **How to Set Up Your Development Environment** - Complete setup guide
- **How to Run Tests Efficiently** - Test execution strategies
- **How to Debug Failing Tests** - Troubleshooting test failures
- **How to Add New Features** - Feature development workflow

### Telemetry & Validation
- **How to Create OTel Schemas** - Schema design and validation
- **How to Fix Weaver Validation Errors** - Common issues and solutions
- **How to Emit Proper Telemetry** - Instrumentation best practices
- **How to Optimize Performance** - Meet the ≤8 tick constraint

### Build & Deployment
- **How to Build the C Library** - C compilation workflow
- **How to Build Rust Binaries** - Cargo build optimization
- **How to Run Performance Tests** - Performance validation
- **How to Generate Documentation** - Doc generation workflow

### Integration Patterns
- **How to Integrate with OpenTelemetry Collectors** - OTLP setup
- **How to Use Knowledge Hooks** - Practical K-hook patterns
- **How to Implement Workflow Patterns** - Apply the 43 patterns
- **How to Validate Production Readiness** - Pre-deployment checklist

---

## 📖 How-to Guide Structure

Each guide will follow this structure:

1. **Goal** - What you'll accomplish
2. **Prerequisites** - What you need first
3. **Steps** - Direct, numbered instructions
4. **Verification** - How to confirm it worked
5. **Troubleshooting** - Common problems and fixes
6. **Related** - Links to relevant docs

---

## 🎯 For Now

While we build comprehensive how-to guides, you can:

### Check Existing Documentation
Review project files:
```bash
# Build guides
cat Makefile              # See available make targets
cat Cargo.toml            # Understand Rust workspace

# Test guides
make test-chicago-v04     # Run Chicago TDD tests
make test-performance-v04 # Run performance tests
cargo test --workspace    # Run all Rust tests
```

### Explore CLAUDE.md
The [`/CLAUDE.md`](/CLAUDE.md) file contains:
- Development workflow guidelines
- SPARC methodology usage
- Agent coordination patterns
- Build and test commands

### Review Reference Documentation
Check [**Reference**](../reference/) for:
- Technical specifications
- Formal papers with implementation details
- Architecture documentation

---

## 🔗 Related Documentation

**Other Diátaxis Categories**:
- [**Tutorials**](../tutorials/) - Learning-oriented guides (coming soon)
- [**Reference**](../reference/) - Technical specifications and papers
- [**Explanation**](../explanation/) - Conceptual understanding

**Project Documentation**:
- [`/CLAUDE.md`](/CLAUDE.md) - Development guidelines
- [`/docs/SITE_MAP.md`](/docs/SITE_MAP.md) - Full documentation index
- [`/README.md`](/README.md) - Project overview

---

## 💡 How-to Guides vs. Tutorials

**How-to Guides** (this section):
- **Goal**: Solve a specific problem
- **Approach**: Direct steps to outcome
- **Example**: "How to Fix Weaver Errors" - Solve a problem
- **When**: You know basics, need to do something

**Tutorials** (learning-focused):
- **Goal**: Help you learn through practice
- **Approach**: Step-by-step learning journey
- **Example**: "Your First KNHK Workflow" - Learn fundamentals
- **When**: You're new and want to learn

---

## 📚 Diátaxis Framework

This documentation follows the [Diátaxis framework](https://diataxis.fr/):

| Type | Orientation | What it does | Example |
|------|-------------|--------------|---------|
| **Tutorials** | Learning | Guides through learning experience | "Your first KNHK workflow" |
| **How-to** | Tasks | Shows how to solve problems | "How to optimize telemetry" (this section) |
| **Reference** | Information | Describes technical details | "Chatman Equation specification" |
| **Explanation** | Understanding | Clarifies and discusses | "Why knowledge hooks matter" |

**You are here**: How-to Guides (Task-oriented)

---

## 🤝 Contributing

Want to help create how-to guides? We'd love your help!

### Guide Ideas Welcome
- Share problems you've solved
- Document common workflows
- Contribute troubleshooting tips
- Improve existing guides

### Quality Standards
- Clear, achievable goals
- Step-by-step instructions
- Tested and verified
- Include troubleshooting

**Contact**: Open an issue or PR on the [KNHK repository](https://github.com/seanchatmangpt/knhk)

---

**Last Updated**: 2025-11-15
**Status**: Coming Soon
**Framework**: Diátaxis (Task-oriented How-to Guides)

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

## 📚 Available How-to Guides

### Development Workflows ✅ (Available Now)

1. **[How to Set Up Your Development Environment](01-setup-development-environment.md)**
   - Install Rust, Cargo, and system dependencies
   - Clone and configure the repository
   - Verify your setup with validation checks
   - **Time**: 15-30 minutes | **Level**: Beginner

2. **[How to Run Tests Efficiently](02-run-tests-efficiently.md)**
   - Understand the three-tier validation hierarchy
   - Use `cargo test`, `cargo nextest`, and project-specific commands
   - Optimize test execution for fast feedback
   - Interpret test results and performance constraints
   - **Time**: 10-20 minutes | **Level**: Beginner

3. **[How to Debug Failing Tests](03-debug-failing-tests.md)**
   - Systematically identify test failures
   - Use debugging tools and techniques
   - Common errors and solutions
   - Add debug output and use debuggers
   - **Time**: 15-30 minutes | **Level**: Intermediate

### Coming Soon

#### Development Workflows (3 guides)
- **How to Add New Features** - Feature development workflow
- **How to Build the C Library** - C compilation workflow
- **How to Build Rust Binaries** - Cargo build optimization

#### Telemetry & Validation (4 guides)
- **How to Create OTel Schemas** - Schema design and validation
- **How to Fix Weaver Validation Errors** - Common issues and solutions
- **How to Emit Proper Telemetry** - Instrumentation best practices
- **How to Optimize Performance** - Meet the ≤8 tick constraint

#### Deployment & Integration (4 guides)
- **How to Run Performance Tests** - Performance validation
- **How to Generate Documentation** - Doc generation workflow
- **How to Integrate with OpenTelemetry Collectors** - OTLP setup
- **How to Validate Production Readiness** - Pre-deployment checklist

#### Advanced Patterns (2+ guides)
- **How to Use Knowledge Hooks** - Practical K-hook patterns
- **How to Implement Workflow Patterns** - Apply the 43 patterns

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

## 🎯 Quick Start by Scenario

### New to KNHK?
1. Read: [Explanation: KNHK Overview](../explanation/the_chatman_equation_fortune5.md)
2. Do: [Tutorial: Your First KNHK Workflow](../tutorials/01-getting-started.md)
3. Setup: [How-to: Setup Development Environment](01-setup-development-environment.md)
4. Learn: [How-to: Run Tests Efficiently](02-run-tests-efficiently.md)

### Setup for the First Time?
→ [How-to: Setup Development Environment](01-setup-development-environment.md)

### Tests Are Failing?
→ [How-to: Debug Failing Tests](03-debug-failing-tests.md)

### Need to Run Tests?
→ [How-to: Run Tests Efficiently](02-run-tests-efficiently.md)

### Building a New Feature? (Coming Soon)
→ How-to: Add New Features

### Performance Tests Failing?
→ [How-to: Optimize Performance](11-optimize-performance.md) (coming soon)

### Weaver Validation Failing?
→ How-to: Fix Weaver Validation Errors (coming soon)

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

---

## 📊 How-to Guide Progress

| Guide | Status | Time | Level |
|-------|--------|------|-------|
| Setup Development Environment | ✅ Complete | 15-30min | Beginner |
| Run Tests Efficiently | ✅ Complete | 10-20min | Beginner |
| Debug Failing Tests | ✅ Complete | 15-30min | Intermediate |
| Add New Features | 🔄 Coming Soon | 20-30min | Intermediate |
| Build C Library | 🔄 Coming Soon | 15-20min | Intermediate |
| Create OTel Schemas | 🔄 Coming Soon | 20-25min | Intermediate |
| Fix Weaver Errors | 🔄 Coming Soon | 10-15min | Intermediate |
| Emit Proper Telemetry | 🔄 Coming Soon | 20-30min | Intermediate |
| Optimize Performance | 🔄 Coming Soon | 20-30min | Advanced |
| Validate Production Ready | 🔄 Coming Soon | 15-20min | Advanced |

---

**Last Updated**: 2025-11-15
**Status**: In Progress (3/13 complete)
**Framework**: Diátaxis (Task-oriented How-to Guides)

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

### Development Workflows

#### [1. How to Set Up Your Development Environment](01-setup-development-environment.md)
**Time**: 10-15 minutes | **Level**: Beginner

Complete setup guide for KNHK development:
- Install Rust and build dependencies
- Clone the repository
- Configure Git
- Build the project
- Verify installation
- Set up IDE (optional)
- Troubleshoot common issues

**What you'll accomplish**: Ready-to-develop KNHK environment

#### [2. How to Run Tests Efficiently](02-run-tests-efficiently.md)
**Time**: 10 minutes | **Level**: Beginner

Master the test suites and optimize your development workflow:
- Understand available test suites
- Run fast unit tests during development
- Run comprehensive tests before committing
- Run performance tests before pushing
- Debug failing tests
- Profile test performance
- CI/CD test integration

**What you'll accomplish**: Efficient testing workflow

#### [3. How to Fix Weaver Validation Errors](03-fix-weaver-validation-errors.md)
**Time**: 15 minutes per error | **Level**: Intermediate

Understand and resolve OpenTelemetry schema validation failures:
- Understand Weaver validation concepts
- Identify common error types
- Fix undocumented spans
- Fix missing attributes
- Fix metric issues
- Add telemetry to schema
- Validate schemas and live telemetry

**What you'll accomplish**: Ability to resolve validation issues

#### [4. How to Add New Features](04-add-new-features.md)
**Time**: 30-60 minutes per feature | **Level**: Intermediate

Implement features following KNHK patterns:
- Create feature branch
- Plan the feature
- Design architecture
- Write tests first (TDD)
- Implement code
- Add telemetry
- Create OTel schema
- Validate and commit
- Complete feature checklist

**What you'll accomplish**: New working feature with proper telemetry

### Telemetry & Validation

#### [5. How to Emit Proper Telemetry](05-emit-telemetry.md)
**Time**: 15 minutes | **Level**: Beginner

Add OpenTelemetry instrumentation to your code:
- Add tracing macros to functions
- Use logging levels effectively
- Skip sensitive data
- Add custom attributes
- Record results and performance
- Error handling with telemetry

**What you'll accomplish**: Properly instrumented code with complete telemetry

#### [6. How to Create OTel Schemas](06-create-otel-schemas.md)
**Time**: 20 minutes | **Level**: Intermediate

Design and implement OpenTelemetry schemas:
- Create schema files
- Define spans with attributes
- Define metrics and logs
- Complete schema examples
- Attribute and metric types
- Schema versioning and validation

**What you'll accomplish**: Well-documented telemetry contracts

#### [7. How to Optimize Performance](07-optimize-performance.md)
**Time**: 20-30 minutes | **Level**: Intermediate

Improve code performance to meet Chatman Constant:
- Identify hot paths
- Eliminate allocations
- Use references efficiently
- Inline functions
- Cache computations
- Benchmark improvements

**What you'll accomplish**: Code meeting ≤8 tick performance requirements

### Advanced Topics

#### [8. How to Debug Failing Tests](08-debug-failing-tests.md)
**Time**: 15-45 minutes | **Level**: Intermediate

Systematically identify and fix test failures:
- Common failure patterns
- Step-by-step debugging workflow
- Decision tree for diagnostics
- Instrumentation strategies
- Advanced debugging techniques
- Async and integration test debugging

**What you'll accomplish**: Ability to fix any failing test efficiently

#### [9. How to Validate Production Readiness](09-validate-production-readiness.md)
**Time**: 30-45 minutes | **Level**: Advanced

Ensure code is ready for production deployment:
- Code quality checklist
- Testing validation (100% pass rate)
- Performance validation (≤8 ticks)
- Telemetry validation (Weaver)
- Security audit checklist
- Observability requirements
- Deployment procedures
- Post-deployment monitoring

**What you'll accomplish**: Production-ready code with validation checklist

### Coming Soon

We're developing additional guides for:

#### Advanced Topics (Coming Soon)
- **How to Build the C Library** - C compilation workflow
- **How to Profile Code** - Deep performance analysis
- **How to Handle Errors** - Error handling best practices
- **How to Document APIs** - API documentation generation
- **How to Use Knowledge Hooks** - Practical K-hook patterns
- **How to Implement Workflow Patterns** - The 43 patterns

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

## 🎯 Getting Started

Choose a guide based on what you need to do:

### Just Starting Out?
1. **Start with**: [Setup Development Environment](01-setup-development-environment.md)
2. **Then learn**: [How to Run Tests Efficiently](02-run-tests-efficiently.md)

### Ready to Add Features?
1. **First understand**: [Tutorials](../tutorials/) - Learn the basics
2. **Then follow**: [How to Add New Features](04-add-new-features.md)

### Fixing Issues?
1. **Schema errors?** → [How to Fix Weaver Validation Errors](03-fix-weaver-validation-errors.md)
2. **Tests failing?** → [How to Run Tests Efficiently](02-run-tests-efficiently.md)
3. **Setup issues?** → [How to Setup Development Environment](01-setup-development-environment.md)

### Need More Details?
- **CLAUDE.md**: [`/CLAUDE.md`](/CLAUDE.md) - Development workflow guidelines
- **Reference**: [Technical specifications](../reference/)
- **Explanation**: [Conceptual understanding](../explanation/)
- **Tutorials**: [Learning-focused guides](../tutorials/)

---

## 🔗 Related Documentation

**Other Diátaxis Categories**:
- [**Tutorials**](../tutorials/) - Learning-oriented guides for getting started
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
**Status**: ✅ 9 Guides Complete | 🔄 Advanced Topics Coming (6+ planned)
**Framework**: Diátaxis (Task-oriented How-to Guides)
**Estimated Coverage**: Development to Advanced (90% of core workflows)
**Total Lines**: 3,500+ (production-grade solution guides)

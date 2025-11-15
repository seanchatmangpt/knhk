# Quick-Reference Cards

One-page checklists for common KNHK tasks. Print or use digitally. Each card covers essential items for a specific workflow.

---

## 📋 Available Cards

### [Production Readiness Checklist](PRODUCTION_READINESS_CHECKLIST.md)
**Purpose**: Pre-deployment validation checklist
- 15 items organized by category
- Categories: Build, Code Quality, Telemetry, Performance, Security, Config, Testing
- Time: 1.5-2 hours to complete
- **Use when**: About to deploy to production

### [Telemetry Checklist](TELEMETRY_CHECKLIST.md)
**Purpose**: Ensure proper telemetry instrumentation
- 12 items for complete instrumentation
- Categories: Spans, Metrics, Logs, Context, Sampling
- Performance budget verification
- **Use when**: Implementing telemetry for a new feature

### [Performance Optimization Checklist](PERFORMANCE_OPTIMIZATION_CHECKLIST.md)
**Purpose**: Optimize code to meet ≤8 tick Chatman Constant
- 10 optimization steps
- Each step shows expected improvement
- Profiling tools included
- **Use when**: Performance tests fail or are too slow

### [Testing Checklist](TESTING_CHECKLIST.md)
**Purpose**: Comprehensive test coverage validation
- Test types: Unit, Integration, Performance, Security
- Coverage targets: >90%
- Chicago TDD pattern verification
- Quick test commands
- **Use when**: Preparing code for review/deployment

### [Deployment Checklist](DEPLOYMENT_CHECKLIST.md)
**Purpose**: Pre-, during, and post-deployment steps
- Pre-deployment: 8 items
- During deployment: 5 items
- Post-deployment: 5 items
- Rollback procedures
- **Use when**: Deploying to any environment

---

## 🎯 Quick Selection Guide

| Task | Card |
|------|------|
| Getting ready for production | [Production Readiness](PRODUCTION_READINESS_CHECKLIST.md) |
| Adding telemetry to code | [Telemetry](TELEMETRY_CHECKLIST.md) |
| Code is running slow | [Performance Optimization](PERFORMANCE_OPTIMIZATION_CHECKLIST.md) |
| Ready for code review | [Testing](TESTING_CHECKLIST.md) |
| About to deploy | [Deployment](DEPLOYMENT_CHECKLIST.md) |

---

## 💡 How to Use These Cards

1. **Print**: Each card is one page (≤100 lines), perfect for printing
2. **Digital**: Use in your IDE's markdown preview
3. **Checklist**: Check items as you complete them
4. **Time Tracking**: Note the estimated time for each card
5. **Reference**: Keep as a bookmark in your development workflow

---

## 📖 Related Documentation

- **How-to Guides**: [Validate Production Readiness](../../../papers/how-to-guides/12-validate-production-readiness.md)
- **Tutorials**: [Building Production-Ready Features](../../../papers/tutorials/04-building-production-ready-features.md)
- **Examples**: [Code examples and patterns](../../examples/)
- **Troubleshooting**: [Common issues and solutions](../troubleshooting/)

---

## 🔄 Workflow Integration

**Typical developer workflow:**

```
1. Write code → Use Testing Checklist
2. Add telemetry → Use Telemetry Checklist
3. Performance issues → Use Performance Optimization Checklist
4. Pre-deployment → Use Production Readiness Checklist
5. Deploying → Use Deployment Checklist
```

---

## 📊 Cards Statistics

| Card | Items | Estimated Time |
|------|-------|-----------------|
| Production Readiness | 15 | 1.5-2 hours |
| Telemetry | 12 | 1-1.5 hours |
| Performance Optimization | 10 | 1-2 hours |
| Testing | 12 | 1-1.5 hours |
| Deployment | 15 | 30-45 min |

**Total items across all cards**: 56
**Average card size**: ~80-100 lines
**Format**: Markdown checklist (copy/paste friendly)

---

**Last Updated**: 2025-11-15
**Version**: v1.1.0
**Framework**: 80/20 Reference Cards

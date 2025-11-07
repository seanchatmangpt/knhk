# Pull Request

## What Changed
<!-- Describe the change in 2-3 sentences -->

## Evidence Required
<!-- Check ONLY what's needed for review -->
- [ ] Code changes (diffs in this PR)
- [ ] Test results (pass/fail status)
- [ ] Performance impact (if hot path changed)
- [ ] Breaking changes (if API modified)

**DO NOT provide** (unless explicitly requested by reviewer):
- [ ] Detailed analysis documents
- [ ] Architecture diagrams (unless design change)
- [ ] Extensive markdown reports
- [ ] Benchmark comparisons (unless performance claim)

## Reviewer Checklist
- [ ] Code compiles (`cargo build --workspace`)
- [ ] Tests pass (`cargo test --workspace`)
- [ ] Clippy clean (`cargo clippy --workspace -- -D warnings`)
- [ ] Meets acceptance criteria from issue

## LEAN Principle
> **Pull, Don't Push**: Provide only what's requested. Generate detailed documentation just-in-time when reviewer asks specific questions.

---

### Quick Commands for Reviewers
```bash
# Request quick status
./scripts/doc-pull.sh status

# Request current blockers
./scripts/doc-pull.sh blockers

# Request metrics
./scripts/doc-pull.sh metrics

# Request full analysis (only if needed)
./scripts/doc-pull.sh full-report
```

*This pull request follows the LEAN documentation policy: create only what's requested, when it's requested.*

# Evidence Index (Single Source of Truth)

**Last Updated**: 2025-11-06
**Purpose**: Eliminate duplicate documentation. One fact, one location, linked everywhere else.

## Quick Links

### Performance Evidence
- **Hot Path Performance**: `/Users/sac/knhk/docs/evidence/performance_8beat_validation.md`
- **Benchmarks**: `/Users/sac/knhk/rust/knhk-etl/benches/`
- **Cycle Time**: See DFLSS Consolidated Report (Section 3)
- **Takt Time**: See DFLSS Consolidated Report (Section 7)

### Quality Evidence
- **Test Results**: `/Users/sac/knhk/reports/test-results.json`
- **Code Quality**: `/Users/sac/knhk/docs/evidence/code_quality_8beat.md`
- **Clippy Results**: Run `cargo clippy --workspace -- -D warnings`
- **First Pass Yield**: See DFLSS Consolidated Report (Section 4)

### Weaver Validation (Source of Truth)
- **Schema Check**: `/Users/sac/knhk/docs/evidence/weaver_validation_final.md`
- **Live Check**: `/Users/sac/knhk/docs/evidence/dfss_weaver_live_validation.md`
- **OTEL Integration**: `/Users/sac/knhk/docs/v1-otel-validation-report.md`

### DFLSS/LEAN Metrics (Consolidated)
- **Consolidated Report**: `/Users/sac/knhk/docs/evidence/dflss_consolidated.md`
  - 8 Wastes Audit
  - Value Stream Mapping
  - Cycle Time Analysis
  - Flow Optimization
  - Kaizen Recommendations
  - Pull System Design
  - WIP Limits
  - Takt Time

### CI/CD Evidence
- **Pipeline Validation**: `/Users/sac/knhk/docs/evidence/cicd_validation.md`
- **GitHub Actions**: `/Users/sac/knhk/.github/workflows/`

### Production Readiness
- **Production Validation**: `/Users/sac/knhk/docs/evidence/production_validation_final.md`
- **Security Audit**: `/Users/sac/knhk/docs/evidence/dfss_security_sprint_validation.md`
- **Stability Tests**: `/Users/sac/knhk/docs/v1-stability-test-report.md`

## Deduplication Rules

1. **Link, Don't Duplicate**: Reference source documents, don't copy content
2. **Single Source of Truth**: Each fact exists in exactly one location
3. **No Redundant Reports**: Delete/consolidate reports covering same topics
4. **Consolidated Where Possible**: Related metrics combined into single reports

## Eliminated Duplicates

### Consolidated into `dflss_consolidated.md`:
- `dflss_8_wastes_audit.md` (719 lines)
- `dflss_lean_waste_analysis.md` (789 lines)
- `dflss_value_stream_map.md` (868 lines)
- `dflss_lean_cycle_time.md` (805 lines)
- `dflss_flow_optimization.md` (840 lines)
- `dflss_kaizen_recommendations.md` (855 lines)
- `dflss_pull_system_design.md` (532 lines)
- `dflss_first_pass_yield.md` (372 lines)
- `dflss_overproduction_waste.md` (527 lines)
- `dflss_skills_waste.md` (578 lines)

**Total Eliminated**: 6,885 lines → ~1,500 lines (78% reduction)

## Before/After Metrics

### Before Deduplication:
- **Total DFLSS docs**: 10 files
- **Total lines**: 6,885
- **Duplicate sections**: 47 (8× Executive Summary, 4× Conclusion, etc.)
- **Motion waste**: ~8.0 hours reading redundant content

### After Deduplication:
- **Total DFLSS docs**: 1 consolidated file + this index
- **Total lines**: ~1,500 (consolidated unique content)
- **Duplicate sections**: 0
- **Motion waste eliminated**: 8.0 hours

## Usage Guide

When looking for evidence:

1. **Check this index first** - find the single source of truth
2. **Link to source** - don't copy into new documents
3. **Update source** - if information changes, update the canonical location
4. **Never duplicate** - resist the urge to copy/paste into new reports

## Maintenance

This index is maintained by the DFLSS Duplication Eliminator agent. If you need to add new evidence:

1. Determine if it fits into existing consolidated reports
2. If yes, add to existing source and link here
3. If no, create new canonical source and add link here
4. **Never create standalone reports for topics already covered**

# KNHK Evidence Repository

This directory contains all evidence artifacts required by PRD Section 18 (8-Beat PRD) and DFLSS Project Charter Section 12.

## Evidence Artifacts

### Complete Evidence ✅

| ID | Name | Location | Format | Machine-Readable |
|----|------|----------|--------|------------------|
| `ev:beat_design` | Beat Scheduler Design | `../docs/8BEAT-PRD.txt` | Turtle RDF | ✅ Yes |
| `ev:weaver_checks` | Weaver Validation Results | `weaver_checks/` | YAML | ✅ Yes |
| `ev:policy_packs` | Policy Budget Configurations | `../rust/knhk-validation/policies/` | Rego | ✅ Yes |

### Evidence to Collect ⚠️

| ID | Name | Location | Format | Collection Procedure |
|----|------|----------|--------|---------------------|
| `ev:pmu_bench` | PMU Benchmark Results | `pmu_bench/` | CSV | `pmu_bench/collection_procedure.md` |
| `ev:receipts_root` | Lockchain Receipt Roots | `receipts_root/` | JSON | `receipts_root/collection_procedure.md` |

### Future Evidence ⚠️

| ID | Name | Location | Format | Dependency |
|----|------|----------|--------|-----------|
| `ev:canary_report` | Canary Deployment Report | `canary_report/` | Markdown | Production deployment |
| `ev:finance_oom` | Financial OOM Analysis | `finance_oom/` | Excel | Measure phase baseline |

## Quick Start

### Collect PMU Benchmarks

```bash
cd pmu_bench
bash collect_benchmarks.sh
```

### Generate Sample Receipts

```bash
cd receipts_root
bash generate_samples.sh
```

### Validate Evidence

```bash
# Validate all evidence artifacts
python3 ../scripts/validate_evidence.py

# Check evidence manifest
python3 ../scripts/check_manifest.py evidence_manifest.json
```

## Evidence Manifest

The evidence manifest (`evidence_manifest.json`) provides machine-readable tracking of all evidence artifacts, their status, and linkage to PRD requirements.

## DFLSS Compliance

This evidence repository supports DFLSS charter requirements (Section 12):
- **Approvals**: Pending signatures
- **Problem/VOC/Baseline**: Documented, baseline measurements in Measure phase
- **CTQs**: Defined with SMART criteria
- **Scope**: Documented
- **Financials**: OOM framework defined
- **Architecture**: Readiness validated
- **Ontology/Policy**: Cataloged
- **Risk Register**: Complete
- **Governance**: Documented
- **Dashboards**: Metrics defined
- **Communications**: Plan pending
- **Acceptance**: Criteria defined

## Collection Priority

**Week 1**:
1. PMU benchmarks
2. Sample receipts
3. Weaver live-check

**Weeks 2-4**:
4. Baseline measurements (Measure phase)
5. Canary deployment plan
6. Financial OOM framework

**Post-Deployment**:
7. Canary execution
8. Financial ROI analysis
9. Acceptance validation

## References

- [Evidence Inventory](../docs/V1-EVIDENCE-INVENTORY.md) - Complete evidence catalog
- [8-Beat PRD](../docs/8BEAT-PRD.txt) - Product requirements (Section 18)
- [DFLSS Charter](../docs/DFLSS_PROJECT_CHARTER.md) - Project charter (Section 12)
- [Performance Compliance](../docs/performance-compliance-report.md) - Compliance analysis

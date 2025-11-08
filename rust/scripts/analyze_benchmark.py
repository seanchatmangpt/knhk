#!/usr/bin/env python3
"""
Analyze KNHK crate benchmark results and generate performance report.
"""

import csv
import json
from pathlib import Path
from typing import List, Dict, Any

def parse_csv(csv_path: Path) -> List[Dict[str, Any]]:
    """Parse benchmark CSV file."""
    results = []
    with open(csv_path, 'r') as f:
        reader = csv.DictReader(f)
        for row in reader:
            # Convert numeric fields
            try:
                results.append({
                    'crate': row['crate'],
                    'loc': int(row['loc']),
                    'build_debug_sec': float(row['build_debug_sec']),
                    'build_release_sec': float(row['build_release_sec']),
                    'test_sec': float(row['test_sec']),
                    'clippy_sec': float(row['clippy_sec']),
                    'binary_size_kb': row['binary_size_kb'] if row['binary_size_kb'] == 'N/A' else int(row['binary_size_kb'])
                })
            except ValueError:
                continue
    return results

def analyze_results(results: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Analyze benchmark results and extract insights."""

    # Sort by different metrics
    slowest_builds_debug = sorted(results, key=lambda x: x['build_debug_sec'], reverse=True)[:5]
    slowest_builds_release = sorted(results, key=lambda x: x['build_release_sec'], reverse=True)[:5]
    slowest_tests = sorted(results, key=lambda x: x['test_sec'], reverse=True)[:5]
    largest_crates = sorted(results, key=lambda x: x['loc'], reverse=True)[:5]

    # Calculate totals
    total_loc = sum(r['loc'] for r in results)
    total_build_debug = sum(r['build_debug_sec'] for r in results)
    total_build_release = sum(r['build_release_sec'] for r in results)
    total_test = sum(r['test_sec'] for r in results)
    total_clippy = sum(r['clippy_sec'] for r in results)

    # Average metrics
    avg_build_debug = total_build_debug / len(results)
    avg_build_release = total_build_release / len(results)
    avg_test = total_test / len(results)
    avg_clippy = total_clippy / len(results)
    avg_loc = total_loc / len(results)

    # Efficiency metrics (LOC per second)
    build_efficiency = [
        {
            'crate': r['crate'],
            'loc_per_sec': r['loc'] / r['build_debug_sec'] if r['build_debug_sec'] > 0 else 0
        }
        for r in results
    ]
    most_efficient = sorted(build_efficiency, key=lambda x: x['loc_per_sec'], reverse=True)[:5]
    least_efficient = sorted(build_efficiency, key=lambda x: x['loc_per_sec'])[:5]

    return {
        'summary': {
            'total_crates': len(results),
            'total_loc': total_loc,
            'total_build_debug_sec': round(total_build_debug, 2),
            'total_build_release_sec': round(total_build_release, 2),
            'total_test_sec': round(total_test, 2),
            'total_clippy_sec': round(total_clippy, 2),
            'avg_build_debug_sec': round(avg_build_debug, 2),
            'avg_build_release_sec': round(avg_build_release, 2),
            'avg_test_sec': round(avg_test, 2),
            'avg_clippy_sec': round(avg_clippy, 2),
            'avg_loc': round(avg_loc, 2)
        },
        'slowest_builds_debug': [
            {'crate': r['crate'], 'time_sec': round(r['build_debug_sec'], 2)}
            for r in slowest_builds_debug
        ],
        'slowest_builds_release': [
            {'crate': r['crate'], 'time_sec': round(r['build_release_sec'], 2)}
            for r in slowest_builds_release
        ],
        'slowest_tests': [
            {'crate': r['crate'], 'time_sec': round(r['test_sec'], 2)}
            for r in slowest_tests
        ],
        'largest_crates': [
            {'crate': r['crate'], 'loc': r['loc']}
            for r in largest_crates
        ],
        'most_efficient': [
            {'crate': r['crate'], 'loc_per_sec': round(r['loc_per_sec'], 2)}
            for r in most_efficient
        ],
        'least_efficient': [
            {'crate': r['crate'], 'loc_per_sec': round(r['loc_per_sec'], 2)}
            for r in least_efficient
        ]
    }

def generate_report(analysis: Dict[str, Any], output_path: Path):
    """Generate markdown report."""

    report = f"""# KNHK Monorepo Performance Benchmark Report

## Workspace Summary

- **Total Crates**: {analysis['summary']['total_crates']}
- **Total Lines of Code**: {analysis['summary']['total_loc']:,}
- **Average LOC per Crate**: {analysis['summary']['avg_loc']:,.0f}

### Build Times

| Metric | Debug | Release |
|--------|-------|---------|
| **Total Time** | {analysis['summary']['total_build_debug_sec']:.2f}s ({analysis['summary']['total_build_debug_sec']/60:.2f}m) | {analysis['summary']['total_build_release_sec']:.2f}s ({analysis['summary']['total_build_release_sec']/60:.2f}m) |
| **Average per Crate** | {analysis['summary']['avg_build_debug_sec']:.2f}s | {analysis['summary']['avg_build_release_sec']:.2f}s |

### Test & Quality Times

| Metric | Time |
|--------|------|
| **Total Test Time** | {analysis['summary']['total_test_sec']:.2f}s ({analysis['summary']['total_test_sec']/60:.2f}m) |
| **Total Clippy Time** | {analysis['summary']['total_clippy_sec']:.2f}s |
| **Average Test Time** | {analysis['summary']['avg_test_sec']:.2f}s |
| **Average Clippy Time** | {analysis['summary']['avg_clippy_sec']:.2f}s |

---

## Top 5 Slowest Builds (Debug)

| Rank | Crate | Build Time |
|------|-------|------------|
"""

    for i, item in enumerate(analysis['slowest_builds_debug'], 1):
        report += f"| {i} | `{item['crate']}` | {item['time_sec']:.2f}s |\n"

    report += f"""
---

## Top 5 Slowest Builds (Release)

| Rank | Crate | Build Time |
|------|-------|------------|
"""

    for i, item in enumerate(analysis['slowest_builds_release'], 1):
        report += f"| {i} | `{item['crate']}` | {item['time_sec']:.2f}s |\n"

    report += f"""
---

## Top 5 Slowest Tests

| Rank | Crate | Test Time |
|------|-------|-----------|
"""

    for i, item in enumerate(analysis['slowest_tests'], 1):
        report += f"| {i} | `{item['crate']}` | {item['time_sec']:.2f}s |\n"

    report += f"""
---

## Top 5 Largest Crates

| Rank | Crate | Lines of Code |
|------|-------|---------------|
"""

    for i, item in enumerate(analysis['largest_crates'], 1):
        report += f"| {i} | `{item['crate']}` | {item['loc']:,} |\n"

    report += f"""
---

## Build Efficiency (LOC/sec)

### Most Efficient (Debug Build)

| Rank | Crate | LOC/sec |
|------|-------|---------|
"""

    for i, item in enumerate(analysis['most_efficient'], 1):
        report += f"| {i} | `{item['crate']}` | {item['loc_per_sec']:.2f} |\n"

    report += f"""
### Least Efficient (Debug Build)

| Rank | Crate | LOC/sec |
|------|-------|---------|
"""

    for i, item in enumerate(analysis['least_efficient'], 1):
        report += f"| {i} | `{item['crate']}` | {item['loc_per_sec']:.2f} |\n"

    report += f"""
---

## Optimization Opportunities

### 🔴 Critical (High Impact)

"""

    # Identify optimization opportunities
    slow_builds = [item['crate'] for item in analysis['slowest_builds_debug'][:3]]
    slow_tests = [item['crate'] for item in analysis['slowest_tests'][:3]]
    large_crates = [item['crate'] for item in analysis['largest_crates'][:3]]
    inefficient = [item['crate'] for item in analysis['least_efficient'][:3]]

    if slow_builds:
        report += f"""1. **Optimize slowest builds**: {', '.join(f'`{c}`' for c in slow_builds)}
   - Consider splitting large crates
   - Review dependency graph for circular dependencies
   - Enable incremental compilation features

"""

    if slow_tests:
        report += f"""2. **Optimize slow tests**: {', '.join(f'`{c}`' for c in slow_tests)}
   - Parallelize test execution
   - Mock expensive I/O operations
   - Review test setup/teardown overhead

"""

    report += f"""
### 🟡 Medium Impact

"""

    if large_crates:
        report += f"""1. **Refactor large crates**: {', '.join(f'`{c}`' for c in large_crates)}
   - Split into smaller, focused modules
   - Extract reusable components
   - Reduce coupling between modules

"""

    if inefficient:
        report += f"""2. **Improve build efficiency**: {', '.join(f'`{c}`' for c in inefficient)}
   - Review complex procedural macros
   - Optimize dependency compilation
   - Consider feature flags for optional dependencies

"""

    report += f"""
### 🟢 Low Impact (Nice to Have)

1. **Dependency Optimization**
   - Audit unused dependencies
   - Use `cargo-udeps` to find unused deps
   - Consider workspace-level dependency deduplication

2. **Compilation Cache**
   - Enable `sccache` or `mold` linker
   - Use build caching in CI/CD
   - Pre-compile common dependencies

---

## Recommendations

### Short-term (1-2 weeks)

1. **Profile slowest builds** using `cargo build --timings`
2. **Optimize test parallelization** for slow test suites
3. **Review dependency graph** for optimization opportunities

### Medium-term (1-2 months)

1. **Refactor largest crates** into smaller, focused modules
2. **Implement build caching** in development and CI
3. **Benchmark critical paths** for performance regressions

### Long-term (3-6 months)

1. **Continuous performance monitoring** in CI/CD
2. **Automated performance regression detection**
3. **Developer tooling improvements** (IDE integration, build scripts)

---

*Generated from KNHK workspace benchmark run on {Path.cwd()}*
"""

    with open(output_path, 'w') as f:
        f.write(report)

def main():
    """Main entry point."""
    workspace_root = Path("/Users/sac/knhk/rust")
    csv_path = workspace_root / "crate_metrics.csv"
    json_path = workspace_root / "benchmark_analysis.json"
    report_path = workspace_root / "docs" / "PERFORMANCE_BENCHMARK.md"

    # Parse results
    results = parse_csv(csv_path)

    # Analyze
    analysis = analyze_results(results)

    # Save JSON
    with open(json_path, 'w') as f:
        json.dump(analysis, f, indent=2)

    # Generate report
    report_path.parent.mkdir(exist_ok=True)
    generate_report(analysis, report_path)

    print(f"✅ Analysis complete!")
    print(f"   - JSON: {json_path}")
    print(f"   - Report: {report_path}")

if __name__ == "__main__":
    main()

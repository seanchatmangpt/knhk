#!/usr/bin/env python3
"""
KNHK Compilation Benchmark Analysis
Generates performance profiles and optimization recommendations
"""

import json
import sys
from pathlib import Path
from typing import List, Dict, Any
from datetime import datetime

def load_benchmark_data(filepath: Path) -> List[Dict[str, Any]]:
    """Load benchmark JSON data"""
    with open(filepath, 'r') as f:
        return json.load(f)

def format_time(seconds: float) -> str:
    """Format time in human-readable format"""
    if seconds is None or seconds == "null":
        return "N/A"
    if seconds < 1:
        return f"{seconds*1000:.0f}ms"
    elif seconds < 60:
        return f"{seconds:.2f}s"
    else:
        mins = int(seconds // 60)
        secs = seconds % 60
        return f"{mins}m {secs:.1f}s"

def generate_summary_table(data: List[Dict[str, Any]]) -> str:
    """Generate summary table of all packages"""
    output = []
    output.append("\n" + "="*120)
    output.append("KNHK COMPILATION BENCHMARK SUMMARY")
    output.append("="*120)
    output.append("")

    # Header
    output.append(f"{'Package':<25} {'LOC':>8} {'Files':>6} {'Deps':>10} {'Clean':>12} {'Incr':>12} {'Test':>12}")
    output.append("-"*120)

    # Sort by clean build time (descending)
    sorted_data = sorted(data, key=lambda x: float(x.get('clean_build_time_sec', 0) or 0), reverse=True)

    total_loc = 0
    total_files = 0
    total_clean = 0
    total_incr = 0
    total_test = 0

    for pkg in sorted_data:
        name = pkg['package']
        loc = pkg.get('lines_of_code', 0)
        files = pkg.get('file_count', 0)
        deps = f"{pkg.get('direct_dependencies', 0)}/{pkg.get('transitive_dependencies', 0)}"
        clean = pkg.get('clean_build_time_sec')
        incr = pkg.get('incremental_build_time_sec')
        test = pkg.get('test_build_time_sec')

        total_loc += loc
        total_files += files
        if clean and clean != "null":
            total_clean += float(clean)
        if incr and incr != "null":
            total_incr += float(incr)
        if test and test != "null":
            total_test += float(test)

        output.append(
            f"{name:<25} {loc:>8,} {files:>6} {deps:>10} "
            f"{format_time(float(clean) if clean and clean != 'null' else None):>12} "
            f"{format_time(float(incr) if incr and incr != 'null' else None):>12} "
            f"{format_time(float(test) if test and test != 'null' else None):>12}"
        )

    output.append("-"*120)
    output.append(
        f"{'TOTALS':<25} {total_loc:>8,} {total_files:>6} {' ':>10} "
        f"{format_time(total_clean):>12} {format_time(total_incr):>12} {format_time(total_test):>12}"
    )
    output.append("="*120)

    return "\n".join(output)

def generate_performance_profiles(data: List[Dict[str, Any]]) -> str:
    """Generate performance profiles"""
    output = []
    output.append("\n" + "="*120)
    output.append("PERFORMANCE PROFILES")
    output.append("="*120)

    # Filter out packages with null build times
    valid_data = [pkg for pkg in data if pkg.get('clean_build_time_sec') and pkg['clean_build_time_sec'] != "null"]

    # 1. Fastest to build
    output.append("\n🚀 FASTEST TO BUILD (Clean Build):")
    output.append("-"*60)
    fastest = sorted(valid_data, key=lambda x: float(x['clean_build_time_sec']))[:5]
    for i, pkg in enumerate(fastest, 1):
        time = format_time(float(pkg['clean_build_time_sec']))
        output.append(f"  {i}. {pkg['package']:<30} {time:>12}")

    # 2. Slowest to build
    output.append("\n🐌 SLOWEST TO BUILD (Clean Build):")
    output.append("-"*60)
    slowest = sorted(valid_data, key=lambda x: float(x['clean_build_time_sec']), reverse=True)[:5]
    for i, pkg in enumerate(slowest, 1):
        time = format_time(float(pkg['clean_build_time_sec']))
        output.append(f"  {i}. {pkg['package']:<30} {time:>12}")

    # 3. Largest codebases
    output.append("\n📊 LARGEST CODEBASES (Lines of Code):")
    output.append("-"*60)
    largest = sorted(data, key=lambda x: x.get('lines_of_code', 0), reverse=True)[:5]
    for i, pkg in enumerate(largest, 1):
        loc = pkg.get('lines_of_code', 0)
        files = pkg.get('file_count', 0)
        output.append(f"  {i}. {pkg['package']:<30} {loc:>8,} lines in {files} files")

    # 4. Smallest packages
    output.append("\n📦 SMALLEST PACKAGES (Lines of Code):")
    output.append("-"*60)
    smallest = sorted(data, key=lambda x: x.get('lines_of_code', 0))[:5]
    for i, pkg in enumerate(smallest, 1):
        loc = pkg.get('lines_of_code', 0)
        files = pkg.get('file_count', 0)
        output.append(f"  {i}. {pkg['package']:<30} {loc:>8,} lines in {files} files")

    # 5. Most dependencies
    output.append("\n🔗 MOST DEPENDENCIES (Transitive):")
    output.append("-"*60)
    most_deps = sorted(data, key=lambda x: x.get('transitive_dependencies', 0), reverse=True)[:5]
    for i, pkg in enumerate(most_deps, 1):
        direct = pkg.get('direct_dependencies', 0)
        trans = pkg.get('transitive_dependencies', 0)
        output.append(f"  {i}. {pkg['package']:<30} {direct} direct, {trans} transitive")

    # 6. Best incremental build performance
    valid_incr = [pkg for pkg in data if pkg.get('incremental_build_time_sec') and pkg['incremental_build_time_sec'] != "null"]
    if valid_incr:
        output.append("\n⚡ BEST INCREMENTAL BUILD TIMES:")
        output.append("-"*60)
        best_incr = sorted(valid_incr, key=lambda x: float(x['incremental_build_time_sec']))[:5]
        for i, pkg in enumerate(best_incr, 1):
            time = format_time(float(pkg['incremental_build_time_sec']))
            output.append(f"  {i}. {pkg['package']:<30} {time:>12}")

    output.append("\n" + "="*120)
    return "\n".join(output)

def generate_optimization_recommendations(data: List[Dict[str, Any]]) -> str:
    """Generate optimization recommendations"""
    output = []
    output.append("\n" + "="*120)
    output.append("OPTIMIZATION RECOMMENDATIONS")
    output.append("="*120)

    recommendations = []

    # 1. Packages with excessive build times
    valid_data = [pkg for pkg in data if pkg.get('clean_build_time_sec') and pkg['clean_build_time_sec'] != "null"]
    if valid_data:
        avg_build_time = sum(float(pkg['clean_build_time_sec']) for pkg in valid_data) / len(valid_data)
        slow_packages = [pkg for pkg in valid_data if float(pkg['clean_build_time_sec']) > avg_build_time * 2]

        if slow_packages:
            output.append("\n⚠️  SLOW BUILD TIMES (>2x average):")
            output.append("-"*60)
            for pkg in sorted(slow_packages, key=lambda x: float(x['clean_build_time_sec']), reverse=True):
                time = format_time(float(pkg['clean_build_time_sec']))
                output.append(f"  • {pkg['package']}: {time}")
                recommendations.append({
                    'package': pkg['package'],
                    'issue': 'Excessive build time',
                    'suggestion': 'Consider splitting into smaller crates or reviewing dependency tree'
                })

    # 2. Packages with many dependencies
    high_dep_packages = [pkg for pkg in data if pkg.get('transitive_dependencies', 0) > 100]
    if high_dep_packages:
        output.append("\n🔗 HIGH DEPENDENCY COUNT (>100 transitive):")
        output.append("-"*60)
        for pkg in sorted(high_dep_packages, key=lambda x: x.get('transitive_dependencies', 0), reverse=True):
            deps = pkg.get('transitive_dependencies', 0)
            output.append(f"  • {pkg['package']}: {deps} transitive dependencies")
            recommendations.append({
                'package': pkg['package'],
                'issue': 'High dependency count',
                'suggestion': 'Review dependency tree with `cargo tree -p <package>` and eliminate unnecessary dependencies'
            })

    # 3. Packages with poor incremental build performance
    valid_incr = [pkg for pkg in data if
                  pkg.get('incremental_build_time_sec') and pkg['incremental_build_time_sec'] != "null" and
                  pkg.get('clean_build_time_sec') and pkg['clean_build_time_sec'] != "null"]

    poor_incr = []
    for pkg in valid_incr:
        clean = float(pkg['clean_build_time_sec'])
        incr = float(pkg['incremental_build_time_sec'])
        ratio = incr / clean if clean > 0 else 0
        if ratio > 0.3:  # Incremental build is >30% of clean build
            poor_incr.append((pkg, ratio))

    if poor_incr:
        output.append("\n⚡ POOR INCREMENTAL BUILD PERFORMANCE (>30% of clean):")
        output.append("-"*60)
        for pkg, ratio in sorted(poor_incr, key=lambda x: x[1], reverse=True):
            output.append(f"  • {pkg['package']}: {ratio*100:.1f}% of clean build time")
            recommendations.append({
                'package': pkg['package'],
                'issue': 'Poor incremental build performance',
                'suggestion': 'Review module structure and consider reducing cross-module dependencies'
            })

    # 4. Large packages that could be split
    large_packages = [pkg for pkg in data if pkg.get('lines_of_code', 0) > 3000]
    if large_packages:
        output.append("\n📦 LARGE PACKAGES (>3000 LOC):")
        output.append("-"*60)
        for pkg in sorted(large_packages, key=lambda x: x.get('lines_of_code', 0), reverse=True):
            loc = pkg.get('lines_of_code', 0)
            output.append(f"  • {pkg['package']}: {loc:,} lines")
            recommendations.append({
                'package': pkg['package'],
                'issue': 'Large codebase',
                'suggestion': 'Consider splitting into smaller, focused crates for better compilation parallelism'
            })

    # 5. Workspace optimization opportunities
    output.append("\n🏗️  WORKSPACE OPTIMIZATION OPPORTUNITIES:")
    output.append("-"*60)

    # Check for packages that should use workspace dependencies
    output.append("  • ✓ All packages should inherit common dependencies from workspace Cargo.toml")
    output.append("  • ✓ Verify all packages use workspace.package version = \"1.0.0\"")
    output.append("  • ✓ Consider enabling parallel compilation: CARGO_BUILD_JOBS=<n>")
    output.append("  • ✓ Use `cargo build --timings` for detailed compilation profiling")

    # Parallel build recommendations
    output.append("\n⚙️  PARALLEL BUILD STRATEGY:")
    output.append("-"*60)

    # Group packages by dependency depth
    fast_pkgs = [pkg['package'] for pkg in data if
                 pkg.get('clean_build_time_sec') and pkg['clean_build_time_sec'] != "null" and
                 float(pkg['clean_build_time_sec']) < 5]
    slow_pkgs = [pkg['package'] for pkg in data if
                 pkg.get('clean_build_time_sec') and pkg['clean_build_time_sec'] != "null" and
                 float(pkg['clean_build_time_sec']) >= 20]

    if fast_pkgs:
        output.append(f"  • Fast builds (<5s): {', '.join(fast_pkgs[:5])}")
        output.append("    → Build these first to unblock dependent packages")

    if slow_pkgs:
        output.append(f"  • Slow builds (≥20s): {', '.join(slow_pkgs[:5])}")
        output.append("    → Focus optimization efforts here for maximum impact")

    # Summary of recommendations
    if recommendations:
        output.append("\n📋 ACTIONABLE RECOMMENDATIONS:")
        output.append("-"*60)
        for i, rec in enumerate(recommendations, 1):
            output.append(f"\n  {i}. {rec['package']}")
            output.append(f"     Issue: {rec['issue']}")
            output.append(f"     Recommendation: {rec['suggestion']}")

    output.append("\n" + "="*120)
    return "\n".join(output)

def generate_markdown_report(data: List[Dict[str, Any]], output_file: Path) -> None:
    """Generate markdown report"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    md = []
    md.append("# KNHK Compilation Performance Benchmark Report")
    md.append(f"\n**Generated:** {timestamp}")
    md.append(f"\n**Total Packages:** {len(data)}")
    md.append("")

    # Summary table
    md.append("## Summary")
    md.append("")
    md.append("| Package | LOC | Files | Dependencies | Clean Build | Incremental | Test Build |")
    md.append("|---------|----:|------:|--------------|------------:|------------:|-----------:|")

    sorted_data = sorted(data, key=lambda x: float(x.get('clean_build_time_sec', 0) or 0), reverse=True)
    for pkg in sorted_data:
        name = pkg['package']
        loc = pkg.get('lines_of_code', 0)
        files = pkg.get('file_count', 0)
        deps = f"{pkg.get('direct_dependencies', 0)}/{pkg.get('transitive_dependencies', 0)}"
        clean = format_time(float(pkg['clean_build_time_sec']) if pkg.get('clean_build_time_sec') and pkg['clean_build_time_sec'] != 'null' else None)
        incr = format_time(float(pkg['incremental_build_time_sec']) if pkg.get('incremental_build_time_sec') and pkg['incremental_build_time_sec'] != 'null' else None)
        test = format_time(float(pkg['test_build_time_sec']) if pkg.get('test_build_time_sec') and pkg['test_build_time_sec'] != 'null' else None)

        md.append(f"| `{name}` | {loc:,} | {files} | {deps} | {clean} | {incr} | {test} |")

    md.append("")
    md.append("## Performance Profiles")
    md.append("")

    # Top 5 slowest
    md.append("### Slowest to Build (Clean)")
    md.append("")
    valid_data = [pkg for pkg in data if pkg.get('clean_build_time_sec') and pkg['clean_build_time_sec'] != "null"]
    slowest = sorted(valid_data, key=lambda x: float(x['clean_build_time_sec']), reverse=True)[:5]
    for i, pkg in enumerate(slowest, 1):
        time = format_time(float(pkg['clean_build_time_sec']))
        md.append(f"{i}. **{pkg['package']}**: {time}")

    md.append("")
    md.append("### Largest Codebases")
    md.append("")
    largest = sorted(data, key=lambda x: x.get('lines_of_code', 0), reverse=True)[:5]
    for i, pkg in enumerate(largest, 1):
        loc = pkg.get('lines_of_code', 0)
        files = pkg.get('file_count', 0)
        md.append(f"{i}. **{pkg['package']}**: {loc:,} lines in {files} files")

    md.append("")
    md.append("## Optimization Recommendations")
    md.append("")
    md.append("1. **Slow builds**: Focus on packages with >20s clean build times")
    md.append("2. **High dependencies**: Review packages with >100 transitive dependencies")
    md.append("3. **Large codebases**: Consider splitting packages >3000 LOC")
    md.append("4. **Incremental builds**: Improve packages where incremental is >30% of clean")
    md.append("")

    with open(output_file, 'w') as f:
        f.write('\n'.join(md))

def main():
    if len(sys.argv) < 2:
        print("Usage: analyze_compilation_benchmark.py <benchmark_json_file>")
        sys.exit(1)

    input_file = Path(sys.argv[1])
    if not input_file.exists():
        print(f"Error: File not found: {input_file}")
        sys.exit(1)

    # Load data
    data = load_benchmark_data(input_file)

    # Generate reports
    print(generate_summary_table(data))
    print(generate_performance_profiles(data))
    print(generate_optimization_recommendations(data))

    # Generate markdown report
    md_file = input_file.with_suffix('.md')
    generate_markdown_report(data, md_file)
    print(f"\n📄 Markdown report saved to: {md_file}")

if __name__ == '__main__':
    main()

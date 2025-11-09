#!/usr/bin/env python3
"""
Collect Code Quality Metrics for c-Chart

Scans codebase for defects and outputs weighted count.
"""

import subprocess
import json
from pathlib import Path

def count_clippy_errors():
    """Count clippy errors."""
    try:
        result = subprocess.run(
            ['cargo', 'clippy', '--workspace', '--', '-D', 'warnings'],
            capture_output=True,
            text=True,
            cwd='/Users/sac/knhk'
        )

        # Count errors
        errors = result.stderr.count('error:')
        return errors
    except Exception as e:
        print(f"Warning: Could not run clippy: {e}", file=sys.stderr)
        return 0

def count_clippy_warnings():
    """Count clippy warnings."""
    try:
        result = subprocess.run(
            ['cargo', 'clippy', '--workspace'],
            capture_output=True,
            text=True,
            cwd='/Users/sac/knhk'
        )

        # Count warnings (excluding errors)
        warnings = result.stderr.count('warning:')
        return warnings
    except Exception as e:
        print(f"Warning: Could not run clippy: {e}", file=sys.stderr)
        return 0

def count_unwrap_in_production():
    """Count .unwrap() and .expect() in production code."""
    try:
        result = subprocess.run(
            ['bash', '-c',
             'grep -r "\\.unwrap()\\|\\.expect(" rust/*/src --include="*.rs" | '
             'grep -v test | grep -v cli | grep -v examples | grep -v build.rs | wc -l'],
            capture_output=True,
            text=True,
            cwd='/Users/sac/knhk'
        )

        count = int(result.stdout.strip())
        return count
    except Exception as e:
        print(f"Warning: Could not count unwraps: {e}", file=sys.stderr)
        return 0

def count_println_in_production():
    """Count println! in production code."""
    try:
        result = subprocess.run(
            ['bash', '-c',
             'grep -r "println!" rust/*/src --include="*.rs" | '
             'grep -v test | grep -v cli | grep -v examples | wc -l'],
            capture_output=True,
            text=True,
            cwd='/Users/sac/knhk'
        )

        count = int(result.stdout.strip())
        return count
    except Exception as e:
        print(f"Warning: Could not count printlns: {e}", file=sys.stderr)
        return 0

def count_unimplemented():
    """Count unimplemented!() calls."""
    try:
        result = subprocess.run(
            ['bash', '-c',
             'grep -r "unimplemented!" rust/*/src --include="*.rs" | wc -l'],
            capture_output=True,
            text=True,
            cwd='/Users/sac/knhk'
        )

        count = int(result.stdout.strip())
        return count
    except Exception as e:
        print(f"Warning: Could not count unimplemented: {e}", file=sys.stderr)
        return 0

def main():
    """Collect all code quality metrics."""
    import sys

    # Collect defect counts
    metrics = {
        'clippy_errors': count_clippy_errors(),
        'clippy_warnings': count_clippy_warnings(),
        'unwrap_production': count_unwrap_in_production(),
        'println_production': count_println_in_production(),
        'unimplemented': count_unimplemented()
    }

    # Calculate weighted defect count
    weights = {
        'clippy_errors': 10,
        'unimplemented': 10,
        'unwrap_production': 5,
        'println_production': 5,
        'clippy_warnings': 3
    }

    weighted_count = sum(metrics[k] * weights.get(k, 1) for k in metrics)

    # Categorize defects
    critical = metrics['clippy_errors'] + metrics['unimplemented']
    high = metrics['unwrap_production'] + metrics['println_production']
    medium = metrics['clippy_warnings']
    low = 0  # Placeholder for future low-priority defects

    output = {
        'timestamp': subprocess.run(['date', '-u', '+%Y-%m-%dT%H:%M:%SZ'],
                                   capture_output=True, text=True).stdout.strip(),
        'metrics': metrics,
        'categories': {
            'critical': critical,
            'high': high,
            'medium': medium,
            'low': low
        },
        'weighted_total': weighted_count
    }

    print(json.dumps(output, indent=2))

    return 0

if __name__ == '__main__':
    exit(main())

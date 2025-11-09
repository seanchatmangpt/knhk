#!/usr/bin/env python3
"""
Update p-Chart for Weaver Validation Pass Rate

Tracks proportion of Weaver validation failures.
"""

import argparse
import pandas as pd
import numpy as np
from pathlib import Path
from datetime import datetime

def calculate_control_limits(df, p_bar=None):
    """Calculate p-chart control limits."""
    if p_bar is None:
        # Calculate from historical data
        if len(df) < 20:
            # Use baseline (target 0% defects, but allow 1% for chart stability)
            p_bar = 0.01
        else:
            total_failures = df['failures'].sum()
            total_validations = df['validations'].sum()
            p_bar = total_failures / total_validations if total_validations > 0 else 0.01

    n_bar = df['validations'].mean() if len(df) > 0 else 75

    # p-chart control limits: p̄ ± 3√(p̄(1-p̄)/n̄)
    std_dev = np.sqrt(p_bar * (1 - p_bar) / n_bar)

    ucl = min(1.0, p_bar + 3 * std_dev)  # Cannot exceed 100%
    cl = p_bar
    lcl = max(0.0, p_bar - 3 * std_dev)  # Cannot be negative

    return ucl, cl, lcl, n_bar

def main():
    parser = argparse.ArgumentParser(description='Update p-chart for Weaver validation')
    parser.add_argument('--result', required=True, choices=['PASS', 'FAIL'], help='Validation result')
    parser.add_argument('--validations', type=int, required=True, help='Total validations run')
    parser.add_argument('--failures', type=int, default=0, help='Number of failures')
    parser.add_argument('--output-dir', default='docs/evidence/spc/weaver', help='Output directory')
    args = parser.parse_args()

    # Determine failures from result
    failures = args.failures if args.result == 'FAIL' else 0

    # Calculate proportion
    p = failures / args.validations if args.validations > 0 else 0

    print(f"📊 Weaver Validation Metrics:")
    print(f"  Result: {args.result}")
    print(f"  Validations: {args.validations}")
    print(f"  Failures: {failures}")
    print(f"  Proportion (p): {p:.4f} ({p*100:.2f}%)")

    # Update chart
    timestamp = datetime.utcnow().isoformat() + 'Z'
    year_month = datetime.utcnow().strftime('%Y_%m')

    chart_file = Path(args.output_dir) / f'p_chart_{year_month}.csv'
    Path(args.output_dir).mkdir(parents=True, exist_ok=True)

    # Create or load chart data
    if not chart_file.exists():
        df = pd.DataFrame(columns=['timestamp', 'validations', 'failures', 'p', 'ucl', 'cl', 'lcl'])
        df.to_csv(chart_file, index=False)

    df = pd.read_csv(chart_file)

    # Calculate control limits
    ucl, cl, lcl, n_bar = calculate_control_limits(df)

    # Append new data point
    new_row = {
        'timestamp': timestamp,
        'validations': args.validations,
        'failures': failures,
        'p': p,
        'ucl': ucl,
        'cl': cl,
        'lcl': lcl
    }

    df = pd.concat([df, pd.DataFrame([new_row])], ignore_index=True)
    df.to_csv(chart_file, index=False)

    print(f"\n📈 p-Chart Updated:")
    print(f"  UCL: {ucl:.4f} ({ucl*100:.2f}%)")
    print(f"  CL:  {cl:.4f} ({cl*100:.2f}%)")
    print(f"  LCL: {lcl:.4f} ({lcl*100:.2f}%)")

    # Check for out of control
    if p > ucl:
        print(f"\n🚨 OUT OF CONTROL: Proportion {p:.4f} exceeds UCL {ucl:.4f}")
        print(f"   CRITICAL: Weaver validation failure rate too high!")
        return 2

    # Check for any failures (zero-defect policy)
    if failures > 0:
        print(f"\n⚠️  WEAVER VALIDATION FAILURES DETECTED: {failures}")
        print(f"   Target: 0 failures (100% pass rate)")
        print(f"   Immediate investigation required!")

        # Log failure details
        failure_log = Path(args.output_dir) / f'failure_log_{datetime.utcnow().strftime("%Y_%m_%d")}.json'
        with open(failure_log, 'a') as f:
            import json
            json.dump({
                'timestamp': timestamp,
                'validations': args.validations,
                'failures': failures,
                'proportion': p
            }, f)
            f.write('\n')

        return 1

    print("\n✅ Weaver validation: 100% pass rate maintained")
    return 0

if __name__ == '__main__':
    exit(main())

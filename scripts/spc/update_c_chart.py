#!/usr/bin/env python3
"""
Update c-Chart for Code Quality Defect Count

Tracks count of defects per inspection.
"""

import argparse
import pandas as pd
import numpy as np
from pathlib import Path
from datetime import datetime
import json

def calculate_control_limits(df, c_bar=None):
    """Calculate c-chart control limits."""
    if c_bar is None:
        if len(df) < 20:
            # Use baseline (from MEASURE phase after cleanup)
            c_bar = 5.0
        else:
            c_bar = df['c'].mean()

    # c-chart control limits: c̄ ± 3√c̄ (Poisson distribution)
    std_dev = np.sqrt(c_bar)

    ucl = c_bar + 3 * std_dev
    cl = c_bar
    lcl = max(0, c_bar - 3 * std_dev)  # Cannot be negative

    return ucl, cl, lcl

def main():
    parser = argparse.ArgumentParser(description='Update c-chart for code quality')
    parser.add_argument('--data', required=True, help='JSON file with code quality metrics')
    parser.add_argument('--output-dir', default='docs/evidence/spc/code_quality', help='Output directory')
    args = parser.parse_args()

    # Load code quality data
    with open(args.data, 'r') as f:
        data = json.load(f)

    weighted_total = data['weighted_total']
    categories = data['categories']

    print(f"📊 Code Quality Metrics:")
    print(f"  Weighted Defects: {weighted_total}")
    print(f"  Critical: {categories['critical']}")
    print(f"  High: {categories['high']}")
    print(f"  Medium: {categories['medium']}")
    print(f"  Low: {categories['low']}")

    # Update chart
    timestamp = data['timestamp']
    year_month = datetime.fromisoformat(timestamp.replace('Z', '+00:00')).strftime('%Y_%m')

    chart_file = Path(args.output_dir) / f'c_chart_{year_month}.csv'
    detail_file = Path(args.output_dir) / f'defect_detail_{datetime.utcnow().strftime("%Y_%m_%d")}.json'

    Path(args.output_dir).mkdir(parents=True, exist_ok=True)

    # Create or load chart data
    if not chart_file.exists():
        df = pd.DataFrame(columns=['timestamp', 'c', 'c_critical', 'c_high', 'c_medium', 'c_low', 'ucl', 'cl', 'lcl'])
        df.to_csv(chart_file, index=False)

    df = pd.read_csv(chart_file)

    # Calculate control limits
    ucl, cl, lcl = calculate_control_limits(df)

    # Append new data point
    new_row = {
        'timestamp': timestamp,
        'c': weighted_total,
        'c_critical': categories['critical'],
        'c_high': categories['high'],
        'c_medium': categories['medium'],
        'c_low': categories['low'],
        'ucl': ucl,
        'cl': cl,
        'lcl': lcl
    }

    df = pd.concat([df, pd.DataFrame([new_row])], ignore_index=True)
    df.to_csv(chart_file, index=False)

    print(f"\n📈 c-Chart Updated:")
    print(f"  UCL: {ucl:.1f}")
    print(f"  CL:  {cl:.1f}")
    print(f"  LCL: {lcl:.1f}")

    # Save detailed metrics
    with open(detail_file, 'w') as f:
        json.dump(data, f, indent=2)

    # Check for out of control
    if weighted_total > ucl:
        print(f"\n🚨 OUT OF CONTROL: Defect count {weighted_total} exceeds UCL {ucl:.1f}")
        print(f"   Code quality regression detected!")
        return 2

    # Check for critical defects (zero-tolerance)
    if categories['critical'] > 0:
        print(f"\n🔴 CRITICAL DEFECTS DETECTED: {categories['critical']}")
        print(f"   Immediate fix required!")
        return 1

    # Check for trend
    if len(df) >= 5:
        last_5 = df.tail(5)
        if (last_5['c'].diff().dropna() > 0).all():
            print(f"\n⚠️  TREND DETECTED: Defect count increasing for 5 consecutive inspections")
            print(f"   Technical debt accumulating!")
            return 1

    print("\n✅ Code quality within control limits")
    return 0

if __name__ == '__main__':
    exit(main())

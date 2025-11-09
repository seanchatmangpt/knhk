#!/usr/bin/env python3
"""
Update X-bar and R Control Charts for Performance Monitoring

Collects performance test results and updates SPC control charts.
"""

import argparse
import pandas as pd
import numpy as np
from pathlib import Path
from datetime import datetime
import json
import re

def parse_performance_results(results_file):
    """Parse performance test results from make test-performance-v04 output."""
    with open(results_file, 'r') as f:
        content = f.read()

    # Extract tick counts for each operation
    # Expected format: "operation_name: X ticks"
    pattern = r'(\w+):\s+(\d+(?:\.\d+)?)\s+ticks'
    matches = re.findall(pattern, content)

    operations = {}
    for op, ticks in matches:
        operations[op] = float(ticks)

    return operations

def calculate_xbar_r(operations_dict):
    """Calculate X-bar (mean) and R (range) from operations."""
    tick_counts = list(operations_dict.values())

    x_bar = np.mean(tick_counts)
    r = max(tick_counts) - min(tick_counts)

    return x_bar, r, tick_counts

def update_chart(chart_file, timestamp, value, subgroup_data=None):
    """Update SPC chart CSV with new data point."""
    # Create file with headers if doesn't exist
    if not Path(chart_file).exists():
        df = pd.DataFrame(columns=['timestamp', 'value', 'ucl', 'cl', 'lcl'])
        df.to_csv(chart_file, index=False)

    # Load existing data
    df = pd.read_csv(chart_file)

    # Calculate control limits (recalculate based on historical data)
    if len(df) > 20:  # Need enough data for meaningful control limits
        process_mean = df['value'].mean()
        process_std = df['value'].std()

        # For X-bar chart: UCL = X̿ + A₂ * R̄ (using simplified 3σ approach)
        ucl = process_mean + 3 * process_std
        cl = process_mean
        lcl = max(0, process_mean - 3 * process_std)  # Cannot be negative for ticks
    else:
        # Initial baseline (from MEASURE phase)
        ucl = 7.2
        cl = 6.1
        lcl = 5.0

    # Append new data point
    new_row = {
        'timestamp': timestamp,
        'value': value,
        'ucl': ucl,
        'cl': cl,
        'lcl': lcl
    }

    if subgroup_data:
        new_row['subgroup_data'] = json.dumps(subgroup_data)

    df = pd.concat([df, pd.DataFrame([new_row])], ignore_index=True)
    df.to_csv(chart_file, index=False)

    return ucl, cl, lcl

def check_western_electric_rules(df):
    """Check Western Electric Rules for special causes."""
    alerts = []

    if len(df) < 9:
        return alerts  # Not enough data for most rules

    recent = df.tail(9)
    latest = recent.iloc[-1]

    # Rule 1: Point beyond UCL or LCL
    if latest['value'] > latest['ucl'] or latest['value'] < latest['lcl']:
        alerts.append({
            'rule': 'Rule 1 - Out of Control',
            'severity': 'CRITICAL',
            'description': f"Value {latest['value']} outside control limits [{latest['lcl']:.2f}, {latest['ucl']:.2f}]",
            'timestamp': latest['timestamp']
        })

    # Rule 2: 9 consecutive points on same side of center line
    if len(recent) >= 9:
        above_cl = (recent['value'] > recent['cl']).all()
        below_cl = (recent['value'] < recent['cl']).all()
        if above_cl or below_cl:
            alerts.append({
                'rule': 'Rule 2 - Shift',
                'severity': 'HIGH',
                'description': f"9 consecutive points on {'above' if above_cl else 'below'} center line",
                'timestamp': latest['timestamp']
            })

    # Rule 3: 6 consecutive points increasing or decreasing
    if len(recent) >= 6:
        last_6 = recent.tail(6)
        increasing = (last_6['value'].diff().dropna() > 0).all()
        decreasing = (last_6['value'].diff().dropna() < 0).all()
        if increasing or decreasing:
            alerts.append({
                'rule': 'Rule 3 - Trend',
                'severity': 'HIGH',
                'description': f"6 consecutive points {'increasing' if increasing else 'decreasing'}",
                'timestamp': latest['timestamp']
            })

    return alerts

def main():
    parser = argparse.ArgumentParser(description='Update X-bar and R control charts')
    parser.add_argument('--results', required=True, help='Performance test results file')
    parser.add_argument('--output-dir', default='docs/evidence/spc/performance', help='Output directory')
    args = parser.parse_args()

    # Parse performance results
    operations = parse_performance_results(args.results)

    if not operations:
        print("❌ No performance data found in results file")
        return 1

    # Calculate X-bar and R
    x_bar, r, tick_counts = calculate_xbar_r(operations)

    print(f"📊 Performance Metrics:")
    print(f"  X-bar (Mean): {x_bar:.2f} ticks")
    print(f"  R (Range): {r:.2f} ticks")
    print(f"  Operations: {len(operations)}")

    # Update charts
    timestamp = datetime.utcnow().isoformat() + 'Z'
    year_month = datetime.utcnow().strftime('%Y_%m')

    xbar_file = Path(args.output_dir) / f'x_bar_chart_{year_month}.csv'
    r_file = Path(args.output_dir) / f'r_chart_{year_month}.csv'

    Path(args.output_dir).mkdir(parents=True, exist_ok=True)

    # Update X-bar chart
    ucl_x, cl_x, lcl_x = update_chart(xbar_file, timestamp, x_bar, tick_counts)
    print(f"\n📈 X-bar Chart Updated:")
    print(f"  Value: {x_bar:.2f}")
    print(f"  UCL: {ucl_x:.2f}, CL: {cl_x:.2f}, LCL: {lcl_x:.2f}")

    # Update R chart
    ucl_r, cl_r, lcl_r = update_chart(r_file, timestamp, r)
    print(f"\n📊 R Chart Updated:")
    print(f"  Value: {r:.2f}")
    print(f"  UCL: {ucl_r:.2f}, CL: {cl_r:.2f}, LCL: {lcl_r:.2f}")

    # Check for special causes
    df_xbar = pd.read_csv(xbar_file)
    alerts = check_western_electric_rules(df_xbar)

    if alerts:
        print(f"\n🚨 SPECIAL CAUSES DETECTED ({len(alerts)}):")
        for alert in alerts:
            print(f"  [{alert['severity']}] {alert['rule']}: {alert['description']}")
        return 2  # Special cause detected
    else:
        print("\n✅ No special causes detected. Process is in control.")

    # Check specification limits (8-tick rule)
    violations = [op for op, ticks in operations.items() if ticks > 8.0]
    if violations:
        print(f"\n⚠️  SPECIFICATION VIOLATIONS:")
        for op in violations:
            print(f"  {op}: {operations[op]:.2f} ticks (>8.0 limit)")
        return 3  # Spec violation

    print("\n✅ All operations within specification (≤8.0 ticks)")
    return 0

if __name__ == '__main__':
    exit(main())

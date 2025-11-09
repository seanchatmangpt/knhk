#!/usr/bin/env python3
"""
Check All SPC Charts for Special Causes

Scans X-bar, R, p, and c charts for Western Electric rule violations.
"""

import argparse
import pandas as pd
import sys

def check_western_electric_rules(df, chart_name):
    """Check Western Electric Rules for special causes."""
    alerts = []

    if len(df) < 9:
        return alerts  # Not enough data for most rules

    recent = df.tail(9)
    latest = recent.iloc[-1]

    # Rule 1: Point beyond UCL or LCL
    if 'ucl' in latest and 'lcl' in latest:
        if latest['value'] > latest['ucl'] or latest['value'] < latest['lcl']:
            alerts.append({
                'chart': chart_name,
                'rule': 'Rule 1 - Out of Control',
                'severity': 'CRITICAL',
                'description': f"Value {latest['value']:.4f} outside control limits [{latest['lcl']:.4f}, {latest['ucl']:.4f}]",
                'timestamp': latest['timestamp']
            })

    # Rule 2: 9 consecutive points on same side of center line
    if len(recent) >= 9 and 'cl' in latest:
        above_cl = (recent['value'] > recent['cl']).all()
        below_cl = (recent['value'] < recent['cl']).all()
        if above_cl or below_cl:
            alerts.append({
                'chart': chart_name,
                'rule': 'Rule 2 - Shift',
                'severity': 'HIGH',
                'description': f"9 consecutive points {'above' if above_cl else 'below'} center line",
                'timestamp': latest['timestamp']
            })

    # Rule 3: 6 consecutive points increasing or decreasing
    if len(recent) >= 6:
        last_6 = recent.tail(6)
        increasing = (last_6['value'].diff().dropna() > 0).all()
        decreasing = (last_6['value'].diff().dropna() < 0).all()
        if increasing or decreasing:
            alerts.append({
                'chart': chart_name,
                'rule': 'Rule 3 - Trend',
                'severity': 'HIGH',
                'description': f"6 consecutive points {'increasing' if increasing else 'decreasing'}",
                'timestamp': latest['timestamp']
            })

    return alerts

def check_chart(chart_file, chart_name, value_col='value'):
    """Check a single chart for special causes."""
    try:
        df = pd.read_csv(chart_file)

        if len(df) == 0:
            print(f"⚠️  {chart_name}: No data")
            return []

        # Rename column to 'value' if needed
        if value_col != 'value' and value_col in df.columns:
            df['value'] = df[value_col]

        alerts = check_western_electric_rules(df, chart_name)

        if alerts:
            print(f"🚨 {chart_name}: {len(alerts)} special cause(s) detected")
        else:
            print(f"✅ {chart_name}: In control")

        return alerts

    except FileNotFoundError:
        print(f"⚠️  {chart_name}: File not found ({chart_file})")
        return []
    except Exception as e:
        print(f"❌ {chart_name}: Error checking chart: {e}")
        return []

def main():
    parser = argparse.ArgumentParser(description='Check all SPC charts for special causes')
    parser.add_argument('--xbar-chart', required=True, help='X-bar chart CSV file')
    parser.add_argument('--r-chart', required=True, help='R chart CSV file')
    parser.add_argument('--p-chart', required=True, help='p-chart CSV file')
    parser.add_argument('--c-chart', required=True, help='c-chart CSV file')
    args = parser.parse_args()

    print("📊 Checking SPC Charts for Special Causes...\n")

    all_alerts = []

    # Check each chart
    all_alerts.extend(check_chart(args.xbar_chart, "X-bar (Performance Mean)"))
    all_alerts.extend(check_chart(args.r_chart, "R (Performance Range)"))
    all_alerts.extend(check_chart(args.p_chart, "p (Weaver Validation)", value_col='p'))
    all_alerts.extend(check_chart(args.c_chart, "c (Code Quality)", value_col='c'))

    # Summary
    print(f"\n{'='*60}")
    print(f"SUMMARY: {len(all_alerts)} special cause(s) detected")
    print(f"{'='*60}")

    if all_alerts:
        # Group by severity
        critical = [a for a in all_alerts if a['severity'] == 'CRITICAL']
        high = [a for a in all_alerts if a['severity'] == 'HIGH']

        if critical:
            print(f"\n🔴 CRITICAL ALERTS ({len(critical)}):")
            for alert in critical:
                print(f"  [{alert['chart']}] {alert['rule']}")
                print(f"    {alert['description']}")
                print(f"    Timestamp: {alert['timestamp']}")

        if high:
            print(f"\n🟡 HIGH PRIORITY ALERTS ({len(high)}):")
            for alert in high:
                print(f"  [{alert['chart']}] {alert['rule']}")
                print(f"    {alert['description']}")
                print(f"    Timestamp: {alert['timestamp']}")

        print(f"\n⚠️  Special causes require investigation (see SOP-002)")
        print(f"   Create GitHub issue with tag 'spc-alert'")

        return 1  # Special causes detected
    else:
        print("\n✅ All charts in control. No special causes detected.")
        return 0

if __name__ == '__main__':
    exit(main())

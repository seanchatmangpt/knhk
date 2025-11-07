#!/bin/bash
# Flow Efficiency Metrics Dashboard
# Calculates Lean flow metrics from git history and workspace state

set -e

echo "📊 Flow Efficiency Metrics Dashboard"
echo "======================================"
echo ""

# Configuration
DAYS_BACK=7
METRICS_FILE=".flow-metrics.csv"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper function: Format duration
format_duration() {
  local seconds=$1
  local hours=$((seconds / 3600))
  local minutes=$(((seconds % 3600) / 60))
  echo "${hours}h ${minutes}m"
}

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📈 WORK IN PROGRESS (WIP) ANALYSIS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Metric 1: Work In Progress (WIP)
# Count active branches (excluding main)
wip=$(git branch 2>/dev/null | grep -v -E '(main|master|\*)' | wc -l | tr -d ' ')
wip_limit=2

echo -n "WIP (active branches):        "
if [ "$wip" -le "$wip_limit" ]; then
  echo -e "${GREEN}$wip / $wip_limit${NC} ✅ (within limit)"
else
  echo -e "${RED}$wip / $wip_limit${NC} ⚠️  (EXCEEDS LIMIT - finish tasks!)"
fi

# Show active branches
if [ "$wip" -gt 0 ]; then
  echo ""
  echo "Active branches:"
  git branch 2>/dev/null | grep -v -E '(main|master|\*)' | sed 's/^/  - /'
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "⏱️  TIMING ANALYSIS (Last $DAYS_BACK Days)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Metric 2: Lead Time (time from commit to commit)
# Average time between commits
commits=$(git log --since="$DAYS_BACK days ago" --format="%ct" | sort -n)
commit_count=$(echo "$commits" | wc -l | tr -d ' ')

if [ "$commit_count" -gt 1 ]; then
  first_commit=$(echo "$commits" | head -1)
  last_commit=$(echo "$commits" | tail -1)
  total_time=$((last_commit - first_commit))
  avg_lead_time=$((total_time / (commit_count - 1)))

  echo "Lead Time (avg):              $(format_duration $avg_lead_time)"

  # Compare to baseline (120 hours = 432000 seconds)
  baseline_lead=432000
  improvement=$(((baseline_lead - avg_lead_time) * 100 / baseline_lead))
  if [ $improvement -gt 0 ]; then
    echo -e "  Improvement vs baseline:    ${GREEN}-${improvement}%${NC} (was 120h)"
  else
    echo -e "  Improvement vs baseline:    ${RED}+${improvement}%${NC} (was 120h)"
  fi
else
  echo "Lead Time (avg):              N/A (need more commits)"
fi

echo ""

# Metric 3: Cycle Time (time commits are authored within work hours)
# Approximate as time between consecutive commits by same author
author=$(git config user.name)
author_commits=$(git log --since="$DAYS_BACK days ago" --author="$author" --format="%ct" | sort -n)
author_count=$(echo "$author_commits" | wc -l | tr -d ' ')

if [ "$author_count" -gt 1 ]; then
  total_cycle=0
  prev_time=$(echo "$author_commits" | head -1)

  while IFS= read -r commit_time; do
    if [ "$commit_time" != "$prev_time" ]; then
      cycle=$((commit_time - prev_time))
      # Cap at 8 hours (28800s) to exclude overnight gaps
      if [ $cycle -lt 28800 ]; then
        total_cycle=$((total_cycle + cycle))
      fi
      prev_time=$commit_time
    fi
  done <<< "$author_commits"

  avg_cycle_time=$((total_cycle / (author_count - 1)))
  echo "Cycle Time (avg):             $(format_duration $avg_cycle_time)"

  # Compare to baseline (12 hours = 43200 seconds)
  baseline_cycle=43200
  improvement=$(((baseline_cycle - avg_cycle_time) * 100 / baseline_cycle))
  if [ $improvement -gt 0 ]; then
    echo -e "  Improvement vs baseline:    ${GREEN}-${improvement}%${NC} (was 12h)"
  else
    echo -e "  Improvement vs baseline:    ${RED}+${improvement}%${NC} (was 12h)"
  fi
else
  echo "Cycle Time (avg):             N/A (need more commits)"
fi

echo ""

# Metric 4: Flow Efficiency = (Cycle Time / Lead Time) * 100%
if [ "$commit_count" -gt 1 ] && [ "$author_count" -gt 1 ]; then
  # Avoid division by zero
  if [ "$avg_lead_time" -gt 0 ]; then
    flow_efficiency=$((avg_cycle_time * 100 / avg_lead_time))

    echo -n "Flow Efficiency:              "
    if [ "$flow_efficiency" -ge 80 ]; then
      echo -e "${GREEN}${flow_efficiency}%${NC} ✅ (target: >80%)"
    elif [ "$flow_efficiency" -ge 50 ]; then
      echo -e "${YELLOW}${flow_efficiency}%${NC} ⚠️  (target: >80%)"
    else
      echo -e "${RED}${flow_efficiency}%${NC} ❌ (target: >80%)"
    fi

    # Compare to baseline (12.5%)
    baseline_flow=12
    improvement=$((flow_efficiency - baseline_flow))
    if [ $improvement -gt 0 ]; then
      improvement_pct=$(((flow_efficiency - baseline_flow) * 100 / baseline_flow))
      echo -e "  Improvement vs baseline:    ${GREEN}+${improvement_pct}%${NC} (was 12.5%)"
    fi
  else
    echo "Flow Efficiency:              N/A (lead time zero)"
  fi
else
  echo "Flow Efficiency:              N/A (need more data)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📦 THROUGHPUT ANALYSIS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Metric 5: Throughput (commits per day)
throughput_total=$(git log --since="$DAYS_BACK days ago" --oneline 2>/dev/null | wc -l | tr -d ' ')
throughput_per_day=$(echo "scale=2; $throughput_total / $DAYS_BACK" | bc)

echo "Throughput (last $DAYS_BACK days):   $throughput_total commits"
echo "  Per day average:          $throughput_per_day commits/day"

# Target: >5 tasks/day
target_throughput=5
if (( $(echo "$throughput_per_day >= $target_throughput" | bc -l) )); then
  echo -e "  Status:                   ${GREEN}✅ Exceeds target${NC} (>5/day)"
else
  echo -e "  Status:                   ${YELLOW}⚠️  Below target${NC} (>5/day)"
fi

echo ""

# Metric 6: Task completion from metrics file
if [ -f "$METRICS_FILE" ]; then
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "✅ COMPLETED TASKS (from flow tracking)"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  # Count completed tasks in last N days
  cutoff_time=$(($(date +%s) - (DAYS_BACK * 86400)))
  completed=$(awk -F',' -v cutoff="$cutoff_time" '$1 >= cutoff && $3 == "completed" {print $2}' "$METRICS_FILE" 2>/dev/null | wc -l | tr -d ' ')

  echo "Tasks completed:              $completed tasks"
  echo ""

  # Show recent completions
  echo "Recent completions:"
  awk -F',' -v cutoff="$cutoff_time" '$1 >= cutoff && $3 == "completed" {print "  - " $2 " (" strftime("%Y-%m-%d %H:%M", $1) ")"}' "$METRICS_FILE" 2>/dev/null | tail -5
else
  echo "No task tracking file found (.flow-metrics.csv)"
  echo "Tasks will be tracked as flow-agent.sh is used"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🎯 RECOMMENDATIONS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Generate recommendations based on metrics
if [ "$wip" -gt "$wip_limit" ]; then
  echo -e "${RED}🚨 CRITICAL:${NC} WIP exceeds limit ($wip > $wip_limit)"
  echo "   → Finish current tasks before starting new ones"
  echo "   → Review KANBAN.md for priority tasks"
  echo ""
fi

if [ "$commit_count" -gt 1 ] && [ "$flow_efficiency" -lt 80 ]; then
  echo -e "${YELLOW}⚠️  WARNING:${NC} Flow efficiency below target ($flow_efficiency% < 80%)"
  echo "   → Reduce WIP to increase flow efficiency"
  echo "   → Eliminate waiting/batching waste"
  echo "   → Use single-piece flow (./scripts/flow-agent.sh)"
  echo ""
fi

if (( $(echo "$throughput_per_day < $target_throughput" | bc -l) )); then
  echo -e "${YELLOW}⚠️  WARNING:${NC} Throughput below target ($throughput_per_day < $target_throughput tasks/day)"
  echo "   → Break tasks into smaller pieces"
  echo "   → Reduce task cycle time"
  echo "   → Eliminate blockers and waste"
  echo ""
fi

echo -e "${GREEN}✅ Good practices:${NC}"
echo "   → Use ./scripts/flow-agent.sh for single-piece flow"
echo "   → Maintain WIP ≤ 2 (check docs/KANBAN.md)"
echo "   → Finish tasks before starting new ones"
echo "   → Run this script daily to track improvement"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary: Flow Efficiency at $(date +%Y-%m-%d)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

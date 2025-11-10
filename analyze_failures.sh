#!/bin/bash

# Analyze failure patterns from 100 runs
echo "========================================="
echo "Failure Pattern Analysis (100 runs)"
echo "========================================="
echo ""

# Get failure data
failures=$(cat /tmp/aggregated_failures.txt)

# Group by failure type
echo "## Grouping failures by category..."
echo ""

declare -A categories
categories["space_gap"]=0
categories["rounded"]=0
categories["touch"]=0
categories["break"]=0
categories["divide"]=0
categories["snap"]=0
categories["other"]=0

while read count pattern; do
    if [[ $pattern =~ space.*gap|gap.*space ]]; then
        categories["space_gap"]=$((categories["space_gap"] + count))
    elif [[ $pattern =~ rounded ]]; then
        categories["rounded"]=$((categories["rounded"] + count))
    elif [[ $pattern =~ touch ]]; then
        categories["touch"]=$((categories["touch"] + count))
    elif [[ $pattern =~ break ]]; then
        categories["break"]=$((categories["break"] + count))
    elif [[ $pattern =~ divide ]]; then
        categories["divide"]=$((categories["divide"] + count))
    elif [[ $pattern =~ snap ]]; then
        categories["snap"]=$((categories["snap"] + count))
    else
        categories["other"]=$((categories["other"] + count))
    fi
done < /tmp/aggregated_failures.txt

total=0
for cat in "${!categories[@]}"; do
    total=$((total + ${categories[$cat]}))
done

echo "Category | Count | % of Failures"
echo "---------|-------|---------------"
for cat in space_gap rounded touch divide break snap other; do
    count=${categories[$cat]}
    pct=$(echo "scale=1; $count * 100 / $total" | bc)
    printf "%-15s | %5d | %5s%%\n" "$cat" "$count" "$pct"
done

echo ""
echo "Total failure instances: $total"
echo "Total tests: 10000"
echo "Overall failure rate: $(echo "scale=2; $total * 100 / 10000" | bc)%"

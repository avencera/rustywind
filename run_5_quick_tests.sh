#!/bin/bash

echo "Running 5 quick fuzz tests..."
cd /home/user/rustywind/tests/fuzz

declare -a rates

for i in {1..5}; do
    echo "Run $i/5..."
    output=$(npm test 2>&1)

    passed=$(echo "$output" | grep -oP '📊 Results: \K\d+(?= passed)')
    rate=$(echo "$output" | grep -oP '\(\K[\d.]+(?=% pass rate)')
    seed=$(echo "$output" | grep -oP '🎲 Seed: \K\w+' | head -1)

    rates+=("$rate")
    echo "  Result: $passed/100 ($rate%) - Seed: $seed"
done

echo ""
echo "=== SUMMARY ==="
sum=0
for rate in "${rates[@]}"; do
    sum=$(echo "$sum + $rate" | bc)
done
avg=$(echo "scale=1; $sum / 5" | bc)

echo "Pass rates: ${rates[*]}"
echo "Average: $avg%"
echo "Baseline: 94%"
echo "Previous (with regression): 85.7%"

if [ $(echo "$avg > 94" | bc) -eq 1 ]; then
    echo "Status: ✅ ABOVE BASELINE! (+$(echo "$avg - 94" | bc)%)"
else
    echo "Status: ⚠️ Below baseline"
fi

#!/bin/bash

echo "========================================="
echo "Running 10 Fuzz Test Runs"
echo "========================================="
echo ""

cd /home/user/rustywind/tests/fuzz

declare -a pass_rates
declare -a pass_counts
declare -a fail_counts
declare -a seeds

for i in {1..10}; do
    echo "Run $i/10..."
    output=$(npm test 2>&1)

    # Extract pass/fail from line like: "📊 Results: 85 passed, 15 failed (85.0% pass rate)"
    passed=$(echo "$output" | grep -oP '📊 Results: \K\d+(?= passed)')
    failed=$(echo "$output" | grep -oP '\d+ passed, \K\d+(?= failed)')
    rate=$(echo "$output" | grep -oP '\(\K[\d.]+(?=% pass rate)')
    seed=$(echo "$output" | grep -oP '🎲 Seed: \K\w+' | head -1)

    pass_rates+=("$rate")
    pass_counts+=("$passed")
    fail_counts+=("$failed")
    seeds+=("$seed")

    echo "  Result: $passed/100 passed ($rate%) - Seed: $seed"
    echo ""
done

echo "========================================="
echo "SUMMARY"
echo "========================================="
echo ""

# Calculate average
sum=0
for rate in "${pass_rates[@]}"; do
    sum=$(echo "$sum + $rate" | bc)
done
avg=$(echo "scale=1; $sum / 10" | bc)

echo "Individual pass rates: ${pass_rates[*]}"
echo "Average pass rate: $avg%"
echo ""

echo "Detailed Results:"
for i in {0..9}; do
    run=$((i + 1))
    echo "  Run $run: ${pass_counts[$i]}/100 (${pass_rates[$i]}%) - Seed: ${seeds[$i]}"
done

echo ""
echo "Baseline: ~94% (before fixes)"
echo "Current:  $avg% (after fixes)"

if [ $(echo "$avg < 94" | bc) -eq 1 ]; then
    echo "Status: ⚠️  Below baseline - investigation needed"
elif [ $(echo "$avg >= 94 && $avg < 96" | bc) -eq 1 ]; then
    echo "Status: ✓ At baseline"
elif [ $(echo "$avg >= 96" | bc) -eq 1 ]; then
    echo "Status: ✅ Improvement achieved!"
fi

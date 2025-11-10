#!/bin/bash

echo "========================================="
echo "Running 25 Fuzz Test Runs"
echo "========================================="
echo ""

declare -a pass_rates
declare -a pass_counts
declare -a fail_counts
declare -a seeds

# Create failure log
FAILURE_LOG="fuzz_25run_failures.txt"
rm -f "$FAILURE_LOG"
touch "$FAILURE_LOG"

for i in {1..25}; do
    echo "Run $i/25..."
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

    # Save failures to log
    if [ "$failed" -gt 0 ]; then
        echo "" >> "$FAILURE_LOG"
        echo "=== Run $i - Seed: $seed - Failed: $failed ===" >> "$FAILURE_LOG"
        echo "$output" | grep -A 8 "Test #" >> "$FAILURE_LOG"
    fi

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
avg=$(echo "scale=2; $sum / 25" | bc)

echo "Individual pass rates:"
for i in {0..24}; do
    run=$((i + 1))
    printf "  Run %2d: %3s/100 (%5s%%) - Seed: %s\n" "$run" "${pass_counts[$i]}" "${pass_rates[$i]}" "${seeds[$i]}"
done

echo ""
echo "Average pass rate: $avg%"
echo "Failures logged to: $FAILURE_LOG"
echo ""
echo "Baseline: ~94% (before fixes)"
echo "Current:  $avg% (after fixes)"

if [ $(echo "$avg < 94" | bc) -eq 1 ]; then
    echo "Status: ⚠️  Below baseline - investigation needed"
elif [ $(echo "$avg >= 94 && $avg < 96" | bc) -eq 1 ]; then
    echo "Status: ✓ At/near baseline"
elif [ $(echo "$avg >= 96" | bc) -eq 1 ]; then
    echo "Status: ✅ Improvement achieved!"
fi

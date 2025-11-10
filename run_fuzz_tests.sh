#!/bin/bash

# Run fuzz tests 10 times and collect results
echo "Running fuzz tests 10 times with different seeds..."
echo "Start time: $(date)"
echo ""

cd tests/fuzz

results=()
pass_counts=()
fail_counts=()
seeds=()

for i in {1..10}; do
    echo "========================================="
    echo "Run $i/10"
    echo "========================================="

    # Run the test and capture output
    output=$(npm test 2>&1)

    # Extract pass/fail counts and seed
    pass=$(echo "$output" | grep -oP '✅ \K\d+(?= pass)' | tail -1)
    fail=$(echo "$output" | grep -oP '❌ \K\d+(?= fail)' | tail -1)
    seed=$(echo "$output" | grep -oP '🎲 Seed: \K\w+' | tail -1)

    if [ -z "$pass" ]; then pass=0; fi
    if [ -z "$fail" ]; then fail=0; fi

    total=$((pass + fail))
    if [ $total -eq 0 ]; then total=100; fi

    pass_rate=$((pass * 100 / total))

    echo "Pass: $pass | Fail: $fail | Rate: ${pass_rate}% | Seed: $seed"
    echo ""

    results+=("$pass_rate")
    pass_counts+=("$pass")
    fail_counts+=("$fail")
    seeds+=("$seed")

    # Small delay between runs
    sleep 1
done

echo "========================================="
echo "SUMMARY OF 10 RUNS"
echo "========================================="

# Calculate average
total_rate=0
for rate in "${results[@]}"; do
    total_rate=$((total_rate + rate))
done
avg_rate=$((total_rate / 10))

echo "Pass rates: ${results[*]}"
echo "Average pass rate: ${avg_rate}%"
echo ""

# Show individual results
echo "Detailed results:"
for i in {0..9}; do
    echo "Run $((i+1)): ${pass_counts[$i]}/100 passed (${results[$i]}%) - Seed: ${seeds[$i]}"
done

echo ""
echo "End time: $(date)"

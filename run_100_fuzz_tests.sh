#!/bin/bash

echo "========================================="
echo "Running 100 Fuzz Test Iterations"
echo "========================================="
echo "Start time: $(date)"
echo ""

cd /home/user/rustywind/tests/fuzz

# Arrays to store all data
declare -a pass_rates
declare -a pass_counts
declare -a fail_counts
declare -a seeds
declare -A failure_patterns

total_tests=0
total_passed=0
total_failed=0

for i in {1..100}; do
    if [ $((i % 10)) -eq 0 ]; then
        echo "Progress: $i/100 completed..."
    fi

    output=$(npm test 2>&1)

    # Extract metrics
    passed=$(echo "$output" | grep -oP '📊 Results: \K\d+(?= passed)')
    failed=$(echo "$output" | grep -oP '\d+ passed, \K\d+(?= failed)')
    rate=$(echo "$output" | grep -oP '\(\K[\d.]+(?=% pass rate)')
    seed=$(echo "$output" | grep -oP '🎲 Seed: \K\w+' | head -1)

    pass_rates+=("$rate")
    pass_counts+=("$passed")
    fail_counts+=("$failed")
    seeds+=("$seed")

    total_tests=$((total_tests + 100))
    total_passed=$((total_passed + passed))
    total_failed=$((total_failed + failed))

    # Extract failure patterns
    echo "$output" | grep -A 1 "Mismatch at position" | grep "Prettier=" | while read line; do
        # Extract the classes that are misordered
        prettier=$(echo "$line" | grep -oP 'Prettier="\K[^"]+')
        rusty=$(echo "$line" | grep -oP 'RustyWind="\K[^"]+')

        if [ -n "$prettier" ] && [ -n "$rusty" ]; then
            echo "$prettier vs $rusty" >> /tmp/failure_patterns_${i}.txt
        fi
    done
done

echo ""
echo "========================================="
echo "100-RUN SUMMARY"
echo "========================================="

# Calculate statistics
sum=0
min=100
max=0
for rate in "${pass_rates[@]}"; do
    sum=$(echo "$sum + $rate" | bc)
    if [ $(echo "$rate < $min" | bc) -eq 1 ]; then
        min=$rate
    fi
    if [ $(echo "$rate > $max" | bc) -eq 1 ]; then
        max=$rate
    fi
done
avg=$(echo "scale=2; $sum / 100" | bc)

echo "Total tests: $total_tests"
echo "Total passed: $total_passed"
echo "Total failed: $total_failed"
echo "Overall pass rate: $(echo "scale=2; $total_passed * 100 / $total_tests" | bc)%"
echo ""
echo "Average: $avg%"
echo "Min: $min%"
echo "Max: $max%"
echo "Range: $(echo "$max - $min" | bc)%"
echo ""

# Distribution analysis
count_90_plus=0
count_85_90=0
count_80_85=0
count_below_80=0

for rate in "${pass_rates[@]}"; do
    if [ $(echo "$rate >= 95" | bc) -eq 1 ]; then
        ((count_90_plus++))
    elif [ $(echo "$rate >= 90" | bc) -eq 1 ]; then
        ((count_85_90++))
    elif [ $(echo "$rate >= 85" | bc) -eq 1 ]; then
        ((count_80_85++))
    else
        ((count_below_80++))
    fi
done

echo "Distribution:"
echo "  95%+: $count_90_plus runs"
echo "  90-94%: $count_85_90 runs"
echo "  85-89%: $count_80_85 runs"
echo "  <85%: $count_below_80 runs"
echo ""

echo "Baseline: 94%"
echo "Current: $avg%"
echo "Difference: $(echo "$avg - 94" | bc)%"
echo ""

if [ $(echo "$avg >= 94" | bc) -eq 1 ]; then
    echo "Status: ✅ At or above baseline"
else
    echo "Status: ⚠️ Below baseline by $(echo "94 - $avg" | bc)%"
fi

echo ""
echo "End time: $(date)"
echo ""
echo "Analyzing failure patterns..."

# Aggregate all failure patterns
cat /tmp/failure_patterns_*.txt 2>/dev/null | sort | uniq -c | sort -rn > /tmp/aggregated_failures.txt
rm /tmp/failure_patterns_*.txt 2>/dev/null

echo "Top 20 failure patterns saved to /tmp/aggregated_failures.txt"
head -20 /tmp/aggregated_failures.txt

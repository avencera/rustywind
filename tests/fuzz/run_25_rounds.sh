#!/bin/bash
# Run 25 rounds of fuzz testing with improved bug fixes

echo "=========================================="
echo "RustyWind Fuzz Test - 25 Rounds"
echo "Testing with 3 critical bugs fixed:"
echo "  1. Property count comparison"
echo "  2. --tw-ring-inset position"
echo "  3. Group/peer variant ordering"
echo "=========================================="
echo ""

total_tests=0
total_passed=0
total_failed=0

for round in {1..25}; do
  echo "Round $round/25..."

  # Run comparison and capture output
  output=$(node compare.js 2>&1)

  # Extract pass/fail counts - format: "92 passed, 8 failed (92.0% pass rate)"
  passed=$(echo "$output" | grep -oP "\d+ passed" | grep -oP "\d+")
  failed=$(echo "$output" | grep -oP "\d+ failed" | grep -oP "\d+")

  if [[ -n "$passed" && -n "$failed" ]]; then
    tests=$((passed + failed))
    total_tests=$((total_tests + tests))
    total_passed=$((total_passed + passed))
    total_failed=$((total_failed + failed))

    pass_rate=$(echo "scale=1; $passed * 100 / $tests" | bc)
    echo "  Round $round: $passed/$tests passed ($pass_rate%)"
  else
    echo "  Round $round: ERROR - Could not parse results"
  fi
done

echo ""
echo "=========================================="
echo "FINAL RESULTS"
echo "=========================================="
echo "Total rounds: 25"
echo "Total tests run: $total_tests"
echo "Total passed: $total_passed"
echo "Total failed: $total_failed"

if [ $total_tests -gt 0 ]; then
  overall_pass_rate=$(echo "scale=2; $total_passed * 100 / $total_tests" | bc)
  echo "Overall pass rate: $overall_pass_rate%"
  echo ""
  echo "Previous baseline: ~96%"

  # Compare with baseline
  if (( $(echo "$overall_pass_rate > 96" | bc -l) )); then
    echo "✅ IMPROVEMENT! Pass rate increased!"
  elif (( $(echo "$overall_pass_rate >= 95" | bc -l) )); then
    echo "✓ Maintained high pass rate"
  else
    echo "⚠️  Pass rate below baseline - investigate"
  fi
else
  echo "ERROR: No tests completed successfully"
fi
echo "=========================================="

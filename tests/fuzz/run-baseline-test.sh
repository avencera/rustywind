#!/bin/bash
# Run 25 rounds of fuzz tests and collect pass rates

echo "Running 25 rounds of fuzz tests to establish baseline..."
echo "=========================================================="
echo ""

RESULTS_FILE="baseline-results.txt"
> $RESULTS_FILE

total_passed=0
total_failed=0
rounds=25

for i in $(seq 1 $rounds); do
  echo "Round $i/$rounds..."

  # Run the test and capture output
  output=$(npm test 2>&1)

  # Extract pass rate from the output
  # Looking for patterns like "91 passed, 9 failed (91.0% pass rate)"
  passed=$(echo "$output" | grep -oP '\d+(?= passed)' | head -1)
  failed=$(echo "$output" | grep -oP '\d+(?= failed)' | head -1)
  pass_rate=$(echo "$output" | grep -oP '\d+\.\d+(?=% pass rate)' | head -1)

  if [ -n "$passed" ] && [ -n "$failed" ]; then
    total_passed=$((total_passed + passed))
    total_failed=$((total_failed + failed))
    echo "Round $i: $passed passed, $failed failed ($pass_rate%)" | tee -a $RESULTS_FILE
  else
    echo "Round $i: ERROR - Could not parse results" | tee -a $RESULTS_FILE
    echo "$output" | tail -20
  fi

  echo ""
done

echo "" | tee -a $RESULTS_FILE
echo "=========================================================="  | tee -a $RESULTS_FILE
echo "BASELINE RESULTS SUMMARY" | tee -a $RESULTS_FILE
echo "=========================================================="  | tee -a $RESULTS_FILE
echo "" | tee -a $RESULTS_FILE

# Calculate overall statistics
total_tests=$((total_passed + total_failed))
overall_pass_rate=$(echo "scale=2; $total_passed * 100 / $total_tests" | bc)

echo "Total rounds: $rounds" | tee -a $RESULTS_FILE
echo "Total tests run: $total_tests" | tee -a $RESULTS_FILE
echo "Total passed: $total_passed" | tee -a $RESULTS_FILE
echo "Total failed: $total_failed" | tee -a $RESULTS_FILE
echo "Overall pass rate: $overall_pass_rate%" | tee -a $RESULTS_FILE
echo "" | tee -a $RESULTS_FILE
echo "Results saved to: $RESULTS_FILE"

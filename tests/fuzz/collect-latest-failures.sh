#!/bin/bash
for i in {1..30}; do
  result=$(node compare.js 2>&1)
  if echo "$result" | grep -q "❌ Failures:"; then
    echo "=== FAILURE SAMPLE $i ==="
    echo "$result" | grep -A 25 "Test #" | head -30
  fi
done

#!/bin/bash
# Collect sample failures

for i in {1..20}; do
  result=$(node compare.js 2>&1)
  if echo "$result" | grep -q "❌ Failures:"; then
    echo "$result" | grep -A 20 "Test #" | head -80
    echo "=========================================="
  fi
done

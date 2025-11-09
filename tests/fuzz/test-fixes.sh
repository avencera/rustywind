#!/bin/bash

tests=(
  "size-2 h-auto"
  "rounded-none rounded-br-none"
  "select-all space-y-1"
  "pt-2 py-0"
  "border-r-0 border-x-0"
  "row-start-auto bg-opacity-50"
)

cd /home/user/rustywind

for test in "${tests[@]}"; do
  echo "Test: $test"
  result=$(echo "<div class=\"$test\"></div>" | ./target/release/rustywind --stdin 2>&1 | grep -o 'class="[^"]*"' | sed 's/class="//;s/"//')
  echo "  Result: $result"
  echo
done

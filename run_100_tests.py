#!/usr/bin/env python3
import subprocess
import re
import json
from collections import defaultdict

results = []
all_failures = []

for i in range(1, 101):
    print(f"Run {i}/100...", flush=True)
    result = subprocess.run(['npm', 'test'], capture_output=True, text=True, cwd='/home/user/rustywind/tests/fuzz')
    output = result.stdout + result.stderr

    # Extract results
    passed_match = re.search(r'📊 Results: (\d+) passed', output)
    failed_match = re.search(r'(\d+) failed', output)
    rate_match = re.search(r'\((\d+\.?\d*)% pass rate\)', output)
    seed_match = re.search(r'🎲 Seed: (\w+)', output)

    if passed_match and rate_match:
        passed = int(passed_match.group(1))
        failed = int(failed_match.group(1)) if failed_match else 0
        rate = float(rate_match.group(1))
        seed = seed_match.group(1) if seed_match else 'unknown'

        results.append({
            'run': i,
            'passed': passed,
            'failed': failed,
            'rate': rate,
            'seed': seed
        })

        print(f"  Result: {passed}/100 ({rate}%) - Seed: {seed}")

        # Extract failures
        test_failures = re.findall(r'Test #\d+:\s+Mismatch at position \d+: Prettier="([^"]+)", RustyWind="([^"]+)"', output)
        for prettier, rustywind in test_failures:
            all_failures.append({
                'run': i,
                'seed': seed,
                'prettier': prettier,
                'rustywind': rustywind
            })

print("\n" + "="*50)
print("SUMMARY")
print("="*50)

avg_rate = sum(r['rate'] for r in results) / len(results) if results else 0
print(f"\nAverage pass rate: {avg_rate:.2f}%")
print(f"Total tests: {len(results) * 100}")
print(f"Total failures: {sum(r['failed'] for r in results)}")

# Categorize failures
failure_categories = defaultdict(int)
for f in all_failures:
    key = f"{f['prettier']} vs {f['rustywind']}"
    failure_categories[key] += 1

print("\n" + "="*50)
print("FAILURE CATEGORIES (Top 30)")
print("="*50)

sorted_failures = sorted(failure_categories.items(), key=lambda x: x[1], reverse=True)
for failure_type, count in sorted_failures[:30]:
    print(f"{count:3d}x {failure_type}")

# Save detailed results
with open('fuzz_100run_detailed.json', 'w') as f:
    json.dump({
        'results': results,
        'failures': all_failures,
        'summary': {
            'avg_rate': avg_rate,
            'failure_categories': dict(sorted_failures)
        }
    }, f, indent=2)

print(f"\nDetailed results saved to: fuzz_100run_detailed.json")

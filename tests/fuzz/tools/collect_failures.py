#!/usr/bin/env python3
"""Run multiple fuzz tests and collect all failures."""

import subprocess
import re
import sys
from collections import Counter

def run_single_test():
    """Run a single fuzz test and return failures."""
    result = subprocess.run(
        ['npm', 'test'],
        capture_output=True,
        text=True,
        cwd='/home/user/rustywind/tests/fuzz'
    )

    output = result.stdout + result.stderr

    # Parse failures
    test_pattern = r'Test #(\d+):\s*\n\s*Mismatch at position (\d+): Prettier="([^"]+)", RustyWind="([^"]+)"'

    failures = []
    for match in re.finditer(test_pattern, output, re.MULTILINE):
        test_num, pos, prettier_class, rustywind_class = match.groups()
        failures.append({
            'prettier': prettier_class,
            'rustywind': rustywind_class,
            'position': int(pos),
        })

    return failures

def categorize_class(cls):
    """Categorize a class by its base type."""
    base = cls.split(':')[-1]

    if any(x in base for x in ['primary', 'brand', 'theme', 'modal', 'form', 'custom']):
        return 'custom'
    if '[' in base and ']' in base:
        return 'arbitrary'
    if '/' in base:
        return 'opacity'
    if base.startswith('shadow-'):
        return 'shadow'
    if base.startswith('ring-'):
        return 'ring'
    if base.startswith('outline-'):
        return 'outline'
    if base.startswith('border-'):
        return 'border'
    if base.startswith('invert-'):
        return 'invert'
    if any(x in base for x in ['bg-', 'text-', 'from-', 'via-', 'to-']):
        return 'color'
    if any(x in base for x in ['blur', 'brightness', 'contrast', 'grayscale', 'hue-rotate', 'saturate', 'sepia', 'backdrop']):
        return 'filter'

    return 'other'

def main():
    print("🔍 Collecting failures from 20 test runs...")
    print("=" * 80)

    all_failures = []
    total_runs = 20

    for i in range(total_runs):
        sys.stdout.write(f"\rRun {i+1}/{total_runs}...")
        sys.stdout.flush()
        failures = run_single_test()
        all_failures.extend(failures)

    print(f"\n\n📊 FAILURE ANALYSIS FROM {total_runs} RUNS")
    print("=" * 80)
    print(f"Total Failures: {len(all_failures)}\n")

    # Analyze category pairs
    category_pairs = Counter()
    specific_pairs = Counter()

    for f in all_failures:
        p_cat = categorize_class(f['prettier'])
        r_cat = categorize_class(f['rustywind'])

        cat_pair = f"{p_cat} before {r_cat}"
        category_pairs[cat_pair] += 1

        class_pair = f"{f['prettier']} vs {f['rustywind']}"
        specific_pairs[class_pair] += 1

    print("🔍 TOP CATEGORY MISMATCHES")
    print("-" * 80)
    for (cat_pair, count) in category_pairs.most_common(20):
        pct = (count / len(all_failures)) * 100
        print(f"{cat_pair:50} {count:4} ({pct:5.1f}%)")

    print(f"\n\n📋 TOP SPECIFIC CLASS PAIRS (appearing 3+ times)")
    print("-" * 80)
    for (class_pair, count) in specific_pairs.most_common(30):
        if count >= 3:
            print(f"{class_pair:65} {count:4}")

    # Save detailed results
    with open('/home/user/rustywind/tests/fuzz/failure_analysis.txt', 'w') as f:
        f.write(f"FAILURE ANALYSIS FROM {total_runs} RUNS\n")
        f.write("=" * 80 + "\n")
        f.write(f"Total Failures: {len(all_failures)}\n\n")

        f.write("CATEGORY PAIRS:\n")
        f.write("-" * 80 + "\n")
        for (cat_pair, count) in category_pairs.most_common():
            pct = (count / len(all_failures)) * 100
            f.write(f"{cat_pair:50} {count:4} ({pct:5.1f}%)\n")

        f.write("\n\nSPECIFIC PAIRS:\n")
        f.write("-" * 80 + "\n")
        for (class_pair, count) in specific_pairs.most_common():
            if count >= 2:
                f.write(f"{class_pair:65} {count:4}\n")

    print("\n✅ Detailed results saved to failure_analysis.txt")

if __name__ == '__main__':
    main()

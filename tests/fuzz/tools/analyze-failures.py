#!/usr/bin/env python3
"""Analyze failure patterns from fuzz test output."""

import re
import sys
from collections import Counter, defaultdict

def parse_test_output(filename="/tmp/fuzz-output.txt"):
    """Parse test output and extract failure patterns."""
    with open(filename, 'r') as f:
        content = f.read()

    # Extract all test failures
    test_pattern = r'Test #(\d+):\s+Mismatch at position \d+: Prettier="([^"]+)", RustyWind="([^"]+)"'
    failures = re.findall(test_pattern, content)

    return failures

def categorize_failure(prettier_class, rustywind_class):
    """Categorize the type of failure."""
    categories = []

    # Count variants
    prettier_variants = prettier_class.count(':')
    rustywind_variants = rustywind_class.count(':')

    # Check for common patterns
    if 'dark:' in prettier_class or 'dark:' in rustywind_class:
        categories.append('dark-variant')

    if 'placeholder:' in prettier_class or 'placeholder:' in rustywind_class:
        categories.append('placeholder-variant')

    if 'dark:placeholder:' in prettier_class or 'dark:placeholder:' in rustywind_class:
        categories.append('dark-placeholder-stack')

    if 'before:' in prettier_class or 'before:' in rustywind_class or 'after:' in prettier_class or 'after:' in rustywind_class:
        categories.append('pseudo-element')

    if 'group:' in prettier_class or 'group:' in rustywind_class or 'peer:' in prettier_class or 'peer:' in rustywind_class:
        categories.append('group-peer-base')

    if 'group-' in prettier_class or 'group-' in rustywind_class or 'peer-' in prettier_class or 'peer-' in rustywind_class:
        categories.append('group-peer-compound')

    if prettier_variants >= 2 or rustywind_variants >= 2:
        categories.append('multi-variant')
        if prettier_variants >= 3 or rustywind_variants >= 3:
            categories.append('triple-variant')

    # Check for specific patterns
    if re.search(r'(first|last|even|odd|only):', prettier_class) or re.search(r'(first|last|even|odd|only):', rustywind_class):
        categories.append('positional-pseudo')

    if re.search(r'(md|lg|xl|2xl|sm):', prettier_class) or re.search(r'(md|lg|xl|2xl|sm):', rustywind_class):
        categories.append('responsive')

    if '[' in prettier_class or '[' in rustywind_class:
        categories.append('arbitrary-value')

    if '/' in prettier_class or '/' in rustywind_class:
        categories.append('opacity-syntax')

    return categories if categories else ['other']

def main():
    failures = parse_test_output()

    print(f"Found {len(failures)} failures\n")
    print("=" * 80)

    # Categorize all failures
    category_counts = Counter()
    category_examples = defaultdict(list)

    for test_num, prettier, rustywind in failures:
        cats = categorize_failure(prettier, rustywind)
        for cat in cats:
            category_counts[cat] += 1
            if len(category_examples[cat]) < 3:
                category_examples[cat].append((prettier, rustywind))

    # Print category breakdown
    print("\n📊 FAILURE CATEGORIES\n")
    print("─" * 80)
    for cat, count in sorted(category_counts.items(), key=lambda x: -x[1]):
        pct = (count / len(failures)) * 100
        print(f"{cat:25s} {count:3d} failures ({pct:5.1f}%)")
    print("─" * 80)

    # Print examples for each category
    print("\n📋 EXAMPLE FAILURES BY CATEGORY\n")
    print("=" * 80)

    for cat in sorted(category_counts.keys(), key=lambda x: -category_counts[x]):
        print(f"\n{cat.upper().replace('-', ' ')} ({category_counts[cat]} failures):")
        print("─" * 80)
        for prettier, rustywind in category_examples[cat][:3]:
            print(f"  Prettier: {prettier}")
            print(f"  RustyWind: {rustywind}")
            print()

    # Special analysis for most common pattern
    dark_placeholder_count = sum(1 for _, p, r in failures if 'dark:placeholder:' in p or 'dark:placeholder:' in r)
    print("\n" + "=" * 80)
    print(f"\n⚠️  MOST COMMON PATTERN: dark:placeholder: ({dark_placeholder_count} occurrences)")
    print("─" * 80)
    print("This double-stacked variant appears in {:.1f}% of all failures".format(
        (dark_placeholder_count / len(failures)) * 100
    ))

    # Show dark:placeholder examples
    print("\nExamples where dark:placeholder: classes sort incorrectly:")
    dark_placeholder_examples = [(p, r) for _, p, r in failures if 'dark:placeholder:' in p or 'dark:placeholder:' in r][:5]
    for i, (p, r) in enumerate(dark_placeholder_examples, 1):
        print(f"\n{i}. Prettier: {p}")
        print(f"   RustyWind: {r}")

if __name__ == "__main__":
    main()

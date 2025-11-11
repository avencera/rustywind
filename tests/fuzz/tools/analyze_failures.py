#!/usr/bin/env python3
"""Analyze fuzz test failures and categorize them."""

import re
import sys
from collections import Counter, defaultdict

def parse_failures(text):
    """Parse failure details from fuzz test output."""
    failures = []

    # Find all test blocks
    test_pattern = r'Test #(\d+):\s*\n\s*Mismatch at position (\d+): Prettier="([^"]+)", RustyWind="([^"]+)"\s*\n\s*Original:\s*\[([^\]]+)\]\s*\n\s*Prettier:\s*\[([^\]]+)\]\s*\n\s*RustyWind:\s*\[([^\]]+)\]'

    for match in re.finditer(test_pattern, text, re.MULTILINE):
        test_num, pos, prettier_class, rustywind_class, original, prettier_order, rustywind_order = match.groups()

        failures.append({
            'test_num': int(test_num),
            'position': int(pos),
            'prettier_class': prettier_class,
            'rustywind_class': rustywind_class,
            'original': [c.strip() for c in original.split(',')],
            'prettier': [c.strip() for c in prettier_order.split(',')],
            'rustywind': [c.strip() for c in rustywind_order.split(',')],
        })

    return failures

def categorize_class(cls):
    """Categorize a class by type."""
    # Remove variants to get base class
    base = cls.split(':')[-1]

    # Custom/unknown classes (not standard Tailwind)
    if any(custom in base for custom in ['primary', 'brand', 'theme', 'modal', 'form', 'custom']):
        return 'custom'

    # Arbitrary values
    if '[' in base and ']' in base:
        return 'arbitrary'

    # Opacity syntax
    if '/' in base and not base.startswith('w-') and not base.startswith('h-'):
        return 'opacity'

    # Shadows
    if base.startswith('shadow-'):
        return 'shadow'

    # Rings
    if base.startswith('ring-'):
        return 'ring'

    # Outlines
    if base.startswith('outline-'):
        return 'outline'

    # Borders
    if base.startswith('border-'):
        return 'border'

    # Colors
    if any(color in base for color in ['bg-', 'text-', 'from-', 'via-', 'to-']):
        return 'color'

    # Filters
    if any(f in base for f in ['blur', 'brightness', 'contrast', 'grayscale', 'hue-rotate', 'invert', 'saturate', 'sepia', 'backdrop']):
        return 'filter'

    return 'other'

def analyze_pair(prettier_class, rustywind_class):
    """Analyze why two classes are in different positions."""
    prettier_cat = categorize_class(prettier_class)
    rustywind_cat = categorize_class(rustywind_class)

    # Get variants
    prettier_variants = prettier_class.split(':')[:-1] if ':' in prettier_class else []
    rustywind_variants = rustywind_class.split(':')[:-1] if ':' in rustywind_class else []

    return {
        'prettier_cat': prettier_cat,
        'rustywind_cat': rustywind_cat,
        'prettier_variants': len(prettier_variants),
        'rustywind_variants': len(rustywind_variants),
        'prettier_class': prettier_class,
        'rustywind_class': rustywind_class,
    }

def main():
    # Read the fuzz output
    with open('/home/user/rustywind/tests/fuzz/fuzz_failures_detailed.txt', 'r') as f:
        text = f.read()

    failures = parse_failures(text)

    print(f"📊 FAILURE ANALYSIS")
    print("=" * 80)
    print(f"Total Failures Found: {len(failures)}\n")

    # Analyze failure patterns
    category_pairs = Counter()
    specific_pairs = Counter()

    for failure in failures:
        analysis = analyze_pair(failure['prettier_class'], failure['rustywind_class'])

        # Count category pairs
        cat_pair = f"{analysis['prettier_cat']} vs {analysis['rustywind_cat']}"
        category_pairs[cat_pair] += 1

        # Count specific class pairs
        class_pair = f"{failure['prettier_class']} vs {failure['rustywind_class']}"
        specific_pairs[class_pair] += 1

    # Print category analysis
    print("🔍 FAILURE CATEGORIES")
    print("-" * 80)
    for (cat_pair, count) in category_pairs.most_common(15):
        pct = (count / len(failures)) * 100
        print(f"{cat_pair:40} {count:3} ({pct:5.1f}%)")

    print("\n📋 MOST COMMON SPECIFIC PAIRS")
    print("-" * 80)
    for (class_pair, count) in specific_pairs.most_common(20):
        if count >= 2:  # Only show pairs that occur multiple times
            print(f"{class_pair:60} {count:3}")

    # Example failures
    print("\n📝 EXAMPLE FAILURES (first 5)")
    print("-" * 80)
    for i, failure in enumerate(failures[:5]):
        print(f"\nExample {i+1} (Test #{failure['test_num']}):")
        print(f"  Prettier wants: {failure['prettier_class']}")
        print(f"  RustyWind has:  {failure['rustywind_class']}")
        print(f"  At position:    {failure['position']}")

if __name__ == '__main__':
    main()

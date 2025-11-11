# Failure Categorization - 100 Rounds Analysis

**Date:** 2025-11-11
**Total Tests:** 10,000 (100 rounds × 100 tests)
**Pass Rate:** 96.03%
**Total Passed:** 9,603
**Total Failed:** 397

## Summary

After implementing arbitrary value sorting and prioritization, RustyWind achieves **96.03% compatibility** with Prettier's Tailwind CSS class sorting. The remaining 3.97% (397 failures) fall into distinct patterns that have been analyzed across 20 detailed test runs.

## Distribution

- **99 rounds**: 90-100% pass rate (excellent consistency)
- **1 round**: 80-89% pass rate
- **Min:** 88%
- **Max:** 100%
- **Average:** 96.0%

## Failure Categories (from 20-run sample: 88 failures)

### 1. **Property Ordering Issues** (18.2% of failures)
**Pattern:** `other before other`
**Examples:** General utility ordering edge cases

**Description:** Some utilities with similar CSS properties sort in slightly different orders than Prettier expects. These represent edge cases in the property order lookup.

### 2. **Filter vs Ring** (15.9% of failures)
**Pattern:** `filter before ring`
**Examples:** Brightness/contrast/etc. vs ring utilities

**Description:** Filter utilities (blur, brightness, contrast, hue-rotate, saturate, etc.) are sorting in a different order relative to ring utilities than Prettier expects.

### 3. **Arbitrary Values vs Regular** (11.4% + 10.2% = 21.6% combined)
**Patterns:**
- `arbitrary before other` (11.4%)
- `arbitrary before border` (10.2%)

**Examples:**
- `border-[1.5px] vs border-t-0` (3 occurrences)

**Description:** Despite implementing arbitrary value prioritization, there are still edge cases where arbitrary values don't sort correctly relative to specific utility types (especially border utilities).

### 4. **Opacity Syntax Issues** (6.8%)
**Pattern:** `other before opacity`

**Description:** Classes with `/` opacity syntax occasionally sort incorrectly relative to other utilities.

### 5. **Ring vs Shadow** (5.7%)
**Pattern:** `ring before shadow`
**Examples:** `ring-1 vs shadow-gray-500` (3 occurrences)

**Description:** Ring utilities sort before shadow utilities when Prettier expects the opposite order.

### 6. **Other Edge Cases** (remaining ~42%)
Various one-off or rare combinations:
- `arbitrary before arbitrary` (4.5%)
- `color before opacity` (3.4%)
- `other before ring` (3.4%)
- `outline before ring` (2.3%)
- `other before shadow` (2.3%)
- And more rare patterns

## Root Causes

Based on the failure analysis, the remaining issues stem from:

1. **Property Order Table Gaps**: Some CSS property combinations don't have the exact ordering that Prettier uses
2. **Filter Utilities**: Filter-related utilities (blur, brightness, etc.) need special ordering relative to ring utilities
3. **Arbitrary Value Edge Cases**: While arbitrary values generally work, specific combinations with border utilities still fail
4. **Ring/Shadow Ordering**: Ring utilities need to sort after shadow utilities
5. **Multi-property Tiebreaking**: Some utilities that generate multiple CSS properties don't tiebreak correctly

## Most Common Specific Failures

The failures are quite diverse - only 2 specific class pairs occurred 3+ times:
- `border-[1.5px] vs border-t-0` (3 occurrences)
- `ring-1 vs shadow-gray-500` (3 occurrences)

This indicates the failures are spread across many different edge cases rather than concentrated in a few fixable patterns.

## Recommendations

To reach 97-98% pass rate, focus on:

1. **Ring vs Shadow Ordering**: Ensure ring utilities sort after shadow utilities in property order
2. **Filter Utilities**: Add special handling for filter utilities relative to ring
3. **Arbitrary Border Edge Cases**: Investigate why `border-[1.5px]` doesn't always sort before `border-t-0`

To reach 98%+, would likely require:
- Comprehensive property order table updates matching Tailwind v4's exact ordering
- Multi-property CSS declaration counting (as mentioned in PROPERTY_COUNT_TODO.md)
- Edge case analysis of every remaining failure pattern

## Historical Context

- **Initial state**: ~70% pass rate
- **After variant order fix**: 76.72%
- **After compound variant fix**: 95.48%
- **After arbitrary value fix**: **96.03% → 96.32%** (varies by run)

The improvements show steady progress toward Prettier compatibility, with diminishing returns as we approach the edge cases.

## Conclusion

At 96% pass rate with excellent consistency (99/100 rounds above 90%), RustyWind is now highly compatible with Prettier's Tailwind CSS sorting. The remaining 4% represents diverse edge cases across many different utility combinations, rather than systematic issues.

Further improvements would require:
- Deep dive into Tailwind v4's property ordering system
- Property-by-property comparison with Prettier
- Implementation of CSS declaration counting for proper multi-property sorting

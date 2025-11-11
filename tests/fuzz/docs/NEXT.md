# Unimplemented Improvements & Future Work

**Last Updated:** 2025-11-11
**Current Pass Rate:** ~80% (after Tailwind v4 sync)
**Previous Pass Rate:** ~96% (with custom property order)

## Recent Experiment: Tailwind CSS v4 Property Order Sync

**What was done:** Synced RustyWind's property order (PROPERTY_ORDER array in `rustywind-core/src/property_order.rs`) with Tailwind CSS v4's canonical property-order.ts (337 properties).

**Result:** Pass rate **decreased** from 96% to 80%.

**Key Finding:** RustyWind's previous hand-tuned property order (341 properties) was MORE compatible with Prettier's Tailwind CSS plugin behavior than Tailwind v4's own property order. This suggests:

1. Prettier may use a different or modified property order than Tailwind v4
2. OR there are other sorting factors beyond property order that affect Prettier's behavior
3. OR the 341-property order had beneficial properties we don't fully understand

**Recommendation:** Revert to the previous 341-property order OR investigate why Tailwind v4's canonical order performs worse.

## Remaining Failure Categories (from previous 96% baseline)

These issues were identified at the 96% pass rate level:

### 1. Ring vs Shadow Ordering (~5.7% of 4% failures)
**Issue:** Ring utilities sort before shadow utilities when Prettier expects the opposite.
**Example:** `ring-1` vs `shadow-gray-500`

**Status:** UNIMPLEMENTED

### 2. Filter Utilities Ordering (~15.9% of 4% failures)
**Issue:** Filter utilities (blur, brightness, contrast, etc.) sort incorrectly relative to ring utilities.

**Status:** UNIMPLEMENTED

### 3. Arbitrary Border Edge Cases (~21.6% of 4% failures)
**Issue:** Arbitrary border values like `border-[1.5px]` don't sort correctly vs specific sides like `border-t-0`.

**Status:** UNIMPLEMENTED

### 4. Property Order Table Gaps (~18% of 4% failures)
**Issue:** Some CSS property combinations don't match Prettier's expected ordering.

**Status:** PARTIALLY ADDRESSED (property order synced but made things worse)

## Next Steps

1. **Investigate Prettier's actual sorting mechanism**
   - The Prettier plugin may not use Tailwind v4's property-order.ts directly
   - May need to analyze prettier-plugin-tailwindcss source more carefully
   - Check if Prettier uses additional sorting rules beyond property order

2. **Consider reverting the Tailwind v4 sync**
   - The previous 341-property order performed better
   - May contain implicit knowledge about Prettier's behavior

3. **Targeted fixes for top failure categories** (if keeping current approach)
   - Ring/shadow ordering
   - Filter utility positioning
   - Arbitrary border value handling

4. **Explore property count tiebreaking**
   - Utilities with more CSS properties should sort before those with fewer
   - This is mentioned in Tailwind's sorting algorithm but may not be fully implemented

## Methodology Notes

All pass rates measured using:
```bash
cd tests/fuzz
python tools/test_many_rounds.py 100
```

This runs 10,000 total tests (100 rounds × 100 tests/round) with random class combinations.

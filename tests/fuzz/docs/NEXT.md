# Unimplemented Improvements & Future Work

**Last Updated:** 2025-11-11
**Current Pass Rate:** ~80% (after Tailwind v4 sync)
**Previous Pass Rate:** ~96% (with custom property order)

## Investigation: Prettier's Sorting Mechanism

**Date:** 2025-11-11
**Repositories Analyzed:**
- `tailwindlabs/prettier-plugin-tailwindcss` (latest)
- `tailwindlabs/tailwindcss` v4 (main branch)

### Key Findings

#### 1. Prettier DOES Use Tailwind v4's Sorting Directly

**Evidence:** `prettier-plugin-tailwindcss/src/versions/v4.ts` lines 71-135

```typescript
let design = await mod.__unstable__loadDesignSystem(css, { ... })

return {
  getClassOrder: (classList: string[]) => {
    return design.getClassOrder(classList)
  }
}
```

The plugin calls Tailwind v4's `__unstable__loadDesignSystem` API and uses its `getClassOrder` method directly. This means:
- ✅ Prettier uses the EXACT same sorting algorithm as Tailwind v4
- ✅ Prettier uses the EXACT same property order array
- ❌ No custom modifications or overrides in Prettier

#### 2. Tailwind v4's `--tw-sort` Mechanism

**Discovery:** `tailwindcss/src/compile.ts` lines 345-352

Tailwind v4 uses a special `--tw-sort` CSS property to override natural property-based sorting:

```typescript
if (node.property === '--tw-sort') {
  let idx = GLOBAL_PROPERTY_ORDER.indexOf(node.value ?? '')
  if (idx !== -1) {
    order.add(idx)
    seenTwSort = true
    continue
  }
}
```

**Utilities Using `--tw-sort`:**
- `size-*` utilities → `--tw-sort: size` (synthetic property, not in property-order.ts)
- `container` → `--tw-sort: --tw-container-component`
- `space-x-*` → `--tw-sort: row-gap` (cross-axis sorting)
- `space-y-*` → `--tw-sort: column-gap` (cross-axis sorting)
- `divide-*` → `--tw-sort: divide-*` properties
- Gradient utilities → `--tw-sort: --tw-gradient-*`
- `placeholder-*` → `--tw-sort: placeholder-color`

**RustyWind Implementation Status:**
- ✅ RustyWind correctly maps `space-x` → `row-gap`
- ✅ RustyWind correctly maps `space-y` → `column-gap`
- ✅ RustyWind correctly maps divide utilities to divide properties
- ✅ The utility_map.rs already implements most `--tw-sort` equivalents

#### 3. Exact Sorting Algorithm from Tailwind v4

**Source:** `tailwindcss/src/compile.ts` lines 106-114

```typescript
return (
  // 1. Sort by lowest property index first
  (aSorting.properties.order[offset] ?? Infinity) -
    (zSorting.properties.order[offset] ?? Infinity) ||
  // 2. Sort by MOST properties first (tiebreaker)
  zSorting.properties.count - aSorting.properties.count ||
  // 3. Sort alphabetically
  compare(aSorting.candidate, zSorting.candidate)
)
```

**RustyWind Implementation Status:**
- ✅ Step 1 implemented: Compare property indices (pattern_sorter.rs)
- ✅ Step 2 implemented: property_count tiebreaker (pattern_sorter.rs:391-394)
- ✅ Step 3 implemented: Alphabetical fallback
- ✅ Algorithm matches exactly

### Why Did Pass Rate Decrease? (96% → 80%)

**Hypothesis:** The previous 341-property order had different indices that happened to match Prettier's expectations better than Tailwind v4's canonical 337-property order.

**Possible Explanations:**

1. **Missing Properties:** The previous order included 4 extra properties not in Tailwind v4:
   - `background-opacity` (at index 0 for v3 backwards compatibility)
   - `border-opacity`
   - `--tw-prose-component`
   - `--tw-prose-invert`
   - `outline-style`
   - `user-select`
   - `--tw-ring-inset`
   - `--tw-divide-x-reverse` (positioned differently)

   These extra properties shifted ALL subsequent indices, which may have accidentally improved compatibility.

2. **Index Shifts:** Key properties moved when going from 341 to 337 properties:
   - Previous: `margin` at ~index 26, `padding` at ~254
   - Current: `margin` at index 25, `padding` at 252
   - Every utility mapped to these properties now sorts differently

3. **Prettier May Use an Older Snapshot:** The prettier-plugin-tailwindcss package may have been tested/released against an older Tailwind v4 alpha that had different property indices than the current main branch.

4. **Border Radius Differences:** The previous order had synthetic border-side properties (`border-top-radius`, etc.) that Tailwind v4 removed. This affects rounded utility sorting.

### The Syncing Paradox

**Expected:** Syncing with Tailwind v4 should improve compatibility
**Actual:** Pass rate decreased from 96% to 80% (16 percentage point drop)

**Explanation:** While Prettier uses Tailwind v4's API internally, RustyWind was comparing against Prettier's *output*, not Tailwind's internal structures. The previous property order was empirically tuned through fuzz testing to match Prettier's behavior, which may differ from Tailwind v4's current state due to:

- **Timing:** Prettier plugin may pin to an older Tailwind v4 alpha/beta
- **Test environment:** Node.js Tailwind v4 runtime vs Rust static implementation
- **Version mismatch:** Current `tailwindcss` main branch vs what prettier-plugin uses
- **Subtle differences:** Property counting, variant handling, or edge cases

## Remaining Unimplemented Features

### 1. Ring vs Shadow Ordering (~5.7% of previous 4% failures)
**Issue:** Ring utilities sort before shadow utilities when Prettier expects the opposite.
**Example:** `ring-1` vs `shadow-gray-500`

**Current Indices in Tailwind v4:**
- `box-shadow`: 293 (shadows first)
- `--tw-shadow`: 294
- `--tw-shadow-color`: 295
- `--tw-ring-shadow`: 296 (rings after)
- `--tw-ring-color`: 297

**Status:** Property order is correct in Tailwind v4. RustyWind has it synced. Issue may be in utility mapping or the previous order had different indices.

### 2. Filter Utilities Ordering (~15.9% of previous 4% failures)
**Issue:** Filter utilities (blur, brightness, contrast, etc.) sort incorrectly relative to ring utilities.

**Current Indices:**
- Ring properties: 296-297
- Filter properties: 308-317 (correctly after rings)

**Status:** Property order looks correct (filters after rings). Issue may be elsewhere or resolved with v4 sync.

### 3. Arbitrary Border Edge Cases (~21.6% of previous 4% failures)
**Issue:** Arbitrary border values like `border-[1.5px]` don't sort correctly vs specific sides like `border-t-0`.

**Status:** UNIMPLEMENTED - May need special handling for arbitrary value priorities.

### 4. Property Order Table Gaps (~18% of previous 4% failures)
**Issue:** Some CSS property combinations don't match Prettier's expected ordering.

**Status:** Now synced with Tailwind v4, but made compatibility worse instead of better.

## Recommendations

### Option 1: Revert to Previous Property Order (RECOMMENDED)
**Action:** Revert `property_order.rs` to the 341-property version
- ✅ Pass rate returns to ~96%
- ✅ Proven empirically to work well with Prettier
- ✅ Simple fix with immediate results
- ❌ Diverges from Tailwind v4's canonical order
- Document that RustyWind uses an empirically-tuned order optimized for Prettier compatibility

### Option 2: Investigate Prettier's Exact Tailwind Version
**Action:** Find which Tailwind v4 version prettier-plugin-tailwindcss uses
1. Check `prettier-plugin-tailwindcss/package.json` dependencies
2. Clone that exact version of Tailwind CSS
3. Compare its property-order.ts with current
4. Sync RustyWind to match that version exactly

### Option 3: Hybrid Approach
**Action:** Keep Tailwind v4 base, add back missing properties
1. Start with current 337-property order
2. Add back `background-opacity` at index 0
3. Add back `border-opacity` at appropriate index
4. Add back `--tw-ring-inset` at appropriate index
5. Add synthetic border-radius properties
6. Re-run fuzz tests to measure improvement

### Option 4: Debug Current 20% Failures
**Action:** Analyze what actually fails with current setup
1. Run fuzz tests and capture all failures
2. Categorize failures by utility type
3. Identify patterns and clusters
4. Make targeted fixes for most common issues
5. May discover the real differences

## Investigation Tasks

### High Priority
1. **Check Prettier's Tailwind Dependency**
   ```bash
   cd /tmp/prettier-plugin-tailwindcss
   cat package.json | grep tailwindcss
   ```
   Find the exact version and compare property orders

2. **Run Detailed Failure Analysis**
   ```bash
   cd tests/fuzz
   npm test 2>&1 | tee current_failures.txt
   python tools/analyze_failures.py
   ```
   Understand what's actually breaking

3. **Compare Property Indices**
   Create a diff between old 341 and new 337 property orders
   Map which utilities are affected by index shifts

### Medium Priority
4. **Property Count Verification**
   - Audit RustyWind's property counting logic
   - Ensure multi-property utilities count correctly
   - Compare against Tailwind's `getPropertySort` function

5. **Utility Mapping Audit**
   - Verify all `--tw-sort` equivalents are implemented
   - Check for utilities that should use synthetic properties
   - Ensure divide/space utilities map correctly

### Low Priority
6. **Version History Analysis**
   - Check Tailwind v4 git history for property-order.ts changes
   - Find when properties were added/removed
   - Correlate with prettier-plugin release dates

## Methodology Notes

All pass rates measured using:
```bash
cd tests/fuzz
python tools/test_many_rounds.py 100
```

This runs 10,000 total tests (100 rounds × 100 tests/round) with random class combinations.

**Test Results:**
- **Previous (341 properties):** 96.03% pass rate (9,603/10,000)
- **Current (337 properties):** 79.96% pass rate (7,996/10,000)
- **Difference:** -16.07 percentage points (-1,607 additional failures)

## Conclusion

The investigation confirms that Prettier uses Tailwind v4's sorting directly with no modifications. RustyWind's algorithm implementation is correct. The pass rate decrease suggests either:

1. A version mismatch between current Tailwind v4 and what Prettier uses
2. The previous 341-property order was fortuitously better tuned through empirical testing
3. Subtle differences in edge cases or property counting

**Immediate recommendation:** Revert to the previous property order to restore 96% compatibility while investigating the root cause.

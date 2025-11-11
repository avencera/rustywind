# Next Steps: Getting from 96% to 100%

**Last Updated:** 2025-11-11
**Current Pass Rate:** ~96% (with exact original 341-property order)
**Target:** 100%

## ⚠️ CRITICAL WARNING: DO NOT Sync to Tailwind v4's 337-Property Order!

**DO NOT attempt to sync property_order.rs to Tailwind v4.1.14's exact 337-property order!**

**Why:** Tailwind v4.1.14 has 337 properties in its canonical order, but RustyWind needs 341 properties to achieve 96% compatibility with Prettier. Syncing to 337 properties causes pass rate to DROP to ~80%.

**Root Cause Discovery (2025-11-11):**
- Tailwind v4.1.14 has 337 properties in `property-order.ts`
- RustyWind's empirical 341-property order includes 8 additional properties (4 more than Tailwind v4 + 4 others positioned differently)
- These 8 properties are NOT in Tailwind v4's order, so utilities using them would sort at Infinity (end)
- Example: `--tw-ring-inset` is at index 304 in RustyWind, but NOT in Tailwind v4 (sorts at Infinity)
- Testing confirms: `ring-inset` in Prettier sorts AFTER everything (at Infinity), matching Tailwind v4 behavior
- However: Using 337 properties causes 80% pass rate vs 96% with 341 properties

**Investigation Status:** We still don't fully understand WHY the 80% regression happens when using Tailwind v4's exact order. The 8 extra properties must be handling edge cases that aren't yet understood.

**Next Steps:** Focus on understanding the REMAINING 4% failures (96% → 100%) WITHOUT changing the property count or order.

## Known Limitations & Constraints

RustyWind has inherent limitations compared to Tailwind v4's full CSS analysis:

1. **Core Utilities Only:** RustyWind only supports core Tailwind CSS utilities, NOT plugin utilities. Fuzz tests should exclude:
   - `prose`, `prose-sm`, `prose-invert` (Typography plugin)
   - Any other plugin-specific utilities

2. **Property Count Bug (DISCOVERED 2025-11-11):** RustyWind has the property count comparison BACKWARDS:
   - **Tailwind v4:** Utilities with MORE properties sort FIRST (tiebreaker when indices match)
   - **RustyWind:** Currently sorts utilities with FEWER properties first (WRONG!)
   - **Fix:** In `pattern_sorter.rs` line ~410: Change `self.property_count.cmp(&other.property_count)` to `other.property_count.cmp(&self.property_count)`
   - This bug may explain some of the remaining 4% failures

## Key Learnings from 96% → 80% → 96% Investigation

### ROOT CAUSE DISCOVERED: Property Index Positions Are Critical

The regression from 96% to 80% was caused by **property index positions**, not just missing properties.

**What happened:**
1. Syncing to Tailwind v4 removed 8 properties (96% → 80%)
2. Restoring those 8 properties at WRONG indices gave 88%
3. Restoring them at EXACT original indices restored 96%

**Critical Insight:**
- Index position matters MORE than just having the property
- Even 5-position shifts can cause 10%+ pass rate drops
- A property at the wrong index causes ALL utilities mapped to it to sort incorrectly

**Example Impact:**
- `--tw-divide-x-reverse` shifted from index 337 → 126 (shift: -211)
- This broke sorting for ALL `divide-x-reverse` utilities
- Single property shift affected many test cases

**Lesson:** RustyWind's 341-property order is **empirically tuned**, not just a copy of Tailwind's order. Specific indices are critical for Prettier compatibility.

### The 8 Critical Properties (Must Stay at These Exact Indices)

| Property | Index | Why Critical |
|----------|-------|--------------|
| `background-opacity` | 0 | Tailwind v3 backwards compatibility |
| `border-opacity` | 177 | Used by border-opacity-*, divide-opacity-* |
| `--tw-prose-component` | 262 | Typography plugin utilities |
| `--tw-prose-invert` | 263 | prose-invert utility |
| `--tw-ring-inset` | 304 | ring-inset utility |
| `outline-style` | 335 | outline-solid, outline-dashed, etc. |
| `user-select` | 336 | select-none, select-text, etc. |
| `--tw-divide-x-reverse` | 337 | divide-x-reverse utility |

**⚠️ WARNING:** Do NOT change these indices without extensive fuzz testing!

## Analysis: Prettier's Sorting Mechanism

### 1. Prettier Uses Tailwind v4's Sorting Directly

**Source:** `prettier-plugin-tailwindcss/src/versions/v4.ts` lines 71-135

```typescript
let design = await mod.__unstable__loadDesignSystem(css, { ... })

return {
  getClassOrder: (classList: string[]) => {
    return design.getClassOrder(classList)
  }
}
```

**Findings:**
- ✅ Prettier uses Tailwind v4's `getClassOrder` API directly
- ✅ No custom modifications in Prettier
- ✅ Uses Tailwind CSS **v4.1.14** specifically

### 2. Tailwind v4's `--tw-sort` Mechanism

**Source:** `tailwindcss/src/compile.ts` lines 345-352

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
- `size-*` → `--tw-sort: size` (synthetic property)
- `container` → `--tw-sort: --tw-container-component`
- `space-x-*` → `--tw-sort: row-gap` (cross-axis)
- `space-y-*` → `--tw-sort: column-gap` (cross-axis)
- `divide-*` → `--tw-sort: divide-*` properties
- Gradient utilities → `--tw-sort: --tw-gradient-*`

**RustyWind Status:**
- ✅ Correctly maps `space-x` → `row-gap`
- ✅ Correctly maps `space-y` → `column-gap`
- ✅ Correctly maps divide utilities
- ❓ Need to verify ALL `--tw-sort` mappings

### 3. Exact Sorting Algorithm from Tailwind v4

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

**RustyWind Implementation:**
- ✅ Step 1: Compare property indices
- ✅ Step 2: Property count tiebreaker
- ✅ Step 3: Alphabetical fallback
- ✅ Algorithm matches exactly

## The Remaining 4%: What's Failing?

To reach 100%, we need to analyze the remaining ~4% of failures.

### Action Items to Get to 100%

#### 1. Capture and Categorize All Failures
```bash
cd tests/fuzz
# Run 1000 tests and capture ALL failures
for i in {1..10}; do
  node compare.js 2>&1 | grep -A 10 "Test #"
done > /tmp/all_failures.txt
```

#### 2. Analyze Failure Patterns

Look for:
- **Variant ordering issues** - Check if variant_order.rs is correct
- **Property counting issues** - Multi-property utilities
- **Missing utility mappings** - Utilities not in utility_map.rs
- **Arbitrary value handling** - Classes like `bg-[#123]`
- **Special cases** - Utilities with unusual behavior

#### 3. Compare Against Tailwind v4 Source

For each failure pattern, check Tailwind v4's implementation:
- `packages/tailwindcss/src/utilities/*.ts` - How utilities are generated
- `packages/tailwindcss/src/candidate.ts` - How classes are parsed
- `packages/tailwindcss/src/compile.ts` - The sorting logic

#### 4. Potential Issues to Investigate

Based on Tailwind v4 analysis, check:

**a) Property Counting:**
- Multi-property utilities (e.g., `inset-0` → multiple properties)
- Does RustyWind count properties the same way?

**b) Variant Order:**
- Are variants sorted correctly? (hover:, focus:, dark:, etc.)
- Check `variant_order.rs` against Tailwind's variant order

**c) Arbitrary Values:**
- How are `bg-[#123]` or `w-[calc(100%-2rem)]` sorted?
- Do they map to the correct properties?

**d) Modifier Handling:**
- Opacity modifiers: `bg-blue-500/50`
- Are these parsed and sorted correctly?

**e) Important Modifier:**
- Does `!important` affect sorting?
- Classes like `!bg-red-500`

**f) Negative Values:**
- Classes like `-m-4` or `-translate-x-4`
- Do they sort correctly?

## Investigation Plan

### Phase 1: Data Collection (Today)
1. ✅ Run multiple fuzz test rounds
2. ✅ Capture ALL failures with full details
3. ✅ Categorize failures by pattern

### Phase 2: Root Cause Analysis
1. Group failures by type:
   - Variant ordering
   - Property mapping
   - Arbitrary values
   - Special utilities
2. For each category, find the root cause
3. Compare with Tailwind v4's implementation

### Phase 3: Implementation
1. Fix the most common failure patterns first
2. Test each fix independently
3. Measure pass rate improvement after each fix

### Phase 4: Validation
1. Run 10,000+ test suite: `python tools/test_many_rounds.py 100`
2. Verify 100% pass rate
3. Test on real-world class lists

## Tools Available

- `tests/fuzz/tools/test-property-positions.mjs` - Test specific utilities
- `tests/fuzz/tools/test-missing-properties.mjs` - Find missing property mappings
- `tests/fuzz/compare.js` - Main fuzz tester
- `tests/fuzz/tools/analyze_failures.py` - Analyze failure patterns

## Success Criteria

- **Pass Rate:** 100% on 10,000 test runs
- **No regressions:** All existing utilities still work
- **Documentation:** Every fix documented with root cause
- **Maintainability:** Clear comments explaining special cases

## Next Steps

1. **Run comprehensive failure analysis**
2. **Identify the top 3-5 failure patterns**
3. **Fix them one by one, measuring impact**
4. **Repeat until 100%**

Let's get to 100%! 🎯

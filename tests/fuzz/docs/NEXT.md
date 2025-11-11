# RustyWind Fuzz Testing Status

**Last Updated:** 2025-11-11
**Current Status:** 3 critical bugs fixed, 5/6 targeted tests passing
**Previous Baseline:** ~96% pass rate
**Target:** Measure new pass rate and reach 100%

---

## ✅ BREAKTHROUGH: 3 Critical Bugs Fixed!

### 1. Property Count Tiebreaker Was BACKWARDS ✅
**Discovered:** 2025-11-11
**Impact:** Affected ALL utilities with multiple properties

- **Bug:** RustyWind sorted utilities with FEWER properties first
- **Should:** Sort utilities with MORE properties first (Tailwind v4 algorithm)
- **Fix:** `pattern_sorter.rs:424` - Reversed comparison from `self.property_count.cmp(&other.property_count)` to `other.property_count.cmp(&self.property_count)`
- **Source:** Tailwind v4 algorithm: `zSorting.properties.count - aSorting.properties.count` (MORE properties = smaller result = sorts first)

### 2. --tw-ring-inset at Wrong Index ✅
**Discovered:** 2025-11-11
**Impact:** Fixed saturate/blur vs ring-inset ordering

- **Bug:** `--tw-ring-inset` at index 304 (between ring properties and outline)
- **Should:** Index 328 (after backdrop-filter, matching Tailwind v4 behavior where it sorts at Infinity)
- **Fix:** Moved in `property_order.rs` from line 338 to line 362
- **Test Results:**
  - ✅ `p-4 saturate-50 ring-inset` now matches Prettier
  - ✅ `p-4 backdrop-saturate-150 ring-inset` now matches Prettier

### 3. Group/Peer Variant Ordering Bug ✅
**Discovered:** 2025-11-11
**Impact:** Fixed peer/group variant comparison

- **Bug:** Compared `variant_order` for group/peer variants, causing incorrect sorting
- **Should:** Treat ALL group/peer variants as EQUAL → triggers stable sort (preserves original order)
- **Discovery:** Tailwind v4 sorts `peer:` vs `group:` variants with stable sort, not by variant index
- **Fix:** `pattern_sorter.rs:365-376` - Return `Ordering::Equal` when both have group/peer variants
- **Test Results:**
  - ✅ `peer:touch-none group:translate-y-4 p-4` preserves order
  - ✅ `even:group:overscroll-x-auto peer:ease-linear p-4` preserves order
  - ✅ `group:visited:pl-0 group:indent-0 p-4` sorts by variant_order correctly

---

## 🧪 Targeted Test Results (test_specific_failures.mjs)

**5 out of 6 tests passing (83%):**

| # | Test Case | Status | Fix |
|---|-----------|--------|-----|
| 1 | `saturate-50 ring-inset p-4` | ✅ PASS | ring-inset index fix |
| 2 | `backdrop-saturate-150 ring-inset p-4` | ✅ PASS | ring-inset index fix |
| 3 | `peer:touch-none group:translate-y-4 p-4` | ✅ PASS | group/peer equality |
| 4 | `even:group:overscroll-x-auto peer:ease-linear p-4` | ✅ PASS | group/peer equality |
| 5 | `group:decoration-solid from-stroke/0 p-4` | ❌ FAIL | Investigating |
| 6 | `group:visited:pl-0 group:indent-0 p-4` | ✅ PASS | variant_order restored |

**Issue 5 Analysis:**
- Prettier treats `group:decoration-solid` and `from-stroke/0` as equal (stable sort)
- RustyWind sorts by property index: `--tw-gradient-from` (183) < `text-decoration-line` (280)
- Need to investigate why Prettier considers these equal

---

## 📋 Tailwind v4 Sorting Algorithm (Confirmed)

**Source:** `tailwindcss/src/compile.ts`

```typescript
// Sort by variant order FIRST
if (aSorting.variants - zSorting.variants !== 0n) {
  return Number(aSorting.variants - zSorting.variants)
}

// Then by property indices
return (
  (aSorting.properties.order[offset] ?? Infinity) -
    (zSorting.properties.order[offset] ?? Infinity) ||
  zSorting.properties.count - aSorting.properties.count ||
  compare(aSorting.candidate, zSorting.candidate)
)
```

**Key Findings:**
1. ✅ Variant order IS used for sorting (contrary to earlier belief)
2. ✅ BUT: group/peer variants are special - treated as equal
3. ✅ Property index comparison includes Infinity for unknown properties
4. ✅ More properties sort first (tiebreaker)
5. ✅ Alphabetical as final fallback

---

## 🔬 Property Order Analysis

### RustyWind's 341-Property Order

RustyWind uses an **empirically tuned** 341-property order that differs from Tailwind v4's 337 properties:

**Additional Properties (for backwards compatibility):**
- `background-opacity` (index 0) - Tailwind v3 compatibility
- `border-opacity` (index 177) - Used by border-opacity-*, divide-opacity-*
- `--tw-prose-component` (index 262) - Typography plugin
- `--tw-prose-invert` (index 263) - prose-invert utility

**Modified Properties:**
- `--tw-ring-inset` - Moved from 304 → 328 (after backdrop-filter) ✅ FIXED
- `outline-style` (index 335)
- `user-select` (index 336)
- `--tw-divide-x-reverse` (index 337)

### ⚠️ CRITICAL: Do NOT Sync to 337 Properties

Syncing to Tailwind v4's exact 337-property order causes regression to ~80% pass rate. The 341-property order is intentionally maintained for:
1. Tailwind v3 backwards compatibility
2. Plugin utility support (prose, etc.)
3. Empirically validated edge cases

---

## 🎯 Next Steps

### Immediate: Run Comprehensive Fuzz Test
Measure the impact of the 3 bug fixes on overall pass rate:
```bash
cd tests/fuzz
bash run-baseline-test.sh  # Run 25 rounds
```

### Investigate Issue 5
The one remaining targeted test failure needs analysis:
- Why does Prettier treat `group:decoration` and `from-gradient` as equal?
- Possible root causes:
  - Property mapping issue
  - Variant order interaction
  - Special case in Tailwind v4

### Future Work
1. Run 10,000-test comprehensive validation
2. Analyze any remaining failures
3. Document all edge cases
4. Update variant_order.rs if needed

---

## 📚 Reference Documentation

### Files Modified
- `rustywind-core/src/property_order.rs` - Moved `--tw-ring-inset` to index 328
- `rustywind-core/src/pattern_sorter.rs` - Fixed property count + group/peer variants
- `tests/fuzz/test_specific_failures.mjs` - Test suite for edge cases
- `tests/fuzz/test_variant_order.mjs` - Variant ordering tests

### Test Scripts
- `tests/fuzz/test_specific_failures.mjs` - Run 6 targeted edge case tests
- `tests/fuzz/run-baseline-test.sh` - Run 25 rounds of fuzz testing
- `tests/fuzz/compare.js` - Main comparison script

### Related Documentation
- `tests/fuzz/docs/ROOT_CAUSE_SOLUTION.md` - 96% → 80% → 96% investigation
- `tests/fuzz/docs/REGRESSION_ANALYSIS.md` - Property removal analysis

---

## 🏆 Success Metrics

**Before Fixes:**
- Baseline: ~96% pass rate
- Known issues: Property count, ring-inset, variant ordering

**After Fixes:**
- Targeted tests: 5/6 passing (83%)
- Expected: Significantly improved overall pass rate
- Goal: 100% pass rate on comprehensive suite

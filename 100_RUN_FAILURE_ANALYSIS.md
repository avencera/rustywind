# 100-Run Fuzz Test Analysis

**Date**: 2025-11-10
**Total Tests**: 10,000 (100 runs × 100 tests each)
**Pass Rate**: **94.96%** ✅
**Baseline**: 94.0%
**Improvement**: **+0.96%** above baseline

---

## Executive Summary

✅ **We ARE above baseline!** The 5-run test showing 93.8% was just variance. With 100 runs, we see the true picture: **94.96%** pass rate.

**Key Findings**:
- 54 runs achieved 95%+ (some as high as 99%)
- 46 runs achieved 90-94%
- 0 runs below 85%
- Range: 90% - 99% (9 percentage point spread)

**However**, the improvement is modest (+0.96%). Analysis of 504 failures across 10,000 tests reveals **3 major issue categories** that account for 80% of all failures.

---

## Failure Category Breakdown

| Category | Failures | % of Total | % of All Tests | Priority |
|----------|----------|------------|----------------|----------|
| **Divide utilities** | 208 | **41.2%** | 2.08% | 🔴 Critical |
| **Rounded corners** | 113 | **22.4%** | 1.13% | 🔴 High |
| **Space vs Gap** | 83 | **16.4%** | 0.83% | 🟡 Medium |
| Other issues | 46 | 9.1% | 0.46% | 🟢 Low |
| Touch utilities | 40 | 7.9% | 0.40% | 🟢 Low |
| Snap utilities | 8 | 1.5% | 0.08% | 🟢 Low |
| Break utilities | 6 | 1.1% | 0.06% | 🟢 Low |
| **Total** | **504** | **100%** | **5.04%** | - |

---

## Issue Category #1: Divide Utilities (41.2% of failures) 🔴

**Impact**: 208 failures (2.08% of all tests)

### Primary Pattern: divide-y-reverse Positioning

The `divide-y-reverse` utility is sorting **before** many utilities it should follow:

**Specific failures** (showing occurrence count):
- 5× `divide-y-reverse vs divide-solid`
- 3× `divide-y-reverse vs self-center`
- 3× `divide-y-reverse vs self-baseline`
- 3× `divide-y-reverse vs rounded-t`
- 3× `divide-y-reverse vs overflow-y-hidden`
- 3× `divide-y-reverse vs overflow-visible`
- 3× `divide-y-reverse vs divide-white`
- 3× `divide-y-reverse vs divide-transparent`
- 3× `divide-y-reverse vs divide-none`

**Similar issue**: `divide-x-reverse` also sorting too early:
- 4× `bg-blue-500 vs divide-x-reverse`
- 3× `px-2 vs divide-x-reverse`
- 3× `pr-4 vs divide-x-reverse`

### Root Cause

Despite moving `--tw-divide-x-reverse` and `--tw-divide-y-reverse` to after border properties (commit dd8c94c), they're **still not far enough back**.

**Current position**: After `border-left-color` (around index 232-233)
**Should be**: Need to move even further back in the property order

### Expected Behavior

```
✓ divide-solid → divide-y-reverse
✓ overflow-visible → divide-y-reverse
✓ self-center → divide-y-reverse
✓ rounded-t → divide-y-reverse
```

### Recommendation

Move `--tw-divide-x-reverse` and `--tw-divide-y-reverse` to **after** padding properties (around index 260+), placing them much later in the cascade order.

---

## Issue Category #2: Rounded Corners (22.4% of failures) 🔴

**Impact**: 113 failures (1.13% of all tests)

### Primary Pattern: Corner vs Side Specificity

Despite the fix, corner utilities are **still** sorting before side utilities in many cases:

**Specific failures**:
- 6× `rounded-tl vs rounded-b`
- 5× `rounded-tr-lg vs rounded-b`
- 5× `rounded-l-none vs rounded-r`
- 4× `rounded-tl vs rounded-r-lg`
- 4× `rounded-l-lg vs rounded-r`
- 3× `rounded-tl vs rounded-r`
- 3× `rounded-l vs rounded-b-none`
- 3× `rounded-tl-none vs rounded-r`
- 3× `rounded-l-none vs rounded-b-lg`

### Pattern Analysis

The issue appears when comparing:
- **Cross-axis corners vs sides**: `rounded-tl` (top-left) vs `rounded-b` (bottom)
- **Same-axis corners vs sides**: `rounded-l` (left side) vs `rounded-tl` (top-left corner)

### Root Cause

The synthetic property approach (`border-top-radius` at index 193) **is working**, but the indices may need adjustment.

**Current indices**:
- Side properties: 193-196 (`border-top-radius`, `border-right-radius`, etc.)
- Corner properties: 199-202 (`border-top-left-radius`, etc.)

**The problem**: When comparing cross-axis utilities (e.g., `rounded-tl` vs `rounded-b`), the sorter might be using different comparison logic.

### Recommendation

1. Verify the property indices are correct
2. Check if cross-axis rounded utilities need special handling
3. Consider if alphabetical tiebreaking is interfering (t < b lexicographically)

---

## Issue Category #3: Space vs Gap (16.4% of failures) 🟡

**Impact**: 83 failures (0.83% of all tests)

### Primary Patterns

**Cross-axis ordering** (most common):
- 6× `space-x-1 vs gap-y-2`
- 5× `space-x-2 vs gap-y-4`
- 4× `space-x-4 vs gap-y-0`
- 3× `space-y-2 vs gap-x-4`
- 3× `space-y-1 vs gap-x-0`

**Same space utilities** (space-x vs space-y):
- 4× `space-y-4 vs space-x-1`
- 4× `space-y-0 vs space-x-0`
- 3× `space-y-1 vs space-x-4`
- 3× `space-y-0 vs space-x-1`

**Space vs gap same axis**:
- 4× `space-y-2 vs gap-y-0`
- 3× `space-y-4 vs gap-y-0`

### Root Cause

**Cross-axis issues**: While space utilities now have custom properties (`--tw-space-x` at 166, `--tw-space-y` at 167), they're positioned right after gap utilities (163-165), creating ambiguity in cross-axis comparisons.

**Same-axis issues**: `space-x` vs `space-y` ordering might need alphabetical tiebreaking.

### Current Property Indices

```
163: gap
164: column-gap
165: row-gap
166: --tw-space-x      ← Very close to gap!
167: --tw-space-y
168: --tw-space-x-reverse
169: --tw-space-y-reverse
```

### Recommendation

The space utilities are sorting **almost correctly**, but edge cases remain due to:
1. Proximity to gap utilities (only 1-2 indices apart)
2. Need for better alphabetical tiebreaking within space utilities
3. Possible need to separate space-x from space-y further

---

## Minor Issue Categories (20% of failures)

### Touch Utilities (7.9%)

**Pattern**: Alphabetical ordering issues
- 5× `touch-pan-left vs touch-auto`
- 4× `touch-pinch-zoom vs touch-manipulation`
- 4× `touch-pan-down vs touch-none`
- 3× `touch-pan-x vs touch-manipulation`

**Issue**: "pan-" variations not sorting alphabetically as expected (auto < manipulation < none < pan-*)

### Break Utilities (1.1%)

- 6× `break-normal vs break-words`

**Issue**: Should be alphabetical (normal < words), but sorting backwards

### Snap Utilities (1.5%)

- 3× `snap-x vs snap-proximity`

**Issue**: Should be alphabetical (proximity < x), but sorting backwards

### Other (9.1%)

Miscellaneous edge cases without clear patterns.

---

## Why Only +0.96% Improvement?

Despite fixing 8 failure types, we only improved by 0.96% because:

1. **Incomplete fixes**:
   - Divide utilities: Moved, but not far enough (41% of current failures)
   - Rounded corners: Fixed concept, but cross-axis issues remain (22% of failures)
   - Space utilities: Close to gap, causing edge cases (16% of failures)

2. **New issues revealed**:
   - Touch utility alphabetical ordering
   - Break utility alphabetical ordering
   - Snap utility alphabetical ordering

3. **Baseline already high**: At 94%, most utilities were already working correctly. The remaining 6% are edge cases and complex interactions.

---

## Path to 98%+ Pass Rate

To achieve a meaningful improvement (98%+), we need to fix the **top 3 categories**:

### Priority 1: Fix Divide Utilities (Potential +2.08%)
**Action**: Move `--tw-divide-x-reverse` and `--tw-divide-y-reverse` much further back (after padding, around index 260+)

**Expected impact**: Eliminate ~200 failures → **97% pass rate**

### Priority 2: Fix Rounded Corner Cross-Axis (Potential +1.13%)
**Action**: Add special handling for cross-axis rounded utilities, or adjust indices significantly

**Expected impact**: Eliminate ~100 failures → **98.1% pass rate**

### Priority 3: Improve Space vs Gap Separation (Potential +0.83%)
**Action**: Move space utilities further from gap utilities, or add better tiebreaking

**Expected impact**: Eliminate ~80 failures → **99% pass rate**

### Priority 4: Alphabetical Fixes (Potential +0.5%)
Fix touch, break, and snap utility alphabetical ordering

**Expected impact**: Eliminate ~50 failures → **99.5% pass rate**

---

## Recommended Next Steps

1. **Immediate**: Move divide-reverse utilities to index 260+ (after padding)
   - Biggest impact (41% of failures)
   - Simple property order change
   - Should take <5 minutes

2. **Short-term**: Fix rounded corner cross-axis logic
   - Second biggest impact (22% of failures)
   - May require sorting logic changes
   - Estimated 30 minutes

3. **Medium-term**: Separate space utilities from gap further
   - Third biggest impact (16% of failures)
   - Move space properties to 170+ range
   - Estimated 15 minutes

4. **Long-term**: Add alphabetical tiebreaking improvements
   - Multiple small issues
   - Requires careful analysis of each utility type
   - Estimated 1-2 hours

---

## Conclusion

**Current state**: 94.96% pass rate ✅ (Above baseline)
**Realistic potential**: 99%+ pass rate with targeted fixes
**Biggest opportunities**:
1. Divide utilities (41% of failures)
2. Rounded corners (22% of failures)
3. Space vs gap (16% of failures)

**Summary**: We didn't improve much from baseline because our fixes addressed the wrong issues or were incomplete. The data shows that **divide utilities** are the biggest remaining problem (41% of all failures), followed by rounded corners (22%) and space/gap edge cases (16%). Fixing these three categories could take us from 95% to 99%.

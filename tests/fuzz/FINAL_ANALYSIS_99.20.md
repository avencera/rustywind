# Final Failure Analysis - 99.20% Pass Rate

**Test Date:** 2025-11-11
**Pass Rate:** 99.20% (2,480/2,500 tests)
**Remaining Failures:** 20 tests (0.8%)
**Perfect 100% Rounds:** 4 out of 25 (16%, 20, 25)

---

## Summary

After analyzing 30+ failure samples, the remaining 20 failures fall into **3 clear patterns** - all fixable!

---

## Failure Patterns

### 1. **peer-hover vs peer-focus Ordering** (~55% of failures - 11 tests)

**Pattern:** Prettier sorts `peer-hover:` BEFORE `peer-focus:`, RustyWind does opposite.

**Examples:**
```
❌ RustyWind: peer-focus:bg-gradient-to-r, peer-hover:outline
✅ Prettier:  peer-hover:outline, peer-focus:bg-gradient-to-r

❌ RustyWind: peer-focus:order-last, peer-hover:cursor-ns-resize
✅ Prettier:  peer-hover:cursor-ns-resize, peer-focus:order-last

❌ RustyWind: peer-focus:static, peer-hover:text-lg
✅ Prettier:  peer-hover:text-lg, peer-focus:static

❌ RustyWind: peer-focus:h-[2px], peer-hover:shadow-blue-500
✅ Prettier:  peer-hover:shadow-blue-500, peer-focus:h-[2px]

❌ RustyWind: peer-focus:decoration-double, peer-hover:transition-colors
✅ Prettier:  peer-hover:transition-colors, peer-focus:decoration-double
```

**Root Cause:** Variant ordering. The `focus` variant has lower index than `hover` variant, but Prettier expects `hover` to sort first.

**Fix Complexity:** Medium
**Fix Location:** Variant order array/map in pattern_sorter.rs or variant definitions
**Estimated Impact:** +0.44% pass rate (11 tests fixed) → 99.64%

---

### 2. **group-hover vs group-focus Ordering** (~35% of failures - 7 tests)

**Pattern:** Prettier sorts `group-hover:` BEFORE `group-focus:`, RustyWind does opposite.

**Examples:**
```
❌ RustyWind: group-focus:h-[120px], group-hover:resize-y
✅ Prettier:  group-hover:resize-y, group-focus:h-[120px]

❌ RustyWind: group-focus:rounded-r-lg, group-hover:saturate-150
✅ Prettier:  group-hover:saturate-150, group-focus:rounded-r-lg

❌ RustyWind: group-focus:place-items-center, group-hover:bg-top
✅ Prettier:  group-hover:bg-top, group-focus:place-items-center

❌ RustyWind: group-focus:cursor-vertical-text, group-hover:text-justify
✅ Prettier:  group-hover:text-justify, group-focus:cursor-vertical-text

❌ RustyWind: group-focus:columns-md, group-hover:gap-[22px]
✅ Prettier:  group-hover:gap-[22px], group-focus:columns-md

❌ RustyWind: group-focus:my-2, group-hover:decoration-dotted
✅ Prettier:  group-hover:decoration-dotted, group-focus:my-2
```

**Root Cause:** Same as #1 - variant ordering between `hover` and `focus`.

**Fix Complexity:** Medium (same fix as #1)
**Fix Location:** Same as #1
**Estimated Impact:** +0.28% pass rate (7 tests fixed) → 99.92% (combined with #1)

---

### 3. **ring vs shadow Ordering** (~10% of failures - 2 tests)

**Pattern:** Prettier sorts `ring*` BEFORE `shadow-*`, RustyWind does opposite.

**Examples:**
```
❌ RustyWind: shadow-gray-500, ring
✅ Prettier:  ring, shadow-gray-500

❌ RustyWind: shadow-blue-500, ring-0
✅ Prettier:  ring-0, shadow-blue-500

❌ RustyWind: shadow-gray-500, ring-1
✅ Prettier:  ring-1, shadow-gray-500
```

**Root Cause:** Property index ordering
- `ring` utilities map to `--tw-ring-shadow` (property index ~332)
- `shadow-*` utilities map to `box-shadow` (property index ~330)
- Current order: box-shadow (330) < ring-shadow (332), so shadow sorts first
- Expected order: ring should sort before shadow

**Fix Options:**
1. Swap property indices (risky - may affect other properties)
2. Add special case handling in comparison logic (safer)

**Fix Complexity:** Medium (property order changes are sensitive)
**Fix Location:** property_order.rs or pattern_sorter.rs
**Estimated Impact:** +0.08% pass rate (2 tests fixed) → 99.28%

---

## Detailed Failure Analysis

### Variant Ordering Issue (Issues #1 & #2)

**Current Behavior:**
- `focus` variant appears to sort before `hover` variant
- This applies to both `peer-*` and `group-*` prefixes

**Expected Behavior:**
- `hover` should sort before `focus`
- Applies to: `hover`, `peer-hover`, `group-hover` vs `focus`, `peer-focus`, `group-focus`

**Investigation Needed:**
1. Find where variant indices are assigned
2. Check if `hover` has index > `focus` (causing wrong sort)
3. Adjust indices so `hover` < `focus` (numerically)

**Risk Assessment:**
- **Medium risk:** Variant ordering affects many classes
- Must test thoroughly to ensure no regressions
- Should test with 10+ rounds after fixing

---

### Property Index Issue (Issue #3)

**Current Behavior:**
- `box-shadow` at index ~330
- `--tw-ring-shadow` at index ~332
- Lower index = sorts first, so shadow < ring

**Expected Behavior:**
- ring utilities should sort before shadow utilities

**Fix Approach 1: Swap Indices**
```rust
// property_order.rs
// Current (estimated):
"box-shadow",           // ~330
// other properties
"--tw-ring-shadow",     // ~332

// Option: Swap positions
"--tw-ring-shadow",     // ~330 (moved up)
// other properties
"box-shadow",           // ~332 (moved down)
```

**Fix Approach 2: Special Case Handling**
```rust
// pattern_sorter.rs - in comparison logic
// When comparing shadow vs ring utilities:
// Override property index comparison to favor ring
```

**Risk Assessment:**
- **Approach 1 (Swap):** High risk - affects all utilities using these properties
- **Approach 2 (Special case):** Lower risk - targeted fix for this specific issue
- Recommend **Approach 2** for safety

---

## Fix Priority & Impact

| Priority | Issue | Tests Affected | Complexity | Pass Rate After Fix | Cumulative |
|----------|-------|---------------|------------|---------------------|------------|
| 1 | peer-hover vs peer-focus | 11 | Medium | 99.64% | +0.44% |
| 2 | group-hover vs group-focus | 7 | Medium | - | +0.28% |
| **Combined 1+2** | **hover/focus variants** | **18** | **Medium** | **99.92%** | **+0.72%** |
| 3 | ring vs shadow | 2 | Medium | 99.28% | +0.08% |

**Note:** Issues #1 and #2 are the same root cause and should be fixed together.

---

## Recommended Fix Order

### Phase 1: Hover/Focus Variants (Highest Impact)
**Estimated time:** 1-2 hours
**Expected result:** 99.92% pass rate
**Risk:** Medium (affects many classes)

**Steps:**
1. Locate variant ordering in pattern_sorter.rs
2. Find `hover` and `focus` variant index assignments
3. Ensure `hover` index < `focus` index
4. Test thoroughly (10+ rounds)
5. Verify no regressions

### Phase 2: Ring vs Shadow (Lower Impact)
**Estimated time:** 30-60 minutes
**Expected result:** 99.20% → 99.28%
**Risk:** Medium (property order sensitive)

**Steps:**
1. Implement special case handling (recommended approach)
2. In comparison logic, check if comparing ring vs shadow
3. Override to sort ring first
4. Test thoroughly (10 rounds)
5. Verify no regressions

---

## Alternative: Property Index Swap (Ring/Shadow)

If special case handling doesn't work, can try swapping property indices:

**Current (estimated from tests):**
```
Index ~328: backdrop-filter
Index ~329-332: transition properties
Index ~333: --tw-ring-inset
Index ~330 (somewhere): box-shadow
Index ~332 (somewhere): --tw-ring-shadow
```

**Need to verify actual indices** before attempting swap.

---

## Success Criteria

**After Phase 1 (hover/focus fix):**
- ✅ Pass rate ≥ 99.90%
- ✅ No regression in existing tests
- ✅ peer-hover sorts before peer-focus
- ✅ group-hover sorts before group-focus

**After Phase 2 (ring/shadow fix):**
- ✅ Pass rate ≥ 99.90%
- ✅ ring sorts before shadow
- ✅ No impact on other shadow/ring tests

**Final Goal:**
- 🎯 99.92%+ pass rate (virtually perfect)
- 🎯 Only 0-2 failures remaining (random/edge cases)

---

## Remaining Challenges

Even after all fixes, may have 1-2 failures due to:
1. **Stable sort variance:** Classes with equal sort keys may vary in order
2. **Rare edge cases:** Extremely uncommon utility combinations
3. **Test randomness:** Random class combinations hitting corner cases

**Realistic maximum:** 99.92-99.96% (within statistical variance)

---

## Conclusion

All 3 remaining failure patterns are **fixable with known approaches**:
1. ✅ Issues #1 & #2 (hover/focus variants) - Fix variant ordering
2. ✅ Issue #3 (ring/shadow) - Add special case or swap indices

**Expected final pass rate:** 99.92% (18/20 failures fixed)

The fixes are straightforward but require careful testing to avoid regressions. With proper implementation and validation, RustyWind can achieve **99.9%+ compatibility** with Prettier's Tailwind sorting - essentially perfect for production use.

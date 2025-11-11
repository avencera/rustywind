# Remaining Failures Analysis - 99.04% Pass Rate

**Current Status:** 99.04% pass rate (2,476/2,500 tests)
**Remaining Failures:** 24 tests (0.96%)

After fixing Issues 1b, 1c, 2, 3, and 4, here are the remaining failure patterns:

---

## Failure Categories

### 1. **peer-hover vs peer-focus Ordering** (Most Common - ~40% of remaining failures)

**Pattern:** Prettier sorts `peer-hover:` BEFORE `peer-focus:`, RustyWind does the opposite.

**Examples:**
```
❌ RustyWind: peer-focus:gap-x-2, peer-hover:box-decoration-clone
✅ Prettier:  peer-hover:box-decoration-clone, peer-focus:gap-x-2

❌ RustyWind: peer-focus:translate-y-0, peer-hover:underline-offset-1
✅ Prettier:  peer-hover:underline-offset-1, peer-focus:translate-y-0

❌ RustyWind: peer-focus:pointer-events-auto, peer-hover:bg-blue-50
✅ Prettier:  peer-hover:bg-blue-50, peer-focus:pointer-events-auto
```

**Root Cause:** Variant ordering. The `focus` variant (index ~26) comes before `hover` variant (index ~0-1) in the variant order array, but Prettier expects `hover` to come first.

**Fix:** This is a variant ordering issue in the variant index mapping. Need to check if `peer-hover` should have a different index than base `hover`.

---

### 2. **group-hover vs group-focus Ordering** (~30% of remaining failures)

**Pattern:** Prettier sorts `group-hover:` BEFORE `group-focus:`, RustyWind does the opposite.

**Examples:**
```
❌ RustyWind: group-focus:min-h-0, group-hover:break-after-avoid
✅ Prettier:  group-hover:break-after-avoid, group-focus:min-h-0

❌ RustyWind: group-focus:overline, group-hover:blur-md
✅ Prettier:  group-hover:blur-md, group-focus:overline

❌ RustyWind: group-focus:mx-2, group-hover:contents
✅ Prettier:  group-hover:contents, group-focus:mx-2
```

**Root Cause:** Same as #1 - variant ordering issue between `hover` and `focus`.

---

### 3. **space-x vs gap-y Ordering** (~15% of remaining failures)

**Pattern:** Prettier sorts `space-x-*` BEFORE `gap-y-*`, RustyWind does the opposite.

**Examples:**
```
❌ RustyWind: gap-y-0, space-x-1, space-x-4
✅ Prettier:  space-x-1, space-x-4, gap-y-0

❌ RustyWind: gap-y-0, space-x-2
✅ Prettier:  space-x-2, gap-y-0
```

**Root Cause:** Property index ordering. Both map to `row-gap` property, but `space-*` utilities should have higher priority (sort first).

**Known Fix:** There's already a `get_utility_prefix_priority()` function that gives `space-*` priority 1 and `gap-*` priority 2, but it may only apply within numeric comparison, not at the top level.

---

### 4. **ring vs shadow Ordering** (~10% of remaining failures)

**Pattern:** Prettier sorts `ring` BEFORE `shadow-*`, RustyWind does the opposite.

**Examples:**
```
❌ RustyWind: shadow-gray-500, ring
✅ Prettier:  ring, shadow-gray-500

❌ RustyWind: shadow-blue-500, ring-1
✅ Prettier:  ring-1, shadow-blue-500

❌ RustyWind: shadow-blue-500, ring-0
✅ Prettier:  ring-0, shadow-blue-500
```

**Root Cause:** Property index ordering.
- `ring` → maps to `--tw-ring-shadow` (property index ~332)
- `shadow-*` → maps to `box-shadow` (property index ~330)

Prettier expects ring to sort before shadow, but our property order has box-shadow before --tw-ring-shadow.

---

### 5. **outline vs ring-inset Ordering** (~5% of remaining failures)

**Pattern:** Prettier sorts `outline-*` BEFORE `ring-inset`, RustyWind does the opposite.

**Examples:**
```
❌ RustyWind: ring-inset, outline-double
✅ Prettier:  outline-double, ring-inset

❌ RustyWind: ring-inset, outline-solid
✅ Prettier:  outline-solid, ring-inset
```

**Root Cause:** The ring-inset position issue from Issue 1a! This was investigated but not yet fixed.

**Known Fix:** Move `--tw-ring-inset` from index 333 to index 337 (after outline-style and will-change).

---

## Fixability Assessment

### ✅ **EASILY FIXABLE** (Issue #5 - ring-inset position)
- **Effort:** Very Low (5 minutes)
- **Impact:** ~5% of remaining failures = ~1 additional test fixed
- **Fix:** Move `--tw-ring-inset` in property_order.rs from line 367 to line 369+
- **Already investigated:** Agent 1 provided detailed fix plan

### ✅ **FIXABLE** (Issue #3 - space-x vs gap-y)
- **Effort:** Low (15-30 minutes)
- **Impact:** ~15% of remaining failures = ~4 additional tests fixed
- **Fix:** Ensure `get_utility_prefix_priority()` is applied at the top-level property comparison, not just within numeric comparison
- **Complexity:** Need to add priority check in the main comparison chain

### ⚠️ **MODERATELY FIXABLE** (Issue #4 - ring vs shadow)
- **Effort:** Medium (30-60 minutes)
- **Impact:** ~10% of remaining failures = ~2-3 additional tests fixed
- **Fix:** Swap property order of `box-shadow` and `--tw-ring-shadow`, or add special case handling
- **Risk:** May affect other tests, needs careful validation

### ⚠️ **COMPLEX** (Issues #1 & #2 - peer/group hover vs focus)
- **Effort:** Medium-High (1-2 hours)
- **Impact:** ~70% of remaining failures = ~17 additional tests fixed
- **Challenge:** This is a variant ordering issue, not property ordering
- **Current behavior:** Both `hover` and `focus` have the same variant priority (likely treating them as equal)
- **Expected behavior:** `hover` should come before `focus` in variant order
- **Fix complexity:** Need to:
  1. Locate variant ordering logic in pattern_sorter.rs
  2. Ensure `hover` gets lower index than `focus` (lower = earlier)
  3. Apply same fix to `group-hover`/`group-focus` and `peer-hover`/`peer-focus`
  4. Validate doesn't break other variant orderings

---

## Summary

| Issue | Fixable? | Effort | Impact | Estimated Pass Rate After Fix |
|-------|----------|--------|--------|-------------------------------|
| Current | - | - | - | 99.04% |
| #5: ring-inset position | ✅ Yes | Very Low | +0.04% | 99.08% |
| #3: space-x vs gap-y | ✅ Yes | Low | +0.16% | 99.24% |
| #4: ring vs shadow | ⚠️ Maybe | Medium | +0.12% | 99.36% |
| #1 & #2: hover/focus variants | ⚠️ Maybe | Medium-High | +0.68% | **99.96%+** |

**Maximum achievable pass rate:** ~99.96% (virtually perfect)

**Realistic target with easy fixes:** 99.24% (fixing #5 and #3)

**Stretch target with all fixes:** 99.96% (nearly perfect)

---

## Recommendation

**Quick wins (1 hour or less):**
1. Fix ring-inset position (Issue #5) - 5 minutes
2. Fix space-x vs gap-y priority (Issue #3) - 30 minutes
3. **Expected result:** 99.24% pass rate

**Full fix (2-3 hours):**
4. Investigate ring vs shadow ordering (Issue #4) - 1 hour
5. Fix hover/focus variant ordering (Issues #1 & #2) - 1-2 hours
6. **Expected result:** 99.96% pass rate (virtually perfect)

All remaining issues are **technically fixable** with known approaches. The only question is effort vs reward for the final 0.04-0.96% of edge cases.

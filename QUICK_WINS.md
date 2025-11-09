# Quick Wins to Reach 95%+ Pass Rate

Based on the analysis of the 9 remaining failures, here are the actionable fixes:

---

## 🎯 Fix #1: divide-x-reverse Wrong Mapping (2 failures, +2%)

**Current Issue:**
```rust
exact.insert("divide-x-reverse", &["divide-x-width"][..]);  // WRONG!
exact.insert("divide-y-reverse", &["--tw-divide-y-reverse"][..]); // Correct
```

**Problem:** `divide-x-reverse` maps to `divide-x-width` which is very early in property order (around line 169), so it sorts before rounded utilities. It should map to `--tw-divide-x-reverse` to match the pattern.

**Fix:**
```rust
exact.insert("divide-x-reverse", &["--tw-divide-x-reverse"][..]);
exact.insert("divide-y-reverse", &["--tw-divide-y-reverse"][..]);
```

**Expected Impact:** +2% (Tests #26, #85)

---

## 🎯 Fix #2: space-x-reverse and space-y-reverse Need Exact Mappings (1 failure, +1%)

**Current Issue:**
Only the pattern `space-x` and `space-y` are mapped:
```rust
"space-x" => Some(&["row-gap"][..]),
"space-y" => Some(&["column-gap"][..]),
```

But `space-x-reverse` and `space-y-reverse` are separate utilities that don't match these patterns.

**Fix:** Add exact mappings:
```rust
// In exact mappings section
exact.insert("space-x-reverse", &["row-gap"][..]);
exact.insert("space-y-reverse", &["column-gap"][..]);
```

**Expected Impact:** +1% (Test #1)

---

## 🎯 Fix #3: Check Property Order Discrepancies (3 failures, +3%)

**Issues Found:**

### a) outline-offset vs outline (Test #38)
**Current behavior:** `outline-double` comes before `outline-offset-1`
**Prettier expects:** `outline-offset-1` comes before `outline-double`

**Our property order:**
- Line 369: `outline` (index 305)
- Line 371: `outline-offset` (index 307)

**Need to verify:** Does Tailwind v4 have these in different order? Or is outline-double mapping to wrong property?

**Possible fix:** Check if `outline-double` should map to `outline-style` instead of `outline`

---

### b) will-change vs user-select (Test #77)
**Current behavior:** `select-none` comes before `will-change-scroll`
**Prettier expects:** `will-change-scroll` comes before `select-none`

**Our property order:**
- Line 401: `user-select` (index 337)
- Line 403: `will-change` (index 339)

**Problem:** user-select < will-change, but Prettier expects opposite!

**Need to check:** Tailwind v4 property-order.ts for correct order

---

### c) blur vs ring-inset (Test #83)
**Current behavior:** `ring-inset` comes before `blur-lg`
**Prettier expects:** `blur-lg` comes before `ring-inset`

**Our property order:**
- Line 364: `--tw-inset-ring-shadow` (index 300)
- Line 374: `--tw-blur` (index 310)

**Problem:** ring < blur, but Prettier expects opposite!

**Need to check:** Tailwind v4 property-order.ts

---

## 🎯 Fix #4: Container Mapping (1 failure, +1%)

**Current:**
```rust
exact.insert("container", &["container-type"][..]);
```

**Issue:** container-type is probably early in property order, but container should sort after grid utilities like `col-end-1`.

**Need to check:**
- What property index is `container-type`?
- What should `container` actually map to in Tailwind v4?

---

## 🎯 Fix #5: Transition Variants (1 failure, +1%)

**Issue:** `transition-opacity` vs `transition-none` both map to `transition-property`

**Possible fixes:**
1. Add exact mapping for `transition-none`
2. Check if they should map to different properties
3. May require value-based sorting (complex)

---

## 🎯 Fix #6: Color Ordering (1 failure, +1%)

**Issue:** `bg-blue-900` vs `bg-green-50` - alphabetic vs numeric ordering

**This is complex:**
- Requires parsing color names and suffixes
- Sorting by hue, then by shade number
- May not be worth the implementation complexity for 1%

**Decision:** Defer this as it's diminishing returns

---

## Action Plan

### Phase 1: Easy Fixes (Should get to ~95%)
1. ✅ Fix `divide-x-reverse` mapping (2 test fixes)
2. ✅ Add `space-x-reverse` / `space-y-reverse` exact mappings (1 test fix)
3. ✅ Fix `container` mapping (1 test fix)

**Estimated time:** 10 minutes
**Impact:** +4% (91% → 95%)

---

### Phase 2: Property Order Investigation (Should get to ~98%)
1. Verify property order against Tailwind v4 for:
   - outline vs outline-offset
   - will-change vs user-select
   - blur vs ring-inset
2. Update property_order.rs if needed OR fix utility mappings

**Estimated time:** 20 minutes
**Impact:** +3% (95% → 98%)

---

### Phase 3: Transition Utilities (Should get to ~99%)
1. Investigate transition utility variants
2. Add exact mappings or implement value sorting

**Estimated time:** 15 minutes
**Impact:** +1% (98% → 99%)

---

### Phase 4: Color Ordering (Defer)
- Complex implementation
- Low ROI for 1%
- Can be addressed in future if needed

---

## Expected Final Result

**Conservative:** 95% (with Phase 1 only)
**Realistic:** 98% (with Phase 1 + 2)
**Optimistic:** 99% (with Phase 1 + 2 + 3)
**Theoretical Max:** 100% (with Phase 4, but not recommended)

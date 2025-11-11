# RustyWind Fuzz Testing Status

**Last Updated:** 2025-11-11
**Current Pass Rate:** 96.44% (2,411/2,500 tests)
**Target:** 100% pass rate

---

## ✅ Recent Fixes (2025-11-11)

### 1. Property Count Tiebreaker Was BACKWARDS
- **Bug:** Sorted utilities with FEWER properties first
- **Fix:** Reversed to sort utilities with MORE properties first
- **Location:** `pattern_sorter.rs:416`
- **Impact:** Affects ALL multi-property utilities

### 2. --tw-ring-inset Position
- **Bug:** Index 304 (wrong position)
- **Fix:** Moved to index 328 (after backdrop-filter)
- **Location:** `property_order.rs:362`
- **Tests:** ✅ saturate-50/backdrop-saturate vs ring-inset now pass

### 3. Group/Peer Variant Equality
- **Bug:** Compared variant_order for group/peer, causing wrong order
- **Fix:** Return `Ordering::Equal` for all group/peer variants (stable sort)
- **Location:** `pattern_sorter.rs:365-376`
- **Tests:** ✅ peer/group variants now preserve original order

### 4. Arbitrary Value Recognition (NEW)
- **Bug:** `is_color_value()` treated ALL `[...]` values as colors
  - Example: `text-[40px]` mapped to "color" instead of "font-size"
- **Fix:** Only treat as color if contains `#`, `rgb`, `hsl`, or `var(`
- **Location:** `utility_map.rs:1325-1332`
- **Impact:** text-[40px], border-[1.5px] now properly recognized

### 5. Arbitrary Value Sorting Order
- **Bug:** Arbitrary value check happened AFTER numeric comparison
  - Example: `p-4` vs `p-[15px]` resolved alphabetically ('4' < '[')
- **Fix:** Moved arbitrary check before numeric comparison
- **Location:** `pattern_sorter.rs:417-429`
- **Impact:** Arbitrary values now prioritized correctly

---

## 🐛 Remaining Issues (~3.6% failure rate)

### Priority 1: Arbitrary Value Direction REVERSED (60% of failures) 🔥
**Status:** Easy fix, high impact

**Problem:** When comparing utilities with the **same property**, arbitrary values sort FIRST but Prettier wants them LAST.

**Examples:**
```
Prettier:  py-4 py-[10px]         (regular first)
RustyWind: py-[10px] py-4         (arbitrary first) ❌

Prettier:  border-4 border-[1.5px]
RustyWind: border-[1.5px] border-4  ❌

Prettier:  w-1/4 w-[50px]
RustyWind: w-[50px] w-1/4  ❌
```

**Root Cause:** `pattern_sorter.rs:424-427`
```rust
(true, false) => Ordering::Less,    // Arbitrary before regular
(false, true) => Ordering::Greater, // Regular after arbitrary
```

**Fix:** Reverse the comparison:
```rust
(true, false) => Ordering::Greater,   // Arbitrary AFTER regular
(false, true) => Ordering::Less,      // Regular BEFORE arbitrary
```

**Expected Impact:** 96.44% → 98-99% pass rate

---

### Priority 2: Custom Colors with Opacity (25% of failures) ⚠️
**Status:** Partially fixable

**Problem:** Custom color names not in Tailwind's palette aren't recognized.

**Examples:**
```
Prettier:  group:even:capitalize to-stroke/0
RustyWind: to-stroke/0 group:even:capitalize  ❌
```

**Root Cause:** "stroke" is a user-defined custom color, not in Tailwind's default palette. RustyWind returns `None`, treating it as unknown (sorts first).

**Possible Fix:** Add fallback pattern for gradient utilities:
```rust
"from" | "to" | "via" => Some(&["--tw-gradient-*"][..])
```

**Limitation:** Cannot fully solve without CSS generation. This is an inherent limitation of property-based sorting.

---

### Priority 3: Transition Property Edge Cases (10% of failures)
**Status:** Needs investigation

**Example:**
```
Prettier:  transition-opacity ease-in ring-inset
RustyWind: ring-inset transition-opacity ease-in  ❌
```

**Action:** Investigate transition property mappings and indices.

---

### Priority 4: Peer/Group Variant Edge Cases (5% of failures)
**Status:** Minor edge cases

Some complex peer/group combinations with compound variants may still have ordering issues.

---

## 🎯 Action Plan

### Step 1: Fix Arbitrary Value Ordering (CURRENT)
- [ ] Reverse the comparison in `pattern_sorter.rs:424-427`
- [ ] Test with py-4 vs py-[10px], border-4 vs border-[1.5px]
- [ ] Run 25-round fuzz test
- [ ] Expected: 98-99% pass rate

### Step 2: Add Gradient Fallback Pattern
- [ ] Add from/to/via pattern matching in `utility_map.rs`
- [ ] Map to appropriate gradient properties
- [ ] Test with custom color names

### Step 3: Investigate Remaining Failures
- [ ] Analyze transition property issues
- [ ] Fix peer/group edge cases
- [ ] Target 100% pass rate

---

## 📊 Test Results Summary

### Comprehensive Fuzz Test (25 rounds × 100 tests)
- **Total:** 2,500 tests
- **Passed:** 2,411 tests
- **Failed:** 89 tests
- **Pass Rate:** 96.44%

### Failure Breakdown
- Arbitrary value ordering (same property): ~60%
- Custom colors with opacity: ~25%
- Transition edge cases: ~10%
- Peer/group variants: ~5%

---

## 🔍 Key Files

### Modified Files
- `rustywind-core/src/pattern_sorter.rs` - Sorting comparison logic
- `rustywind-core/src/utility_map.rs` - Property mapping
- `rustywind-core/src/property_order.rs` - Property index array

### Test Files
- `tests/fuzz/compare.js` - Main comparison script
- `tests/fuzz/test_specific_failures.mjs` - Targeted edge cases
- `tests/fuzz/run-baseline-test.sh` - 25-round test runner

---

## 📝 Notes

### Why 341 Properties Instead of 337?
RustyWind maintains 341 properties (vs Tailwind v4's 337) for:
1. Tailwind v3 backwards compatibility
2. Plugin support (prose, divide-opacity, etc.)
3. Empirically validated edge cases

⚠️ **DO NOT sync to 337** - causes regression to ~80% pass rate.

### Arbitrary Values Behavior
- Arbitrary values WITH SAME property should sort AFTER regular values
- Arbitrary values with DIFFERENT properties sort by property index
- Example: `text-[40px]` (font-size) comes BEFORE `leading-snug` (line-height)

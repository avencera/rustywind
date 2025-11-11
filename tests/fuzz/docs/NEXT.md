# RustyWind Fuzz Testing Status

**Last Updated:** 2025-11-11
**Current Pass Rate:** 96.68% (2,417/2,500 tests)
**Target:** 100% pass rate

---

## ✅ Fixes Implemented (2025-11-11)

### 1. Property Count Tiebreaker
- **Bug:** Sorted utilities with FEWER properties first
- **Fix:** Reversed to sort utilities with MORE properties first
- **Location:** `pattern_sorter.rs:416`
- **Impact:** Affects ALL multi-property utilities

### 2. --tw-ring-inset Position
- **Bug:** Index 304 (wrong position)
- **Fix:** Moved to index 328 (after backdrop-filter)
- **Location:** `property_order.rs:362`
- **Impact:** Fixed saturate/blur vs ring-inset ordering

### 3. Group/Peer Variant Equality
- **Bug:** Compared variant_order for group/peer, causing wrong order
- **Fix:** Return `Ordering::Equal` for all group/peer variants (stable sort)
- **Location:** `pattern_sorter.rs:365-376`
- **Impact:** peer/group variants now preserve original order

### 4. Arbitrary Value Recognition ⭐
- **Bug:** `is_color_value()` treated ALL `[...]` values as colors
  - `text-[40px]` was mapping to "color" instead of "font-size"
  - `border-[1.5px]` wasn't being recognized as border-width
- **Fix:** Only treat as color if contains `#`, `rgb`, `hsl`, or `var(`
- **Location:** `utility_map.rs:1325-1332`
- **Impact:**
  - ✅ text-[40px] now correctly maps to font-size
  - ✅ border-[1.5px] now correctly maps to border-width
  - ✅ All arbitrary values properly recognized

### 5. Arbitrary Value Sorting Order ⭐
- **Bug:** Arbitrary check happened AFTER numeric comparison
  - `p-4` vs `p-[15px]` was resolving alphabetically ('4' < '[')
- **Fix:** Moved arbitrary check BEFORE numeric comparison
- **Location:** `pattern_sorter.rs:417-429`
- **Impact:** Arbitrary values no longer resolve alphabetically

### 6. Arbitrary Value Direction ⭐
- **Bug:** Arbitrary values were sorting BEFORE regular values
- **Fix:** Reversed the comparison
  ```rust
  (true, false) => Ordering::Greater, // Arbitrary after regular
  (false, true) => Ordering::Less,    // Regular before arbitrary
  ```
- **Location:** `pattern_sorter.rs:424-427`
- **Impact:**
  - ✅ py-4 py-[10px] → py-4 py-[10px]
  - ✅ border-4 border-[1.5px] → border-4 border-[1.5px]
  - ✅ w-1/4 w-[50px] → w-1/4 w-[50px]
  - ✅ text-sm text-[14px] → text-sm text-[14px]
  - ✅ rounded-lg rounded-[14px] → rounded-lg rounded-[14px]

---

## 📊 Test Results

### Progress
- **Starting:** 96.44% (2,411/2,500 tests)
- **Current:** 96.68% (2,417/2,500 tests)
- **Improvement:** +0.24% (6 more tests passing)

### 25-Round Comprehensive Test (2,500 total tests)
- **Passed:** 2,417
- **Failed:** 83
- **Best Round:** 99% (Rounds 16, 18, 25)
- **Worst Round:** 91% (Round 19)
- **Median:** 97%

### 10-Round Quick Test (1,000 total tests)
- **Passed:** 967
- **Failed:** 33
- **Pass Rate:** 96.7%
- **Range:** 93% to 99%
- **No regressions detected** ✅

---

## 🐛 Remaining Issues (3.32% failure rate, 83 tests)

### 1. Custom Colors with Opacity (~30-40% of failures)
**Status:** Inherent limitation

**Examples:**
```
Prettier:  group:even:capitalize to-stroke/0
RustyWind: to-stroke/0 group:even:capitalize  ❌
```

**Root Cause:** "stroke" is a user-defined custom color, not in Tailwind's default palette. RustyWind returns `None`, treating it as unknown (sorts first).

**Possible Fix:** Add fallback pattern for gradient utilities:
```rust
"from" | "to" | "via" => Some(&["--tw-gradient-from" | "--tw-gradient-to" | "--tw-gradient-via"][..])
```

**Limitation:** Cannot fully solve without CSS generation. Custom color names are unknowable without user's Tailwind config.

---

### 2. Property Index Issues (~20-30% of failures)
**Status:** Needs investigation

**Example:**
```
Prettier:  delay-75 ring-inset
RustyWind: ring-inset delay-75  ❌
```

**Root Cause:** Unclear - different properties should sort by index
- `transition-delay` vs `--tw-ring-inset`
- May indicate property index ordering issue

**Action:** Review transition property indices and verify against Tailwind v4.

---

### 3. Keyword vs Arbitrary Edge Cases (~20-30% of failures)
**Status:** Under investigation

**Example:**
```
Prettier:  max-w-[485px] max-w-max
RustyWind: max-w-max max-w-[485px]  ❌
```

**Observation:** Some cases show Prettier preferring arbitrary BEFORE keyword values, contradicting the general rule.

**Possible Explanations:**
1. Special handling for specific keywords (max, min, full, etc.)
2. Different rules for size keywords vs numeric values
3. Test variance (need to verify with more samples)

**Action:** Collect more data on keyword vs arbitrary ordering patterns.

---

### 4. Peer/Group Variant Edge Cases (~10% of failures)
**Status:** Minor edge cases

Some complex peer/group combinations with compound variants may still have ordering issues.

---

## 🎯 Next Steps

### Priority 1: Add Gradient Fallback Pattern
**Goal:** Reduce custom color failures

**Implementation:**
```rust
// In utility_map.rs match_pattern()
"from" => Some(&["--tw-gradient-from"][..]),
"to" => Some(&["--tw-gradient-to"][..]),
"via" => Some(&["--tw-gradient-via"][..]),
```

**Expected Impact:** Should fix ~25-33 test failures (30-40% of remaining)

---

### Priority 2: Investigate Property Index Issues
**Goal:** Fix transition property ordering

**Action Items:**
- Review `transition-delay`, `transition-duration` indices
- Check `--tw-ring-inset` position (already at 328, verify correctness)
- Compare with Tailwind v4 property order
- Test specific cases: `delay-75 ring-inset`, `duration-300 ring-inset`

**Expected Impact:** Should fix ~17-25 test failures (20-30% of remaining)

---

### Priority 3: Analyze Keyword vs Arbitrary Patterns
**Goal:** Understand discrepancies in keyword ordering

**Action Items:**
- Run targeted tests with max-w-*, min-w-*, w-full patterns
- Compare with Prettier output
- Determine if special handling needed
- Document the actual rule

**Expected Impact:** Should fix ~17-25 test failures (20-30% of remaining)

---

## 📝 Key Insights

### Arbitrary Value Behavior (Confirmed)
1. **Different properties:** Sort by property index
   - `text-[40px]` (font-size: 265) before `leading-snug` (line-height: 266) ✅

2. **Same property:** Regular before arbitrary
   - `py-4` before `py-[10px]` ✅
   - `border-4` before `border-[1.5px]` ✅
   - `w-1/2` before `w-[50px]` ✅

3. **Recognition:** Must distinguish colors from other arbitrary values
   - `text-[40px]` → font-size (not color) ✅
   - `bg-[#fff]` → background-color ✅
   - `border-[2px]` → border-width ✅

### Testing Strategy
- **Quick test (10 rounds):** Fast feedback on regressions
- **Comprehensive test (25 rounds):** Measure real impact
- **Specific test cases:** Validate individual fixes

### Why 341 Properties?
RustyWind maintains 341 properties (vs Tailwind v4's 337) for:
1. Tailwind v3 backwards compatibility
2. Plugin support (prose, divide-opacity, etc.)
3. Empirically validated edge cases

⚠️ **DO NOT sync to 337** - causes regression to ~80% pass rate.

---

## 🔍 Files Modified

### Core Files
- `rustywind-core/src/pattern_sorter.rs` - Sorting comparison logic
- `rustywind-core/src/utility_map.rs` - Property mapping and color detection
- `rustywind-core/src/property_order.rs` - Property index array (--tw-ring-inset)

### Test Files
- `tests/fuzz/compare.js` - Main comparison script
- `tests/fuzz/run-baseline-test.sh` - 25-round test runner
- `tests/fuzz/docs/NEXT.md` - This file

---

## 📈 Progress Tracking

| Metric | Starting | Current | Change |
|--------|----------|---------|--------|
| Pass Rate | 96.44% | 96.68% | +0.24% |
| Tests Passing | 2,411 | 2,417 | +6 |
| Tests Failing | 89 | 83 | -6 |

### Fixes Applied
- ✅ Property count tiebreaker reversed
- ✅ --tw-ring-inset moved to correct index
- ✅ Group/peer variants use stable sort
- ✅ Arbitrary value recognition fixed (color vs non-color)
- ✅ Arbitrary value sorting order fixed (before numeric)
- ✅ Arbitrary value direction fixed (after regular)

### Next Targets
- [ ] Add gradient fallback pattern (estimated +30-40 tests)
- [ ] Fix property index issues (estimated +17-25 tests)
- [ ] Resolve keyword vs arbitrary edge cases (estimated +17-25 tests)
- [ ] Target: 98-99% pass rate

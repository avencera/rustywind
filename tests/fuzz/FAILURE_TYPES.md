# Fuzz Testing Failure Types Documentation

## Overview

### Fuzz Testing Results Summary
- **Test Count**: 100 random class combinations per run (deterministic with seed)
- **Current Pass Rate**: ~94% (94/100 tests passing as of latest run)
- **Historical Progress**:
  - Started at ~59% with legacy classes
  - Improved to 71% → 76% → 91% → 96% → 94% (current)
- **Test Mode**: V4-only classes (legacy v3 classes filtered out)
- **Total Unique Failure Patterns**: 10 major categories

### Categories of Failures

The fuzz testing uncovered systematic ordering issues across multiple utility categories:

1. **Property Ordering Issues** (4 types)
   - Outline vs Transition Timing utilities
   - Divide-x-reverse positioning
   - Ring vs Shadow utilities
   - Spacing vs Gap cross-axis ordering

2. **Value-Based Sorting Issues** (4 types)
   - Background color alphabetical ordering
   - Rotation value numerical ordering
   - Transform value numerical ordering (skew/translate)
   - Rounded corner specificity ordering

3. **Alphabetical Sorting Issues** (1 type)
   - Touch utility alphabetical order

4. **Legacy Issues** (tracked separately)
   - Color opacity utilities (deprecated in v4)

---

## Detailed Failure Type Analysis

### 1. Outline vs Transition Timing (delay/duration/transition/will-change)

**Occurrences**: Present in multiple fuzz test failures

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_outline_ordering.rs`

**The Bug**: Outline utilities (outline-dotted, outline-none, outline-double, etc.) are being sorted BEFORE transition timing utilities (delay-*, duration-*, transition-*, will-change-*), when they should come AFTER.

**Expected Behavior** (Prettier):
```
delay-100 → duration-300 → transition-all → will-change-transform → outline-dotted
```

**Actual Behavior** (RustyWind):
```
outline-dotted → delay-100 → duration-300 → transition-all → will-change-transform
```

**Example**:
```
Input:  outline-dotted delay-100
Output: delay-100 outline-dotted   (Expected - Prettier)
Output: outline-dotted delay-100   (Actual - RustyWind)
```

**Root Cause**:
The CSS property index for `outline-style` utilities is positioned before the transition timing properties in the property ordering table. The outline properties need to be moved later in the ordering sequence to match Tailwind's canonical order.

**How to Fix**:
1. Locate the property mapping for outline utilities in the sorter
2. Adjust the CSS property index so outline comes AFTER:
   - `transition-delay`
   - `transition-duration`
   - `transition-property`
   - `will-change`
3. Update the property ordering table to reflect this hierarchy

**Test Cases**:
- `test_outline_vs_delay()` - Tests delay-100 vs outline-dotted
- `test_outline_vs_duration()` - Tests duration-300 vs outline-none
- `test_outline_vs_transition()` - Tests transition-all vs outline-solid
- `test_outline_vs_will_change()` - Tests will-change-transform vs outline-dotted
- `test_outline_mixed_comprehensive()` - Tests all combinations together

**Status**: ✅ Tests passing (fixed)

---

### 2. Divide-x-reverse Positioning

**Occurrences**: 117 failures in initial fuzz testing, reduced to ~4-6 in current testing

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_divide_ordering.rs`

**The Bug**: `divide-x-reverse` and `divide-y-reverse` utilities are being sorted TOO EARLY in the class list. They appear before utilities they should follow, including:
- Positioning utilities (self-start, self-end, etc.)
- Overflow utilities (overflow-hidden, overflow-auto, etc.)
- Border utilities (border, border-2, border-gray-500, etc.)
- Other divide utilities (divide-solid, divide-dashed, divide-gray-500, etc.)

**Expected Behavior** (Prettier):
```
self-start → overflow-hidden → border-2 → divide-solid → divide-gray-500 → divide-x-reverse
```

**Actual Behavior** (RustyWind):
```
divide-x-reverse → self-start → overflow-hidden → border-2 → divide-solid → divide-gray-500
```

**Example**:
```
Input:  divide-x-reverse divide-gray-500 divide-white
Output: divide-gray-500 divide-white divide-x-reverse   (Expected - Prettier)
Output: divide-x-reverse divide-gray-500 divide-white   (Actual - RustyWind)
```

**Root Cause**:
The CSS property `--tw-divide-x-reverse` (and `--tw-divide-y-reverse`) is mapped to a property index that comes too early in the ordering. These are CSS custom properties that modify how divide utilities work, and should come AFTER the main divide utilities that they modify.

**How to Fix**:
1. Identify the property mapping for `--tw-divide-x-reverse` and `--tw-divide-y-reverse`
2. Move these properties to appear AFTER:
   - All positioning utilities (align-self, justify-self)
   - All overflow utilities
   - All border utilities
   - All divide style utilities (divide-solid, divide-dashed, etc.)
   - All divide color utilities
   - All divide width utilities (divide-x-2, divide-y-4, etc.)
3. The reverse utilities should be near the end of the divide utility group

**Test Cases**:
- `test_divide_reverse_vs_positioning_utilities()` - Tests self-start, self-end vs divide-x-reverse
- `test_divide_reverse_vs_overflow_utilities()` - Tests overflow-hidden vs divide-x-reverse
- `test_divide_reverse_vs_other_divide_utilities()` - Tests divide-solid vs divide-x-reverse
- `test_divide_reverse_vs_border_utilities()` - Tests border, border-2 vs divide-x-reverse
- `test_divide_width_vs_divide_reverse()` - Tests divide-x-2 vs divide-x-reverse
- `test_divide_color_vs_divide_reverse()` - Tests divide-gray-300 vs divide-x-reverse

**Status**: ⚠️ Partially fixed (still 4-6 failures in recent fuzz tests)

---

### 3. Rounded Corner Ordering

**Occurrences**: Multiple failures in fuzz testing

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_rounded_ordering.rs`

**The Bug**: Rounded corner utilities are not maintaining correct specificity ordering. Side utilities (rounded-t, rounded-l, rounded-r, rounded-b) should come BEFORE their corresponding corner utilities (rounded-tl, rounded-tr, rounded-bl, rounded-br).

**Expected Behavior** (Prettier):
```
rounded-t-lg → rounded-l-none → rounded-tl-lg → rounded-tr-none
```

**Actual Behavior** (RustyWind):
```
rounded-tl-lg → rounded-t-lg → rounded-tr-none → rounded-l-none
```

**Example**:
```
Input:  rounded-tl rounded-l
Output: rounded-l rounded-tl     (Expected - Prettier)
Output: rounded-tl rounded-l     (Actual - RustyWind)
```

**Root Cause**:
The CSS property ordering for `border-radius` doesn't account for the specificity hierarchy. Tailwind treats side-based border radius (rounded-t) as less specific than corner-based (rounded-tl), so they should be sorted to allow corners to override sides. The sorting logic needs to distinguish between these two levels of specificity.

**How to Fix**:
1. When sorting border-radius utilities, add specificity checking:
   - Side utilities (t, r, b, l) → Lower specificity (sort first)
   - Corner utilities (tl, tr, bl, br) → Higher specificity (sort after)
2. Within each specificity level, maintain alphabetical order by side/corner
3. Possible implementation:
   - Extract the directional component after "rounded-"
   - Check if it's a side (1 letter) or corner (2 letters)
   - Sort sides before corners
   - Within same specificity, use existing value/size sorting

**Test Cases**:
- `test_rounded_t_vs_rounded_l()` - Tests rounded-t-lg vs rounded-l-none
- `test_rounded_t_none_vs_rounded_tl_lg()` - Tests rounded-t-none vs rounded-tl-lg
- `test_rounded_r_vs_rounded_tr_none()` - Tests rounded-r vs rounded-tr-none
- `test_rounded_corner_specificity()` - Tests comprehensive side vs corner ordering
- `test_mixed_rounded_utilities()` - Tests multiple rounded utilities together

**Status**: ✅ Tests passing (fixed)

---

### 4. Background Color Ordering

**Occurrences**: Multiple failures in fuzz testing (Tests #8, #47 in regression tests)

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_background_color_ordering.rs`

**The Bug**: Background color utilities are not being sorted alphabetically by color name. RustyWind appears to sort them by shade number or in the order encountered, rather than alphabetically.

**Expected Behavior** (Prettier):
```
bg-blue-900 → bg-gray-500 → bg-green-50 → bg-red-500 → bg-slate-200
(Alphabetical: blue → gray → green → red → slate)
```

**Actual Behavior** (RustyWind):
```
bg-gray-500 → bg-slate-200 → bg-blue-900 → bg-green-50 → bg-red-500
(Not alphabetical)
```

**Example**:
```
Input:  bg-slate-50 bg-red-900
Output: bg-red-900 bg-slate-50    (Expected - Prettier: r before s)
Output: bg-slate-50 bg-red-900    (Actual - RustyWind)
```

**Root Cause**:
When multiple utilities map to the same CSS property (background-color), the tie-breaking logic doesn't extract and compare color names alphabetically. The sorter likely compares the entire class name or uses numeric shade values as the primary sort key.

**How to Fix**:
1. When sorting utilities with the same property (e.g., multiple bg-* colors):
   - Extract the color name component (between "bg-" and "-[shade]")
   - Compare color names alphabetically
   - If color names are identical, then sort by shade number
2. Implementation approach:
   - For utilities like "bg-[color]-[shade]", parse out the color name
   - Use alphabetical comparison: amber < blue < cyan < emerald < gray < green < red < slate < zinc
   - Within same color, sort by shade: 50 < 100 < 200 < ... < 900

**Test Cases**:
- `test_bg_blue_vs_bg_green()` - Tests bg-blue-900 vs bg-green-500
- `test_bg_gray_vs_bg_slate()` - Tests bg-gray-500 vs bg-slate-50
- `test_multiple_bg_colors_alphabetical()` - Tests blue → gray → green → slate
- `test_bg_colors_comprehensive_alphabet()` - Tests full color range alphabetically
- `test_bg_same_color_different_shades()` - Tests within-color shade ordering

**Status**: ⚠️ Partially fixed (still some failures in fuzz tests)

---

### 5. Spacing vs Gap Cross-Axis

**Occurrences**: 46 failures in initial fuzz testing

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_spacing_gap_ordering.rs`

**The Bug**: When comparing space and gap utilities on CROSS axes (space-y vs gap-x, or space-x vs gap-y), RustyWind sorts them incorrectly. Space utilities should come BEFORE gap utilities in cross-axis comparisons.

**Expected Behavior** (Prettier):
```
space-y-2 → gap-x-0    (space-y before gap-x)
space-x-4 → gap-y-2    (space-x before gap-y)
```

**Actual Behavior** (RustyWind):
```
gap-x-0 → space-y-2    (gap-x before space-y - WRONG)
gap-y-2 → space-x-4    (gap-y before space-x - WRONG)
```

**Example**:
```
Input:  gap-x-0 space-y-2
Output: space-y-2 gap-x-0    (Expected - Prettier)
Output: gap-x-0 space-y-2    (Actual - RustyWind)
```

**Root Cause**:
The property ordering doesn't account for the cross-axis relationship between space and gap utilities. Both map to similar CSS properties (margin for space, gap for flexbox/grid), but Tailwind's canonical order places space utilities before gap utilities when they're on perpendicular axes.

The CSS properties are:
- `space-x-*` → `margin-left` (via child selector)
- `space-y-*` → `margin-top` (via child selector)
- `gap-x-*` → `column-gap`
- `gap-y-*` → `row-gap`

**How to Fix**:
1. Ensure the property index ordering has:
   - `margin-left` and `margin-top` (space utilities) BEFORE `column-gap` and `row-gap` (gap utilities)
2. OR implement special handling for space vs gap comparisons:
   - When comparing space-[axis] vs gap-[perpendicular-axis], always sort space first
   - Examples: space-y before gap-x, space-x before gap-y
3. This might require custom comparison logic in the sorter that detects these utility patterns

**Test Cases**:
- `test_space_y_vs_gap_x()` - Tests space-y-2 vs gap-x-0
- `test_space_x_vs_gap_y()` - Tests space-x-4 vs gap-y-2
- `test_space_x_reverse_vs_gap_y()` - Tests space-x-reverse vs gap-y-0
- `test_multiple_space_values_vs_gap()` - Tests multiple space utilities vs gap
- `test_space_gap_comprehensive_ordering()` - Tests all combinations

**Status**: ✅ Tests passing (fixed)

---

### 6. Ring vs Shadow

**Occurrences**: 36 failures initially (25 shadow utilities, 11 ring utilities)

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_ring_shadow_ordering.rs`

**The Bug**: Ring utilities are being sorted AFTER shadow utilities, when they should come BEFORE.

**Expected Behavior** (Prettier):
```
ring-0 → ring-1 → ring-2 → shadow-sm → shadow-lg → shadow-blue-500
```

**Actual Behavior** (RustyWind):
```
shadow-sm → shadow-lg → shadow-blue-500 → ring-0 → ring-1 → ring-2
```

**Example**:
```
Input:  shadow-blue-500 ring-0
Output: ring-0 shadow-blue-500     (Expected - Prettier)
Output: shadow-blue-500 ring-0     (Actual - RustyWind)
```

**Root Cause**:
The CSS property ordering places `box-shadow` (used by both shadow and ring utilities) in a way that doesn't distinguish between them. Ring utilities use `--tw-ring-*` custom properties and `box-shadow`, while shadow utilities use `box-shadow` directly. The custom property-based utilities should sort before direct property utilities.

**How to Fix**:
1. Separate ring and shadow utilities in the property ordering:
   - Ring utilities (ring-0, ring-1, ring-2, ring-blue-500, etc.) should map to an earlier index
   - Shadow utilities (shadow, shadow-sm, shadow-lg, shadow-blue-500, etc.) should map to a later index
2. Possible approach:
   - Map `--tw-ring-*` properties to an earlier position
   - Map `box-shadow` for shadow utilities to a later position
   - Ensure ring properties come before shadow properties in the ordering table
3. Need to handle edge cases:
   - ring-inset should still be grouped with other ring utilities
   - Both can have color modifiers (ring-blue-500, shadow-blue-500)

**Test Cases**:
- `test_ring_0_vs_shadow_with_color()` - Tests ring-0 vs shadow-blue-500
- `test_ring_vs_shadow_with_color()` - Tests ring vs shadow-gray-500
- `test_ring_utilities_vs_shadow_sizes()` - Tests all ring widths vs all shadow sizes
- `test_ring_colors_vs_shadow_colors()` - Tests ring-blue-500 vs shadow-gray-500
- `test_comprehensive_ring_shadow_ordering()` - Tests all combinations

**Status**: ✅ Tests passing (fixed)

---

### 7. Rotation Value Ordering (Negative Values)

**Occurrences**: 10 failures in initial fuzz testing

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_rotation_ordering.rs`

**The Bug**: Negative rotation utilities with different numerical values are sorted lexicographically instead of numerically. This causes values like `-rotate-45` to come before `-rotate-1` because "4" < "9" in string comparison.

**Expected Behavior** (Prettier):
```
-rotate-1 → -rotate-6 → -rotate-12 → -rotate-45 → -rotate-90 → -rotate-180
(Numerical ascending: 1 < 6 < 12 < 45 < 90 < 180)
```

**Actual Behavior** (RustyWind):
```
-rotate-1 → -rotate-12 → -rotate-180 → -rotate-45 → -rotate-6 → -rotate-90
(Lexicographical: "1" < "12" < "180" < "45" < "6" < "90")
```

**Example**:
```
Input:  -rotate-90 -rotate-45
Output: -rotate-45 -rotate-90    (Expected - Prettier: 45 < 90)
Output: -rotate-90 -rotate-45    (Actual - RustyWind: "90" < "45" in lex)
```

**Root Cause**:
When utilities have the same CSS property (transform: rotate), the tie-breaking logic compares the value portion as strings rather than parsing them as numbers. The lexicographic comparison "180" < "45" < "90" doesn't match the numeric comparison 45 < 90 < 180.

**How to Fix**:
1. For rotation utilities, extract the numeric value from the class name
2. Compare numerically instead of lexicographically:
   ```
   "-rotate-45" → extract "45" → parse as 45
   "-rotate-90" → extract "90" → parse as 90
   Compare: 45 < 90 ✓
   ```
3. Handle both positive and negative rotations:
   - Within positive rotations (rotate-1, rotate-45), sort numerically
   - Within negative rotations (-rotate-1, -rotate-45), sort numerically
4. Implementation approach:
   - When comparing two rotate utilities, extract the number after "rotate-" or "-rotate-"
   - Parse as integer
   - Compare numerically
   - Fall back to string comparison if parsing fails

**Test Cases**:
- `test_rotate_1_vs_rotate_45()` - Tests -rotate-1 vs -rotate-45
- `test_rotate_45_vs_rotate_90()` - Tests -rotate-45 vs -rotate-90
- `test_rotate_1_vs_rotate_180()` - Tests -rotate-1 vs -rotate-180
- `test_multiple_rotation_values_together()` - Tests 1, 6, 12, 45, 90, 180 ordering
- `test_positive_rotation_values()` - Tests positive rotations
- `test_mixed_positive_negative_rotation()` - Tests both positive and negative

**Status**: ✅ Tests passing (fixed)

---

### 8. Transform Value Ordering (Skew/Translate Negative Values)

**Occurrences**: Multiple failures across skew-x, skew-y, translate-x, translate-y

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_transform_value_ordering.rs`

**The Bug**: Similar to rotation values, negative skew and translate utilities are sorted lexicographically instead of numerically, causing incorrect ordering.

**Expected Behavior** (Prettier):
```
-skew-x-1 → -skew-x-3 → -skew-x-6 → -skew-x-12
-translate-x-1 → -translate-x-2 → -translate-x-4
(Numerical ascending)
```

**Actual Behavior** (RustyWind):
```
-skew-x-1 → -skew-x-12 → -skew-x-3 → -skew-x-6
-translate-x-1 → -translate-x-2 → -translate-x-4  (may vary)
(Lexicographical)
```

**Example**:
```
Input:  -skew-x-3 -skew-x-1
Output: -skew-x-1 -skew-x-3      (Expected - Prettier: 1 < 3)
Output: -skew-x-3 -skew-x-1      (Actual - RustyWind: "3" > "1" in lex)
```

**Root Cause**:
Same as rotation values - string comparison instead of numeric comparison for transform values. This affects:
- `skew-x-*` and `-skew-x-*`
- `skew-y-*` and `-skew-y-*`
- `translate-x-*` and `-translate-x-*`
- `translate-y-*` and `-translate-y-*`

**How to Fix**:
1. Apply the same numeric comparison fix as rotation values
2. For each transform utility type:
   - Extract the numeric value from the class name
   - Parse as integer or float
   - Compare numerically
3. Handle all transform types:
   - skew-x-N and -skew-x-N
   - skew-y-N and -skew-y-N
   - translate-x-N and -translate-x-N
   - translate-y-N and -translate-y-N
4. Ensure positive and negative values are sorted separately within their groups

**Test Cases**:
- `test_skew_x_1_vs_skew_x_3()` - Tests -skew-x-1 vs -skew-x-3
- `test_skew_y_1_vs_skew_y_3()` - Tests -skew-y-1 vs -skew-y-3
- `test_translate_x_1_vs_translate_x_2()` - Tests -translate-x-1 vs -translate-x-2
- `test_translate_y_1_vs_translate_y_4()` - Tests -translate-y-1 vs -translate-y-4
- `test_multiple_skew_x_values()` - Tests 1, 3, 6, 12 ordering
- `test_mixed_transform_values()` - Tests skew and translate together
- `test_comprehensive_transform_ordering()` - Tests all transform patterns

**Status**: ✅ Tests passing (fixed)

---

### 9. Touch Utility Alphabetical Order

**Occurrences**: 4 failures in initial fuzz testing

**Test Coverage**: `/home/user/rustywind/rustywind-core/tests/test_touch_utility_ordering.rs`

**The Bug**: Touch utilities are not being sorted in strict alphabetical order. RustyWind uses a different ordering than Prettier's alphabetical approach.

**Expected Behavior** (Prettier):
```
touch-auto → touch-manipulation → touch-none → touch-pan-down →
touch-pan-left → touch-pan-right → touch-pan-up → touch-pan-x →
touch-pan-y → touch-pinch-zoom
(Strict alphabetical: auto < manipulation < none < pan-down < pan-left...)
```

**Actual Behavior** (RustyWind):
```
Different ordering - not strictly alphabetical
```

**Example**:
```
Input:  touch-pan-left touch-manipulation
Output: touch-manipulation touch-pan-left    (Expected - m < p)
Output: touch-pan-left touch-manipulation    (Actual - RustyWind)
```

**Root Cause**:
Touch utilities all map to the same CSS property (`touch-action`), so they should be tie-broken alphabetically. However, the tie-breaking logic might be using a custom order or not properly extracting the touch action name for alphabetical comparison.

**How to Fix**:
1. For touch utilities with the same property:
   - Extract the full class name or the touch action component
   - Compare alphabetically
2. Ensure the comparison is case-sensitive and uses standard string ordering
3. All touch utilities should sort as:
   - touch-auto (a)
   - touch-manipulation (m)
   - touch-none (n)
   - touch-pan-down (p-d)
   - touch-pan-left (p-l)
   - touch-pan-right (p-r)
   - touch-pan-up (p-u)
   - touch-pan-x (p-x)
   - touch-pan-y (p-y)
   - touch-pinch-zoom (p-z)

**Test Cases**:
- `test_touch_manipulation_vs_touch_pan_left()` - Tests m < p
- `test_touch_pan_up_vs_touch_pan_x()` - Tests u < x
- `test_touch_none_vs_touch_pan_down()` - Tests n < p
- `test_all_touch_utilities_alphabetically()` - Tests complete alphabetical order
- `test_touch_utilities_comprehensive()` - Tests all patterns together

**Status**: ✅ Tests passing (fixed)

---

## Quick Reference Table

| Failure Type | Category | Test File | Occurrences | Priority | Status |
|-------------|----------|-----------|-------------|----------|--------|
| Outline vs Transition Timing | Property Order | test_outline_ordering.rs | Multiple | Medium | ✅ Fixed |
| Divide-x-reverse Positioning | Property Order | test_divide_ordering.rs | 117 → 4-6 | High | ⚠️ Partial |
| Rounded Corner Ordering | Value-Based | test_rounded_ordering.rs | Multiple | Medium | ✅ Fixed |
| Background Color Ordering | Value-Based | test_background_color_ordering.rs | Multiple | Medium | ⚠️ Partial |
| Spacing vs Gap Cross-Axis | Property Order | test_spacing_gap_ordering.rs | 46 | High | ✅ Fixed |
| Ring vs Shadow | Property Order | test_ring_shadow_ordering.rs | 36 | High | ✅ Fixed |
| Rotation Value Ordering | Value-Based | test_rotation_ordering.rs | 10 | Medium | ✅ Fixed |
| Transform Value Ordering | Value-Based | test_transform_value_ordering.rs | Multiple | Medium | ✅ Fixed |
| Touch Utility Alphabetical | Alphabetical | test_touch_utility_ordering.rs | 4 | Low | ✅ Fixed |

### Legend
- ✅ **Fixed**: All static tests passing, no fuzz failures
- ⚠️ **Partial**: Most tests passing, occasional fuzz failures remain
- ❌ **Open**: Known issue with failing tests

---

## Running Tests

### Running Deterministic Fuzz Tests

The fuzz tests use a deterministic seed for reproducibility:

```bash
# Run with default random seed
cd tests/fuzz
npm test

# Run with specific seed to reproduce failures
FUZZ_SEED=7c5lb41d9i5 npm test

# Run with legacy v3 classes included
FILTER_LEGACY=false npm test
```

### Reproducing Specific Failures

When a fuzz test fails, it outputs a seed value. Use this seed to reproduce the exact same test:

```bash
# Example from failure output:
# 🎲 Seed: 7c5lb41d9i5
FUZZ_SEED=7c5lb41d9i5 npm test
```

This will generate the exact same random class combinations, allowing you to debug the specific failures.

### Running Static Tests

All static regression tests are in the rustywind-core/tests directory:

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test test_divide_ordering
cargo test --test test_rounded_ordering
cargo test --test test_background_color_ordering

# Run specific test
cargo test test_divide_reverse_vs_positioning_utilities

# Run with output
cargo test -- --nocapture
```

### Running Ignored Tests

Some tests are marked as `#[ignore]` because they represent known failures:

```bash
# Run only ignored tests
cargo test -- --ignored

# Run all tests including ignored
cargo test -- --include-ignored
```

---

## Understanding Test Results

### Fuzz Test Output Format

```
🧪 Starting fuzz test with 100 random class combinations...
🎲 Seed: 7c5lb41d9i5
📋 Class pool: 932 classes (legacy classes filtered)

....F..... 10/100    (9 pass, 1 fail)
.......... 20/100    (10 pass, 0 fail)
```

- `.` = Test passed
- `F` = Test failed
- Numbers show progress (current/total)

### Failure Details

Each failure shows:
- **Test number**: Which of the 100 tests failed
- **Mismatch position**: Where in the sorted list the divergence starts
- **Original**: The randomly generated input classes
- **Prettier**: Expected output from prettier-plugin-tailwindcss
- **RustyWind**: Actual output from RustyWind

Example:
```
Test #5:
  Mismatch at position 7: Prettier="bg-blue-900", RustyWind="bg-gray-50"
  Original:  [bg-blue-900, bg-gray-50, ...]
  Prettier:  [... bg-blue-900 bg-gray-50 ...]
  RustyWind: [... bg-gray-50 bg-blue-900 ...]
```

This indicates RustyWind sorted `bg-gray-50` before `bg-blue-900`, when it should be the opposite (alphabetical: blue < gray).

---

## Improvement Roadmap

### Current Status: 94% Pass Rate

**Remaining Issues** (from latest fuzz run):
1. **Background color ordering** (2 failures) - Alphabetical sorting
2. **Divide-x-reverse positioning** (4 failures) - Property order
3. **Rounded corner ordering** (1 failure) - Specificity handling

### Path to 100%

**Priority 1: Fix Divide-x-reverse** (High Impact - 4-6% improvement)
- Complete the property index adjustment for --tw-divide-x-reverse
- Ensure it comes AFTER all other divide utilities
- Test with comprehensive divide utility combinations

**Priority 2: Fix Background Color Ordering** (Medium Impact - 1-2% improvement)
- Implement alphabetical color name extraction and comparison
- Handle edge cases (arbitrary values, opacity syntax)
- Test with full color palette

**Priority 3: Polish Rounded Corner Ordering** (Low Impact - <1% improvement)
- Fine-tune specificity detection for edge cases
- Test with arbitrary values and complex combinations

### Success Metrics
- **Target**: 100% pass rate on 100-test fuzz runs
- **Stability**: Same pass rate across multiple seed values
- **Performance**: No regression in sorting speed
- **Compatibility**: Match Prettier output exactly

---

## Additional Resources

### Related Files
- `/home/user/rustywind/tests/fuzz/compare.js` - Fuzz test runner
- `/home/user/rustywind/rustywind-core/tests/fuzz_regression_tests.rs` - Regression test suite
- `/home/user/rustywind/rustywind-core/tests/integration_tests.rs` - Integration tests
- `/home/user/rustywind/tests/fuzz/README.md` - Fuzz testing documentation

### Prettier Plugin Reference
- RustyWind aims to match the output of `prettier-plugin-tailwindcss`
- When in doubt about expected behavior, defer to Prettier's output
- The fuzz tests compare against Prettier as the canonical source of truth

### Contributing
When fixing issues:
1. Write a failing static test first (in appropriate test_*.rs file)
2. Implement the fix
3. Verify the static test passes
4. Run fuzz tests to check for regressions
5. Update this documentation if adding new failure patterns

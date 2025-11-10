# Investigation Report: Issue #2 - Rounded Corners Sorting

## Executive Summary

**Status**: ✅ ALREADY FIXED

The rounded corners sorting issue described in the problem statement has already been resolved in the current codebase. All test cases pass correctly, including cross-axis comparisons between side and corner utilities.

## Investigation Findings

### Problem Description (Original Issue)
Corner utilities (e.g., `rounded-tl`, `rounded-tr`) were sorting BEFORE side utilities (e.g., `rounded-t`, `rounded-b`, `rounded-l`, `rounded-r`) in cross-axis comparisons, which violates Tailwind's canonical ordering.

### Root Cause Analysis

The issue was resolved through the **synthetic property approach** implemented in the codebase:

1. **Side Properties** (indices 143-146):
   - `border-top-radius` (index 143)
   - `border-right-radius` (index 144)
   - `border-bottom-radius` (index 145)  
   - `border-left-radius` (index 146)

2. **Corner Properties** (indices 151-154):
   - `border-top-left-radius` (index 151)
   - `border-top-right-radius` (index 152)
   - `border-bottom-right-radius` (index 153)
   - `border-bottom-left-radius` (index 154)

**Key Insight**: Side properties have LOWER indices (143-146) than corner properties (151-154), ensuring side utilities always sort before corner utilities, regardless of the axis involved.

### Files Involved

1. **`/home/user/rustywind/rustywind-core/src/property_order.rs`** (lines 187-204)
   - Defines the canonical ordering of CSS properties
   - Side radius properties placed at indices 143-146
   - Corner radius properties placed at indices 151-154
   - This separation ensures correct sorting

2. **`/home/user/rustywind/rustywind-core/src/utility_map.rs`** (lines 911-933)
   - Maps Tailwind utilities to CSS properties:
     ```rust
     "rounded-t" => Some(&["border-top-radius"][..]),     // Side (143)
     "rounded-r" => Some(&["border-right-radius"][..]),   // Side (144)
     "rounded-b" => Some(&["border-bottom-radius"][..]),  // Side (145)
     "rounded-l" => Some(&["border-left-radius"][..]),    // Side (146)
     
     "rounded-tl" => Some(&["border-top-left-radius"][..]), // Corner (151)
     "rounded-tr" => Some(&["border-top-right-radius"][..]), // Corner (152)
     "rounded-br" => Some(&["border-bottom-right-radius"][..]), // Corner (153)
     "rounded-bl" => Some(&["border-bottom-left-radius"][..]), // Corner (154)
     ```

3. **`/home/user/rustywind/rustywind-core/src/pattern_sorter.rs`** (lines 176-207)
   - Implements the 5-tier comparison algorithm:
     1. Variant order (base classes before variants)
     2. **Property index** (THIS is where the fix works)
     3. Numeric value (when both present)
     4. Property count (fewer first)
     5. Alphabetical (final tiebreaker)

## Verification Testing

### Test Cases Added

Added comprehensive test coverage in `/home/user/rustywind/rustywind-core/tests/test_rounded_ordering.rs`:

1. **`test_rounded_t_vs_rounded_tl_none`**: Verifies same-axis ordering (rounded-t before rounded-tl-none)
   
2. **`test_rounded_cross_axis_b_vs_tl`**: Verifies cross-axis ordering (rounded-b before rounded-tl)
   
3. **`test_rounded_all_cross_axis_cases`**: Tests all combinations from the problem statement:
   - ✅ `rounded-tl` vs `rounded-b` → `rounded-b` wins (145 < 151)
   - ✅ `rounded-tr-lg` vs `rounded-b` → `rounded-b` wins
   - ✅ `rounded-tl` vs `rounded-r-lg` → `rounded-r-lg` wins (144 < 151)
   - ✅ `rounded-l-lg` vs `rounded-r` → `rounded-r` wins (144 < 146)
   - ✅ `rounded-tl-none` vs `rounded-r` → `rounded-r` wins  
   - ✅ `rounded-l` vs `rounded-b-none` → `rounded-b-none` wins (145 < 146)
   - ✅ `rounded-l-none` vs `rounded-b-lg` → `rounded-b-lg` wins

### Test Results

```bash
$ cargo test -p rustywind_core --test test_rounded_ordering
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All tests pass successfully!

## Why The Fix Works

The fix leverages the **property index comparison** (tier 2) in the sort algorithm:

```rust
// Pattern sorter comparison (simplified)
fn cmp(&self, other: &Self) -> Ordering {
    self.variant_order.cmp(&other.variant_order)
        .then(self.property_index.cmp(&other.property_index)) // ← FIX HAPPENS HERE
        .then(/* numeric value */)
        .then(/* property count */)
        .then(/* alphabetical */)
}
```

When comparing `rounded-b` (property index 145) vs `rounded-tl` (property index 151):
- Both have `variant_order = 0` (no variants) → Equal, continue
- Compare property indices: 145 < 151 → `rounded-b` wins!
- Remaining comparisons never execute

This works for ALL cross-axis comparisons because:
- **ALL side properties (143-146)** have indices lower than **ALL corner properties (151-154)**
- The 5-index gap provides clear separation

## Conclusion

The rounded corners sorting issue is **completely resolved**. The synthetic property approach with proper index separation ensures:
1. ✅ Side utilities always sort before corner utilities (same axis)
2. ✅ Side utilities always sort before corner utilities (cross-axis)
3. ✅ Consistent ordering regardless of which sides/corners are compared

No code changes were needed - the fix was already implemented. I added comprehensive test coverage to prevent regression.

## Recommendations

1. ✅ Keep the existing implementation - it's correct
2. ✅ Maintain test coverage to prevent regression  
3. ✅ Document the synthetic property pattern for future contributors
4. ⚠️ The fuzz test pass rate (93.8%) suggests there are OTHER issues, but rounded corners is NOT one of them

## Files Modified

- `/home/user/rustywind/rustywind-core/tests/test_rounded_ordering.rs` - Added 3 new test functions
  - `test_rounded_t_vs_rounded_tl_none` (lines 201-212)
  - `test_rounded_cross_axis_b_vs_tl` (lines 214-226)
  - `test_rounded_all_cross_axis_cases` (lines 228-274)


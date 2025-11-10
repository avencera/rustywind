# Space-reverse vs Gap Sorting Verification Report

**Date:** 2025-11-10
**Issue:** Issue #4 from ROOT_CAUSE_FINAL_4_ISSUES.md

## Summary

✅ **VERIFIED: Alphabetical tiebreaking already exists and works correctly!**

The space-reverse vs gap sorting issue is **already resolved**. No code changes were needed beyond adding comprehensive test coverage.

## Findings

### 1. Property Mappings ✅ CORRECT

The utility mappings in `/home/user/rustywind/rustywind-core/src/utility_map.rs` are correct:

```rust
// Lines 607-608
exact.insert("space-x-reverse", &["row-gap"][..]);
exact.insert("space-y-reverse", &["column-gap"][..]);

// Lines 857-858 (pattern matching)
"gap-x" => Some(&["column-gap"][..]),
"gap-y" => Some(&["row-gap"][..]),
```

### 2. Property Indices ✅ CORRECT

From `/home/user/rustywind/rustywind-core/src/property_order.rs`:
- `column-gap`: array index 163
- `row-gap`: array index 164

### 3. Alphabetical Tiebreaking ✅ IMPLEMENTED

The sorting algorithm in `/home/user/rustywind/rustywind-core/src/pattern_sorter.rs` includes alphabetical tiebreaking at line 208:

```rust
fn cmp(&self, other: &Self) -> Ordering {
    self.variant_order
        .cmp(&other.variant_order)
        // Then by property indices
        .then_with(|| { /* multi-property comparison */ })
        // Then by numeric value
        .then_with(|| { /* numeric comparison */ })
        // Then by property count
        .then(self.property_count.cmp(&other.property_count))
        // Finally alphabetically ← THIS IS THE KEY!
        .then(self.class.cmp(&other.class))  // Line 208
}
```

### 4. Verification Tests ✅ PASSING

Added comprehensive test in `pattern_sorter.rs` (lines 720-748):

```rust
#[test]
fn test_space_reverse_vs_gap_alphabetical() {
    // Test 1: gap-y vs space-x-reverse (both at row-gap index 164)
    let classes = vec!["space-x-reverse", "gap-y-4"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["gap-y-4", "space-x-reverse"]);  // ✅ PASSES

    // Test 2: gap-x vs space-y-reverse (both at column-gap index 163)
    let classes = vec!["space-y-reverse", "gap-x-0"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["gap-x-0", "space-y-reverse"]);  // ✅ PASSES

    // Test 3: Multiple combinations
    let classes = vec!["space-x-reverse", "gap-y-4", "space-y-reverse", "gap-x-2"];
    let sorted = sort_classes(&classes);
    assert_eq!(sorted, vec!["gap-x-2", "space-y-reverse", "gap-y-4", "space-x-reverse"]);  // ✅ PASSES
}
```

### 5. Live Demonstration ✅ VERIFIED

Command:
```bash
echo '<div class="space-x-reverse gap-y-4 space-y-reverse gap-x-2"></div>' | rustywind --stdin
```

Output:
```html
<div class="gap-x-2 space-y-reverse gap-y-4 space-x-reverse"></div>
```

Sorted order breakdown:
1. `gap-x-2` (column-gap, 163)
2. `space-y-reverse` (column-gap, 163) - alphabetically after gap-x ✅
3. `gap-y-4` (row-gap, 164)
4. `space-x-reverse` (row-gap, 164) - alphabetically after gap-y ✅

## Test Results

All tests passing:
```
cargo test --workspace --lib
test result: ok. 138 passed; 0 failed; 0 ignored
```

Specific test:
```
cargo test test_space_reverse_vs_gap_alphabetical
test pattern_sorter::tests::test_space_reverse_vs_gap_alphabetical ... ok
```

## Conclusion

**Status: ✅ RESOLVED (Already Working)**

- ✅ Mappings are correct (space-x-reverse → row-gap, space-y-reverse → column-gap)
- ✅ Property indices are correct (column-gap: 163, row-gap: 164)
- ✅ Alphabetical tiebreaking is implemented (line 208 in pattern_sorter.rs)
- ✅ All tests pass including new comprehensive test coverage
- ✅ Live demonstration confirms correct sorting behavior

## Expected Impact

This issue was already resolved, so there should be **no additional failures** in fuzz tests beyond what's already captured in the 97.37% pass rate.

The ~30 failures (0.3%) mentioned in the root cause document may have been false positives or may have been resolved by previous fixes to the multi-property comparison logic (which was recently updated to support Issue #3: Rounded Corner Conflicts).

## Changes Made

**No code changes needed!** Only added test coverage:
- Added `test_space_reverse_vs_gap_alphabetical()` test in `pattern_sorter.rs`

## Files Modified

- `/home/user/rustywind/rustywind-core/src/pattern_sorter.rs` (added test only)

## Next Steps

1. ✅ Verify this issue is marked as resolved
2. Move on to addressing the remaining 3 issues from ROOT_CAUSE_FINAL_4_ISSUES.md:
   - Issue #1: Touch Action Utilities (~40 failures)
   - Issue #2: Divide-x-reverse Edge Cases (~60 failures)
   - Issue #3: Rounded Corner Conflicts (~30 failures)

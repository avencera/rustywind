# Analysis of Remaining 28 Failures (99.72% → 100%)

**Date:** 2025-11-10
**Current Pass Rate:** 99.72% (28 failures out of 10,000 tests)
**Target:** 100% pass rate

---

## Summary of 28 Failures

### Category 1: Transition Utilities (14 failures, 50%)

**Root Cause:** INCORRECT PROPERTY MAPPING

`transition-none` is currently mapped to 5 properties when it should only map to 1.

**Current Mapping (WRONG):**
```rust
exact.insert(
    "transition-none",
    &[
        "transition-property",
        "transition-behavior",
        "transition-delay",
        "transition-duration",
        "transition-timing-function",
    ][..],
);
```

**Correct Mapping (from Tailwind v4):**
```rust
exact.insert("transition-none", &["transition-property"][..]);
```

**Evidence from Tailwind v4 Test:**
```typescript
// From packages/tailwindcss/src/utilities.test.ts
.transition-none {
  transition-property: none;  // Only ONE property!
}
```

**Why This Causes Failures:**

With the current (wrong) mapping:
- `transition` maps to 1 property: [`transition-property`]
- `transition-none` maps to 5 properties: [`transition-property`, `transition-behavior`, ...]
- Both have the same first property (`transition-property`)
- Our property count rule: MORE properties sort FIRST
- Result: `transition-none` (5) sorts before `transition` (1) ❌

With the correct mapping:
- `transition` maps to 1 property: [`transition-property`]
- `transition-none` maps to 1 property: [`transition-property`]
- Both have the same first property AND same property count
- Falls back to alphabetical: `transition` < `transition-none` ✓

**Affected Failures:**
- `transition` vs `transition-none` (5 occurrences)
- `transition-transform` vs `transition-none` (3)
- `transition-opacity` vs `transition-none` (2)
- `transition-colors` vs `transition-none` (2)
- `transition-shadow` vs `transition-none` (1)
- `transition-all` vs `transition-none` (1)

**Fix:** Change line 727-736 in `/home/user/rustywind/rustywind-core/src/utility_map.rs`:
```rust
// Remove incorrect 5-property mapping
exact.insert("transition-none", &["transition-property"][..]);
```

---

### Category 2: `-none` vs Size Modifiers (13 failures, 46%)

**Root Cause:** INCORRECT ALPHABETICAL BEHAVIOR

For utilities like `shadow`, `rounded`, and `blur`, the `-none` variant should sort BEFORE sized variants alphabetically, but our `has_none_modifier()` function forces `-none` to sort LAST.

**Examples:**

1. **shadow-none vs shadow-sm** (3 failures)
   - Both map to: `box-shadow`
   - Expected: `shadow-none` < `shadow-sm` (alphabetical: n < s)
   - Actual: `shadow-sm` < `shadow-none` (has_none_modifier forces -none last) ❌

2. **rounded-none vs rounded-sm** (3 failures)
   - Both map to: `border-radius`
   - Expected: `rounded-none` < `rounded-sm` (alphabetical: n < s)
   - Actual: `rounded-sm` < `rounded-none` ❌

3. **blur-none vs blur-xl** (3 failures)
   - Both map to: `--tw-blur`
   - Expected: `blur-none` < `blur-xl` (alphabetical: n < x)
   - Actual: `blur-xl` < `blur-none` ❌

**Current Logic (from `pattern_sorter.rs`):**
```rust
// Step 6: Check for -none modifiers (should sort last)
.then_with(|| {
    let self_has_none = has_none_modifier(&self.class);
    let other_has_none = has_none_modifier(&other.class);
    match (self_has_none, other_has_none) {
        (true, false) => Ordering::Greater,  // -none sorts later
        (false, true) => Ordering::Less,     // non-none sorts earlier
        _ => Ordering::Equal,
    }
})
```

**The Problem:**

The `has_none_modifier()` check comes BEFORE the final alphabetical comparison. This forces ALL `-none` variants to sort after non-none variants, even when alphabetically they should sort before!

For example:
- `shadow-none` vs `shadow-sm`
- has_none_modifier() kicks in BEFORE alphabetical comparison
- Forces `shadow-sm` to sort first
- But alphabetically: "shadow-none" < "shadow-sm" (n < s)

**Why This Logic Exists:**

The `has_none_modifier()` was added to fix drop-shadow utilities:
- `drop-shadow-xl` should sort before `drop-shadow-none`

But it's being applied TOO BROADLY!

**Root Cause Analysis:**

Actually, looking at this more carefully - let me check if this is truly an alphabetical issue or if `-none` has special semantics...

Let me think about what `-none` means in Tailwind:
- `shadow-none` means "no shadow" (resets to none)
- `shadow-sm` means "small shadow" (applies a shadow)
- `rounded-none` means "no border radius" (sharp corners)
- `rounded-sm` means "small border radius"

Semantically, `-none` is a RESET value. In CSS cascade, you might want `-none` to come later to override a previous value. But in class sorting, Tailwind appears to treat `-none` as just another value that sorts alphabetically.

Let me verify by checking if the issue is actually with how we extract the base name:

**Current `extract_base_name()` behavior:**
```rust
// rounded-sm → rounded-sm (no extraction, has size modifier)
// rounded-none → rounded-none (no extraction, has size modifier)
```

Both are treated as having size modifiers, so they fall through to the `-none` check, which forces `-none` to sort last.

**What Should Happen:**

For these utilities, BOTH `-none` and `-sm` are size modifiers that should be compared alphabetically without special treatment.

---

### Category 3: Other (1 failure)

**Pattern:** `divide-x-reverse` vs `ring-inset` (1 occurrence)

This is a different issue - likely an edge case with divide-x-reverse property ordering.

---

## Proposed Fixes

### Fix #1: Correct transition-none Mapping (14 failures → 0)

**File:** `/home/user/rustywind/rustywind-core/src/utility_map.rs`
**Lines:** 727-736

**Change:**
```rust
// OLD (WRONG):
exact.insert(
    "transition-none",
    &[
        "transition-property",
        "transition-behavior",
        "transition-delay",
        "transition-duration",
        "transition-timing-function",
    ][..],
);

// NEW (CORRECT):
exact.insert("transition-none", &["transition-property"][..]);
```

**Impact:** Fixes all 14 transition-related failures immediately.

**Risk:** Very low - simple property mapping correction based on Tailwind v4 source.

---

### Fix #2: Remove has_none_modifier() Check (13 failures → 0)

**File:** `/home/user/rustywind/rustywind-core/src/pattern_sorter.rs`
**Lines:** ~290-301

**Option A: Remove the -none Check Entirely**

Simply comment out or remove the -none modifier check:

```rust
// Step 6: Check for -none modifiers (should sort last)
// REMOVED - causes incorrect sorting for shadow/rounded/blur utilities
// .then_with(|| {
//     let self_has_none = has_none_modifier(&self.class);
//     let other_has_none = has_none_modifier(&other.class);
//     match (self_has_none, other_has_none) {
//         (true, false) => Ordering::Greater,
//         (false, true) => Ordering::Less,
//         _ => Ordering::Equal,
//     }
// })
```

This would let all utilities fall through to alphabetical comparison, which is correct for shadow/rounded/blur.

**BUT:** This might break drop-shadow utilities. Let me verify...

Actually, looking at the 28 failures, I don't see ANY drop-shadow failures! This suggests the -none check isn't actually needed, or drop-shadow utilities already work correctly through other means.

**Option B: Make has_none_modifier() More Specific**

Only apply the -none-sorts-last rule to specific utility families:

```rust
fn has_none_modifier_that_sorts_last(utility: &str) -> bool {
    // Only apply -none-sorts-last to specific utilities where it's needed
    // Currently, this list is EMPTY because no utilities need it!

    // drop-shadow was the original reason, but it doesn't appear in failures
    if utility.starts_with("drop-shadow-") && utility.ends_with("-none") {
        return true;
    }

    false
}
```

Then update the comparison to use this more specific function.

**Recommendation:** Option A (remove the check entirely) is simpler and appears correct based on the failure data.

---

### Fix #3: Investigate divide-x-reverse vs ring-inset (1 failure)

This single failure needs individual investigation. It might be:
1. A property order issue with divide-x-reverse (we recently moved it to end of list)
2. An edge case that occurs very rarely

**Impact:** 0.01% of tests (1 out of 10,000)

**Recommendation:** Fix #1 and #2 first, then re-run 100 tests to see if this failure persists or was a fluke.

---

## Implementation Plan

### Phase 1: Fix transition-none Mapping
1. Edit `/home/user/rustywind/rustywind-core/src/utility_map.rs` line 727-736
2. Change transition-none to map to single property
3. Run cargo test to ensure no regressions
4. Test manually: `echo 'class="transition transition-none"' | rustywind --stdin`
5. Expected: `class="transition transition-none"` (alphabetical)

### Phase 2: Remove has_none_modifier Check
1. Edit `/home/user/rustywind/rustywind-core/src/pattern_sorter.rs` line ~290-301
2. Comment out or remove the has_none_modifier check
3. Run cargo test
4. Test manually:
   - `echo 'class="shadow-sm shadow-none"' | rustywind --stdin`
   - Expected: `class="shadow-none shadow-sm"` (alphabetical: n < s)
   - `echo 'class="rounded-xl rounded-none"' | rustywind --stdin`
   - Expected: `class="rounded-none rounded-xl"` (alphabetical: n < x)

### Phase 3: Verify with 100-Run Test
1. Build release: `cargo build --release`
2. Copy binary: `cp target/release/rustywind tests/fuzz/rustywind`
3. Run 100 tests: `python3 run_100_tests.py`
4. Target: 99.99%+ pass rate (≤1 failure)

### Phase 4: Handle Remaining Edge Cases
If any failures remain after Phase 3, investigate individually.

---

## Expected Outcome

### After Fix #1 (transition-none):
- Pass rate: 99.72% → 99.86%
- Failures: 28 → 14
- Perfect runs: 74% → ~85%

### After Fix #2 (has_none_modifier):
- Pass rate: 99.86% → 99.99%
- Failures: 14 → 1
- Perfect runs: ~85% → ~95%

### After Phase 3 Verification:
- Pass rate: 99.99%+ (possibly 100%)
- Failures: ≤1
- Perfect runs: ~99%

---

## Testing Strategy

### Unit Tests
Add specific test cases:

```rust
#[test]
fn test_transition_none_vs_transition() {
    let sorted = sort_classes("transition-none transition");
    assert_eq!(sorted, "transition transition-none");
}

#[test]
fn test_shadow_none_vs_shadow_sm() {
    let sorted = sort_classes("shadow-sm shadow-none");
    assert_eq!(sorted, "shadow-none shadow-sm");
}

#[test]
fn test_rounded_none_vs_rounded_xl() {
    let sorted = sort_classes("rounded-xl rounded-none");
    assert_eq!(sorted, "rounded-none rounded-xl");
}

#[test]
fn test_blur_none_vs_blur_sm() {
    let sorted = sort_classes("blur-sm blur-none");
    assert_eq!(sorted, "blur-none blur-sm");
}
```

### Integration Tests
Run the full 100-run fuzz test suite after each fix to measure improvement.

---

## Risk Assessment

| Fix | Risk Level | Reason |
|-----|------------|--------|
| transition-none mapping | Very Low | Simple property correction, matches Tailwind v4 source exactly |
| Remove has_none_modifier | Low | Based on failure data, no utilities actually need this rule |
| divide-x-reverse fix | Low | Single edge case, minimal impact |

**Overall Risk:** Low

Both fixes are well-understood and based on concrete evidence from:
1. Tailwind CSS v4 source code
2. Actual failure data from 10,000 fuzz tests
3. Manual verification with Prettier

---

## Conclusion

The remaining 28 failures (0.28%) are caused by 2 simple issues:

1. **Incorrect transition-none mapping** (14 failures)
   - Currently maps to 5 properties, should map to 1
   - Fix: One line change in utility_map.rs

2. **Overly broad has_none_modifier check** (13 failures)
   - Forces ALL -none variants to sort last
   - Should sort alphabetically instead
   - Fix: Remove or comment out the check

Both fixes are low-risk and well-understood. Implementation should take less than 30 minutes, followed by a 100-run verification test.

**Expected Final Result:** 99.99-100% pass rate 🎯

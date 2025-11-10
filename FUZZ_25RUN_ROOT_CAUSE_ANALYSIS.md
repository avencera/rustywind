# Fuzz 25-Run Root Cause Analysis

**Date:** 2025-11-10
**Pass Rate:** 95.96% (101 failures out of 2500 tests)
**Branch:** claude/fix-top-three-fuzz-issues-011CUyaP5RH5jAXudN4oRkMs

---

## Executive Summary

After 25 fuzz runs, we identified **4 primary root causes** accounting for 80+ failures (80% of all issues):

1. **Space vs Gap Ordering** - 35 failures (35%)
2. **Divide-x-reverse NOT in property-order.ts** - 25 failures (25%)
3. **Rounded Corner Cross-Axis** - 15 failures (15%)
4. **Snap/Touch Utilities** - 10 failures (10%)

---

## Root Cause #1: Space vs Gap Cross-Axis (35% of failures)

### The Problem

Space utilities are sorting incorrectly relative to gap utilities, especially in cross-axis scenarios:

**Examples:**
- `space-y-reverse` should come BEFORE `gap-y-2` (but doesn't)
- `space-x-2` should come BEFORE `gap-y-0` (but doesn't)
- `space-y-0` should come BEFORE `space-x-2` (cross-axis space issue)

### Root Cause from Tailwind Source

From `tmp/tailwindcss/packages/tailwindcss/src/property-order.ts` lines 151-155:

```typescript
'gap',                      // 151
'column-gap',               // 152
'row-gap',                  // 153
'--tw-space-x-reverse',     // 154
'--tw-space-y-reverse',     // 155
```

**Current RustyWind Order (rustywind-core/src/property_order.rs:163-171):**
```rust
"gap",                    // 163
"column-gap",             // 164
"row-gap",                // 165
"--tw-space-x-reverse",   // 166
"--tw-space-y-reverse",   // 167
"--tw-space-x",           // 168
"--tw-space-y",           // 169
```

**Issue:** `--tw-space-x` and `--tw-space-y` are NOT in Tailwind's property-order.ts!

### Investigation Needed

Space utilities use `--tw-sort: row-gap` in their CSS output (from utilities.ts:2024):

```typescript
styleRule(':where(& > :not(:last-child))', [
  decl('--tw-sort', 'row-gap'),  // ← Key finding!
  decl('--tw-space-x-reverse', '0'),
  decl('margin-inline-start', `calc(${value} * var(--tw-space-x-reverse))`),
  decl('margin-inline-end', `calc(${value} * calc(1 - var(--tw-space-x-reverse)))`),
]),
```

**The `--tw-sort` property overrides the default sorting!** Space utilities should sort using `row-gap` (index 153), not `--tw-space-x`/`--tw-space-y`.

### Recommendation

**Option A (Correct):** Remove `--tw-space-x` and `--tw-space-y` from property_order.rs entirely. Let them fall back to row-gap sorting.

**Option B (Workaround):** Keep them but position them at the exact same index as row-gap to match the --tw-sort behavior.

### Frequency
- 35 failures out of 101 (35%)
- Most common: `space-y-reverse vs gap-y-2` (5x)

---

## Root Cause #2: Divide-x-reverse Missing from Tailwind v4 (25% of failures)

### The Problem

`divide-x-reverse` is sorting incorrectly with many properties, appearing TOO EARLY in output.

**Examples:**
- Prettier: `divide-transparent divide-x-reverse`
- RustyWind: `divide-x-reverse divide-transparent`

### Root Cause from Tailwind Source

From `tmp/tailwindcss/packages/tailwindcss/src/property-order.ts` lines 158-162:

```typescript
'divide-x-width',          // 158
'divide-y-width',          // 159
'--tw-divide-y-reverse',   // 160  ← Only Y, not X!
'divide-style',            // 161
'divide-color',            // 162
```

**CRITICAL FINDING:** `--tw-divide-x-reverse` is NOT in the property-order.ts list!

However, it exists in utilities.ts (line 2441-2444):
```typescript
staticUtility('divide-x-reverse', [
  () => atRoot([property('--tw-divide-x-reverse', '0')]),
  () => styleRule(':where(& > :not(:last-child))', [decl('--tw-divide-x-reverse', '1')]),
])
```

**Current RustyWind Order (rustywind-core/src/property_order.rs:170-176):**
```rust
"divide-x-width",         // 170
"divide-y-width",         // 171
"--tw-divide-y-reverse",  // 172
"--tw-divide-x-reverse",  // 173  ← Should NOT be here!
"divide-style",           // 174
"divide-color",           // 175
```

### Why This Causes Failures

When a property is NOT in Tailwind's property-order.ts, it sorts by a different mechanism (likely alphabetically or at end of list). Our implementation has it at index 173, which causes it to sort before many properties it should follow.

### Recommendation

**Remove `--tw-divide-x-reverse` from property_order.rs entirely.** Let it fall back to whatever mechanism Tailwind uses for unlisted properties.

OR

**Investigate how Prettier/Tailwind sorts unlisted properties** and replicate that behavior.

### Frequency
- 25 failures out of 101 (25%)
- Most common patterns:
  - `divide-{color} vs divide-x-reverse` (5x)
  - `divide-{style} vs divide-x-reverse` (3x)
  - `justify-self-{value} vs divide-x-reverse` (5x)

---

## Root Cause #3: Rounded Corner Cross-Axis (15% of failures)

### The Problem

Corner utilities (rounded-tl, rounded-tr, rounded-bl, rounded-br) are sorting incorrectly relative to side utilities (rounded-t, rounded-r, rounded-b, rounded-l).

**Examples:**
- Prettier: `rounded-tl rounded-b-lg`
- RustyWind: `rounded-b-lg rounded-tl`

### Current Implementation

We use synthetic properties:
- `border-top-radius` for `rounded-t`
- `border-top-left-radius` for `rounded-tl`

### Issue

The index difference between these synthetic properties isn't large enough, or there's a modifier handling issue.

### Recommendation

**Increase index separation** between side and corner properties, or investigate how Tailwind v4 actually sorts these (they might use a different mechanism).

### Frequency
- 15 failures out of 101 (15%)

---

## Root Cause #4: Snap and Touch Utilities (10% of failures)

### Snap Utilities

**Problem:** `snap-x` sorting incorrectly relative to `snap-proximity`

From property-order.ts (lines 102-103):
```typescript
'scroll-snap-type',           // 102  ← snap-x/snap-y map here
'--tw-scroll-snap-strictness', // 103  ← snap-proximity maps here
```

So snap-x/snap-y should sort BEFORE snap-proximity.

**Fuzz failures show:**
- Prettier: `snap-x snap-proximity`
- RustyWind: `snap-proximity snap-x`

This means our implementation has snap-proximity at a LOWER index than snap-type.

**Current RustyWind:** Likely snap-x maps to `scroll-snap-align` or wrong property.

### Touch Utilities

**Problem:** `touch-pan-*` sorting incorrectly relative to `touch-manipulation`

From property-order.ts (line 95):
```typescript
'touch-action',     // 95
```

All touch utilities map to `touch-action`, so they should sort together at the same index.

**Fuzz failures suggest** they're mapping to different properties or using different indices.

### Recommendation

**Snap:** Verify snap-x/snap-y map to `scroll-snap-type` (not scroll-snap-align)
**Snap:** Verify snap-proximity maps to `--tw-scroll-snap-strictness`

**Touch:** Verify ALL touch utilities map to `touch-action` at the same index

### Frequency
- Snap: 5 failures (5%)
- Touch: 5 failures (5%)

---

## Other Edge Cases (15% of failures)

### Truncate vs Overflow
- `truncate vs overflow-auto` (2x)
- Both should map to overflow properties

### Drop Shadow
- `drop-shadow-xl vs drop-shadow-none` (1x)

### Various Misc
- 13 other one-off failures

---

## Priority Fix Order

1. **Remove/Fix Space Properties** (35% impact)
   - Remove `--tw-space-x` and `--tw-space-y` from property_order.rs
   - Let them fall back to row-gap sorting via --tw-sort

2. **Remove divide-x-reverse** (25% impact)
   - Remove `--tw-divide-x-reverse` from property_order.rs
   - Match Tailwind v4's behavior of not having it in the list

3. **Fix Snap/Touch** (10% impact)
   - Correct snap-x/y to scroll-snap-type
   - Correct snap-proximity to --tw-scroll-snap-strictness
   - Verify touch utilities all map to touch-action

4. **Investigate Rounded Corners** (15% impact)
   - Deeper investigation needed
   - May require different approach than synthetic properties

---

## Files to Modify

### Primary File
- `rustywind-core/src/property_order.rs`
  - Remove `--tw-space-x` (line 168)
  - Remove `--tw-space-y` (line 169)
  - Remove `--tw-divide-x-reverse` (line 173)
  - Verify snap property mappings
  - Verify touch property mappings

### Test Files
- Update test assertions for property count (will decrease by 3)
- Update index assertions
- Add regression tests for these specific cases

---

## Expected Impact

**If all 4 root causes are fixed:**
- Estimated pass rate: **98-99%**
- Failure reduction: **65-70 failures eliminated**
- Remaining failures: ~30-35 (mostly rounded corners and misc edge cases)

---

## Commands for Verification

```bash
# Rebuild
cargo build --release
cp target/release/rustywind tests/fuzz/rustywind

# Test specific seed
cd tests/fuzz && FUZZ_SEED=hkgz6s9bbkk npm test  # 90% run with many divide issues

# Run full 25-test suite
cd /home/user/rustywind && python3 run_25_tests.py
```

---

## Next Steps

1. Launch 3 parallel agents to fix:
   - Agent 1: Remove space and divide-x-reverse properties
   - Agent 2: Fix snap utility property mappings
   - Agent 3: Fix touch utility property mappings

2. After fixes, run 10-25 fuzz tests to measure improvement

3. Investigate rounded corners if still needed

# Root Cause Analysis: Space Cross-Axis & Rounded Corners

**Date:** 2025-11-10
**Issues:** 2 remaining sorting issues from fuzz tests

---

## Issue #1: Space Cross-Axis Sorting (3% of failures)

### The Problem
`space-y` and `space-x` utilities are sorting incorrectly relative to each other.

**Test Failures:**
- Prettier: `space-y-0, space-x-2` (space-y BEFORE space-x)
- RustyWind: `space-x-2, space-y-0` (space-x BEFORE space-y)

### Root Cause from Tailwind Source

From `./tmp/tailwindcss/packages/tailwindcss/src/utilities.ts`:

**Line 2024 (space-x):**
```typescript
styleRule(':where(& > :not(:last-child))', [
  decl('--tw-sort', 'row-gap'),  // ← space-x uses row-gap
  decl('--tw-space-x-reverse', '0'),
  decl('margin-inline-start', `calc(${value} * var(--tw-space-x-reverse))`),
])
```

**Line 2039 (space-y):**
```typescript
styleRule(':where(& > :not(:last-child))', [
  decl('--tw-sort', 'column-gap'),  // ← space-y uses column-gap
  decl('--tw-space-y-reverse', '0'),
  decl('margin-block-start', `calc(${value} * var(--tw-space-y-reverse))`),
])
```

**Key Finding:** They use DIFFERENT --tw-sort properties!
- `space-x` → `row-gap` (property-order.ts index 153)
- `space-y` → `column-gap` (property-order.ts index 152)

Since **152 < 153**, `space-y` should sort BEFORE `space-x`.

### Current RustyWind Mapping (WRONG)

In `rustywind-core/src/utility_map.rs`:
```rust
"space-x" => Some(&["--tw-space-x-reverse"][..]),  // Index 166
"space-y" => Some(&["--tw-space-y-reverse"][..]),  // Index 167
```

Since 166 < 167, RustyWind puts `space-x` BEFORE `space-y` (backwards!).

### The Fix

Change mappings to use the actual --tw-sort properties:
```rust
"space-x" => Some(&["row-gap"][..]),     // Index 153
"space-y" => Some(&["column-gap"][..]),  // Index 152
```

Now 152 < 153, so `space-y` will correctly come BEFORE `space-x`.

---

## Issue #2: Rounded Corners Cross-Axis (1-2% of failures)

### The Problem
Corner utilities (`rounded-tl`, `rounded-tr`, etc.) are sorting incorrectly relative to side utilities (`rounded-t`, `rounded-b`, etc.).

**Test Failures:**
- Prettier: `rounded-tl-lg, rounded-b` (corner BEFORE side)
- RustyWind: `rounded-b, rounded-tl-lg` (side BEFORE corner)

### Root Cause from Tailwind Source

From `./tmp/tailwindcss/packages/tailwindcss/src/utilities.ts` (line 2178-2189):

```typescript
['rounded-t', ['border-top-left-radius', 'border-top-right-radius']],
['rounded-r', ['border-top-right-radius', 'border-bottom-right-radius']],
['rounded-b', ['border-bottom-right-radius', 'border-bottom-left-radius']],
['rounded-l', ['border-top-left-radius', 'border-bottom-left-radius']],
...
['rounded-tl', ['border-top-left-radius']],
['rounded-tr', ['border-top-right-radius']],
['rounded-br', ['border-bottom-right-radius']],
['rounded-bl', ['border-bottom-left-radius']],
```

**Key Finding:** Side utilities map to MULTIPLE corner properties, NOT synthetic side properties!

From `./tmp/tailwindcss/packages/tailwindcss/src/property-order.ts` (line 181-192):
```typescript
'border-top-radius',    // 181 (synthetic, "not real")
'border-right-radius',  // 182 (synthetic, "not real")
'border-bottom-radius', // 183 (synthetic, "not real")
'border-left-radius',   // 184 (synthetic, "not real")
'border-start-start-radius',  // 185
'border-start-end-radius',    // 186
'border-end-end-radius',      // 187
'border-end-start-radius',    // 188
'border-top-left-radius',     // 189 ← ACTUAL corner property
'border-top-right-radius',    // 190 ← ACTUAL corner property
'border-bottom-right-radius', // 191 ← ACTUAL corner property
'border-bottom-left-radius',  // 192 ← ACTUAL corner property
```

**Sorting Logic for Multiple Properties:**
When a utility maps to multiple properties, Tailwind uses the MINIMUM index.

**Examples:**
- `rounded-b` → [border-bottom-right-radius (191), border-bottom-left-radius (192)] → sorts by min(191, 192) = **191**
- `rounded-tl` → [border-top-left-radius (189)] → sorts by **189**

Since **189 < 191**, `rounded-tl` should sort BEFORE `rounded-b`. ✅ This matches Prettier!

### Current RustyWind Mapping (WRONG)

In `rustywind-core/src/utility_map.rs`:
```rust
"rounded-t" => Some(&["border-top-radius"][..]),    // RustyWind index 196
"rounded-b" => Some(&["border-bottom-radius"][..]), // RustyWind index 198
"rounded-tl" => Some(&["border-top-left-radius"][..]), // RustyWind index 204
```

In `rustywind-core/src/property_order.rs`:
```
border-top-radius:       196
border-bottom-radius:    198
border-top-left-radius:  204
```

Since **198 < 204**, RustyWind puts `rounded-b` BEFORE `rounded-tl` (backwards!).

### The Fix

**Option 1 (Simple):** Map side utilities to their MINIMUM corner property instead of synthetic properties:

```rust
"rounded-t" => Some(&["border-top-left-radius"][..]),      // 189 (min of 189, 190)
"rounded-r" => Some(&["border-top-right-radius"][..]),     // 190 (min of 190, 191)
"rounded-b" => Some(&["border-bottom-right-radius"][..]),  // 191 (min of 191, 192)
"rounded-l" => Some(&["border-top-left-radius"][..]),      // 189 (min of 189, 192)
```

**Option 2 (Correct but complex):** Support multiple properties in utility_map.rs and use minimum index for sorting.

**Recommendation:** Use Option 1 (simpler, achieves same result).

**Additional:** Remove synthetic side properties from property_order.rs:
- Remove `border-top-radius` (line 196)
- Remove `border-right-radius` (line 197)
- Remove `border-bottom-radius` (line 198)
- Remove `border-left-radius` (line 199)

These are marked as "not real" in Tailwind v4 and aren't used for actual sorting.

---

## Summary of Fixes

### Fix #1: Space Utilities
**File:** `rustywind-core/src/utility_map.rs`

```rust
// Change from:
"space-x" => Some(&["--tw-space-x-reverse"][..]),
"space-y" => Some(&["--tw-space-y-reverse"][..]),

// Change to:
"space-x" => Some(&["row-gap"][..]),
"space-y" => Some(&["column-gap"][..]),
```

### Fix #2: Rounded Utilities
**File:** `rustywind-core/src/utility_map.rs`

```rust
// Change from:
"rounded-t" => Some(&["border-top-radius"][..]),
"rounded-r" => Some(&["border-right-radius"][..]),
"rounded-b" => Some(&["border-bottom-radius"][..]),
"rounded-l" => Some(&["border-left-radius"][..]),

// Change to:
"rounded-t" => Some(&["border-top-left-radius"][..]),
"rounded-r" => Some(&["border-top-right-radius"][..]),
"rounded-b" => Some(&["border-bottom-right-radius"][..]),
"rounded-l" => Some(&["border-top-left-radius"][..]),
```

**File:** `rustywind-core/src/property_order.rs`

Remove lines 196-199:
- `"border-top-radius",`
- `"border-right-radius",`
- `"border-bottom-radius",`
- `"border-left-radius",`

Property count will decrease from 342 → 338.

---

## Expected Impact

### Space Fix
- **Estimated Impact:** Fixes 3% of remaining failures (~3-5 per 100 tests)
- **Examples Fixed:**
  - `space-y-0 vs space-x-2`
  - `space-y-1 vs space-x-1`
  - `space-y-2 vs space-x-4`

### Rounded Fix
- **Estimated Impact:** Fixes 1-2% of remaining failures (~1-2 per 100 tests)
- **Examples Fixed:**
  - `rounded-tl vs rounded-b`
  - `rounded-tr vs rounded-b-lg`
  - `rounded-bl vs rounded-r`

### Total Expected Improvement
- **Current:** 95.5% pass rate
- **After fixes:** 97-99% pass rate (+1.5-3.5%)
- **Remaining issues:** <1% (minor edge cases)

---

## Commands for Testing

```bash
# Build release
cargo build --release
cp target/release/rustywind tests/fuzz/rustywind

# Run 10 fuzz tests
./run_10_fuzz_tests.sh

# Run specific failing seeds
cd tests/fuzz && FUZZ_SEED=u5ebqet5yk npm test
```

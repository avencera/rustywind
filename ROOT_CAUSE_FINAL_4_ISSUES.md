# Root Cause Analysis: Final 4 Remaining Issues

**Date:** 2025-11-10
**Current Pass Rate:** 97.37%
**Remaining Failures:** 263 out of 10,000 tests (2.63%)

---

## Issue #1: Touch Action Utilities (~40 failures, 0.4%)

### The Problem
Touch-pan utilities are sorting incorrectly relative to each other.

**Failure Examples:**
- Prettier: `touch-pan-left, touch-auto`
- RustyWind: `touch-auto, touch-pan-left`

**Other failures:**
- `touch-pan-x vs touch-pan-up`
- `touch-pan-y vs touch-auto`

### Root Cause from Tailwind Source

From `/tmp/tailwindcss/packages/tailwindcss/src/utilities.ts` (lines 1698-1725):

```typescript
// Simple touch utilities
for (let value of ['auto', 'none', 'manipulation']) {
  staticUtility(`touch-${value}`, [['touch-action', value]])
}

// touch-pan-x, left, right
for (let value of ['x', 'left', 'right']) {
  staticUtility(`touch-pan-${value}`, [
    touchProperties,
    ['--tw-pan-x', `pan-${value}`],
    ['touch-action', 'var(--tw-pan-x,) var(--tw-pan-y,) var(--tw-pinch-zoom,)'],
  ])
}

// touch-pan-y, up, down
for (let value of ['y', 'up', 'down']) {
  staticUtility(`touch-pan-${value}`, [
    touchProperties,
    ['--tw-pan-y', `pan-${value}`],
    ['touch-action', 'var(--tw-pan-x,) var(--tw-pan-y,) var(--tw-pinch-zoom,)'],
  ])
}

staticUtility('touch-pinch-zoom', [
  touchProperties,
  ['--tw-pinch-zoom', `pinch-zoom`],
  ['touch-action', 'var(--tw-pan-x,) var(--tw-pan-y,) var(--tw-pinch-zoom,)'],
])
```

From `property-order.ts` (lines 95-98):
```typescript
'touch-action',       // 95
'--tw-pan-x',        // 96
'--tw-pan-y',        // 97
'--tw-pinch-zoom',   // 98
```

**Key Finding:** Touch utilities should map to DIFFERENT properties:
- `touch-auto`, `touch-none`, `touch-manipulation` → `touch-action` (index 95)
- `touch-pan-x`, `touch-pan-left`, `touch-pan-right` → `--tw-pan-x` (index 96)
- `touch-pan-y`, `touch-pan-up`, `touch-pan-down` → `--tw-pan-y` (index 97)
- `touch-pinch-zoom` → `--tw-pinch-zoom` (index 98)

### Current RustyWind Mapping (WRONG)
All touch utilities map to `touch-action`, so they tie and sort alphabetically.

### The Fix
Map touch utilities to their specific custom properties:
```rust
// touch-auto, touch-none, touch-manipulation
"touch-auto" => Some(&["touch-action"][..]),
"touch-none" => Some(&["touch-action"][..]),
"touch-manipulation" => Some(&["touch-action"][..]),

// touch-pan-x, left, right
"touch-pan-x" => Some(&["--tw-pan-x"][..]),
"touch-pan-left" => Some(&["--tw-pan-x"][..]),
"touch-pan-right" => Some(&["--tw-pan-x"][..]),

// touch-pan-y, up, down
"touch-pan-y" => Some(&["--tw-pan-y"][..]),
"touch-pan-up" => Some(&["--tw-pan-y"][..]),
"touch-pan-down" => Some(&["--tw-pan-y"][..]),

// touch-pinch-zoom
"touch-pinch-zoom" => Some(&["--tw-pinch-zoom"][..]),
```

Expected sorting order:
1. touch-auto/none/manipulation (95)
2. touch-pan-x/left/right (96)
3. touch-pan-y/up/down (97)
4. touch-pinch-zoom (98)

---

## Issue #2: Divide-x-reverse Edge Cases (~60 failures, 0.6%)

### The Problem
`divide-x-reverse` is sorting BEFORE many properties it should sort AFTER.

**Failure Examples:**
- Prettier: `overflow-x-scroll, divide-x-reverse`
- RustyWind: `divide-x-reverse, overflow-x-scroll`

**Other failures:**
- `divide-none vs divide-x-reverse`
- `scroll-auto vs divide-x-reverse`
- `rounded vs divide-x-reverse`

### Root Cause from Tailwind Source

From `utilities.ts` (lines 2390-2443):
```typescript
// divide-x uses --tw-divide-x-reverse
handle: (value) => [
  atRoot([property('--tw-divide-x-reverse', '0')]),
  styleRule(':where(& > :not(:last-child))', [
    decl('--tw-sort', 'divide-x-width'),  // ← Uses divide-x-width for sorting!
    decl('--tw-divide-x-reverse', '0'),
    ...
  ]),
]

staticUtility('divide-x-reverse', [
  () => atRoot([property('--tw-divide-x-reverse', '0')]),
  () => styleRule(':where(& > :not(:last-child))', [decl('--tw-divide-x-reverse', '1')]),
])
```

**Key Finding:** `divide-x-reverse` does NOT have a `--tw-sort` declaration in its styleRule!

From `property-order.ts`:
- `divide-x-width` is at index 158
- `--tw-divide-y-reverse` is at index 160
- `--tw-divide-x-reverse` is NOT in the list

**Implication:** When a utility doesn't have `--tw-sort`, it should sort by the FIRST property it sets. For `divide-x-reverse`, that's `--tw-divide-x-reverse`.

Since `--tw-divide-x-reverse` is NOT in property-order.ts, Tailwind must use a fallback mechanism.

Looking at the utilities.ts more carefully, `divide-x-reverse` is a static utility that only sets `--tw-divide-x-reverse` to `'1'`. It doesn't have a --tw-sort, so it likely falls back to some default behavior.

### Current RustyWind Mapping (WRONG)
We map `divide-x-reverse` to `--tw-divide-y-reverse` (index 170), causing it to sort too early.

### The Fix
Looking at the failures, divide-x-reverse should sort very late (after overflow, scroll, rounded, etc.).

**Option 1:** Map to a very late property like `forced-color-adjust` (last property)
**Option 2:** Map to a property near the end of the border section

Looking at where divide properties are:
- divide-x-width: 158
- divide-y-width: 159
- --tw-divide-y-reverse: 160
- divide-style: 161
- divide-color: 162

And rounded/border properties are at 178-207...

Actually, looking at the test failures more carefully - divide-x-reverse is consistently sorting BEFORE properties it shouldn't. This suggests it's sorting at index 170 (--tw-divide-y-reverse).

Let me check what property would make divide-x-reverse sort AFTER all these properties...

The failures show divide-x-reverse sorting before:
- overflow-x-scroll (overflow-x is around 182)
- scroll-auto (scroll-behavior is 176)
- rounded (border-radius is 178)

So divide-x-reverse needs to sort AFTER index ~182.

**Best Fix:** Don't include divide-x-reverse in the PROPERTY_ORDER at all. Instead, handle it specially in the sorter to always sort at the end of the divide section, similar to how we might handle unknown properties.

**Simpler Fix:** Map divide-x-reverse to a property that sorts after these, like at the end of the property list. Looking at property-order.ts, we could use a late property.

Actually, the simplest fix based on Tailwind's behavior: divide-x-reverse should probably sort right AFTER divide-color (162), not at 170.

Let me reconsider...  Looking at the actual Tailwind utilities.ts code again:

```typescript
staticUtility('divide-x-reverse', [
  () => atRoot([property('--tw-divide-x-reverse', '0')]),
  () => styleRule(':where(& > :not(:last-child))', [decl('--tw-divide-x-reverse', '1')]),
])
```

It doesn't set --tw-sort. So it should sort by --tw-divide-x-reverse property. But that property isn't in property-order.ts!

**Solution:** Since --tw-divide-x-reverse isn't in Tailwind's property-order.ts, we need to handle it specially. The failures suggest it should sort very late.

Looking at the ACTUAL test output, divide-x-reverse is sorting BEFORE these properties, but Prettier expects it AFTER. This means divide-x-reverse should have a HIGHER index than we currently give it.

**Best approach:** Map divide-x-reverse to a custom property that sorts at the very end, OR add --tw-divide-x-reverse to property_order.rs at the END of the list.

---

## Issue #3: Rounded Corner Conflicts (~30 failures, 0.3%)

### The Problem
Side rounded utilities (rounded-t, rounded-l) are sorting incorrectly relative to each other when they affect the same corner.

**Failure Examples:**
- Prettier: `rounded-t-none, rounded-l`
- RustyWind: `rounded-l, rounded-t-none`

**Other failures:**
- `rounded-t vs rounded-l` (13 failures)
- `rounded-t-none vs rounded-l-lg`

### Root Cause

Both `rounded-t` and `rounded-l` map to `border-top-left-radius` (their minimum corner):
- `rounded-t` → border-top-left-radius (189)
- `rounded-l` → border-top-left-radius (189)

They TIE at index 189, so they need a secondary sort (alphabetical).

**Alphabetically:**
- `rounded-l` < `rounded-t` (l comes before t)

But Prettier expects:
- `rounded-t` < `rounded-l`

This suggests Prettier is NOT sorting alphabetically for the tiebreaker!

### Investigation Needed

Looking at Tailwind's utilities.ts:
```typescript
['rounded-t', ['border-top-left-radius', 'border-top-right-radius']],
['rounded-l', ['border-top-left-radius', 'border-bottom-left-radius']],
```

Both set border-top-left-radius, but as the FIRST vs FIRST property respectively.

**Hypothesis:** When utilities map to multiple properties and tie on the first property, Tailwind might use:
1. The SECOND property as a tiebreaker
2. Or some other ordering logic

**Second property indices:**
- rounded-t: border-top-right-radius (190)
- rounded-l: border-bottom-left-radius (192)

Since 190 < 192, this would make rounded-t come BEFORE rounded-l! ✅

### The Fix

RustyWind needs to support utilities mapping to MULTIPLE properties and use ALL properties for sorting (not just the first).

**Implementation:**
1. Modify utility_map.rs to allow multiple properties per utility
2. Modify sorting logic to compare all properties in order
3. Update rounded side utilities:
```rust
"rounded-t" => Some(&["border-top-left-radius", "border-top-right-radius"][..]),
"rounded-r" => Some(&["border-top-right-radius", "border-bottom-right-radius"][..]),
"rounded-b" => Some(&["border-bottom-right-radius", "border-bottom-left-radius"][..]),
"rounded-l" => Some(&["border-top-left-radius", "border-bottom-left-radius"][..]),
```

When sorting, compare:
1. First property indices
2. If tie, compare second property indices
3. If still tie, use alphabetical

---

## Issue #4: Space-reverse vs Gap (~30 failures, 0.3%)

### The Problem
space-reverse utilities are sorting incorrectly relative to gap utilities.

**Failure Examples:**
- Prettier: `gap-y-4, space-x-reverse`
- RustyWind: `space-x-reverse, gap-y-4`

**Other failures:**
- `space-y-reverse vs gap-x-0`
- `space-x-4 vs gap-y-2`

### Root Cause from Tailwind Source

From utilities.ts (lines 2048-2064):
```typescript
staticUtility('space-x-reverse', [
  () => atRoot([property('--tw-space-x-reverse', '0')]),
  () => styleRule(':where(& > :not(:last-child))', [
    decl('--tw-sort', 'row-gap'),           // ← space-x-reverse uses row-gap!
    decl('--tw-space-x-reverse', '1'),
  ]),
])

staticUtility('space-y-reverse', [
  () => atRoot([property('--tw-space-y-reverse', '0')]),
  () => styleRule(':where(& > :not(:last-child))', [
    decl('--tw-sort', 'column-gap'),        // ← space-y-reverse uses column-gap!
    decl('--tw-space-y-reverse', '1'),
  ]),
])
```

From gap utilities:
```typescript
spacingUtility('gap-x', ..., (value) => [decl('column-gap', value)])  // column-gap
spacingUtility('gap-y', ..., (value) => [decl('row-gap', value)])     // row-gap
```

**Key Finding:**
- `space-x-reverse` uses --tw-sort: `row-gap` (153)
- `space-y-reverse` uses --tw-sort: `column-gap` (152)
- `gap-x` uses `column-gap` (152)
- `gap-y` uses `row-gap` (153)

So:
- space-y-reverse (152) == gap-x (152) → tie, sort alphabetically → "gap-x" < "space-y-reverse"
- space-x-reverse (153) == gap-y (153) → tie, sort alphabetically → "gap-y" < "space-x-reverse"

But failures show Prettier expects:
- `gap-y-4, space-x-reverse` (gap before space-reverse)
- `gap-x-0, space-y-reverse` (gap before space-reverse)

This confirms alphabetical tiebreaking: "gap" < "space"

### Current RustyWind Mapping (WRONG)

We currently map:
```rust
exact.insert("space-x-reverse", &["row-gap"][..]);
exact.insert("space-y-reverse", &["column-gap"][..]);
```

This should be CORRECT and cause gap vs space-reverse to tie and sort alphabetically.

Let me check our actual sorting code to see if alphabetical tiebreaking is working...

**Actually, this might already be fixed!** Let me verify by checking if we have alphabetical sorting for ties.

---

## Summary of Fixes Needed

### Fix #1: Touch Action Utilities
**File:** `rustywind-core/src/utility_map.rs`

Add specific mappings for touch utilities to use custom properties:
- Map touch-auto/none/manipulation → touch-action
- Map touch-pan-x/left/right → --tw-pan-x
- Map touch-pan-y/up/down → --tw-pan-y
- Map touch-pinch-zoom → --tw-pinch-zoom

**Impact:** Fixes ~40 failures (0.4%)

### Fix #2: Divide-x-reverse
**File:** `rustywind-core/src/property_order.rs`

Add `--tw-divide-x-reverse` to the END of PROPERTY_ORDER (after `forced-color-adjust`) or use a special late-sorting mechanism.

**Impact:** Fixes ~60 failures (0.6%)

### Fix #3: Rounded Corners
**Files:** `rustywind-core/src/utility_map.rs` and sorting logic

1. Allow utilities to map to multiple properties
2. Update rounded side utilities to map to both corners
3. Modify sorting to compare all properties in order

**Impact:** Fixes ~30 failures (0.3%)

### Fix #4: Space-reverse vs Gap
**File:** Verify alphabetical tiebreaking in sorting code

This might already be working. Need to verify current behavior.

**Impact:** Fixes ~30 failures (0.3%)

---

## Expected Total Impact

- **Current:** 97.37% pass rate
- **After fixes:** 98.5-99% pass rate
- **Failure reduction:** 160 failures → <100 failures (per 10k tests)

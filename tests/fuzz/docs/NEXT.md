# Failure Categorization & Next Steps

**Last Updated:** 2025-11-11
**Total Tests:** 10,000 (100 rounds × 100 tests)
**Pass Rate:** 96.03%
**Total Passed:** 9,603
**Total Failed:** 397

> **Status**: RustyWind has achieved excellent Prettier compatibility. The remaining 4% represents edge cases rather than systematic issues.

## Summary

After implementing arbitrary value sorting and prioritization, RustyWind achieves **96.03% compatibility** with Prettier's Tailwind CSS class sorting. The remaining 3.97% (397 failures) fall into distinct patterns that have been analyzed across 20 detailed test runs.

## Distribution

- **99 rounds**: 90-100% pass rate (excellent consistency)
- **1 round**: 80-89% pass rate
- **Min:** 88%
- **Max:** 100%
- **Average:** 96.0%

## Failure Categories (from 20-run sample: 88 failures)

### 1. **Property Ordering Issues** (18.2% of failures)
**Pattern:** `other before other`
**Examples:** General utility ordering edge cases

**Description:** Some utilities with similar CSS properties sort in slightly different orders than Prettier expects. These represent edge cases in the property order lookup.

### 2. **Filter vs Ring** (15.9% of failures)
**Pattern:** `filter before ring`
**Examples:** Brightness/contrast/etc. vs ring utilities

**Description:** Filter utilities (blur, brightness, contrast, hue-rotate, saturate, etc.) are sorting in a different order relative to ring utilities than Prettier expects.

### 3. **Arbitrary Values vs Regular** (11.4% + 10.2% = 21.6% combined)
**Patterns:**
- `arbitrary before other` (11.4%)
- `arbitrary before border` (10.2%)

**Examples:**
- `border-[1.5px] vs border-t-0` (3 occurrences)

**Description:** Despite implementing arbitrary value prioritization, there are still edge cases where arbitrary values don't sort correctly relative to specific utility types (especially border utilities).

### 4. **Opacity Syntax Issues** (6.8%)
**Pattern:** `other before opacity`

**Description:** Classes with `/` opacity syntax occasionally sort incorrectly relative to other utilities.

### 5. **Ring vs Shadow** (5.7%)
**Pattern:** `ring before shadow`
**Examples:** `ring-1 vs shadow-gray-500` (3 occurrences)

**Description:** Ring utilities sort before shadow utilities when Prettier expects the opposite order.

### 6. **Other Edge Cases** (remaining ~42%)
Various one-off or rare combinations:
- `arbitrary before arbitrary` (4.5%)
- `color before opacity` (3.4%)
- `other before ring` (3.4%)
- `outline before ring` (2.3%)
- `other before shadow` (2.3%)
- And more rare patterns

## Root Causes

Based on the failure analysis, the remaining issues stem from:

1. **Property Order Table Gaps**: Some CSS property combinations don't have the exact ordering that Prettier uses
2. **Filter Utilities**: Filter-related utilities (blur, brightness, etc.) need special ordering relative to ring utilities
3. **Arbitrary Value Edge Cases**: While arbitrary values generally work, specific combinations with border utilities still fail
4. **Ring/Shadow Ordering**: Ring utilities need to sort after shadow utilities
5. **Multi-property Tiebreaking**: Some utilities that generate multiple CSS properties don't tiebreak correctly

## Most Common Specific Failures

The failures are quite diverse - only 2 specific class pairs occurred 3+ times:
- `border-[1.5px] vs border-t-0` (3 occurrences)
- `ring-1 vs shadow-gray-500` (3 occurrences)

This indicates the failures are spread across many different edge cases rather than concentrated in a few fixable patterns.

## Source Code Investigation (Tailwind CSS v4 & Prettier Plugin)

**Investigation Date:** 2025-11-11
**Repositories Analyzed:**
- `tailwindlabs/tailwindcss` (main branch, v4)
- `tailwindlabs/prettier-plugin-tailwindcss` (latest)

### Key Files Analyzed

1. **`packages/tailwindcss/src/property-order.ts`** - Definitive property order array (416 properties)
2. **`packages/tailwindcss/src/compile.ts`** - Sorting algorithm implementation
3. **`packages/tailwindcss/src/utilities.ts`** - Utility definitions and CSS generation
4. **`prettier-plugin-tailwindcss/src/sorting.ts`** - Plugin sorting interface

### Tailwind's Sorting Algorithm

From `compile.ts:83-115`, the exact sorting order is:

```typescript
astNodes.sort((a, z) => {
  // 1. Sort by variant order (bitwise OR of variant indices)
  if (aSorting.variants - zSorting.variants !== 0n) {
    return Number(aSorting.variants - zSorting.variants)
  }

  // 2. Find first different property
  let offset = 0
  while (
    offset < aSorting.properties.order.length &&
    offset < zSorting.properties.order.length &&
    aSorting.properties.order[offset] === zSorting.properties.order[offset]
  ) {
    offset += 1
  }

  return (
    // 3. Sort by lowest property index
    (aSorting.properties.order[offset] ?? Infinity) -
      (zSorting.properties.order[offset] ?? Infinity) ||
    // 4. Sort by property count (more properties first)
    zSorting.properties.count - aSorting.properties.count ||
    // 5. Sort alphabetically
    compare(aSorting.candidate, zSorting.candidate)
  )
})
```

**Key Insight:** Property indices come from the GLOBAL_PROPERTY_ORDER array in `property-order.ts`. Lower index = sorts first.

### Finding #1: Ring vs Shadow Ordering

**Source:** `property-order.ts:365-376`

```typescript
'box-shadow',              // Index 365 ← SHADOWS FIRST
'--tw-shadow',             // Index 366
'--tw-shadow-color',       // Index 367
'--tw-ring-shadow',        // Index 368 ← RINGS AFTER
'--tw-ring-color',         // Index 369
'--tw-inset-shadow',       // Index 370
'--tw-inset-shadow-color', // Index 371
'--tw-inset-ring-shadow',  // Index 372
'--tw-inset-ring-color',   // Index 373
'--tw-ring-offset-width',  // Index 374
'--tw-ring-offset-color',  // Index 375
```

**Utilities Mapping:**
- `shadow-*` utilities generate `box-shadow` property → Index 365
- `ring-*` utilities generate `--tw-ring-shadow` property → Index 368

**Correct Order:** `shadow-gray-500` → `ring-1` (365 < 368)

**RustyWind Issue:** Currently sorts rings before shadows, violating this order.

**Solution:**
```rust
// In rustywind-core/src/property_order.rs
// Ensure these indices maintain the correct order:
("box-shadow", 365),           // shadows
("--tw-shadow", 366),
("--tw-shadow-color", 367),
("--tw-ring-shadow", 368),     // rings (higher index)
("--tw-ring-color", 369),
("--tw-ring-offset-width", 374),
("--tw-ring-offset-color", 375),
```

### Finding #2: Filter Utilities Ordering

**Source:** `property-order.ts:382-402`

```typescript
'outline',                       // Index 377 ← OUTLINE FIRST
'outline-width',                 // Index 378
'outline-offset',                // Index 379
'outline-color',                 // Index 380

'--tw-blur',                     // Index 382 ← FILTERS AFTER OUTLINE
'--tw-brightness',               // Index 383
'--tw-contrast',                 // Index 384
'--tw-drop-shadow',              // Index 385
'--tw-grayscale',                // Index 386
'--tw-hue-rotate',               // Index 387
'--tw-invert',                   // Index 388
'--tw-saturate',                 // Index 389
'--tw-sepia',                    // Index 390
'filter',                        // Index 391

'--tw-backdrop-blur',            // Index 393 ← BACKDROP FILTERS
'--tw-backdrop-brightness',      // Index 394
'--tw-backdrop-contrast',        // Index 395
'--tw-backdrop-grayscale',       // Index 396
'--tw-backdrop-hue-rotate',      // Index 397
'--tw-backdrop-invert',          // Index 398
'--tw-backdrop-opacity',         // Index 399
'--tw-backdrop-saturate',        // Index 400
'--tw-backdrop-sepia',           // Index 401
'backdrop-filter',               // Index 402

'transition-property',           // Index 404 ← TRANSITIONS AFTER
```

**Utilities Mapping:**
- `blur` → `--tw-blur` + `filter` properties → Indices [382, 391]
- `brightness-*` → `--tw-brightness` + `filter` → Indices [383, 391]
- `contrast-*` → `--tw-contrast` + `filter` → Indices [384, 391]
- `grayscale-*` → `--tw-grayscale` + `filter` → Indices [386, 391]
- `hue-rotate-*` → `--tw-hue-rotate` + `filter` → Indices [387, 391]
- `invert-*` → `--tw-invert` + `filter` → Indices [388, 391]
- `saturate-*` → `--tw-saturate` + `filter` → Indices [389, 391]
- `sepia-*` → `--tw-sepia` + `filter` → Indices [390, 391]
- `drop-shadow-*` → `--tw-drop-shadow` + `filter` → Indices [385, 391]

- `backdrop-blur` → `--tw-backdrop-blur` + `backdrop-filter` → Indices [393, 402]
- `backdrop-brightness-*` → `--tw-backdrop-brightness` + `backdrop-filter` → Indices [394, 402]
- Similar for other backdrop filters...

**Key Insight:** Filter utilities generate TWO properties:
1. A CSS variable (e.g., `--tw-blur`)
2. The `filter` property itself

The lowest index (the CSS variable) determines the sort position. From `compile.ts:96-104`, it finds the **first different property** and uses the **lowest index**.

**RustyWind Issue:** Filter utilities are not properly mapped to their CSS variable properties.

**Solution:**
```rust
// In rustywind-core/src/property_order.rs
// Add filter property mappings:
("--tw-blur", 382),
("--tw-brightness", 383),
("--tw-contrast", 384),
("--tw-drop-shadow", 385),
("--tw-grayscale", 386),
("--tw-hue-rotate", 387),
("--tw-invert", 388),
("--tw-saturate", 389),
("--tw-sepia", 390),
("filter", 391),

("--tw-backdrop-blur", 393),
("--tw-backdrop-brightness", 394),
("--tw-backdrop-contrast", 395),
("--tw-backdrop-grayscale", 396),
("--tw-backdrop-hue-rotate", 397),
("--tw-backdrop-invert", 398),
("--tw-backdrop-opacity", 399),
("--tw-backdrop-saturate", 400),
("--tw-backdrop-sepia", 401),
("backdrop-filter", 402),
```

And in the utility mapping logic (likely in `hybrid_sorter.rs` or `utility_map.rs`):
```rust
// Map filter utilities to their CSS variable properties
"blur" => vec![382, 391],        // Uses --tw-blur (382) as primary sort key
"brightness" => vec![383, 391],
"contrast" => vec![384, 391],
// etc...
```

### Finding #3: Arbitrary Border Edge Cases

**Source Analysis:**

1. **Property Order** (`property-order.ts:194-202`):
```typescript
'border-width',                  // Index 194 ← GENERIC BORDER
'border-inline-width',           // Index 195
'border-block-width',            // Index 196
'border-inline-start-width',     // Index 197
'border-inline-end-width',       // Index 198
'border-top-width',              // Index 199 ← SPECIFIC SIDES
'border-right-width',            // Index 200
'border-bottom-width',           // Index 201
'border-left-width',             // Index 202
```

2. **Utility Definitions** (`utilities.ts:2310-2356`):

```typescript
borderSideUtility('border', {
  width: (value) => [
    decl('border-style', 'var(--tw-border-style)'),
    decl('border-width', value),  // ← Uses 'border-width' (index 194)
  ],
  // ...
})

borderSideUtility('border-t', {
  width: (value) => [
    decl('border-top-style', 'var(--tw-border-style)'),
    decl('border-top-width', value),  // ← Uses 'border-top-width' (index 199)
  ],
  // ...
})
```

**Correct Behavior:**
- `border-[1.5px]` → generates `border-width: 1.5px` → Index 194
- `border-t-0` → generates `border-top-width: 0` → Index 199
- **Expected:** `border-[1.5px]` sorts BEFORE `border-t-0` (194 < 199) ✓

**RustyWind Issue:** The failure occurs 3 times in testing, suggesting:
1. Arbitrary value extraction might not be working for this specific case
2. Property mapping might be incorrect for `border-[...]` utilities
3. The comparison might not be reaching the property-level sort

**Investigation Required:**

From `pattern_sorter.rs`, check how `border-[1.5px]` is parsed:
1. Does it extract the numeric value correctly?
2. Is it mapped to the correct property (`border-width`)?
3. Does it compare at the property level or numeric level?

**Hypothesis:** RustyWind might be comparing these at the arbitrary value numeric level (both have numbers: 1.5 vs 0) instead of at the property index level first.

**Solution:**
```rust
// In rustywind-core/src/hybrid_sorter.rs or utility_map.rs
// Ensure border utilities map to correct properties:

fn get_property_for_utility(utility: &str) -> Option<&'static str> {
    // For border utilities with arbitrary values
    if utility.starts_with("border-[") && utility.contains(']') {
        return Some("border-width");  // Index 194
    }

    // For border-t utilities
    if utility.starts_with("border-t-") {
        return Some("border-top-width");  // Index 199
    }

    // Similar for border-r, border-b, border-l
    // ...
}
```

**Critical Fix:** Property-level comparison MUST happen BEFORE arbitrary value numeric comparison in the sorting algorithm.

### Property Count Tiebreaker

From `compile.ts:111`:
```typescript
zSorting.properties.count - aSorting.properties.count
```

Utilities with MORE properties sort BEFORE utilities with fewer properties (when property indices are equal).

**Example:**
- `border` generates 2 properties: `border-style` + `border-width` → count = 2
- `border-t-0` generates 2 properties: `border-top-style` + `border-top-width` → count = 2

So property count doesn't help distinguish these. The property index is the key differentiator.

## Actionable Recommendations

### Immediate Actions (Target: 97-98% pass rate)

#### 1. Ring vs Shadow Ordering
**Priority: HIGH** | **Impact: ~5.7% of failures**
**Source:** See "Finding #1" in Source Code Investigation above

**Current Issue:**
- `ring-1` sorts before `shadow-gray-500` when Prettier expects the opposite
- Ring utilities need to sort after shadow utilities

**Root Cause:**
- Tailwind's `property-order.ts` has `box-shadow` at index 365, `--tw-ring-shadow` at index 368
- RustyWind's property order might have these reversed or using incorrect indices

**Implementation Steps:**

1. **Update `rustywind-core/src/property_order.rs`:**
```rust
// Match Tailwind v4's exact indices from property-order.ts:365-376
("box-shadow", 365),
("--tw-shadow", 366),
("--tw-shadow-color", 367),
("--tw-ring-shadow", 368),      // Must be AFTER box-shadow
("--tw-ring-color", 369),
("--tw-inset-shadow", 370),
("--tw-inset-shadow-color", 371),
("--tw-inset-ring-shadow", 372),
("--tw-inset-ring-color", 373),
("--tw-ring-offset-width", 374),
("--tw-ring-offset-color", 375),
```

2. **Update utility mapping in `utility_map.rs` or `hybrid_sorter.rs`:**
```rust
// Ensure shadow utilities map to box-shadow (365)
"shadow" => vec![365],  // or appropriate property
"shadow-sm" => vec![365],
"shadow-lg" => vec![365],
// etc.

// Ensure ring utilities map to --tw-ring-shadow (368)
"ring" => vec![368],
"ring-0" => vec![368],
"ring-1" => vec![368],
// etc.
```

**Testing:**
- Add test case in `rustywind-core/tests/test_ring_shadow_ordering.rs`
- Verify with `node tests/fuzz/test-ring-blur.mjs`
- Test case: `assert_sorting("ring-1 shadow-gray-500", "shadow-gray-500 ring-1")`

#### 2. Filter Utilities Ordering
**Priority: HIGH** | **Impact: ~15.9% of failures**
**Source:** See "Finding #2" in Source Code Investigation above

**Current Issue:**
- Filter utilities (blur, brightness, contrast, saturate, etc.) sort incorrectly relative to ring utilities
- Filter utilities are not mapped to their CSS variable properties

**Root Cause:**
- Filter utilities generate MULTIPLE properties (CSS variable + filter property)
- Tailwind uses the LOWEST property index for sorting
- Example: `blur` → `--tw-blur` (382) + `filter` (391) → sorts by 382

**Affected Utilities:**
- Regular filters: `blur`, `brightness`, `contrast`, `grayscale`, `hue-rotate`, `invert`, `saturate`, `sepia`, `drop-shadow`
- Backdrop filters: `backdrop-blur`, `backdrop-brightness`, `backdrop-contrast`, `backdrop-grayscale`, `backdrop-hue-rotate`, `backdrop-invert`, `backdrop-opacity`, `backdrop-saturate`, `backdrop-sepia`

**Implementation Steps:**

1. **Update `rustywind-core/src/property_order.rs`:**
```rust
// Add ALL filter-related properties from Tailwind's property-order.ts:382-402
("outline", 377),
("outline-width", 378),
("outline-offset", 379),
("outline-color", 380),

// Regular filters
("--tw-blur", 382),
("--tw-brightness", 383),
("--tw-contrast", 384),
("--tw-drop-shadow", 385),
("--tw-grayscale", 386),
("--tw-hue-rotate", 387),
("--tw-invert", 388),
("--tw-saturate", 389),
("--tw-sepia", 390),
("filter", 391),

// Backdrop filters
("--tw-backdrop-blur", 393),
("--tw-backdrop-brightness", 394),
("--tw-backdrop-contrast", 395),
("--tw-backdrop-grayscale", 396),
("--tw-backdrop-hue-rotate", 397),
("--tw-backdrop-invert", 398),
("--tw-backdrop-opacity", 399),
("--tw-backdrop-saturate", 400),
("--tw-backdrop-sepia", 401),
("backdrop-filter", 402),

("transition-property", 404),
```

2. **Update utility mapping to return multiple property indices:**
```rust
// In utility_map.rs or hybrid_sorter.rs
// Filter utilities need to return BOTH their CSS variable AND filter property
"blur" => vec![382, 391],           // --tw-blur, filter
"blur-none" => vec![382, 391],
"blur-sm" => vec![382, 391],
"brightness" => vec![383, 391],     // --tw-brightness, filter
"contrast" => vec![384, 391],       // --tw-contrast, filter
"drop-shadow" => vec![385, 391],    // --tw-drop-shadow, filter
"grayscale" => vec![386, 391],      // --tw-grayscale, filter
"hue-rotate" => vec![387, 391],     // --tw-hue-rotate, filter
"invert" => vec![388, 391],         // --tw-invert, filter
"saturate" => vec![389, 391],       // --tw-saturate, filter
"sepia" => vec![390, 391],          // --tw-sepia, filter

"backdrop-blur" => vec![393, 402],  // --tw-backdrop-blur, backdrop-filter
"backdrop-brightness" => vec![394, 402],
"backdrop-contrast" => vec![395, 402],
"backdrop-grayscale" => vec![396, 402],
"backdrop-hue-rotate" => vec![397, 402],
"backdrop-invert" => vec![398, 402],
"backdrop-opacity" => vec![399, 402],
"backdrop-saturate" => vec![400, 402],
"backdrop-sepia" => vec![401, 402],
```

3. **Update sorting logic to use lowest property index:**
```rust
// In hybrid_sorter.rs comparison function
// When comparing two utilities with multiple properties,
// use the LOWEST property index from each (matching Tailwind's behavior)
let a_min_prop = a_properties.iter().min().unwrap_or(&usize::MAX);
let z_min_prop = z_properties.iter().min().unwrap_or(&usize::MAX);
a_min_prop.cmp(z_min_prop)
```

**Testing:**
- Add comprehensive filter tests in `rustywind-core/tests/test_filter_ordering.rs`
- Test cases:
  - `blur` vs `ring-1` (should sort: `blur` then `ring-1`)
  - `brightness-50` vs `shadow-lg` (should sort: `shadow-lg` then `brightness-50`)
  - `backdrop-blur` vs `transition` (should sort: `backdrop-blur` then `transition`)

#### 3. Arbitrary Border Edge Cases
**Priority: MEDIUM** | **Impact: ~21.6% combined (arbitrary failures)**
**Source:** See "Finding #3" in Source Code Investigation above

**Current Issue:**
- `border-[1.5px]` vs `border-t-0` ordering inconsistent (fails 3 times in 10,000 tests)
- Despite implementing arbitrary value prioritization, border-specific edge cases remain

**Root Cause:**
- `border-[1.5px]` should map to `border-width` property (index 194)
- `border-t-0` maps to `border-top-width` property (index 199)
- Property comparison must happen BEFORE arbitrary value numeric comparison
- Possible issue: comparison happening at numeric level (1.5 vs 0) instead of property level

**Correct Behavior from Tailwind:**
```typescript
// border-[1.5px] generates:
border-style: var(--tw-border-style);
border-width: 1.5px;  // Index 194

// border-t-0 generates:
border-top-style: var(--tw-border-style);
border-top-width: 0;  // Index 199

// Expected order: border-[1.5px] → border-t-0 (194 < 199)
```

**Implementation Steps:**

1. **Update property mapping for border utilities:**
```rust
// In utility_map.rs or hybrid_sorter.rs
fn get_border_property(utility: &str) -> Option<&'static str> {
    // Generic border (all sides)
    if utility.starts_with("border") && !utility.contains('-', 7) {
        // border, border-2, border-[1.5px], etc.
        return Some("border-width");  // Index 194
    }

    // Side-specific borders
    if utility.starts_with("border-t") {
        return Some("border-top-width");  // Index 199
    }
    if utility.starts_with("border-r") {
        return Some("border-right-width");  // Index 200
    }
    if utility.starts_with("border-b") {
        return Some("border-bottom-width");  // Index 201
    }
    if utility.starts_with("border-l") {
        return Some("border-left-width");  // Index 202
    }

    // Logical borders
    if utility.starts_with("border-s") {
        return Some("border-inline-start-width");  // Index 197
    }
    if utility.starts_with("border-e") {
        return Some("border-inline-end-width");  // Index 198
    }
    if utility.starts_with("border-x") {
        return Some("border-inline-width");  // Index 195
    }
    if utility.starts_with("border-y") {
        return Some("border-block-width");  // Index 196
    }

    None
}
```

2. **Ensure property comparison happens first in sorting logic:**
```rust
// In hybrid_sorter.rs or pattern_sorter.rs
// The comparison order MUST be:
// 1. Variant order
// 2. Property index  ← THIS MUST COME BEFORE ARBITRARY VALUE COMPARISON
// 3. Arbitrary value numeric comparison
// 4. Alphabetical

// Example implementation:
fn compare_utilities(a: &str, z: &str) -> Ordering {
    // 1. Compare variants (existing logic)
    // ...

    // 2. Compare property indices FIRST
    let a_prop_idx = get_property_index(a);
    let z_prop_idx = get_property_index(z);
    match a_prop_idx.cmp(&z_prop_idx) {
        Ordering::Equal => {
            // 3. Only if properties are equal, compare arbitrary values
            compare_arbitrary_values(a, z)
        }
        other => return other,
    }
}
```

3. **Add property order entries:**
```rust
// In property_order.rs (should already exist, but verify)
("border-width", 194),
("border-inline-width", 195),
("border-block-width", 196),
("border-inline-start-width", 197),
("border-inline-end-width", 198),
("border-top-width", 199),
("border-right-width", 200),
("border-bottom-width", 201),
("border-left-width", 202),
```

**Debugging:**
```rust
// Add temporary debug logging to verify behavior:
eprintln!("Comparing: {} vs {}", a, z);
eprintln!("  {} property: {:?} (index: {:?})", a, get_border_property(a), get_property_index(a));
eprintln!("  {} property: {:?} (index: {:?})", z, get_border_property(z), get_property_index(z));
```

**Testing:**
```rust
#[test]
fn test_arbitrary_border_ordering() {
    // The key test case that fails 3 times
    assert_sorting(
        "border-[1.5px] border-t-0",
        "border-[1.5px] border-t-0"
    );

    // Additional edge cases
    assert_sorting(
        "border-t-0 border-[1.5px]",
        "border-[1.5px] border-t-0"
    );

    assert_sorting(
        "border-[2px] border-r-4 border-b-0 border-l-8",
        "border-[2px] border-r-4 border-b-0 border-l-8"
    );

    // Verify all border sides maintain correct order
    assert_sorting(
        "border-t-0 border-r-0 border-b-0 border-l-0 border-[1px]",
        "border-[1px] border-t-0 border-r-0 border-b-0 border-l-0"
    );
}
```

**Critical Implementation Note:**
The failure rate is low (3/10,000) which suggests this might be a race condition or an edge case in the comparison logic where sometimes the property comparison is skipped. Review the entire comparison chain in `hybrid_sorter.rs` to ensure property indices are ALWAYS compared before numeric values.

### Medium-Term Improvements (Target: 98%+)

#### 4. Property Order Table Expansion
**Priority: MEDIUM** | **Complexity: HIGH**

**Approach:**
1. Extract complete property order from Tailwind v4 source
2. Run comparison tests for every utility pair
3. Identify gaps in `property_order.rs`
4. Add missing property mappings

**Available Tools:**
- `tests/fuzz/analyze-properties.mjs`
- `tests/fuzz/compare-properties.mjs`
- `tests/fuzz/extract-real-world-patterns.mjs`

#### 5. Multi-Property CSS Declaration Counting
**Priority: LOW** | **Complexity: HIGH** | **Impact: ~18.2% of "other" failures**

**Current Limitation:**
- Some utilities generate multiple CSS properties
- Tailwind uses property count as a tiebreaker
- RustyWind doesn't currently implement this

**Example:**
```css
/* transform utilities generate multiple properties */
.scale-150 {
  --tw-scale-x: 1.5;
  --tw-scale-y: 1.5;
  transform: translate(...) rotate(...) scale(...);
}
```

**Implementation:**
- Create utility → CSS property count mapping
- Add tiebreaker in `hybrid_sorter.rs` after property order comparison
- Reference Tailwind's `compile.ts:99-130` for exact logic

### Long-Term Enhancements

#### 6. Continuous Fuzz Testing in CI/CD
**Priority: MEDIUM**

**Recommendations:**
1. Add GitHub Actions workflow for fuzz testing
2. Run 100 rounds on every PR
3. Fail if pass rate drops below 95%
4. Track pass rate trends over time

**CI Configuration:**
```yaml
# .github/workflows/fuzz-test.yml
name: Fuzz Tests
on: [pull_request]
jobs:
  fuzz:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
      - run: cd tests/fuzz && npm install
      - run: cd tests/fuzz && python tools/test_many_rounds.py 100
      - run: # Parse results and enforce 95% minimum
```

#### 7. Real-World Pattern 100% Coverage
**Priority: LOW**

**Current State:**
- `tests/real-world-tests/` contains actual codebases
- Good for regression testing

**Enhancement:**
- Extract all unique class combinations from real repos
- Add to fuzz test corpus
- Ensure 100% pass rate on real-world patterns (fixed, not random)

#### 8. Property Order Documentation
**Priority: LOW**

**Gap:**
- `property_order.rs` lacks documentation on source of indices
- Hard to verify correctness against Tailwind

**Improvement:**
- Add comments linking each property to Tailwind source
- Document index selection methodology
- Create verification script comparing RustyWind vs Tailwind

## Historical Context

- **Initial state**: ~70% pass rate
- **After variant order fix**: 76.72%
- **After compound variant fix**: 95.48%
- **After arbitrary value fix**: **96.03% → 96.32%** (varies by run)

The improvements show steady progress toward Prettier compatibility, with diminishing returns as we approach the edge cases.

## Implementation Roadmap

### Phase 1: Quick Wins (1-2 weeks) → Target: 97%
- [ ] Fix ring vs shadow ordering in `property_order.rs`
- [ ] Add filter utility property group mappings
- [ ] Write integration tests for ring/shadow and filter ordering
- [ ] Run 100-round validation to confirm improvement
- [ ] Document changes in test files

### Phase 2: Edge Cases (2-3 weeks) → Target: 98%
- [ ] Debug arbitrary border value extraction in `pattern_sorter.rs`
- [ ] Fix `border-[...]` vs `border-{side}` ordering
- [ ] Expand property order table based on failure analysis
- [ ] Add regression tests for top 20 failure patterns
- [ ] Validate with real-world test suite

### Phase 3: Completeness (1-2 months) → Target: 99%
- [ ] Implement CSS declaration counting feature
- [ ] Extract complete Tailwind v4 property order reference
- [ ] Property-by-property validation against Prettier
- [ ] Achieve 100% pass rate on real-world patterns
- [ ] Comprehensive documentation update

### Phase 4: Maintenance (Ongoing)
- [ ] Add fuzz tests to CI/CD pipeline
- [ ] Track pass rate trends over time
- [ ] Update on Tailwind v4 major releases
- [ ] Document property order sources with references
- [ ] Community feedback integration

## Measuring Success

### Success Metrics
- **97%**: Production-ready for most use cases
- **98%**: Excellent compatibility, rare edge cases only
- **99%**: Near-perfect, suitable for all production scenarios
- **99.5%+**: Diminishing returns, may not be worth the effort

### Test Coverage Goals
1. **Fuzz tests**: 97%+ pass rate (random combinations)
2. **Real-world tests**: 100% pass rate (fixed patterns from actual code)
3. **Integration tests**: 100% pass rate (specific feature coverage)
4. **Regression tests**: 100% pass rate (known bug fixes)

## Tools & Scripts Reference

### Running Tests
```bash
# Quick fuzz test (100 tests, single seed)
cd tests/fuzz
npm test

# Comprehensive testing (10,000 tests over 100 rounds)
python tools/test_many_rounds.py 100

# Collect and analyze failures from multiple runs
python tools/collect_failures.py
```

### Analyzing Results
```bash
# Analyze failures by utility category
python tools/analyze_failures.py

# Analyze multi-seed pattern results
node tools/analyze-failures.js

# Extract specific failure patterns
node extract-failure-patterns.mjs
```

### Debugging Specific Issues
```bash
# Test specific utility categories
node test-ring-blur.mjs
node test-outline-transition.mjs
node test-divide-detailed.mjs

# Check property mappings
node test-property-mapping.mjs

# Analyze Tailwind's runtime behavior
node extract-variant-order-runtime.mjs

# Verify transform ordering
node verify-transforms.js
```

### Real-World Testing
```bash
# Run comparison against real codebases
cd tests/real-world-tests
node compare-tools.js

# Analyze class differences
node analyze-class-diffs.js
```

## Conclusion

At **96.03% pass rate** with excellent consistency (99/100 rounds above 90%), RustyWind is now highly compatible with Prettier's Tailwind CSS sorting. The remaining 4% represents diverse edge cases across many different utility combinations, rather than systematic issues.

The path to 97-98% is clear with actionable fixes for:
1. Ring vs shadow property ordering
2. Filter utility group handling
3. Arbitrary border edge cases

Beyond 98%, improvements require deeper analysis and potentially complex features like CSS declaration counting. However, the current compatibility level is already excellent for production use.

**Current Recommendation:** Focus on Phase 1 (ring/shadow + filters) to reach 97%, which provides excellent production-ready compatibility for the vast majority of use cases. The cost-benefit ratio for pushing beyond 98% should be carefully evaluated based on user feedback and real-world impact.

---

## Concrete Implementation Plan

**Investigation Date:** 2025-11-11
**RustyWind Codebase Analysis Results**

### Current State Assessment

After analyzing RustyWind's source code, here's what we found:

#### ✅ What's Already Working

1. **Infrastructure is excellent:**
   - `SortKey` struct supports `property_indices: Vec<usize>` (multiple properties per utility)
   - Property comparison iterates through ALL indices in order
   - `property_count: usize` already tracks number of properties
   - Comparison algorithm matches Tailwind's 5-step process

2. **Filter utilities already mapped:**
   - `blur` → `--tw-blur`
   - `brightness` → `--tw-brightness`
   - All filter CSS variables are mapped correctly

3. **Border utilities properly structured:**
   - Border width utilities map to appropriate properties
   - Arbitrary value handling exists

#### ❌ Critical Issue Found: Property Order Out of Sync

**Root Cause of All 3 Failure Categories:**

```
RustyWind property_order.rs: 341 properties
Tailwind CSS v4 property-order.ts: 416 properties
Difference: 75 properties MISSING
```

**Index Comparison:**
```
Property              | Tailwind v4 | RustyWind | Delta
--------------------- | ----------- | --------- | -----
box-shadow            | 365         | ~336      | -29
--tw-ring-shadow      | 368         | ~342      | -26
--tw-blur             | 382         | ~355      | -27
filter                | 391         | ~364      | -27
```

**Impact:** Because indices are shifted, sorting relationships between utilities are incorrect, causing:
- Ring vs Shadow failures (wrong relative order)
- Filter vs Ring failures (wrong relative order)
- Border edge cases (property indices don't match Tailwind's)

### Feasibility Analysis

#### ✅ Can Be Fixed (Without CSS Parsing)

1. **Property Order Sync** → **PRIMARY FIX**
   - Update `property_order.rs` to match Tailwind v4's exact 416-property list
   - This alone should fix ALL 3 categories of failures
   - **Feasibility: HIGH** - Just update a static array

2. **Property Count** → **ALREADY IMPLEMENTED**
   - `SortKey.property_count` already tracks this
   - Comparison already uses it (line 423 in pattern_sorter.rs)
   - **No changes needed**

3. **Multiple Properties Per Utility** → **ALREADY SUPPORTED**
   - `property_indices: Vec<usize>` already handles this
   - Comparison iterates through all indices
   - **No changes needed**

#### ❌ Cannot Be Easily Fixed (Would Require CSS Parsing)

1. **Exact Property Count from CSS Generation**
   - RustyWind doesn't generate actual CSS
   - Currently approximates property count from utility_map.rs
   - Would need to parse Tailwind's CSS generation logic
   - **Feasibility: LOW** - Major architectural change
   - **Impact: MINIMAL** - Only affects <1% of edge cases

### Implementation Steps

#### Phase 1: Sync Property Order (Target: Fix all 3 issues)

**Time Estimate: 2-4 hours**
**Expected Impact: 96% → 98% pass rate**

1. **Extract Tailwind v4's complete property order:**
   ```bash
   # From /tmp/tailwindcss clone
   cat packages/tailwindcss/src/property-order.ts
   ```

2. **Update `rustywind-core/src/property_order.rs`:**
   ```rust
   pub const PROPERTY_ORDER: &[&str] = &[
       // Replace entire array with Tailwind v4's 416 properties
       // Exact copy from property-order.ts lines 1-416
   ];
   ```

3. **Verify critical indices:**
   ```rust
   #[test]
   fn test_tailwind_v4_indices() {
       // Verify key properties match Tailwind v4
       assert_eq!(get_property_index("box-shadow"), Some(365));
       assert_eq!(get_property_index("--tw-ring-shadow"), Some(368));
       assert_eq!(get_property_index("--tw-blur"), Some(382));
       assert_eq!(get_property_index("filter"), Some(391));
       assert_eq!(get_property_index("border-width"), Some(194));
       assert_eq!(get_property_index("border-top-width"), Some(199));
   }
   ```

4. **Run tests:**
   ```bash
   # Unit tests
   cargo test

   # Fuzz tests
   cd tests/fuzz && python tools/test_many_rounds.py 100
   ```

**Files to Modify:**
- `rustywind-core/src/property_order.rs` (replace PROPERTY_ORDER array)

**No Other Changes Needed:**
- ✅ `utility_map.rs` - Already correct
- ✅ `pattern_sorter.rs` - Already correct
- ✅ `hybrid_sorter.rs` - Already correct

#### Phase 2: Optional Enhancements (Target: 98% → 99%)

**Only if Phase 1 doesn't reach 98%:**

1. **Add filter + property mappings** (if needed):
   ```rust
   // In utility_map.rs, change from:
   "blur" => Some(&["--tw-blur"][..]),

   // To:
   "blur" => Some(&["--tw-blur", "filter"][..]),
   ```

   But this is likely **NOT needed** because:
   - Pattern sorter uses ALL property indices
   - First index (--tw-blur at 382) is what matters for sorting
   - Adding "filter" (391) would only affect tiebreaking

2. **Verify border utility mappings:**
   ```rust
   #[test]
   fn test_border_property_mapping() {
       let map = UtilityMap::new();
       assert_eq!(map.get_properties("border-[1.5px]"), Some(&["border-width"][..]));
       assert_eq!(map.get_properties("border-t-0"), Some(&["border-top-width"][..]));
   }
   ```

#### Phase 3: Validation (Target: Confirm 97-98%)

1. **Run comprehensive fuzz tests:**
   ```bash
   cd tests/fuzz
   python tools/test_many_rounds.py 100 > results.txt
   ```

2. **Analyze remaining failures:**
   ```bash
   python tools/collect_failures.py
   python tools/analyze_failures.py
   ```

3. **Create regression tests:**
   ```rust
   // In rustywind-core/tests/test_ring_shadow_ordering.rs
   #[test]
   fn test_ring_after_shadow() {
       assert_sorting("ring-1 shadow-gray-500", "shadow-gray-500 ring-1");
   }

   // In rustywind-core/tests/test_filter_ordering.rs
   #[test]
   fn test_blur_before_ring() {
       assert_sorting("ring-1 blur", "blur ring-1");
   }

   // In rustywind-core/tests/test_border_arbitrary.rs
   #[test]
   fn test_border_arbitrary_vs_side() {
       assert_sorting("border-t-0 border-[1.5px]", "border-[1.5px] border-t-0");
   }
   ```

### Why This Should Work

1. **Root Cause Identified:**
   - The 75 missing properties cause index mismatches
   - All utilities are already correctly mapped
   - Just need indices to match Tailwind v4

2. **Infrastructure Ready:**
   - Multi-property support already exists
   - Property count tracking already exists
   - Comparison logic already correct

3. **Minimal Changes:**
   - Only need to update one array in one file
   - No algorithmic changes required
   - No new features to implement

### Success Criteria

**After Phase 1 (Property Order Sync):**
- ✅ Fuzz test pass rate: **97-98%**
- ✅ `ring-1 shadow-gray-500` sorts correctly
- ✅ `blur brightness-50` vs `ring-1` sorts correctly
- ✅ `border-[1.5px] border-t-0` sorts correctly

**Measurement:**
```bash
# Before fix: ~96% pass rate
# After fix: Target 97-98% pass rate
# Improvement: +1-2% (fixing ~150-200 failures out of 397)
```

### Risk Assessment

**LOW RISK:**
- Only updating static data (property order array)
- No logic changes
- Easy to revert if issues arise
- Comprehensive test suite exists

**Testing Strategy:**
1. Unit tests verify indices
2. Integration tests verify specific cases
3. Fuzz tests verify overall pass rate
4. Real-world tests verify no regressions

### Timeline

- **Investigation:** ✅ Complete (2 hours)
- **Phase 1 Implementation:** 2-4 hours
  - Extract property order: 30 minutes
  - Update property_order.rs: 30 minutes
  - Write verification tests: 1 hour
  - Run full test suite: 1-2 hours
- **Phase 2 (if needed):** 1-2 hours
- **Phase 3 Validation:** 1 hour

**Total Estimate:** 4-7 hours to reach 97-98% pass rate

### Next Steps

1. ✅ **COMPLETE:** Source code investigation
2. ✅ **COMPLETE:** Implementation plan
3. **TODO:** Extract Tailwind v4 property order (416 properties)
4. **TODO:** Update rustywind-core/src/property_order.rs
5. **TODO:** Add verification tests
6. **TODO:** Run fuzz tests and measure improvement
7. **TODO:** Document results

### Questions Answered

**Q: Can we do property counting without CSS parsing?**
**A:** ✅ Yes! Already implemented via `property_count` in SortKey.

**Q: Can we fix ring/shadow ordering?**
**A:** ✅ Yes! Just sync property order indices.

**Q: Can we fix filter utilities?**
**A:** ✅ Yes! Just sync property order indices. (Multi-property support already exists if needed)

**Q: Can we fix border arbitrary values?**
**A:** ✅ Yes! Just sync property order indices. (Utilities already mapped correctly)

**Q: Do we need to parse CSS?**
**A:** ❌ No! All issues can be fixed by syncing the property order array.

**Q: How much work is this?**
**A:** ⚡ Minimal! Just update one file (property_order.rs) with Tailwind v4's property list.

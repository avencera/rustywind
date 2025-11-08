# How Prettier Plugin Determines Tailwind CSS Class Order

**Research Session:** 011CUvKHNdhS77igNg64EwfD
**Date:** 2025-11-08

## Executive Summary

The Prettier plugin for Tailwind CSS (prettier-plugin-tailwindcss) determines class order by consuming a sorting API provided by the core Tailwind CSS library. Classes are sorted based on the **CSS properties they generate**, not arbitrary categories. The order is determined by a hardcoded list of 416+ CSS properties in `property-order.ts`.

## Architecture Overview

### Two-Part System

```
┌─────────────────────────────────────┐
│  Prettier Plugin (separate repo)   │
│  - Finds class attributes          │
│  - Calls getClassOrder()            │
│  - Performs final sorting           │
└─────────────┬───────────────────────┘
              │ uses API
              ▼
┌─────────────────────────────────────┐
│  Tailwind CSS Core (this repo)     │
│  - Provides getClassOrder()         │
│  - Compiles classes to CSS          │
│  - Assigns sort order numbers       │
└─────────────────────────────────────┘
```

**Key Insight:** The core library doesn't actually sort classes—it assigns each class a sort order number. The Prettier plugin performs the actual sorting.

## The Sorting Algorithm

### Step-by-Step Process

#### 1. Parse Class Names
```typescript
// packages/tailwindcss/src/sort.ts:4
export function getClassOrder(design: DesignSystem, classes: string[]): [string, bigint | null][]
```

Input: `['px-3', 'py-4', 'bg-red-500', 'hover:p-1']`

#### 2. Compile to CSS and Extract Properties
For each class, Tailwind compiles it to CSS and notes which properties are generated:

- `px-3` → `padding-left`, `padding-right`
- `py-4` → `padding-top`, `padding-bottom`
- `bg-red-500` → `background-color`
- `hover:p-1` → (hover variant) + `padding`

#### 3. Look Up Property Positions

From `packages/tailwindcss/src/property-order.ts`:

```typescript
export default [
  'container-type',
  'pointer-events',
  'visibility',
  'position',
  // ... line 8
  'inset',
  // ... line 37
  'margin',
  // ... line 224
  'background-color',
  // ... line 315
  'padding',
  'padding-inline',
  'padding-block',
  // ... line 320
  'padding-top',
  'padding-right',
  'padding-bottom',
  'padding-left',
  // ... 416 total properties
]
```

Each property's position in this array determines its sort order:
- `background-color` is at index ~224
- `padding` is at index ~315
- `padding-left` is at index ~322

#### 4. Assign Sort Order Numbers

From `packages/tailwindcss/src/compile.ts:325` - `getPropertySort()`:

```typescript
function getPropertySort(nodes: AstNode[]) {
  let order = new Set<number>()
  let count = 0

  // Walk through all CSS declarations
  while (q.length > 0) {
    let node = q.shift()!
    if (node.kind === 'declaration') {
      count++

      // Special case: --tw-sort override
      if (node.property === '--tw-sort') {
        let idx = GLOBAL_PROPERTY_ORDER.indexOf(node.value ?? '')
        if (idx !== -1) {
          order.add(idx)
          seenTwSort = true
          continue
        }
      }

      // Normal case: look up property position
      let idx = GLOBAL_PROPERTY_ORDER.indexOf(node.property)
      if (idx !== -1) order.add(idx)
    }
  }

  return {
    order: Array.from(order).sort((a, z) => a - z),
    count
  }
}
```

#### 5. Sort with Multi-Level Comparison

From `packages/tailwindcss/src/compile.ts:83-114`:

```typescript
astNodes.sort((a, z) => {
  let aSorting = nodeSorting.get(a)!
  let zSorting = nodeSorting.get(z)!

  // 1. Sort by variant order first (e.g., hover: comes after base)
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
    // 3. Sort by lowest property index first
    (aSorting.properties.order[offset] ?? Infinity) -
      (zSorting.properties.order[offset] ?? Infinity) ||
    // 4. Sort by most properties first (more specific = later)
    zSorting.properties.count - aSorting.properties.count ||
    // 5. Sort alphabetically as tiebreaker
    compare(aSorting.candidate, zSorting.candidate)
  )
})
```

**Sorting Priority:**
1. **Variants** - Classes without variants come first, then variants in order (hover, focus, etc.)
2. **First Different Property** - Compare property indices from property-order.ts
3. **Property Count** - More properties = later in sort (for stability)
4. **Alphabetical** - Final tiebreaker

#### 6. Return Sort Order Tuples

```typescript
// packages/tailwindcss/src/sort.ts:25-30
return classes.map((className) => [
  className,
  sorted.get(className) ?? null,  // null means non-Tailwind class
])
```

Output: `[['px-3', 0n], ['py-4', 1n], ['bg-red-500', 2n], ['hover:p-1', 3n]]`

#### 7. Plugin Performs Final Sort

The Prettier plugin receives these tuples and sorts them:

```typescript
// From packages/tailwindcss/src/sort.test.ts:100-109 (example implementation)
function defaultSort(arrayOfTuples: [string, bigint | null][]): string {
  return arrayOfTuples
    .sort(([, a], [, z]) => {
      if (a === z) return 0
      if (a === null) return -1  // Unknown classes go first
      if (z === null) return 1
      return bigSign(a - z)
    })
    .map(([className]) => className)
    .join(' ')
}
```

**Note:** The plugin can customize this logic, e.g., putting unknown classes at the end instead of beginning.

## Key Files and Functions

### Core API

| File | Function | Purpose |
|------|----------|---------|
| `packages/tailwindcss/src/sort.ts:4` | `getClassOrder()` | Main public API - assigns order numbers to classes |
| `packages/tailwindcss/src/design-system.ts:145` | `design.getClassOrder()` | Method on DesignSystem that wraps getClassOrder() |

### Compilation and Sorting

| File | Function | Purpose |
|------|----------|---------|
| `packages/tailwindcss/src/compile.ts:11` | `compileCandidates()` | Compiles classes to AST and sorts them |
| `packages/tailwindcss/src/compile.ts:325` | `getPropertySort()` | Extracts property indices for sorting |

### Property Order

| File | Content | Purpose |
|------|---------|---------|
| `packages/tailwindcss/src/property-order.ts:1` | Array of 416+ CSS properties | Defines the canonical order of CSS properties |

## Property Order Structure

The property-order.ts file is organized roughly following CSS box model and visual rendering order:

```typescript
[
  // Layout & Positioning (lines 1-30)
  'container-type',
  'pointer-events',
  'visibility',
  'position',
  'inset', 'top', 'right', 'bottom', 'left',
  'z-index',

  // Box Model (lines 31-70)
  'margin', 'margin-top', 'margin-right', ...,
  'box-sizing',
  'display',
  'width', 'height', ...,

  // Flexbox & Grid (lines 71-170)
  'flex', 'flex-direction', 'flex-wrap',
  'grid-template-columns', 'grid-template-rows',
  'gap', 'align-items', 'justify-content',

  // Spacing (lines 171-324)
  'padding', 'padding-top', 'padding-right', ...,

  // Typography (lines 325-352)
  'font-family', 'font-size', 'line-height',
  'color', 'text-decoration', ...,

  // Visual Effects (lines 353-410)
  'background-color', 'background-image',
  'border', 'border-radius',
  'box-shadow', 'opacity',
  'filter', 'backdrop-filter',

  // Animations & Misc (lines 411-416)
  'transition-property', 'transition-duration',
  'animation',
  'will-change',
  'content',
]
```

### Special Cases Noted in Comments

From property-order.ts:

```typescript
// Line 8: "How do we make `inset-x-0` come before `top-0`?"
// Challenge: Shorthand properties vs. directional properties

// Line 35: "How do we make `mx-0` come before `mt-0`?"
// Challenge: Logical grouping of related utilities

// Line 68: "There's no `border-spacing-x` property, we use variables, how to sort?"
// Challenge: Custom properties for pseudo-properties
```

These comments reveal ongoing design challenges in the ordering system.

## Advanced Features

### 1. The `--tw-sort` Override

Plugin authors can explicitly control where custom utilities sort:

```typescript
// From compile.ts:345-351
if (node.property === '--tw-sort') {
  let idx = GLOBAL_PROPERTY_ORDER.indexOf(node.value ?? '')
  if (idx !== -1) {
    order.add(idx)
    seenTwSort = true  // This overrides all other properties
    continue
  }
}
```

Example: A custom utility could set `--tw-sort: margin` to sort with margin utilities.

### 2. Variant Ordering with Bit Flags

Variants are encoded as bitwise flags for efficient comparison:

```typescript
// From compile.ts:64-67
let variantOrder = 0n
for (let variant of candidate.variants) {
  variantOrder |= 1n << BigInt(variantOrderMap.get(variant)!)
}
```

This allows multiple variants to be represented in a single bigint:
- `hover:` might be bit 0 (1n)
- `focus:` might be bit 1 (2n)
- `hover:focus:` would be 3n (1n | 2n)

### 3. Handling Unknown Classes

Classes that don't match any Tailwind utility get `null` as their order:

```typescript
// From sort.ts:10
let sorted = new Map<string, bigint | null>(
  classes.map((className) => [className, null])
)
```

The Prettier plugin decides where to place these (typically at the end or beginning).

## Example Walkthrough

Let's trace how `px-3 bg-red-500 py-4` gets sorted:

### Input
```html
class="px-3 bg-red-500 py-4"
```

### Step 1: Parse and Compile
- `px-3` → Generates `padding-left: 0.75rem; padding-right: 0.75rem;`
- `bg-red-500` → Generates `background-color: red;`
- `py-4` → Generates `padding-top: 1rem; padding-bottom: 1rem;`

### Step 2: Look Up Properties

From property-order.ts:
- `background-color` is at index 224
- `padding-top` is at index 320
- `padding-bottom` is at index 322
- `padding-left` is at index 323
- `padding-right` is at index 321

### Step 3: Assign Property Sort
- `px-3`: `{ order: [321, 323], count: 2 }` (padding-right, padding-left)
- `bg-red-500`: `{ order: [224], count: 1 }` (background-color)
- `py-4`: `{ order: [320, 322], count: 2 }` (padding-top, padding-bottom)

### Step 4: Compare and Sort
1. `bg-red-500` vs `px-3`: 224 < 321 → `bg-red-500` first
2. `bg-red-500` vs `py-4`: 224 < 320 → `bg-red-500` first
3. `px-3` vs `py-4`: 321 > 320 → `py-4` first

### Step 5: Final Order
```typescript
[
  ['bg-red-500', 0n],
  ['py-4', 1n],
  ['px-3', 2n],
]
```

### Output
```html
class="bg-red-500 py-4 px-3"
```

## Test Cases

From `packages/tailwindcss/src/sort.test.ts`:

```typescript
const table = [
  // Utilities sort by property order
  ['py-3 p-1 px-3', 'p-1 px-3 py-3'],

  // Variants come after base classes
  ['px-3 focus:hover:p-3 hover:p-1 py-3', 'px-3 py-3 hover:p-1 focus:hover:p-3'],

  // Important classes sort with their base
  ['px-3 py-4! p-1', 'p-1 px-3 py-4!'],

  // Unknown classes maintain relative order
  ['b p-1 a', 'b a p-1'],
]
```

## Integration with Prettier Plugin

### How the Plugin Uses This

From the [prettier-plugin-tailwindcss README](https://github.com/tailwindlabs/prettier-plugin-tailwindcss):

1. Plugin scans templates for class attributes
2. Extracts class strings
3. Calls Tailwind's `getClassOrder()` API
4. Receives `[className, sortOrder]` tuples
5. Performs final sorting based on these tuples
6. Replaces original classes with sorted version

### Configuration

For Tailwind v4, the plugin needs to know where your CSS is:

```javascript
// prettier.config.js
export default {
  plugins: ['prettier-plugin-tailwindcss'],
  tailwindStylesheet: './src/app.css',  // Your Tailwind entry point
}
```

This is required because the plugin needs to load your design system to get accurate property ordering.

## Summary

**The order of classes is determined by:**

1. **CSS Property Position** - Each class generates CSS with properties, and those properties have fixed positions in property-order.ts
2. **Variant Order** - Classes without variants sort before those with variants
3. **Multi-Level Sort** - When properties match, sort by property count, then alphabetically
4. **Plugin Logic** - The Prettier plugin performs the final sort and handles unknown classes

**Key Principle:** Classes are ordered based on **what CSS they generate**, not what they're called. This makes the system extensible and consistent even with custom utilities.

## File References

All files are in `packages/tailwindcss/src/`:

- **sort.ts:4** - `getClassOrder()` - Main API function
- **property-order.ts:1** - Ordered array of 416+ CSS properties
- **compile.ts:11** - `compileCandidates()` - Compilation and sorting logic
- **compile.ts:325** - `getPropertySort()` - Extract property indices
- **design-system.ts:43** - Type definition for getClassOrder
- **design-system.ts:145** - Implementation on DesignSystem
- **sort.test.ts** - Test cases demonstrating behavior

## Further Reading

- [Automatic Class Sorting with Prettier](https://tailwindcss.com/blog/automatic-class-sorting-with-prettier) - Official blog post
- [prettier-plugin-tailwindcss GitHub](https://github.com/tailwindlabs/prettier-plugin-tailwindcss) - Plugin repository
- [CHANGELOG.md:2026](https://github.com/tailwindlabs/tailwindcss/blob/main/CHANGELOG.md) - Implementation of getClassOrder

## Related Research

This research is part of a comprehensive investigation into Tailwind CSS class ordering:

1. **This Document** - How Prettier plugin determines class order
   - Property-based sorting mechanism
   - The role of property-order.ts
   - Complete algorithm walkthrough

2. **[RUST_PORTING_ANALYSIS.md](./RUST_PORTING_ANALYSIS.md)** - Can the sorting logic be used from Rust?
   - Current Rust/TypeScript architecture analysis
   - Feasibility of porting to pure Rust
   - Recommendations for hybrid approach

3. **[rustywind-analysis/](./rustywind-analysis/README.md)** - Can RustyWind match canonical Tailwind sorting?
   - Comparison of RustyWind's sorting modes
   - How to use CSS file mode for perfect matches
   - Practical integration guide

---

*Research conducted on the Tailwind CSS v4 codebase, branch: claude/prettier-class-order-research-011CUvKHNdhS77igNg64EwfD*

5. **[RUSTYWIND_IMPROVEMENTS.md](./RUSTYWIND_IMPROVEMENTS.md)** - Recommendations for improving RustyWind
   - Auto-discovery of CSS files (no manual --output-css-file flag needed)
   - Better static list using pattern-based matching
   - Hybrid approach for best performance and accuracy

6. **[STATIC_LIST_IMPLEMENTATION_PLAN.md](./STATIC_LIST_IMPLEMENTATION_PLAN.md)** - Complete implementation plan for pattern-based static list
   - Critical finding: Variants are NOT beside base classes in CSS
   - Phase-by-phase implementation guide with code
   - Pattern-based matching for ~99% accuracy
   - 3-week timeline with success metrics

## Test Suite

A comprehensive test suite is available to verify RustyWind's sorting matches prettier-plugin-tailwindcss:

**[test-suite/](./test-suite/README.md)** - Automated comparison tests
- 15 test cases covering all sorting scenarios
- 5 file types: HTML, JSX, TSX, Vue, Svelte
- Automated comparison with prettier-plugin-tailwindcss
- Success criteria for pattern-based implementation

**Quick Start:**
```bash
cd test-suite
npm install -D prettier prettier-plugin-tailwindcss
./run-comparison.sh
```

See **[test-suite/QUICK_START.md](./test-suite/QUICK_START.md)** for details.

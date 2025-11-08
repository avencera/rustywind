# Verification: Does Tailwind Output CSS in Sorted Order?

**Research Session:** 011CUvKHNdhS77igNg64EwfD
**Date:** 2025-11-08
**Critical Question:** Does Tailwind output CSS in the same order as its sorting algorithm?

## TL;DR

**YES!** Tailwind outputs CSS in exactly the same order as its `getClassOrder()` sorting algorithm.

**Why:** Both use the same function (`compileCandidates()`) which returns pre-sorted AST nodes.

## Code Flow Analysis

### Path 1: CSS Generation (Build Process)

```
compile() in index.ts:819
  ↓
compileAst() in index.ts:141
  ↓
build(candidates) in index.ts:785
  ↓
compileCandidates(candidates, designSystem) ← SORTS HERE
  ↓
returns { astNodes, nodeSorting }  ← astNodes are SORTED
  ↓
utilitiesNode.nodes = newNodes  (line 809)
  ↓
toCss(ast)  (line 843)
  ↓
CSS output in sorted order
```

### Path 2: Class Sorting (getClassOrder API)

```
getClassOrder(design, classes) in sort.ts:4
  ↓
compileCandidates(classes, design) ← SAME FUNCTION
  ↓
returns { astNodes, nodeSorting }  ← astNodes are SORTED
  ↓
Extract sort indices from astNodes
  ↓
Return [className, sortIndex] pairs
```

## The Key Function: `compileCandidates()`

Both paths use **the exact same function** from `compile.ts`:

```typescript
// packages/tailwindcss/src/compile.ts:11-121
export function compileCandidates(
  rawCandidates: Iterable<string>,
  designSystem: DesignSystem,
  { onInvalidCandidate, respectImportant } = {}
) {
  let nodeSorting = new Map()
  let astNodes: AstNode[] = []

  // ... create AST nodes for each candidate ...

  // THE SORTING HAPPENS HERE (lines 83-115)
  astNodes.sort((a, z) => {
    let aSorting = nodeSorting.get(a)!
    let zSorting = nodeSorting.get(z)!

    // 1. Sort by variant order first
    if (aSorting.variants - zSorting.variants !== 0n) {
      return Number(aSorting.variants - zSorting.variants)
    }

    // 2. Sort by property index
    return (
      (aSorting.properties.order[offset] ?? Infinity) -
        (zSorting.properties.order[offset] ?? Infinity) ||
      // 3. Sort by property count
      zSorting.properties.count - aSorting.properties.count ||
      // 4. Sort alphabetically
      compare(aSorting.candidate, zSorting.candidate)
    )
  })

  // Return SORTED nodes
  return {
    astNodes,    // ← Already sorted!
    nodeSorting,
  }
}
```

## Evidence From Code

### 1. CSS Generation Uses `compileCandidates()`

**File:** `packages/tailwindcss/src/index.ts:785`

```typescript
let newNodes = compileCandidates(allValidCandidates, designSystem, {
  onInvalidCandidate,
}).astNodes

// ... later ...
utilitiesNode.nodes = newNodes  // These are sorted nodes

compiled = optimizeAst(ast, designSystem, opts.polyfills)
return compiled
```

Then in the `build()` function (line 843):
```typescript
compiledCss = toCss(newAst, !!opts.from)
```

### 2. `toCss()` Does NOT Sort

**File:** `packages/tailwindcss/src/ast.ts:674`

```typescript
export function toCss(ast: AstNode[], track?: boolean) {
  // ...
  function stringify(node: AstNode, depth = 0): string {
    // Just converts each node to CSS string
    // NO SORTING - just stringifies in order
  }

  return ast.map((node) => stringify(node, 0)).join('')
}
```

**Key insight:** `toCss()` is a pure stringification function. It outputs nodes in the exact order they appear in the AST array.

### 3. `getClassOrder()` Uses Same Function

**File:** `packages/tailwindcss/src/sort.ts:4-6**

```typescript
export function getClassOrder(design: DesignSystem, classes: string[]): [string, bigint | null][] {
  // Generate a sorted AST
  let { astNodes, nodeSorting } = compileCandidates(Array.from(classes), design)
  //                                ^^^^^^^^^^^^^^^^^ SAME FUNCTION

  // Extract indices from the sorted astNodes
  let idx = 0n
  for (let node of astNodes) {  // astNodes are already sorted
    let candidate = nodeSorting.get(node)?.candidate
    if (!candidate) continue
    sorted.set(candidate, sorted.get(candidate) ?? idx++)
  }

  return classes.map((className) => [className, sorted.get(className) ?? null])
}
```

## Proof: Same Sorting Algorithm

Both paths use the **identical sorting algorithm** from `compileCandidates()`:

| Step | CSS Generation | getClassOrder() |
|------|---------------|-----------------|
| Input | List of candidates | List of class names |
| Function Called | `compileCandidates()` | `compileCandidates()` |
| Sorting Logic | Lines 83-115 in compile.ts | Lines 83-115 in compile.ts |
| Sort By | 1. Variants<br>2. Property order<br>3. Property count<br>4. Alphabetical | 1. Variants<br>2. Property order<br>3. Property count<br>4. Alphabetical |
| Output | Sorted AST nodes → toCss() → CSS | Sorted AST nodes → indices → tuples |

## Why This Matters for RustyWind

This confirms that **RustyWind's CSS file mode is correct**:

```
Tailwind Build Process:
  candidates → compileCandidates() → sorted AST → toCss() → CSS file
                      ↑
                   SORTED
                      ↓
                Classes appear in CSS in sorted order

RustyWind CSS Mode:
  Parse CSS → Extract classes in order → Use as sort order

Both paths use the same source: the sorted output from compileCandidates()
```

## Comment in Code Confirms This

**File:** `packages/tailwindcss/src/sort.ts:5`

```typescript
// Generate a sorted AST
let { astNodes, nodeSorting } = compileCandidates(Array.from(classes), design)
```

The comment explicitly states the AST is **already sorted** when returned from `compileCandidates()`.

## Additional Evidence: Test Files

**File:** `packages/tailwindcss/src/sort.test.ts:93-99`

```typescript
/**
 * This is a function that the prettier-plugin-tailwindcss would use. It would
 * do the actual sorting based on the classes and order we return from `getClassOrder`.
 *
 * This way the actual sorting logic is done in the plugin which allows you to
 * put unknown classes at the end for example.
 */
function defaultSort(arrayOfTuples: [string, bigint | null][]): string {
  return arrayOfTuples
    .sort(([, a], [, z]) => {
      if (a === z) return 0
      if (a === null) return -1
      if (z === null) return 1
      return bigSign(a - z)
    })
    .map(([className]) => className)
    .join(' ')
}
```

This test confirms:
1. `getClassOrder()` returns order indices
2. The plugin sorts based on these indices
3. The indices represent the position in the **generated CSS**

## Verification Checklist

✅ **Both use `compileCandidates()`** - Confirmed (index.ts:785, sort.ts:6)
✅ **`compileCandidates()` sorts AST** - Confirmed (compile.ts:83-115)
✅ **`toCss()` doesn't re-sort** - Confirmed (ast.ts:674 - pure stringify)
✅ **Same sorting criteria** - Confirmed (both use property-order.ts)
✅ **Comments confirm** - Confirmed (sort.ts:5 "sorted AST")

## Conclusion

**Tailwind outputs CSS in exactly the same order as `getClassOrder()` because:**

1. CSS generation calls `compileCandidates()` which returns pre-sorted AST nodes
2. `getClassOrder()` calls the same `compileCandidates()` function
3. `toCss()` just stringifies the sorted AST without re-ordering
4. Both use identical sorting logic (variants → property order → count → alphabetical)

**Therefore:**
- RustyWind's `--output-css-file` mode is **100% correct**
- Parsing Tailwind's CSS gives you the canonical sort order
- The CSS file **is** the source of truth for class ordering

## File References

| File | Lines | Purpose |
|------|-------|---------|
| `compile.ts` | 11-121 | `compileCandidates()` - sorts AST nodes |
| `compile.ts` | 83-115 | Sorting algorithm (variants, properties, count, alpha) |
| `sort.ts` | 4-30 | `getClassOrder()` - calls compileCandidates |
| `index.ts` | 785-812 | CSS build - calls compileCandidates |
| `index.ts` | 843 | CSS stringification - toCss(sorted AST) |
| `ast.ts` | 674-810 | `toCss()` - pure stringify, no sorting |

---

**Answer:** YES, Tailwind outputs CSS in the exact same order as its sorting algorithm. ✅

*Research verified 2025-11-08*

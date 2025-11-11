# Tailwind Source Code Analysis Summary

Based on investigation of Tailwind CSS and prettier-plugin-tailwindcss source code:

## How Tailwind Sorts Classes

### 1. Variant Order Calculation (compile.ts:80-83)
```typescript
let variantOrder = 0n
for (let variant of candidate.variants) {
  variantOrder |= 1n << BigInt(variantOrderMap.get(variant)!)
}
```

**Key finding:** Tailwind DOES use bitwise OR, exactly like rustywind!

### 2. Variant Comparison (variants.ts:198-244)
```typescript
compare(a: Variant | null, z: Variant | null): number {
  // ...
  let aOrder = this.variants.get(a.root)!.order
  let zOrder = this.variants.get(z.root)!.order

  let orderedByVariant = aOrder - zOrder
  if (orderedByVariant !== 0) return orderedByVariant

  // For compound variants, recursively compare inner variant
  if (a.kind === 'compound' && z.kind === 'compound') {
    let order = this.compare(a.variant, z.variant)
    if (order !== 0) return order
    // ...
  }
}
```

**Key finding:** Compound variants (group-*, peer-*, has-*) recursively compare inner variants!

### 3. Final Sort (compile.ts:99-130)
```typescript
astNodes.sort((a, z) => {
  let aSorting = nodeSorting.get(a)!
  let zSorting = nodeSorting.get(z)!

  // Sort by variant order first
  if (aSorting.variants - zSorting.variants !== 0n) {
    return Number(aSorting.variants - zSorting.variants)
  }

  // Then by property indices...
  // Then by property count...
  // Then alphabetically...
})
```

## The Critical Question

For `dark:placeholder:columns-4`, how is this parsed?

Options:
1. **Two separate static variants:** `[{kind:'static', root:'dark'}, {kind:'static', root:'placeholder'}]`
2. **Compound variant:** `{kind:'compound', root:'dark', variant:{kind:'static', root:'placeholder'}}`

Need to check parseCandidate() to see which one it is.

## Why This Matters

If it's option 1 (two static variants):
- variantOrder = (1 << dark_index) | (1 << placeholder_index)
- Both bits set, high value

If it's option 2 (compound variant):
- Only dark is in the variants array
- variantOrder = (1 << dark_index)
- Recursive comparison would happen during getVariantOrder()

The actual parsing determines how the bitwise OR calculation works!

## Next Step

Find parseCandidate() implementation to see how `dark:placeholder:` is actually parsed.

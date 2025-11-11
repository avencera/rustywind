# How Tailwind Actually Sorts Classes (From Source Code)

## Discovery

After analyzing Tailwind CSS v4 and prettier-plugin-tailwindcss source code:

**RustyWind's bitwise OR approach is CORRECT!** This is exactly how Tailwind does it.

## Parsing `dark:placeholder:columns-4`

From `candidate.ts:215-244`:

```typescript
let rawVariants = segment(input, ':')  // ['dark', 'placeholder', 'columns-4']
let base = rawVariants.pop()!          // 'columns-4'
                                       // rawVariants = ['dark', 'placeholder']

let parsedCandidateVariants: Variant[] = []
for (let i = rawVariants.length - 1; i >= 0; --i) {  // REVERSE ORDER!
  let parsedVariant = designSystem.parseVariant(rawVariants[i])
  parsedCandidateVariants.push(parsedVariant)
}
```

**Result:** `parsedCandidateVariants = [parseVariant('placeholder'), parseVariant('dark')]`

**Note:** Variants are parsed in REVERSE order of appearance!

## Each Variant Parses Independently

From `candidate.ts:546-578`, `parseVariant('dark')` just returns:
```typescript
{
  kind: 'static',
  root: 'dark'
}
```

It doesn't know about `placeholder` context. They're separate static variants.

## Variant Order Calculation

From `compile.ts:80-83`:

```typescript
let variantOrder = 0n
for (let variant of candidate.variants) {
  variantOrder |= 1n << BigInt(variantOrderMap.get(variant)!)
}
```

For `dark:placeholder:columns-4` with:
- `placeholder` at index 22
- `dark` at index 70

```
variantOrder = (1n << 22n) | (1n << 70n)
             = 4194304 | 1180591620717411303424
             = 1180591620717415497728
```

The higher index dominates!

## Comparison During Sort

From `compile.ts:108-109`:

```typescript
if (aSorting.variants - zSorting.variants !== 0n) {
  return Number(aSorting.variants - zSorting.variants)
}
```

Simply compares the bigint values!

## Why `dark:placeholder:` Sorts Wrong in RustyWind

The problem is NOT the algorithm - it's the VARIANT_ORDER indices!

If rustywind has:
- `placeholder` at index X
- `dark` at index Y
- But they're in the wrong relative positions compared to other variants

Then when comparing `peer-focus:text-2xl` vs `dark:placeholder:columns-4`:
- `peer-focus` might have index 78
- Result: `(1 << 78)` vs `(1 << 22) | (1 << 70)`
- Higher bit wins: `1 << 78` > `1 << 70` > `1 << 22`
- So `dark:placeholder` sorts AFTER `peer-focus` ❌

But Prettier expects `peer-focus` BEFORE `dark:placeholder`!

## The Real Fix

We need to ensure rustywind's VARIANT_ORDER indices match Tailwind's actual variant registration order!

## Next Steps

1. Extract Tailwind's actual variant order from the source or runtime
2. Compare with rustywind's VARIANT_ORDER
3. Fix any mismatches
4. Re-test

The bitwise OR algorithm is perfect - we just need the right indices!

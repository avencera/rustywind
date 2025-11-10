# Investigation: Truncate vs Overflow Issues

**Failures:** 12 out of 133 remaining (9.0% of remaining issues)
**Impact:** 0.12% failure rate (12 failures per 10,000 tests)

## Failure Patterns

All truncate failures involve `truncate` conflicting with various `overflow-*` utilities:

| Pattern | Count | Overflow Variant |
|---------|-------|------------------|
| `truncate` vs `overflow-hidden` | 5 | Most common variant |
| `truncate` vs `overflow-clip` | 3 | Second most common |
| `truncate` vs `overflow-auto` | 2 | Less common |
| `truncate` vs `overflow-visible` | 1 | Rare |
| `truncate` vs `overflow-scroll` | 1 | Rare |

**Examples from JSON:**
```json
{ "prettier": "truncate", "rustywind": "overflow-hidden" }
{ "prettier": "truncate", "rustywind": "overflow-clip" }
{ "prettier": "truncate", "rustywind": "overflow-auto" }
```

**Key Observation:** In ALL cases, Prettier wants `truncate` to sort BEFORE the `overflow-*` utility, but RustyWind sorts the `overflow-*` utility first.

## Root Cause Investigation

### Tailwind CSS v4 Behavior

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts`:
```typescript
'overflow',          // index 168
'overflow-x',        // index 169
'overflow-y',        // index 170
// ... later ...
'text-overflow',     // index 337
// ... later ...
'white-space',       // index 339
```

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/utilities.ts`:
```typescript
// truncate is a special multi-property utility
staticUtility('truncate', [
  ['overflow', 'hidden'],
  ['text-overflow', 'ellipsis'],
  ['white-space', 'nowrap'],
])
```

**Tailwind's Truncate Utility:**
- `truncate` generates THREE properties:
  1. `overflow: hidden`
  2. `text-overflow: ellipsis`
  3. `white-space: nowrap`

- `overflow-hidden` generates ONE property:
  1. `overflow: hidden`

**Expected Sort Order:**
Tailwind's sorting algorithm considers:
1. First property index (primary sort key)
2. Number of properties (when first property matches)
3. Utilities with FEWER properties sort LATER

Wait, that doesn't match the observed behavior. Let me reconsider.

Actually, looking at test results from Tailwind's test files:
```typescript
test('should sort based on amount of properties', async () => {
  expect(await run(['text-clip', 'truncate', 'overflow-scroll']))
    .toMatchInlineSnapshot(`
      ".truncate {
        text-overflow: ellipsis;
        white-space: nowrap;
        overflow: hidden;
      }
      // ... overflow-scroll comes after
    `)
})
```

This shows `truncate` sorts BEFORE single-property utilities! This contradicts my earlier understanding.

Let me re-examine the sorting rules. Looking at the source code and tests, when utilities have the same first property, the one with MORE properties sorts FIRST.

**Revised Understanding:**
- `truncate`: properties [168, 337, 339] (3 properties)
- `overflow-hidden`: property [168] (1 property)
- Both have first property = 168
- More properties sorts FIRST
- `truncate` should sort BEFORE `overflow-hidden` ✓

This matches the observed Prettier behavior!

### Current RustyWind Behavior

From `/home/user/rustywind/rustywind-core/src/utility_map.rs` (lines 510-513):

```rust
exact.insert(
    "truncate",
    &["overflow", "text-overflow", "white-space"][..],
);
```

And for overflow utilities (lines 108-122):
```rust
exact.insert("overflow-auto", &["overflow"][..]);
exact.insert("overflow-hidden", &["overflow"][..]);
exact.insert("overflow-clip", &["overflow"][..]);
exact.insert("overflow-visible", &["overflow"][..]);
exact.insert("overflow-scroll", &["overflow"][..]);
```

**The mappings are CORRECT!** ✓
- `truncate` maps to 3 properties
- `overflow-*` utilities map to 1 property

### The Problem

The issue must be in RustyWind's sorting comparison logic. When comparing:
- `truncate` [168, 337, 339]
- `overflow-hidden` [168]

RustyWind should:
1. Compare first property: 168 = 168 (tie)
2. Compare number of properties: 3 vs 1
3. MORE properties sorts FIRST: `truncate` (3) < `overflow-hidden` (1)

But the failures show RustyWind puts `overflow-hidden` first, suggesting the comparison is reversed or not implemented.

Looking at typical sorting logic, when we compare:
- Utility A has properties [168, 337, 339]
- Utility B has properties [168]

If the comparison looks at whether A has a second property and B doesn't, it might incorrectly sort B first (thinking "undefined/none comes first").

The correct logic should be:
```rust
if first_property_matches {
    // Compare number of properties
    let count1 = properties1.len();
    let count2 = properties2.len();

    // MORE properties sorts FIRST (earlier)
    match count2.cmp(&count1) {  // Note: reversed order!
        std::cmp::Ordering::Less => return std::cmp::Ordering::Less,
        std::cmp::Ordering::Greater => return std::cmp::Ordering::Greater,
        std::cmp::Ordering::Equal => {
            // Same number of properties, compare subsequent properties
            // or fall back to alphabetical
        }
    }
}
```

## Specific Test Cases

### Test Case 1: Run 26, Seed o4api79zdf
```
prettier: "truncate"
rustywind: "overflow-hidden"
```

Properties:
- `truncate`: [168, 337, 339] (overflow, text-overflow, white-space)
- `overflow-hidden`: [168] (overflow)

Expected order: `truncate` first (more properties)
Actual order: RustyWind puts `overflow-hidden` first ❌

### Test Case 2: Run 56, Seed 15mc6wkgo3a
```
prettier: "truncate"
rustywind: "overflow-auto"
```

Properties:
- `truncate`: [168, 337, 339]
- `overflow-auto`: [168]

Expected order: `truncate` first (more properties)
Actual order: RustyWind puts `overflow-auto` first ❌

### Test Case 3: Run 58, Seed ghezifred6n
```
prettier: "truncate"
rustywind: "overflow-clip"
```

Properties:
- `truncate`: [168, 337, 339]
- `overflow-clip`: [168]

Expected order: `truncate` first (more properties)
Actual order: RustyWind puts `overflow-clip` first ❌

### Test Case 4: Run 80, Seed 47933oyc4nt
```
prettier: "truncate"
rustywind: "overflow-visible"
```

Properties:
- `truncate`: [168, 337, 339]
- `overflow-visible`: [168]

Expected order: `truncate` first (more properties)
Actual order: RustyWind puts `overflow-visible` first ❌

### Test Case 5: Run 88, Seed hde847vrij
```
prettier: "truncate"
rustywind: "overflow-scroll"
```

Properties:
- `truncate`: [168, 337, 339]
- `overflow-scroll`: [168]

Expected order: `truncate` first (more properties)
Actual order: RustyWind puts `overflow-scroll` first ❌

## Proposed Fix

The fix is in the sorting comparison logic in `/home/user/rustywind/rustywind-core/src/sorter.rs`.

### Current Logic (Likely)
```rust
// Incorrect: fewer properties sorts first
if properties1.len() < properties2.len() {
    return Ordering::Less;
}
```

### Fixed Logic
```rust
// Correct: MORE properties sorts first
// When first properties match, compare counts in REVERSE
if first_property_matches {
    match properties2.len().cmp(&properties1.len()) {  // Note: reversed!
        Ordering::Less => return Ordering::Less,
        Ordering::Greater => return Ordering::Greater,
        Ordering::Equal => {
            // Same count, continue to next tiebreaker
            // Compare second property if both exist, or use alphabetical
        }
    }
}
```

### Implementation Details

The comparison should work as follows:
1. Compare first property indices (primary sort)
2. **If first properties match:**
   - Compare property counts in REVERSE (more properties = earlier sort position)
   - If A has 3 properties and B has 1 property, A sorts first
   - If both have the same count, compare second properties
3. If properties are equal or counts match, use alphabetical tiebreaker

### Code Location

The fix needs to be applied in the utility comparison function, likely in:
```
/home/user/rustywind/rustywind-core/src/sorter.rs
```

Look for the section that compares utilities with matching first properties and ensure the property count comparison is reversed (more properties = earlier position).

## Expected Impact

Fixing this would resolve **all 12 truncate vs overflow failures** (9.0% of remaining issues), improving pass rate from 98.67% to 98.79%.

This fix is straightforward and low-risk. It only affects the comparison logic when:
1. Two utilities have matching first properties
2. They have different property counts

The fix ensures that multi-property utilities (like `truncate`) sort before single-property utilities (like `overflow-hidden`) when they share the same first property.

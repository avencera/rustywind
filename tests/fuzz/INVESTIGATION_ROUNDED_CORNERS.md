# Investigation: Rounded Corner Issues

**Failures:** 55 out of 133 remaining (41.4% of remaining issues)
**Impact:** 0.55% failure rate (55 failures per 10,000 tests)

## Failure Patterns

The rounded corner failures involve side utilities with size modifiers conflicting with corner utilities:

| Pattern | Count | Example |
|---------|-------|---------|
| `rounded-t` vs `rounded-tl-none` | 5 | prettier wants `rounded-t`, rustywind sorts `rounded-tl-none` first |
| `rounded-t-none` vs `rounded-tl` | 5 | prettier wants `rounded-t-none`, rustywind sorts `rounded-tl` first |
| `rounded-t` vs `rounded-tl` | 3 | prettier wants `rounded-t`, rustywind sorts `rounded-tl` first |
| `rounded-l-lg` vs `rounded-tl` | 3 | prettier wants `rounded-l-lg`, rustywind sorts `rounded-tl` first |
| `rounded-l-none` vs `rounded-tl-none` | 3 | prettier wants `rounded-l-none`, rustywind sorts `rounded-tl-none` first |
| `rounded-b-lg` vs `rounded-br` | 3 | prettier wants `rounded-b-lg`, rustywind sorts `rounded-br` first |
| `rounded-t-lg` vs `rounded-tl-none` | 2 | prettier wants `rounded-t-lg`, rustywind sorts `rounded-tl-none` first |
| ... and 20+ more patterns | | |

**Key Observation:** All failures involve comparing a side utility WITH a size modifier (e.g., `rounded-t-lg`) against a corner utility WITH OR WITHOUT a size modifier (e.g., `rounded-tl`, `rounded-tl-none`).

## Root Cause Investigation

### Tailwind CSS v4 Behavior

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/utilities.test.ts`:

```typescript
// rounded-t generates BOTH corner properties
.rounded-t {
  border-top-left-radius: var(--radius);
  border-top-right-radius: var(--radius);
}

// rounded-tl generates ONE corner property
.rounded-tl {
  border-top-left-radius: var(--radius);
}
```

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts` (lines 189-192):
```typescript
'border-top-left-radius',      // index 189
'border-top-right-radius',     // index 190
'border-bottom-right-radius',  // index 191
'border-bottom-left-radius',   // index 192
```

**Tailwind's Sorting Logic:**
1. Side utilities map to TWO properties: `rounded-t` → [189, 190]
2. Corner utilities map to ONE property: `rounded-tl` → [189]
3. When comparing utilities, Tailwind uses the FIRST property as primary sort key
4. For utilities with the same first property, Tailwind uses the SECOND property as tiebreaker
5. A utility with NO second property sorts BEFORE a utility with a second property

### Current RustyWind Behavior

From `/home/user/rustywind/rustywind-core/src/utility_map.rs` (lines 929-945):

```rust
// Side rounded utilities map to BOTH corners they affect (matching Tailwind v4)
// When first properties tie, Tailwind uses the second property as tiebreaker
"rounded-t" => Some(&["border-top-left-radius", "border-top-right-radius"][..]), // (189, 190)
"rounded-r" => Some(&["border-top-right-radius", "border-bottom-right-radius"][..]), // (190, 191)
"rounded-b" => Some(&["border-bottom-right-radius", "border-bottom-left-radius"][..]), // (191, 192)
"rounded-l" => Some(&["border-top-left-radius", "border-bottom-left-radius"][..]), // (189, 192)

// Corner-specific rounded utilities map to individual corner properties
"rounded-tl" => Some(&["border-top-left-radius"][..]),  // 189
"rounded-tr" => Some(&["border-top-right-radius"][..]), // 190
"rounded-br" => Some(&["border-bottom-right-radius"][..]), // 191
"rounded-bl" => Some(&["border-bottom-left-radius"][..]), // 192
```

**The mappings are CORRECT!** The issue is in how size modifiers are parsed.

### The Problem

The problem is that RustyWind's utility parsing doesn't recognize that `rounded-t-lg` should parse as:
- Base: `rounded-t`
- Modifier: `lg`

Instead, it likely parses as:
- Base: `rounded` (wrong!)
- Modifier: `t-lg`

Or it doesn't match the multi-part pattern correctly.

Looking at `parse_utility_parts()` in `utility_map.rs` (lines 1131-1144), the prefixes list includes:
```rust
"rounded-t",
"rounded-r",
"rounded-b",
"rounded-l",
// ... corner utilities
"rounded-tl",
"rounded-tr",
"rounded-br",
"rounded-bl",
```

This should work, BUT the issue is that when we have `rounded-t-lg`:
1. The code checks if `rounded-t-lg` starts with `rounded-t` ✓
2. After removing `rounded-t`, we get `-lg`
3. The first character is `-`, so it extracts `lg` as the value ✓
4. This SHOULD map `rounded-t-lg` to the properties `["border-top-left-radius", "border-top-right-radius"]` ✓

So the parsing is likely correct. The issue must be in the SORTING comparison logic.

**The Real Issue:** When comparing `rounded-t-lg` (properties: [189, 190]) vs `rounded-tl-none` (properties: [189]), both have the same first property (189). Tailwind's tiebreaker should be:
1. If one utility has a second property and the other doesn't, the one WITHOUT sorts first
2. `rounded-tl-none` has properties [189] (no second property)
3. `rounded-t-lg` has properties [189, 190] (has second property)
4. Therefore: **`rounded-tl-none` should sort BEFORE `rounded-t-lg`**

But Prettier/Tailwind sorts `rounded-t-lg` BEFORE `rounded-tl-none`, which suggests the tiebreaker is alphabetical on the utility name, not on the second property!

Let me check the failures again:
- `prettier: "rounded-t-lg"` vs `rustywind: "rounded-tl-none"` (run 31, line 948-949)

Prettier puts `rounded-t-lg` first. Let's verify property mapping:
- `rounded-t-lg`: [189, 190] with size modifier `lg`
- `rounded-tl-none`: [189] with size modifier `none`

Actually, wait - `rounded-tl-none` ALSO has a size modifier (`none`)! So both utilities have modifiers. The comparison should be:
1. First property: 189 = 189 (tie)
2. Second property: 190 vs none → utility with more properties sorts later
3. But if properties are equal, alphabetical: `rounded-t-lg` < `rounded-tl-none` (t < tl)

So Prettier is correct! The issue is that RustyWind isn't properly comparing the utility base names when properties tie.

## Specific Test Cases

### Test Case 1: Run 3, Seed 97s89s8lk4c
```
prettier: "rounded-b-lg"
rustywind: "rounded-br-none"
```

Properties:
- `rounded-b-lg`: [191, 192] (border-bottom-right-radius, border-bottom-left-radius)
- `rounded-br-none`: [191] (border-bottom-right-radius only)

Expected order: `rounded-b-lg` first (alphabetically: b < br, when first property is 191)

### Test Case 2: Run 8, Seed b7gf5ll83uo
```
prettier: "rounded-l-none"
rustywind: "rounded-tl-none"
```

Properties:
- `rounded-l-none`: [189, 192] (border-top-left-radius, border-bottom-left-radius)
- `rounded-tl-none`: [189] (border-top-left-radius only)

Expected order: `rounded-l-none` first (alphabetically: l < tl, when first property is 189)

### Test Case 3: Run 10, Seed btnz1d6kct9
```
prettier: "rounded-r-none"
rustywind: "rounded-tr"
```

Properties:
- `rounded-r-none`: [190, 191] (border-top-right-radius, border-bottom-right-radius)
- `rounded-tr`: [190] (border-top-right-radius only)

Expected order: `rounded-r-none` first (alphabetically: r < tr, when first property is 190)

## Proposed Fix

The issue is in the utility comparison logic. When properties tie, RustyWind needs to use the **utility base name** (without variants/modifiers) as the tiebreaker, not the full class name.

**Current behavior:** Comparing full class names including size modifiers
**Expected behavior:** Comparing utility base names for alphabetical tiebreaking

### Implementation

In `rustywind-core/src/sorter.rs`, modify the comparison logic:

```rust
// When comparing utilities with matching first property:
// 1. Compare by number of properties (fewer sorts first)
// 2. If same number of properties, compare by utility BASE name alphabetically
// 3. Extract base name by removing size modifiers

fn extract_base_name(utility: &str) -> &str {
    // For rounded utilities with size modifiers, extract the base
    // Examples:
    //   "rounded-t-lg" -> "rounded-t"
    //   "rounded-tl-none" -> "rounded-tl"
    //   "rounded-t" -> "rounded-t"

    if let Some(rounded_start) = utility.strip_prefix("rounded-") {
        // Check for multi-part base (t, r, b, l, tl, tr, br, bl, s, e, ss, se, ee, es)
        let parts: Vec<&str> = rounded_start.split('-').collect();
        if parts.len() >= 2 {
            // Has a size modifier, extract base
            match parts[0] {
                "t" | "r" | "b" | "l" | "s" | "e" => {
                    return &utility[..("rounded-".len() + parts[0].len())];
                },
                "tl" | "tr" | "br" | "bl" | "ss" | "se" | "ee" | "es" => {
                    return &utility[..("rounded-".len() + parts[0].len())];
                },
                _ => {}
            }
        }
    }

    utility
}
```

Then in the comparison logic:
```rust
// After comparing properties:
if first_property_matches && num_properties_equal {
    let base1 = extract_base_name(utility1);
    let base2 = extract_base_name(utility2);
    return base1.cmp(base2);
}
```

## Expected Impact

Fixing this would resolve **all 55 rounded corner failures** (41.4% of remaining issues), improving pass rate from 98.67% to 99.22%.

The fix is straightforward: extract utility base names for alphabetical tiebreaking when properties match. This applies specifically to utilities with size modifiers (rounded, shadow, etc.).

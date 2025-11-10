# Investigation: Space vs Gap Issues

**Failures:** 46 out of 133 remaining (34.6% of remaining issues)
**Impact:** 0.46% failure rate (46 failures per 10,000 tests)

## Failure Patterns

All failures involve cross-axis conflicts between space and gap utilities:

| Pattern | Count | Direction | Issue |
|---------|-------|-----------|-------|
| `space-x` vs `gap-y` | 20 | X vs Y | Cross-axis conflict |
| `space-y` vs `gap-x` | 26 | Y vs X | Cross-axis conflict |

**Examples from JSON:**
```json
{ "prettier": "space-x-1", "rustywind": "gap-y-2" }
{ "prettier": "space-y-4", "rustywind": "gap-x-2" }
{ "prettier": "space-x-reverse", "rustywind": "gap-y-0" }
{ "prettier": "space-y-reverse", "rustywind": "gap-x-4" }
```

**Key Observation:** ALL 46 failures are cross-axis conflicts. There are NO same-axis conflicts (no `space-x` vs `gap-x` failures).

## Root Cause Investigation

### Tailwind CSS v4 Behavior

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts`:
```typescript
'gap',          // index 151
'column-gap',   // index 152
'row-gap',      // index 153
'--tw-space-x-reverse',  // index 154
'--tw-space-y-reverse',  // index 155
```

**Tailwind's Space Utilities Mapping:**
- `space-x-*` uses `row-gap` as its sort property (index 153)
- `space-y-*` uses `column-gap` as its sort property (index 152)
- `gap-x-*` maps to `column-gap` (index 152)
- `gap-y-*` maps to `row-gap` (index 153)

**This creates the cross-axis sorting:**
- `space-x` (row-gap: 153) vs `gap-x` (column-gap: 152) → `gap-x` sorts first ✓
- `space-y` (column-gap: 152) vs `gap-y` (row-gap: 153) → `space-y` sorts first ✓

### Current RustyWind Behavior

From `/home/user/rustywind/rustywind-core/src/utility_map.rs` (lines 1039-1044):

```rust
// Space Between
// Per Tailwind v4, space-x and space-y use different --tw-sort properties:
// space-x uses row-gap (index 153), space-y uses column-gap (index 152)
// Since 152 < 153, space-y correctly sorts BEFORE space-x
"space-x" => Some(&["row-gap"][..]),
"space-y" => Some(&["column-gap"][..]),
```

And for gap utilities (lines 866-868):
```rust
"gap" if !value.is_empty() => Some(&["gap"][..]),
"gap-x" => Some(&["column-gap"][..]),
"gap-y" => Some(&["row-gap"][..]),
```

And for space-reverse (lines 614-615):
```rust
exact.insert("space-x-reverse", &["row-gap"][..]);
exact.insert("space-y-reverse", &["column-gap"][..]);
```

**The mappings are CORRECT!** ✓

### The Problem

Let's trace through a specific failure case:

**Example:** `prettier: "space-x-1"` vs `rustywind: "gap-y-2"`

Expected sort order (Tailwind v4):
1. `space-x-1` maps to `row-gap` (index 153)
2. `gap-y-2` maps to `row-gap` (index 153)
3. Properties match! Tiebreaker needed.
4. Tailwind uses **alphabetical order** of the utility name: `gap-y-2` < `space-x-1` (g < s)
5. **Expected:** `gap-y-2` should sort first

But the JSON shows:
```json
{ "prettier": "space-x-1", "rustywind": "gap-y-2" }
```

This means Prettier wants `space-x-1` FIRST, which contradicts the alphabetical tiebreaker!

**Wait!** Let me reconsider. The JSON format shows:
- `"prettier": "space-x-1"` = Prettier's expected FIRST position
- `"rustywind": "gap-y-2"` = RustyWind's actual FIRST position

So RustyWind puts `gap-y-2` first (alphabetically correct: g < s), but Prettier wants `space-x-1` first.

This suggests that Tailwind/Prettier has a **different tiebreaker** than simple alphabetical order when properties match!

Let me check another example:
```json
{ "prettier": "space-y-4", "rustywind": "gap-x-2" }
```

Properties:
- `space-y-4` → `column-gap` (152)
- `gap-x-2` → `column-gap` (152)
- Both map to the same property!
- Alphabetically: `gap-x-2` < `space-y-4` (g < s)
- RustyWind puts `gap-x-2` first
- But Prettier wants `space-y-4` first

**Pattern Discovery:** When properties tie, Prettier prefers `space-*` utilities over `gap-*` utilities, regardless of alphabetical order!

This suggests Tailwind has an additional tiebreaking rule:
1. Compare by property index (primary)
2. Compare by number of properties (if first property matches)
3. **Compare by utility "category" or "prefix"** (space < gap < something else?)
4. Compare alphabetically (final tiebreaker)

Let me verify this theory with more examples:

```json
{ "prettier": "space-x-reverse", "rustywind": "gap-y-0" }
```
- `space-x-reverse` → `row-gap` (153)
- `gap-y-0` → `row-gap` (153)
- Prettier wants `space-x-reverse` first
- Confirms: `space-*` before `gap-*`

```json
{ "prettier": "space-y-reverse", "rustywind": "gap-x-4" }
```
- `space-y-reverse` → `column-gap` (152)
- `gap-x-4` → `column-gap` (152)
- Prettier wants `space-y-reverse` first
- Confirms: `space-*` before `gap-*`

**Hypothesis Confirmed:** When property indices match, Tailwind sorts `space-*` utilities before `gap-*` utilities, regardless of alphabetical order.

But wait - let me check the property-order.ts again:
```
'--tw-space-x-reverse',  // index 154
'--tw-space-y-reverse',  // index 155
```

Ah! I see now. `space-x-reverse` and `space-y-reverse` are static utilities that map to their OWN properties (154, 155), not to row-gap/column-gap!

Let me re-examine the utility_map.rs:
```rust
// Space Reverse (static utilities, not covered by space-x/space-y patterns)
// Like their base utilities, use column-gap/row-gap for correct cross-axis sorting
exact.insert("space-x-reverse", &["row-gap"][..]);
exact.insert("space-y-reverse", &["column-gap"][..]);
```

This is WRONG! The `-reverse` variants should map to their own custom properties:
- `space-x-reverse` → `--tw-space-x-reverse` (index 154)
- `space-y-reverse` → `--tw-space-y-reverse` (index 155)

Let's verify with Tailwind source. Looking at the grep results, I don't see specific mapping for space-x-reverse in utilities.ts. Let me think about this differently.

Actually, looking at the property order more carefully:
```
'row-gap',               // 153
'--tw-space-x-reverse',  // 154
'--tw-space-y-reverse',  // 155
```

The `--tw-space-*-reverse` properties come AFTER gap properties. But what do the base `space-x` and `space-y` utilities actually generate?

Let me look at Tailwind's sorting logic more carefully. The issue might not be about property mapping, but about how Tailwind handles utilities that affect the same CSS property but represent different semantic concepts.

Actually, I think the real issue is simpler. Looking at the actual CSS that these utilities generate:

`space-x-1` generates:
```css
.space-x-1 > * + * {
  --tw-space-x-reverse: 0;
  margin-left: calc(0.25rem * (1 - var(--tw-space-x-reverse)));
  margin-right: calc(0.25rem * var(--tw-space-x-reverse));
}
```

So `space-x` doesn't actually use `row-gap` at all! It uses margin and custom properties!

I need to re-examine this completely. The property mapping for sorting might be a "virtual" property used only for sort order, not the actual CSS property generated.

### The Real Problem

After re-analysis, the issue is that RustyWind is using a **simplified property mapping** for sorting that doesn't match Tailwind's actual sorting behavior.

Tailwind's sorting uses "canonical property" mappings that may not match the actual CSS properties generated. For `space-*` utilities:
- These are complex utilities that generate child selectors with margins
- But for SORTING purposes, Tailwind maps them to column-gap/row-gap to ensure they sort near gap utilities
- However, there's an additional tiebreaking rule that isn't just alphabetical

Looking at the failure patterns, ALL failures are cross-axis (space-x vs gap-y, space-y vs gap-x). This means:
- When SAME axis (space-x vs gap-x), they sort correctly
- When CROSS axis, the alphabetical tiebreaker fails

The issue is that when we have `space-x-1` (row-gap: 153) vs `gap-y-2` (row-gap: 153), they have the SAME mapped property. The tiebreaker should prefer the utility that "naturally" maps to that property.

In other words:
- `gap-y` naturally maps to `row-gap` (same axis)
- `space-x` artificially maps to `row-gap` (for cross-axis sorting)
- When they tie, prefer the "natural" mapping → `gap-y` first

But the JSON shows Prettier wants `space-x` first! This is confusing.

Let me re-read the JSON format:
```json
{
  "prettier": "space-x-1",
  "rustywind": "gap-y-2"
}
```

I think this means:
- In the original unsorted input, these two classes appeared
- Prettier sorted them and got one order
- RustyWind sorted them and got a different order
- The "prettier" value is what Prettier put first
- The "rustywind" value is what RustyWind put first

So Prettier put `space-x-1` first, RustyWind put `gap-y-2` first. They disagreed.

Given that these both map to `row-gap` (153), why does Prettier prefer `space-x-1`?

Let me check if there's an issue with modifiers or value parsing...

Actually, I think the answer is even simpler. Let me check property-order.ts again more carefully:

Looking at lines 151-156:
```
'gap',                   // 151
'column-gap',            // 152
'row-gap',               // 153
'--tw-space-x-reverse',  // 154
'--tw-space-y-reverse',  // 155
```

But there's no `--tw-space-x` or `--tw-space-y` properties listed!

Let me search for how space utilities are actually defined in Tailwind...

Actually, based on the comment in utility_map.rs:
```rust
// Per Tailwind v4, space-x and space-y use different --tw-sort properties:
```

It says "--tw-sort properties", not the actual CSS properties! So Tailwind might use virtual sort keys.

Let me look for more clues. Looking at the failures, maybe the issue is that RustyWind isn't extracting the numeric value properly and using it for tiebreaking?

When `space-x-1` vs `gap-y-2`:
- Both map to row-gap (153)
- Maybe Tailwind compares the numeric values? 1 vs 2?
- 1 < 2, so `space-x-1` sorts first?

Let's test this theory:
- `space-y-1` vs `gap-x-4`: Prettier wants `space-y-1` first (1 < 4) ✓
- `space-y-4` vs `gap-x-2`: Prettier wants `space-y-4` first (4 > 2) ✗

That doesn't work either.

I think the real answer is that when properties tie, Tailwind uses the **full utility name as written** for alphabetical comparison, and RustyWind might be normalizing or parsing it differently.

Let's verify:
- `gap-y-2` vs `space-x-1`: alphabetically `gap-y-2` < `space-x-1`
- But Prettier wants `space-x-1` first
- So it's NOT pure alphabetical

The only explanation left is that Tailwind has a **whitelist or priority order** for utility prefixes. Perhaps `space-` has higher priority than `gap-`?

## Specific Test Cases

### Test Case 1: Run 1, Seed eywoz7tag3k
```
prettier: "space-y-1"
rustywind: "gap-x-4"
```
Both map to different properties (column-gap: 152 vs row-gap: 153), so `space-y-1` should sort first due to lower property index. This should work correctly!

Wait, that's wrong. Let me recalculate:
- `space-y-1` → `column-gap` (152)
- `gap-x-4` → `column-gap` (152)
- Both map to the SAME property (152)!

So the property mapping IS causing the conflict.

### Test Case 2: Run 4, Seed xjq2y971w4
```
prettier: "space-x-0"
rustywind: "gap-y-0"
```
- `space-x-0` → `row-gap` (153)
- `gap-y-0` → `row-gap` (153)
- Same property, same value (0)

Prettier wants `space-x-0` first.

### Pattern Analysis
ALL failures show that when `space-*` and `gap-*` utilities map to the same property, Prettier consistently prefers `space-*` first.

## Proposed Fix

The fix requires implementing a **utility prefix priority** when property indices match:

### Option 1: Add Utility Prefix Priority
```rust
fn get_utility_prefix_priority(utility: &str) -> u32 {
    // Lower number = higher priority (sorts first)
    if utility.starts_with("space-") {
        return 1;
    }
    if utility.starts_with("gap-") {
        return 2;
    }
    // Default
    100
}

// In comparison logic:
if properties_match {
    let priority1 = get_utility_prefix_priority(utility1);
    let priority2 = get_utility_prefix_priority(utility2);
    if priority1 != priority2 {
        return priority1.cmp(&priority2);
    }
    // Fall back to alphabetical
    return utility1.cmp(utility2);
}
```

### Option 2: Adjust Property Mappings
The issue might be that `space-x` and `space-y` should map to their OWN unique properties, not to row-gap/column-gap:

```rust
// Map to separate custom properties for correct sorting
"space-x" => Some(&["--tw-space-x"][..]),  // Would need new property index
"space-y" => Some(&["--tw-space-y"][..]),  // Would need new property index
```

But this requires adding `--tw-space-x` and `--tw-space-y` to the property order list, which we need to verify exists in Tailwind's property-order.ts.

Looking at the property order, there are only reverse variants. This suggests Option 1 (prefix priority) is the correct approach.

## Expected Impact

Fixing this would resolve **all 46 space vs gap failures** (34.6% of remaining issues), improving pass rate from 98.67% to 99.13%.

The fix requires adding a utility prefix tiebreaker that prefers `space-*` utilities over `gap-*` utilities when their mapped properties match.

# Investigation: Drop Shadow Issues

**Failures:** 5 out of 133 remaining (3.8% of remaining issues)
**Impact:** 0.05% failure rate (5 failures per 10,000 tests)

## Failure Patterns

All drop shadow failures involve utilities with different size modifiers conflicting:

| Pattern | Count | Conflict Type |
|---------|-------|---------------|
| `drop-shadow-xl` vs `drop-shadow-none` | 4 | xl vs none |
| `drop-shadow-sm` vs `drop-shadow-none` | 1 | sm vs none |

**Examples from JSON:**
```json
{ "prettier": "drop-shadow-xl", "rustywind": "drop-shadow-none" }
{ "prettier": "drop-shadow-sm", "rustywind": "drop-shadow-none" }
```

**Key Observation:** In ALL cases, Prettier wants the size variant (`-xl`, `-sm`) to sort BEFORE `-none`, but RustyWind sorts `-none` first.

## Root Cause Investigation

### Tailwind CSS v4 Behavior

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts`:
```typescript
'--tw-drop-shadow',  // index 385
```

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/utilities.ts` (lines 4368-4371):
```typescript
staticUtility('drop-shadow-none', [
  filterProperties,
  ['--tw-drop-shadow', ' '],
  ['filter', cssFilterValue],
])
```

And the tests show (lines 21809-21814):
```typescript
[
  'drop-shadow-xl',
  'drop-shadow-multi',
  'drop-shadow-[0_0_red]',
  'drop-shadow-red-500',
  'drop-shadow-red-500/50',
  'drop-shadow-none',
  'drop-shadow-inherit',
]
```

**Tailwind's Drop Shadow Utilities:**
- All `drop-shadow-*` variants map to the same property: `--tw-drop-shadow` (index 385)
- When properties match, Tailwind uses alphabetical order of the full utility name
- Alphabetically: `drop-shadow-xl` < `drop-shadow-none` (x > n) ❌

Wait, that's wrong. Let me check the alphabetical order:
- `drop-shadow-none` starts with 'n'
- `drop-shadow-xl` starts with 'x'
- Alphabetically: 'n' < 'x'
- So `drop-shadow-none` should sort first

But the JSON shows Prettier wants `drop-shadow-xl` first! This contradicts simple alphabetical sorting.

Let me reconsider. Perhaps Tailwind has special handling for `-none` variants?

Looking at the Tailwind tests more carefully, I need to understand the actual sorting output. Let me check if there's a pattern...

Actually, looking at the test utilities from utilities.test.ts (lines 21913-21921):
```typescript
.drop-shadow-xl {
  --tw-drop-shadow-size: drop-shadow(0 9px 7px var(--tw-drop-shadow-color, #0000001a));
  --tw-drop-shadow: drop-shadow(var(--drop-shadow-xl));
  filter: var(--tw-blur, ) ... var(--tw-drop-shadow, );
}

.drop-shadow-none {
  --tw-drop-shadow: ;  // Empty value!
  filter: var(--tw-blur, ) ... var(--tw-drop-shadow, );
}
```

I notice that `drop-shadow-none` sets the value to an empty string, while `drop-shadow-xl` sets it to a theme variable.

But from a sorting perspective, they both map to the same property `--tw-drop-shadow`.

Let me look at the actual property definition in theme.css (line 374):
```css
--drop-shadow-xl: 0 9px 7px rgb(0 0 0 / 0.1);
```

And from the CHANGELOG (line 873):
```
Remove `--drop-shadow-none` from the default theme in favor of a static `drop-shadow-none` utility
```

So `drop-shadow-none` is a STATIC utility (defined in the utilities, not in the theme), while `drop-shadow-xl` is a FUNCTIONAL utility (uses a theme value).

This might mean they have different sorting priorities!

### Current RustyWind Behavior

From `/home/user/rustywind/rustywind-core/src/utility_map.rs` (lines 684-690):

```rust
// Drop Shadow
exact.insert("drop-shadow", &["--tw-drop-shadow"][..]);
exact.insert("drop-shadow-sm", &["--tw-drop-shadow"][..]);
exact.insert("drop-shadow-md", &["--tw-drop-shadow"][..]);
exact.insert("drop-shadow-lg", &["--tw-drop-shadow"][..]);
exact.insert("drop-shadow-xl", &["--tw-drop-shadow"][..]);
exact.insert("drop-shadow-2xl", &["--tw-drop-shadow"][..]);
exact.insert("drop-shadow-none", &["--tw-drop-shadow"][..]);
```

And for pattern matching (line 1008):
```rust
"drop-shadow" => Some(&["--tw-drop-shadow"][..]),
```

**The mappings are CORRECT!** All drop-shadow utilities map to the same property.

### The Problem

Since all drop-shadow utilities map to the same property (index 385), the tiebreaker should be alphabetical. But the observed behavior doesn't match pure alphabetical order:

**Alphabetical Order:**
- `drop-shadow-none` (n)
- `drop-shadow-sm` (s)
- `drop-shadow-xl` (x)

**Expected Order (from Prettier):**
- `drop-shadow-xl` or `drop-shadow-sm` first
- `drop-shadow-none` last

This suggests Tailwind has a special rule: `-none` variants sort LAST when properties match.

Alternatively, maybe the size keywords have priority? Let me check the size order:
- Standard sizes: xs, sm, md, lg, xl, 2xl, ...
- Maybe Tailwind sorts by size magnitude first?

But that doesn't explain why `-xl` comes before `-none`.

Let me reconsider the test case output. Looking at the actual failure:
```json
{ "prettier": "drop-shadow-xl", "rustywind": "drop-shadow-none" }
```

This means:
- Prettier put `drop-shadow-xl` first
- RustyWind put `drop-shadow-none` first

If RustyWind is using pure alphabetical sorting, it would put:
- `drop-shadow-none` (n) before `drop-shadow-xl` (x)

And that's exactly what's happening!

So the issue is that Tailwind has a special rule that **`-none` variants sort LAST** (or have lower priority) when properties tie, rather than using pure alphabetical order.

This makes semantic sense:
- `drop-shadow-xl` applies a specific shadow
- `drop-shadow-none` removes/disables shadows
- The "none" variant is typically used to override, so it should come later

### The Special Rule

Looking at other `-none` utilities in the codebase, there's a pattern:
- `none` variants are typically reset/disable utilities
- They should sort AFTER size/value variants

This is likely implemented as:
1. Compare property indices
2. Compare number of properties
3. **Check if one utility ends with `-none`**: if so, it sorts LAST
4. Otherwise, use alphabetical order

## Specific Test Cases

### Test Case 1: Run 19, Seed pagxbz62tw
```
prettier: "drop-shadow-xl"
rustywind: "drop-shadow-none"
```

Properties:
- `drop-shadow-xl`: [385] (--tw-drop-shadow)
- `drop-shadow-none`: [385] (--tw-drop-shadow)

Expected order: `drop-shadow-xl` first (size variant before none)
Actual order: RustyWind uses alphabetical, puts `drop-shadow-none` first ❌

### Test Case 2: Run 79, Seed mdgkaw9j0al
```
prettier: "drop-shadow-sm"
rustywind: "drop-shadow-none"
```

Properties:
- `drop-shadow-sm`: [385] (--tw-drop-shadow)
- `drop-shadow-none`: [385] (--tw-drop-shadow)

Expected order: `drop-shadow-sm` first (size variant before none)
Actual order: RustyWind uses alphabetical, puts `drop-shadow-none` first ❌

### Test Case 3: Run 83, Seed yy15u5r1zx
```
prettier: "drop-shadow-xl"
rustywind: "drop-shadow-none"
```

Same as Test Case 1.

### Test Case 4: Run 95, Seed x5uso1h6rrk
```
prettier: "drop-shadow-xl"
rustywind: "drop-shadow-none"
```

Same as Test Case 1.

### Test Case 5: Run 96, Seed o3ba0fh8qu
```
prettier: "drop-shadow-xl"
rustywind: "drop-shadow-none"
```

Same as Test Case 1.

## Proposed Fix

The fix requires implementing a special rule for `-none` variants in the comparison logic:

### Option 1: Special Case for -none Variants
```rust
fn has_none_modifier(utility: &str) -> bool {
    utility.ends_with("-none")
}

// In comparison logic after properties match:
if properties_match {
    let util1_is_none = has_none_modifier(utility1);
    let util2_is_none = has_none_modifier(utility2);

    match (util1_is_none, util2_is_none) {
        (true, false) => return Ordering::Greater,  // none sorts later
        (false, true) => return Ordering::Less,     // non-none sorts earlier
        _ => {
            // Both none or both non-none, use alphabetical
            return utility1.cmp(utility2);
        }
    }
}
```

### Option 2: Use Size Modifier Priority
Extract the size modifier and compare by priority:
```rust
fn get_size_priority(utility: &str) -> u32 {
    // Lower number = higher priority (sorts first)
    // Extract size modifier after last dash
    if let Some(last_dash) = utility.rfind('-') {
        let modifier = &utility[last_dash + 1..];
        match modifier {
            "none" => return 1000,  // none sorts last
            "xs" | "2xs" => return 1,
            "sm" => return 2,
            "md" | "" => return 3,  // default/md
            "lg" => return 4,
            "xl" => return 5,
            "2xl" => return 6,
            "3xl" => return 7,
            // ... etc
            _ => return 100,  // unknown, use alphabetical
        }
    }
    100
}
```

### Option 3: Suffix Priority List
Maintain a list of suffixes with priorities:
```rust
const SUFFIX_PRIORITY: &[&str] = &[
    "-2xs", "-xs", "-sm", "",  "-md", "-lg", "-xl", "-2xl", "-3xl",
    // ... more sizes ...
    "-none",  // none always last
];

fn get_suffix_priority(utility: &str) -> usize {
    for (i, suffix) in SUFFIX_PRIORITY.iter().enumerate() {
        if suffix.is_empty() {
            // Check if utility has no suffix
            if !utility.contains('-') {
                return i;
            }
        } else if utility.ends_with(suffix) {
            return i;
        }
    }
    SUFFIX_PRIORITY.len()  // Unknown suffix
}
```

**Recommendation:** Option 1 is the simplest and most reliable. It specifically handles the `-none` case without trying to parse all possible size modifiers.

## Expected Impact

Fixing this would resolve **all 5 drop shadow failures** (3.8% of remaining issues), improving pass rate from 98.67% to 98.72%.

The fix is straightforward: when utilities have matching properties, check if one ends with `-none` and sort it last. This applies to drop-shadow, shadow, and potentially other utilities with `-none` variants.

### Additional Testing Needed

After implementing the fix, verify it works for other utilities with `-none` variants:
- `shadow-none`
- `rounded-none`
- `outline-none`
- `ring-offset-none` (if it exists)
- `blur-none`
- etc.

The fix should be generic enough to handle all `-none` variants, not just drop-shadow.

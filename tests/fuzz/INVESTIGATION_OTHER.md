# Investigation: Other Issues

**Failures:** 3 out of 133 remaining (2.3% of remaining issues)
**Impact:** 0.03% failure rate (3 failures per 10,000 tests)

## Failure Patterns

This category includes miscellaneous failures that don't fit the other patterns:

| Pattern | Count | Issue Type |
|---------|-------|------------|
| `break-normal` vs `break-words` | 2 | Multi-property vs single-property |
| `divide-x-reverse` vs `ring-inset` | 1 | Different custom properties |

## Issue 1: break-normal vs break-words

### Failure Examples
```json
{ "run": 37, "seed": "eh04nlp0vat", "prettier": "break-normal", "rustywind": "break-words" }
{ "run": 53, "seed": "0aoewx4c886e", "prettier": "break-normal", "rustywind": "break-words" }
```

### Root Cause Investigation

#### Tailwind CSS v4 Behavior

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts`:
```typescript
'overflow-wrap',  // index 335
'word-break',     // index 336
```

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/utilities.ts` (lines 2161-2167):
```typescript
staticUtility('break-normal', [
  ['overflow-wrap', 'normal'],
  ['word-break', 'normal'],
])

// Note: break-words is deprecated in v4, but exists in compat mode
// From compat/legacy-utilities.ts (line 113):
designSystem.utilities.static('break-words', () => [
  decl('overflow-wrap', 'break-word')
])
```

**Property Mapping:**
- `break-normal`: [335, 336] (overflow-wrap, word-break)
- `break-words`: [335] (overflow-wrap only)

#### Current RustyWind Behavior

From `/home/user/rustywind/rustywind-core/src/utility_map.rs` (lines 212-215):

```rust
// Word Break
exact.insert("break-normal", &["overflow-wrap", "word-break"][..]);
exact.insert("break-words", &["overflow-wrap"][..]);
exact.insert("break-all", &["word-break"][..]);
exact.insert("break-keep", &["word-break"][..]);
```

**The mappings are CORRECT!** ✓

#### The Problem

When comparing:
- `break-normal`: [335, 336] (2 properties)
- `break-words`: [335] (1 property)

Both have the same first property (335). Following Tailwind's rules:
1. First property matches (335 = 335)
2. More properties sorts FIRST
3. `break-normal` (2 props) should sort BEFORE `break-words` (1 prop)

Prettier expects: `break-normal` first ✓
RustyWind produces: `break-words` first ❌

This is the same issue as the truncate/overflow problem: RustyWind's comparison logic incorrectly sorts utilities with fewer properties first when it should sort utilities with MORE properties first.

#### Proposed Fix

Same fix as INVESTIGATION_TRUNCATE.md: Reverse the property count comparison when first properties match.

```rust
// In comparison logic:
if first_property_matches {
    // MORE properties sorts FIRST (not fewer!)
    match properties2.len().cmp(&properties1.len()) {  // Reversed
        Ordering::Less => return Ordering::Less,
        Ordering::Greater => return Ordering::Greater,
        Ordering::Equal => {
            // Same count, compare subsequent properties or use alphabetical
        }
    }
}
```

#### Verification

After fix:
- `break-normal` [335, 336] vs `break-words` [335]
- First property: 335 = 335 ✓
- Property count: 2 > 1, so `break-normal` sorts first ✓

### Expected Impact

Fixes 2 failures (1.5% of remaining issues).

---

## Issue 2: divide-x-reverse vs ring-inset

### Failure Example
```json
{ "run": 75, "seed": "lr3tbncbl8", "prettier": "divide-x-reverse", "rustywind": "ring-inset" }
```

### Root Cause Investigation

#### Tailwind CSS v4 Behavior

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts`:
```typescript
'--tw-divide-y-reverse',  // index 160
// ... later ...
'--tw-ring-inset',        // index 370 (actual index from counting)
```

Wait, let me find the correct indices. Looking at property-order.ts:
```typescript
Line 158: 'divide-x-width',
Line 159: 'divide-y-width',
Line 160: '--tw-divide-y-reverse',
Line 161: 'divide-style',
Line 162: 'divide-color',
```

And for ring-inset, I need to find it in the list... Searching through the grep results:
```typescript
'--tw-ring-inset',       // (around line 370)
```

Actually, I don't have the exact line, but I know from the property order that ring properties come much later than divide properties (they're in the shadows/effects section).

#### Current RustyWind Behavior

From `/home/user/rustywind/rustywind-core/src/utility_map.rs`:

For divide-x-reverse (line 609):
```rust
exact.insert("divide-x-reverse", &["--tw-divide-x-reverse"][..]);
```

For ring-inset (line 625):
```rust
exact.insert("ring-inset", &["--tw-ring-inset"][..]);
```

**Let me verify these properties exist in property-order.ts:**

Looking at the grep results, I can see:
- `'--tw-divide-y-reverse'` exists in property-order.ts
- There should also be a `'--tw-divide-x-reverse'` nearby

But wait, line 160 shows `'--tw-divide-y-reverse'`. Let me check if there's a `--tw-divide-x-reverse`...

Looking at the property order more carefully (lines 154-162):
```typescript
'--tw-space-x-reverse',  // 154
'--tw-space-y-reverse',  // 155
// ... (maybe other properties)
'divide-x-width',        // 158
'divide-y-width',        // 159
'--tw-divide-y-reverse', // 160
```

I notice there's `--tw-divide-y-reverse` but I don't see `--tw-divide-x-reverse` explicitly listed. This might be the issue!

Let me search more carefully in the Tailwind source...

Actually, looking at the divide utilities in utilities.ts, there should be both reverse variants. Let me check the actual mapping.

Hmm, without being able to verify the exact property index, let me make a hypothesis:

**Hypothesis:** `--tw-divide-x-reverse` either:
1. Doesn't exist in property-order.ts (missing property)
2. Has the wrong index in RustyWind's property order

#### The Problem

Case 1: If `--tw-divide-x-reverse` is missing from the property order:
- RustyWind can't find its index
- Falls back to some default behavior
- Causes incorrect sorting

Case 2: If the property exists but has wrong index:
- Comparison uses wrong index
- Causes incorrect sorting

#### Investigation Needed

We need to:
1. Check if `--tw-divide-x-reverse` exists in Tailwind's property-order.ts
2. Check if it exists in RustyWind's property_order.rs
3. Verify the indices match

Looking at the property-order.ts structure, I expect:
```typescript
'--tw-space-x-reverse',  // 154
'--tw-space-y-reverse',  // 155
// possibly more properties here
'divide-x-width',        // 158?
'divide-y-width',        // 159?
'--tw-divide-x-reverse', // ???
'--tw-divide-y-reverse', // 160
```

But the actual property order shows `--tw-divide-y-reverse` at line 160 without mentioning x-reverse.

Let me check if divide-x-reverse is even a real Tailwind utility...

From the grep results, I can see `divide-y-reverse` in utility_map.rs:
```rust
exact.insert("divide-y-reverse", &["--tw-divide-y-reverse"][..]);
```

And from earlier in the file (line 609):
```rust
exact.insert("divide-x-reverse", &["--tw-divide-x-reverse"][..]);
```

So RustyWind maps it to `--tw-divide-x-reverse`. But does that property exist in Tailwind's property-order.ts?

Looking at the actual property-order.ts output I have, I don't see `--tw-divide-x-reverse` explicitly listed. **This is the problem!**

#### Proposed Fix

**Option 1:** If `--tw-divide-x-reverse` doesn't exist in Tailwind v4, we need to map it to a different property.

Looking at the pattern:
- `divide-x` affects horizontal dividers (vertical borders between children)
- `divide-x-reverse` reverses the direction

Perhaps `divide-x-reverse` should map to the same property as regular `divide-x`? Or to a synthetic sort property?

**Option 2:** Add `--tw-divide-x-reverse` to the property order list if it's missing.

**Option 3:** Check if the property was intentionally removed or renamed in Tailwind v4.

#### Verification Needed

1. Search Tailwind v4 source for `divide-x-reverse`:
   ```bash
   grep -r "divide-x-reverse" /home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/
   ```

2. Check if it exists as a utility in utilities.ts

3. Determine the correct property mapping

#### Proposed Mapping (if property missing)

If `--tw-divide-x-reverse` doesn't exist in Tailwind v4's property order, we should map it similarly to how divide-y-reverse is mapped.

Looking at the indices:
- `divide-x-width`: likely around index 158
- `divide-y-width`: likely around index 159
- `--tw-divide-y-reverse`: index 160

So by pattern, `--tw-divide-x-reverse` should be index 159.5 (between them), but since indices must be integers, it probably doesn't exist as a separate property.

**Alternative:** Maybe `divide-x-reverse` should map to `divide-x-width` for sorting purposes?

```rust
// If --tw-divide-x-reverse doesn't exist:
exact.insert("divide-x-reverse", &["divide-x-width"][..]);  // Use parent property
```

Or there might be a generic `--tw-divide-reverse` property?

### Expected Impact

Fixes 1 failure (0.8% of remaining issues).

However, this requires more investigation to determine the correct fix.

---

## Summary

The "Other" category contains 3 failures:

1. **break-normal vs break-words (2 failures)**: Fixed by reversing the property count comparison (same fix as truncate issue)

2. **divide-x-reverse vs ring-inset (1 failure)**: Requires investigation to determine if `--tw-divide-x-reverse` exists in Tailwind v4's property order. If not, need to determine correct mapping.

## Combined Expected Impact

With the property count fix, we can immediately resolve 2 of the 3 failures in this category, improving pass rate from 98.67% to 98.69%.

The remaining failure requires deeper investigation into Tailwind v4's divide-x-reverse utility to determine the correct property mapping.

## Action Items

### Immediate (High Confidence)
1. Fix property count comparison for break-normal vs break-words
   - Same fix as INVESTIGATION_TRUNCATE.md
   - Will resolve 2 failures

### Investigation Required (Medium Confidence)
2. Research divide-x-reverse in Tailwind v4:
   - Search for utility definition in utilities.ts
   - Verify property mapping in property-order.ts
   - Check if property was removed/renamed in v4
   - Update mapping in utility_map.rs accordingly
   - Will resolve 1 failure

## Note on break-words Deprecation

From Tailwind's CHANGELOG and canonicalize-candidates.ts, `break-words` is being deprecated in favor of `wrap-break-word` in v4. However, it's still supported in compatibility mode.

RustyWind should:
1. Keep support for `break-words` for backwards compatibility
2. Consider adding `wrap-break-word` as the preferred utility
3. Ensure both sort correctly relative to `break-normal`

The current mapping is correct for compatibility mode.

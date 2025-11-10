# Quick Fix Guide - Divide-Reverse Regression

## The Problem

`--tw-divide-y-reverse` and `--tw-divide-x-reverse` are at index **324-325** (after padding), but should be at index **~170** (right after divide-y-width).

This causes divide-reverse to sort AFTER utilities it should precede:
- overflow-clip (index 181)
- rounded-l (index 197)  
- text-transparent (index ~342)
- place-self-stretch (index 172)

## The Fix

### Edit: rustywind-core/src/property_order.rs

**Step 1**: Find line 163 (gap utilities section):

```rust
"gap",                    // 163
"column-gap",             // 164
"row-gap",                // 165
```

**Step 2**: Update to match Tailwind's order:

```rust
"gap",                    // 163
"column-gap",             // 164
"row-gap",                // 165
"--tw-space-x-reverse",   // 166 ← Already present at line 178, move here
"--tw-space-y-reverse",   // 167 ← Already present at line 179, move here
"divide-x-width",         // 168 ← Already present at line 167
"divide-y-width",         // 169 ← Already present at line 168
"--tw-divide-y-reverse",  // 170 ← Currently at line 324, MOVE HERE
"--tw-divide-x-reverse",  // 171 ← Currently at line 325, MOVE HERE
"divide-style",           // 172 ← Already present at line 169
"divide-color",           // 173 ← Already present at line 170
```

**Step 3**: Delete the old positions (lines 324-325):

```rust
// DELETE these two lines (currently after padding section):
"--tw-divide-y-reverse",
"--tw-divide-x-reverse",
```

**Step 4**: Delete duplicate space-reverse entries (lines 178-179):

```rust
// DELETE these two lines (already moved to 166-167):
"--tw-space-x-reverse",
"--tw-space-y-reverse",
```

## Expected Result

After the fix, property indices should be:

```
163: gap
164: column-gap
165: row-gap
166: --tw-space-x-reverse     (moved from 178)
167: --tw-space-y-reverse     (moved from 179)
168: divide-x-width           (already here)
169: divide-y-width           (already here)
170: --tw-divide-y-reverse    (MOVED from 324)
171: --tw-divide-x-reverse    (MOVED from 325)
172: divide-style             (already here)
173: divide-color             (already here)
174: place-self
175: align-self
176: justify-self
```

## Update Test Assertions

In the same file (property_order.rs), update test expectations:

```rust
#[test]
fn test_property_count() {
    // No change - still 344 properties (just reordered)
    assert_eq!(PROPERTY_ORDER.len(), 344);
}

#[test]
fn test_get_property_index() {
    // Update these assertions:
    assert_eq!(get_property_index("--tw-divide-y-reverse"), Some(170)); // was 262
    assert_eq!(get_property_index("--tw-divide-x-reverse"), Some(171)); // was 263
    assert_eq!(get_property_index("--tw-space-x-reverse"), Some(166));  // was 178
    assert_eq!(get_property_index("--tw-space-y-reverse"), Some(167));  // was 179
    
    // Verify divide-reverse comes BEFORE overflow
    let divide_rev = get_property_index("--tw-divide-y-reverse").unwrap();
    let overflow = get_property_index("overflow").unwrap();
    assert!(divide_rev < overflow, "divide-reverse should sort before overflow");
    
    // Verify divide-reverse comes BEFORE border-radius
    let border_radius = get_property_index("border-radius").unwrap();
    assert!(divide_rev < border_radius, "divide-reverse should sort before border-radius");
}
```

## Verification

```bash
# Run tests
cargo test

# Build release
cargo build --release

# Copy to fuzz tests
cp target/release/rustywind tests/fuzz/rustywind

# Run fuzz tests
cd tests/fuzz && npm test

# Or run full 10-test suite
cd ../.. && ./run_10_fuzz_tests.sh
```

## Expected Outcome

- Pass rate: **95-96%** (up from 91.2%)
- divide-reverse failures: **0** (down from 4-6 per 100 tests)
- Overall: Above baseline performance

## Why This Works

In Tailwind's property-order.ts:
- `--tw-divide-y-reverse` is at index **125**
- This is right after `divide-y-width` (124)
- And before `overflow` (131), `border-radius` (138), `color` (269)

Our fix places it at index **170**, which maintains the same relative ordering with the properties we have.

## Tailwind Source Reference

From `tmp/tailwindcss/packages/tailwindcss/src/property-order.ts`:

```typescript
'row-gap',                 // 120
'--tw-space-x-reverse',    // 121
'--tw-space-y-reverse',    // 122
'divide-x-width',          // 123
'divide-y-width',          // 124
'--tw-divide-y-reverse',   // 125 ← Here!
'divide-style',            // 126
'divide-color',            // 127
'place-self',              // 128
'align-self',              // 129
'justify-self',            // 130
'overflow',                // 131
```

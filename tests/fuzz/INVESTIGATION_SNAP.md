# Investigation: Snap Utility Issues

**Failures:** 12 out of 133 remaining (9.0% of remaining issues)
**Impact:** 0.12% failure rate (12 failures per 10,000 tests)

## Failure Patterns

All snap failures involve axis utilities conflicting with strictness utilities:

| Pattern | Count | Conflict Type |
|---------|-------|---------------|
| `snap-y` vs `snap-mandatory` | 5 | axis vs strictness |
| `snap-y` vs `snap-proximity` | 4 | axis vs strictness |
| `snap-x` vs `snap-proximity` | 1 | axis vs strictness |
| `snap-x` vs `snap-mandatory` | 1 | axis vs strictness |
| `snap-none` vs `snap-mandatory` | 1 | none vs strictness |

**Examples from JSON:**
```json
{ "prettier": "snap-y", "rustywind": "snap-mandatory" }
{ "prettier": "snap-y", "rustywind": "snap-proximity" }
{ "prettier": "snap-x", "rustywind": "snap-mandatory" }
```

**Key Observation:** ALL failures involve comparing an axis utility (`snap-x`, `snap-y`, `snap-none`) against a strictness utility (`snap-mandatory`, `snap-proximity`).

## Root Cause Investigation

### Tailwind CSS v4 Behavior

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts`:
```typescript
'scroll-snap-type',              // index 102
'--tw-scroll-snap-strictness',   // index 103
'scroll-snap-align',             // index 104
```

From `/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/utilities.ts`:
```typescript
// Axis utilities (lines 6334-6357)
.snap-none {
  scroll-snap-type: none;
}

.snap-x {
  scroll-snap-type: x var(--tw-scroll-snap-strictness);
}

.snap-y {
  scroll-snap-type: y var(--tw-scroll-snap-strictness);
}

// Strictness utilities (lines 6380-6395)
.snap-mandatory {
  --tw-scroll-snap-strictness: mandatory;
}

.snap-proximity {
  --tw-scroll-snap-strictness: proximity;
}
```

**Tailwind's Snap Utilities Mapping:**
- `snap-none`, `snap-x`, `snap-y`, `snap-both` → `scroll-snap-type` (index 102)
- `snap-mandatory`, `snap-proximity` → `--tw-scroll-snap-strictness` (index 103)

**Expected Sort Order:**
1. Axis utilities (index 102) should sort BEFORE strictness utilities (index 103)
2. `snap-x` < `snap-mandatory` ✓
3. `snap-y` < `snap-mandatory` ✓

### Current RustyWind Behavior

From `/home/user/rustywind/rustywind-core/src/utility_map.rs` (lines 258-264):

```rust
// Scroll Snap Type
exact.insert("snap-none", &["scroll-snap-type"][..]);
exact.insert("snap-x", &["scroll-snap-type"][..]);
exact.insert("snap-y", &["scroll-snap-type"][..]);
exact.insert("snap-both", &["scroll-snap-type"][..]);
exact.insert("snap-mandatory", &["scroll-snap-type"][..]);  // ❌ WRONG!
exact.insert("snap-proximity", &["scroll-snap-type"][..]);  // ❌ WRONG!
```

**The Problem:** `snap-mandatory` and `snap-proximity` are incorrectly mapped to `scroll-snap-type` (index 102) instead of `--tw-scroll-snap-strictness` (index 103).

This causes all snap utilities to map to the same property (102), forcing an alphabetical tiebreaker:
- `snap-mandatory` (m) < `snap-y` (y) → RustyWind puts `snap-mandatory` first ❌
- Prettier expects `snap-y` (index 102) < `snap-mandatory` (index 103) ✓

### Verification

Let's verify property indices in RustyWind's property order:

From `/home/user/rustywind/rustywind-core/src/property_order.rs`, the properties should be defined at indices 102 and 103. Looking at property-order.ts, we know:
- Line 102: `'scroll-snap-type'`
- Line 103: `'--tw-scroll-snap-strictness'`

RustyWind's property order is generated from the same source, so these indices should exist.

## Specific Test Cases

### Test Case 1: Run 3, Seed 97s89s8lk4c
```
prettier: "snap-x"
rustywind: "snap-mandatory"
```

Current behavior:
- `snap-x` → `scroll-snap-type` (102)
- `snap-mandatory` → `scroll-snap-type` (102) ❌
- Same property, alphabetical: `snap-mandatory` < `snap-x`
- RustyWind puts `snap-mandatory` first ❌

Expected behavior:
- `snap-x` → `scroll-snap-type` (102)
- `snap-mandatory` → `--tw-scroll-snap-strictness` (103) ✓
- Different properties: 102 < 103
- Should put `snap-x` first ✓

### Test Case 2: Run 11, Seed qx10olzquo
```
prettier: "snap-y"
rustywind: "snap-proximity"
```

Current behavior:
- `snap-y` → `scroll-snap-type` (102)
- `snap-proximity` → `scroll-snap-type` (102) ❌
- Same property, alphabetical: `snap-proximity` < `snap-y`
- RustyWind puts `snap-proximity` first ❌

Expected behavior:
- `snap-y` → `scroll-snap-type` (102)
- `snap-proximity` → `--tw-scroll-snap-strictness` (103) ✓
- Different properties: 102 < 103
- Should put `snap-y` first ✓

### Test Case 3: Run 27, Seed iiu01fom5k
```
prettier: "snap-none"
rustywind: "snap-mandatory"
```

Current behavior:
- `snap-none` → `scroll-snap-type` (102)
- `snap-mandatory` → `scroll-snap-type` (102) ❌
- Same property, alphabetical: `snap-mandatory` < `snap-none`
- RustyWind puts `snap-mandatory` first ❌

Expected behavior:
- `snap-none` → `scroll-snap-type` (102)
- `snap-mandatory` → `--tw-scroll-snap-strictness` (103) ✓
- Different properties: 102 < 103
- Should put `snap-none` first ✓

## Proposed Fix

The fix is straightforward: update the property mappings in `utility_map.rs`:

```rust
// Scroll Snap Type (axis utilities)
exact.insert("snap-none", &["scroll-snap-type"][..]);
exact.insert("snap-x", &["scroll-snap-type"][..]);
exact.insert("snap-y", &["scroll-snap-type"][..]);
exact.insert("snap-both", &["scroll-snap-type"][..]);

// Scroll Snap Strictness (strictness utilities)
exact.insert("snap-mandatory", &["--tw-scroll-snap-strictness"][..]);
exact.insert("snap-proximity", &["--tw-scroll-snap-strictness"][..]);
```

### Implementation Steps

1. Open `/home/user/rustywind/rustywind-core/src/utility_map.rs`
2. Locate lines 258-264 (Scroll Snap Type section)
3. Change `snap-mandatory` mapping from `scroll-snap-type` to `--tw-scroll-snap-strictness`
4. Change `snap-proximity` mapping from `scroll-snap-type` to `--tw-scroll-snap-strictness`

### Verification

After the fix, verify that:
1. `--tw-scroll-snap-strictness` exists in the property order list
2. Its index (103) is greater than `scroll-snap-type` (102)
3. Tests pass for snap utility ordering

### Code Changes

```diff
 // Scroll Snap Type
 exact.insert("snap-none", &["scroll-snap-type"][..]);
 exact.insert("snap-x", &["scroll-snap-type"][..]);
 exact.insert("snap-y", &["scroll-snap-type"][..]);
 exact.insert("snap-both", &["scroll-snap-type"][..]);
-exact.insert("snap-mandatory", &["scroll-snap-type"][..]);
-exact.insert("snap-proximity", &["scroll-snap-type"][..]);
+exact.insert("snap-mandatory", &["--tw-scroll-snap-strictness"][..]);
+exact.insert("snap-proximity", &["--tw-scroll-snap-strictness"][..]);
```

## Expected Impact

Fixing this would resolve **all 12 snap utility failures** (9.0% of remaining issues), improving pass rate from 98.67% to 98.79%.

This is a simple, high-confidence fix with zero risk of breaking other utilities. The property exists in Tailwind's canonical property order and is already included in RustyWind's property order list.

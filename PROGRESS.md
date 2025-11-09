# RustyWind Fuzz Test Improvement - Phase 1 & 2

## Session: claude/figure-out-where-to-011CUxiCX4zzbD2tAvsGm2Vw

**Goal**: Continue from 54% pass rate to reach 75-85% through Phase 1 (Variant Order Refinement) and Phase 2 (Value-Based Sub-Sorting)

## Progress Timeline

| Milestone | Pass Rate | Change | Key Fix |
|-----------|-----------|--------|---------|
| Starting Point (from previous session) | 54% | - | Previous work included CLI fixes, negative transforms, property mappings, variant ordering, and Tailwind v4 upgrade |
| Property Order Fix #1 | 61% | +7% | Moved `--tw-space-x-reverse` and `--tw-space-y-reverse` from index 115-116 to after `gap`/`row-gap` (index 166-167) |
| Property Order Fix #2 | 59% | -2% | Fixed outline-style utilities and added divide-reverse utilities |
| **Variant Order Fix (Phase 1 Complete)** | **60%** | **+1%** | **Fixed focus/hover/landscape variant order to match Tailwind v4** |

## Changes Made

### 1. Space Utilities Property Order (61% pass rate)

**Problem**: `space-x` and `space-y` utilities were being sorted too early, before `columns` and `gap` utilities.

**Root Cause**: In `rustywind-core/src/property_order.rs`, `--tw-space-x-reverse` and `--tw-space-y-reverse` were at indices 115-116 (right after `resize`), but in Tailwind's property-order.ts they're at indices 154-155 (after `gap`/`row-gap`).

**Fix**:
- Removed `--tw-space-x-reverse` and `--tw-space-y-reverse` from line 115-116
- Removed `user-select` from line 117 (not in Tailwind's property order)
- Added `--tw-space-x-reverse` and `--tw-space-y-reverse` after `row-gap` at lines 166-167

**Files Changed**:
- `rustywind-core/src/property_order.rs`

**Impact**: +7 percentage points (54% → 61%)

### 2. Filter Utilities Property Mapping (Testing...)

**Problem**: Filter utilities like `grayscale-0`, `sepia-0`, `brightness-50` were being sorted incorrectly.

**Root Cause**: In `rustywind-core/src/utility_map.rs`, filter utilities were mapping to generic `filter` and `backdrop-filter` properties instead of specific custom properties like `--tw-grayscale`, `--tw-sepia`, etc.

**Fix**: Updated utility mappings to use specific custom properties:

Filter utilities:
- `blur` → `--tw-blur` (was `filter`)
- `brightness` → `--tw-brightness` (was `filter`)
- `contrast` → `--tw-contrast` (was `filter`)
- `grayscale` → `--tw-grayscale` (was `filter`)
- `hue-rotate` → `--tw-hue-rotate` (was `filter`)
- `invert` → `--tw-invert` (was `filter`)
- `saturate` → `--tw-saturate` (was `filter`)
- `sepia` → `--tw-sepia` (was `filter`)
- `drop-shadow` → `--tw-drop-shadow` (was `filter`)

Backdrop filter utilities:
- `backdrop-blur` → `--tw-backdrop-blur` (was `backdrop-filter`)
- `backdrop-brightness` → `--tw-backdrop-brightness` (was `backdrop-filter`)
- `backdrop-contrast` → `--tw-backdrop-contrast` (was `backdrop-filter`)
- `backdrop-grayscale` → `--tw-backdrop-grayscale` (was `backdrop-filter`)
- `backdrop-hue-rotate` → `--tw-backdrop-hue-rotate` (was `backdrop-filter`)
- `backdrop-invert` → `--tw-backdrop-invert` (was `backdrop-filter`)
- `backdrop-opacity` → `--tw-backdrop-opacity` (was `backdrop-filter`)
- `backdrop-saturate` → `--tw-backdrop-saturate` (was `backdrop-filter`)
- `backdrop-sepia` → `--tw-backdrop-sepia` (was `backdrop-filter`)

**Files Changed**:
- `rustywind-core/src/utility_map.rs`

**Expected Impact**: Should fix issues with grayscale, sepia, and other filter utilities appearing in wrong positions.

### 3. Outline Style Utilities (Testing...)

**Problem**: `outline-dashed`, `outline-none`, etc. were being sorted incorrectly.

**Root Cause**:
1. In `rustywind-core/src/property_order.rs`, `outline-style` was included but Tailwind's property-order.ts doesn't have it
2. In `rustywind-core/src/utility_map.rs`, outline style utilities were mapping to `outline-style`

**Fix**:
- Removed `outline-style` from property order
- Changed outline style utilities to map to `outline` instead:
  - `outline-none` → `outline` (was `outline-style`)
  - `outline-solid` → `outline` (was `outline-style`)
  - `outline-dashed` → `outline` (was `outline-style`)
  - `outline-dotted` → `outline` (was `outline-style`)
  - `outline-double` → `outline` (was `outline-style`)

**Files Changed**:
- `rustywind-core/src/property_order.rs`
- `rustywind-core/src/utility_map.rs`

**Expected Impact**: Should fix outline utility sorting issues.

### 4. Divide Reverse Utilities (Testing...)

**Problem**: `divide-x-reverse` and `divide-y-reverse` were not recognized, causing sorting issues.

**Root Cause**: These utilities were missing from the utility map.

**Fix**: Added mappings:
- `divide-x-reverse` → `divide-x-width`
- `divide-y-reverse` → `--tw-divide-y-reverse`

**Files Changed**:
- `rustywind-core/src/utility_map.rs`

**Expected Impact**: Should fix divide-reverse sorting issues.

## Next Steps

1. ✅ Test current changes
2. ⏳ Analyze remaining failures
3. ⏳ Implement Phase 1: Variant Order Refinement (if not already achieved)
4. ⏳ Implement Phase 2: Value-Based Sub-Sorting
5. ⏳ Add failing test cases to test suite
6. ⏳ Verify all tests pass
7. ⏳ Commit and push changes

## Test Failure Analysis

### Common Failure Patterns (from 61% pass rate):

1. **Variant ordering issues**:
   - `focus-within:*` vs `focus:*` vs `hover:*`
   - `only:*` vs `checked:*` vs `empty:*`
   - `landscape:*` vs `dark:*` vs `portrait:*`

2. **Value-based sorting** (same property, different values):
   - `scale-y-50` vs `scale-y-100`
   - `rounded-tr` vs `rounded-b`
   - `brightness-50` vs `brightness-100`

3. **Select utilities**: `select-all`, `select-auto` still appearing wrong (user-select not in property order, gets sorted alphabetically)

## Technical Insights

### Property Order Alignment with Tailwind v4

The property order MUST exactly match Tailwind's `packages/tailwindcss/src/property-order.ts`:
- Total properties in Tailwind: 337
- Properties in RustyWind: Now aligned (removed user-select, outline-style)

### Key Differences Found:
- ✅ RustyWind had `user-select` - REMOVED
- ✅ RustyWind had `outline-style` - REMOVED
- ✅ RustyWind had `--tw-space-*-reverse` in wrong position - FIXED
- ✅ Filter utilities mapped to wrong properties - FIXED
- ✅ Divide-reverse utilities missing - FIXED

## Files Modified

1. `rustywind-core/src/property_order.rs` - Property ordering
2. `rustywind-core/src/utility_map.rs` - Utility to property mappings

## Build Status

- ✅ Compiled successfully with `cargo build --release`
- ⏳ Running fuzz tests...

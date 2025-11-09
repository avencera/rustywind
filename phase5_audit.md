# Phase 5: Utility Mapping Audit Results

## Issues Found

### 1. Filter Utilities - INCORRECT ❌

**Current mapping** (utility_map.rs lines 961-969):
```rust
"blur" => Some(&["filter"][..]),
"brightness" => Some(&["filter"][..]),
"contrast" => Some(&["filter"][..]),
"grayscale" => Some(&["filter"][..]),
"hue-rotate" => Some(&["filter"][..]),
"invert" => Some(&["filter"][..]),
"saturate" => Some(&["filter"][..]),
"sepia" => Some(&["filter"][..]),
"drop-shadow" => Some(&["filter"][..]),
```

**Expected mapping** (from Tailwind v4 utilities.ts lines 4037-4200):
```rust
"blur" => Some(&["--tw-blur"][..]),
"brightness" => Some(&["--tw-brightness"][..]),
"contrast" => Some(&["--tw-contrast"][..]),
"grayscale" => Some(&["--tw-grayscale"][..]),
"hue-rotate" => Some(&["--tw-hue-rotate"][..]),
"invert" => Some(&["--tw-invert"][..]),
"saturate" => Some(&["--tw-saturate"][..]),
"sepia" => Some(&["--tw-sepia"][..]),
"drop-shadow" => Some(&["--tw-drop-shadow"][..]),
```

**Impact**: HIGH - Filter utilities won't sort correctly

---

### 2. Backdrop Filter Utilities - INCORRECT ❌

**Current mapping** (utility_map.rs lines 972-980):
```rust
"backdrop-blur" => Some(&["backdrop-filter"][..]),
"backdrop-brightness" => Some(&["backdrop-filter"][..]),
"backdrop-contrast" => Some(&["backdrop-filter"][..]),
"backdrop-grayscale" => Some(&["backdrop-filter"][..]),
"backdrop-hue-rotate" => Some(&["backdrop-filter"][..]),
"backdrop-invert" => Some(&["backdrop-filter"][..]),
"backdrop-opacity" => Some(&["backdrop-filter"][..]),
"backdrop-saturate" => Some(&["backdrop-filter"][..]),
"backdrop-sepia" => Some(&["backdrop-filter"][..]),
```

**Expected mapping** (from Tailwind v4 utilities.ts lines 4049-4250):
```rust
"backdrop-blur" => Some(&["--tw-backdrop-blur"][..]),
"backdrop-brightness" => Some(&["--tw-backdrop-brightness"][..]),
"backdrop-contrast" => Some(&["--tw-backdrop-contrast"][..]),
"backdrop-grayscale" => Some(&["--tw-backdrop-grayscale"][..]),
"backdrop-hue-rotate" => Some(&["--tw-backdrop-hue-rotate"][..]),
"backdrop-invert" => Some(&["--tw-backdrop-invert"][..]),
"backdrop-opacity" => Some(&["--tw-backdrop-opacity"][..]),
"backdrop-saturate" => Some(&["--tw-backdrop-saturate"][..]),
"backdrop-sepia" => Some(&["--tw-backdrop-sepia"][..]),
```

**Impact**: HIGH - Backdrop filter utilities won't sort correctly

---

### 3. Transform Utilities - CORRECT ✅

**Current mapping** (utility_map.rs lines 943-958):
```rust
"rotate" => Some(&["rotate"][..]),
"scale" => Some(&["scale"][..]),
"scale-x" => Some(&["--tw-scale-x"][..]),
"scale-y" => Some(&["--tw-scale-y"][..]),
"translate-x" => Some(&["--tw-translate-x"][..]),
"translate-y" => Some(&["--tw-translate-y"][..]),
"skew-x" => Some(&["--tw-skew-x"][..]),
"skew-y" => Some(&["--tw-skew-y"][..]),
```

**Status**: Matches Tailwind v4 ✅

---

### 4. Ring Utilities - CORRECT ✅

**Current mapping** (utility_map.rs lines 926-931):
```rust
"ring" (width) => Some(&["--tw-ring-shadow"][..]),
"ring" (color) => Some(&["--tw-ring-color"][..]),
"ring-offset" (width) => Some(&["--tw-ring-offset-width"][..]),
"ring-offset" (color) => Some(&["--tw-ring-offset-color"][..]),
```

**Status**: Matches Tailwind v4 ✅

---

### 5. Border Radius Utilities - CORRECT ✅

**Current mapping** (utility_map.rs lines 885-908):
Properly maps to specific properties like `border-top-left-radius`, etc.

**Status**: Matches Tailwind v4 ✅

---

## Summary

- **Issues found**: 2 categories (18 utilities total)
- **Correctly mapped**: Transform, Ring, Border Radius utilities
- **Incorrectly mapped**: Filter utilities (9), Backdrop filter utilities (9)

## Expected Impact

Fixing these 18 utility mappings should improve fuzz test pass rate by **2-5%** as predicted.

## Files to Modify

1. `rustywind-core/src/utility_map.rs` - Update filter and backdrop-filter mappings (lines 961-980)

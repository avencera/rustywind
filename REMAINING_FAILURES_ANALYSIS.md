# Analysis of Remaining 9% Failures (9/100 tests)

## Summary of Failures

From the fuzz test output, here are the 9 remaining failures:

### 1. **space-x-reverse + gap-y** (Test #1)
```
Expected: space-x-reverse, gap-y-4
Actual:   gap-y-4, space-x-reverse
```
**Issue:** `space-x-reverse` should come before `gap-y-4`, but doesn't

### 2. **transition-opacity vs transition-none** (Test #9)
```
Expected: transition-opacity, transition-none
Actual:   transition-none, transition-opacity
```
**Issue:** Both map to `transition-property`, need numeric/value-based sorting

### 3. **divide-x-reverse before rounded-full** (Test #26)
```
Expected: rounded-full, ..., divide-x-reverse
Actual:   divide-x-reverse, rounded-full, ...
```
**Issue:** `divide-x-reverse` sorting way too early

### 4. **bg-blue-900 vs bg-green-50** (Test #27)
```
Expected: bg-blue-900, bg-green-50
Actual:   bg-green-50, bg-blue-900
```
**Issue:** Color utilities with same property sorting alphabetically instead of by specificity

### 5. **outline-offset-1 vs outline-double** (Test #38)
```
Expected: outline-offset-1, outline-double
Actual:   outline-double, outline-offset-1
```
**Issue:** Property order: `outline-offset` should come after `outline` (style)

### 6. **will-change-scroll vs select-none** (Test #77)
```
Expected: will-change-scroll, select-none
Actual:   select-none, will-change-scroll
```
**Issue:** Property order conflict between `will-change` and `user-select`

### 7. **blur-lg vs ring-inset** (Test #83)
```
Expected: blur-lg, ..., ring-inset
Actual:   ring-inset, ..., blur-lg
```
**Issue:** Ring utilities should come after filter utilities

### 8. **rounded-tr before divide-x-reverse** (Test #85)
```
Expected: rounded-tr, ..., divide-x-reverse
Actual:   divide-x-reverse, ..., rounded-tr
```
**Issue:** `divide-x-reverse` sorting too early (same as #3)

### 9. **container vs col-end-1** (Test #89)
```
Expected: col-end-1, container
Actual:   container, col-end-1
```
**Issue:** `container` should come after grid utilities

---

## Root Cause Analysis

### Category 1: **Divide Utilities** (Tests #26, #85) - 2 failures
**Problem:** `divide-x-reverse` sorting way too early (before rounded, before rounded-tr)

**Current mapping:**
```rust
// divide-x-reverse exact mapping
exact.insert("divide-x-reverse", &["--tw-divide-x-reverse"][..]);
```

**Likely issue:**
- `--tw-divide-x-reverse` property index might be wrong
- Or it's not being recognized and falling back to unknown

**To investigate:**
- Check property index of `--tw-divide-x-reverse`
- Compare with Tailwind v4 property-order.ts
- Check if it needs to be after border-radius properties

---

### Category 2: **Space-reverse + Gap Interaction** (Test #1) - 1 failure
**Problem:** `space-x-reverse` should come before `gap-y-4`

**Current mapping:**
```rust
"space-x" => Some(&["row-gap"][..]),
"space-y" => Some(&["column-gap"][..]),
```

**Issue:** `space-x-reverse` and `space-y-reverse` are different utilities that might need exact mappings

**To investigate:**
- Check if `space-x-reverse` and `space-y-reverse` need separate exact mappings
- They might not be covered by the `space-x` pattern

---

### Category 3: **Property Order Conflicts** (Tests #38, #77, #83) - 3 failures

#### Test #38: `outline-offset-1` vs `outline-double`
**Current:**
- `outline-double` → `outline` (style)
- `outline-offset-1` → `outline-offset`

**Issue:** Need to verify property order indices

#### Test #77: `will-change-scroll` vs `select-none`
**Current:**
- `will-change-scroll` → `will-change`
- `select-none` → `user-select`

**Issue:** Need to check which property comes first

#### Test #83: `blur-lg` vs `ring-inset`
**Current:**
- `blur-lg` → `--tw-blur`
- `ring-inset` → `--tw-inset-ring-shadow`

**Issue:** Ring shadow should come before filter properties? Need verification

---

### Category 4: **Transition Utilities** (Test #9) - 1 failure
**Problem:** `transition-opacity` vs `transition-none` ordering

**Current mapping:**
```rust
"transition" => Some(&["transition-property"][..]),
```

**Issue:** Both map to same property, need value-based sorting
- `transition-none` might need exact mapping to `none` value
- Or they're not being differentiated

---

### Category 5: **Container vs Grid** (Test #89) - 1 failure
**Problem:** `container` should come after `col-end-1`

**Issue:** Need to check what property `container` maps to
- Might not have an exact mapping
- Or maps to wrong property

---

### Category 6: **Color Ordering** (Test #27) - 1 failure
**Problem:** `bg-blue-900` vs `bg-green-50` alphabetic ordering

**This is complex:**
- Both map to `background-color`
- Prettier/Tailwind might sort by color name or numeric suffix
- This requires value-based sorting within same property

---

## Priority Order to Fix

### **Priority 1: Divide Utilities** (2 failures, ~2%)
- Simple property mapping issue
- Quick win

### **Priority 2: Space-reverse Utilities** (1 failure, ~1%)
- Add exact mappings for `space-x-reverse`, `space-y-reverse`
- Should be straightforward

### **Priority 3: Property Order Conflicts** (3 failures, ~3%)
- Verify property indices
- May need to check property_order.rs against Tailwind v4

### **Priority 4: Container Utility** (1 failure, ~1%)
- Check mapping or add exact mapping

### **Priority 5: Transition Utilities** (1 failure, ~1%)
- May need exact mappings for transition variants
- Or value-based sorting

### **Priority 6: Color Ordering** (1 failure, ~1%)
- Complex - requires value/name sorting
- May not be worth the complexity (diminishing returns)

---

## Estimated Remaining Potential

| Priority | Failures | Estimated Fix Difficulty | Impact |
|----------|----------|-------------------------|---------|
| 1. Divide | 2 | Easy | +2% |
| 2. Space-reverse | 1 | Easy | +1% |
| 3. Property conflicts | 3 | Medium | +3% |
| 4. Container | 1 | Easy | +1% |
| 5. Transition | 1 | Medium | +1% |
| 6. Color | 1 | Hard | +1% |

**Total potential: 91% → 100%** (but color ordering may not be realistic)

**Realistic target: 91% → 98-99%** with priorities 1-5

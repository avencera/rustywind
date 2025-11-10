# Tailwind CSS `-none` and `-0` Utility Sorting Patterns

## Executive Summary

**Key Finding:** There is **NO universal sorting rule** for `-none` and `-0` utilities in Tailwind CSS/Prettier. Each utility category follows its own sorting logic based on CSS property values, not lexical patterns.

---

## Detailed Findings by Category

### Category A: SIZE-BASED UTILITIES ⚠️ COMPLEX

**Utilities:** `blur-none`, `shadow-none`, `rounded-none`, `drop-shadow-none`

**Pattern:** Custom CSS value-based ordering where `-none` appears in the MIDDLE of the sequence.

**Actual Prettier Ordering:**

```
blur:
  [1] blur
  [2] blur-2xl
  [3] blur-3xl
  [4] blur-lg
  [5] blur-md
  [6] blur-none      ← position 6 of 8
  [7] blur-sm
  [8] blur-xl

shadow:
  [1] shadow
  [2] shadow-2xl
  [3] shadow-inner
  [4] shadow-lg
  [5] shadow-md
  [6] shadow-none    ← position 6 of 8
  [7] shadow-sm
  [8] shadow-xl

rounded:
  [1] rounded
  [2] rounded-2xl
  [3] rounded-3xl
  [4] rounded-full
  [5] rounded-lg
  [6] rounded-md
  [7] rounded-none   ← position 7 of 9
  [8] rounded-sm
  [9] rounded-xl

drop-shadow:
  [1] drop-shadow-2xl
  [2] drop-shadow-lg
  [3] drop-shadow-md
  [4] drop-shadow-sm
  [5] drop-shadow-xl
  [6] drop-shadow-none  ← position 6 of 6 (LAST!)
```

**Key Observation:**

- `-md`, `-lg`, `-2xl`, `-3xl` come BEFORE `-none`
- `-sm`, `-xl` come AFTER `-none`
- This is NOT alphabetical or size-based!
- Likely based on actual CSS values (e.g., `0` vs `1px` vs `2px`)

---

### Category B: ALWAYS SORTS LAST ✅ SIMPLE

**Utilities:** `transition-none`

**Pattern:** Always appears AFTER all other `transition-*` values.

**Example:**

```
Input:  transition-transform transition-shadow transition-opacity transition-colors transition-all transition-none
Output: transition-all transition-colors transition-opacity transition-shadow transition-transform transition-none
```

**Rule:** `transition-none` → LAST position always

---

### Category C: SPECIAL POSITIONING ⚠️ CASE-BY-CASE

**Utilities:** `border-0`, `grayscale-0`

**Pattern:** Sorts after the base value, has special relationship with default.

#### border-0

```
Input:  border-8 border-4 border-2 border border-0
Output: border border-0 border-2 border-4 border-8

Rule: border → border-0 → border-2/4/8 (ascending)
```

#### grayscale-0

```
Input:  grayscale-0 grayscale
Output: grayscale grayscale-0

Rule: grayscale → grayscale-0
```

---

### Category D: NUMERIC SORTING ✅ SIMPLE

**Utilities:** `brightness-0`, `contrast-0`, `saturate-0`, `scale-0`, `rotate-0`, `duration-0`

**Pattern:** `-0` sorts FIRST, followed by ascending numeric order.

**Examples:**

```
brightness-0 brightness-50 brightness-100 brightness-150
contrast-0 contrast-50 contrast-100 contrast-150
saturate-0 saturate-50 saturate-100 saturate-150
scale-0 scale-50 scale-100 scale-150
rotate-0 rotate-45 rotate-90 rotate-180
duration-0 duration-100 duration-500 duration-1000
```

**Rule:** Always sort numerically with 0 first

---

### Category E: ALPHABETICAL ✅ SIMPLE

**Utilities:** `animate-none`

**Pattern:** Sorts alphabetically with other `animate-*` values.

**Example:**

```
Input:  animate-spin animate-pulse animate-ping animate-none animate-bounce
Output: animate-bounce animate-none animate-ping animate-pulse animate-spin
```

**Rule:** Standard alphabetical sorting (animate-none falls between animate-bounce and animate-ping)

---

## Summary Table

| Utility            | Pattern         | Position              | Easy to Implement? |
| ------------------ | --------------- | --------------------- | ------------------ |
| `blur-none`        | CSS-value-based | Middle (6/8)          | ❌ No              |
| `shadow-none`      | CSS-value-based | Middle (6/8)          | ❌ No              |
| `rounded-none`     | CSS-value-based | Middle (7/9)          | ❌ No              |
| `drop-shadow-none` | CSS-value-based | Last (6/6)            | ❌ No              |
| `transition-none`  | Always last     | Last                  | ✅ Yes             |
| `border-0`         | After base      | 2nd                   | ⚠️ Special case    |
| `grayscale-0`      | After base      | 2nd                   | ⚠️ Special case    |
| `brightness-0`     | Numeric         | First                 | ✅ Yes             |
| `contrast-0`       | Numeric         | First                 | ✅ Yes             |
| `saturate-0`       | Numeric         | First                 | ✅ Yes             |
| `scale-0`          | Numeric         | First                 | ✅ Yes             |
| `rotate-0`         | Numeric         | First                 | ✅ Yes             |
| `duration-0`       | Numeric         | First                 | ✅ Yes             |
| `animate-none`     | Alphabetical    | Middle (alphabetical) | ✅ Yes             |

---

## Implications for RustyWind

### Why This is Hard

1. **No Universal Pattern:** The `-none` suffix doesn't have consistent behavior
2. **CSS-Value-Based:** Size-based utilities require knowing the actual CSS values
3. **Non-Lexical:** Cannot be solved with string comparison or regex patterns
4. **Requires Mapping:** Would need to hardcode the exact ordering for each utility

### Possible Solutions

1. **Option A:** Use Tailwind's official sorting order data structure
   - Import from `tailwindcss` package
   - Requires keeping in sync with Tailwind versions

2. **Option B:** Create comprehensive hardcoded maps
   - Manually map each utility to its position
   - Maintenance burden for new utilities

3. **Option C:** Accept limitations
   - Document known sorting differences
   - Focus on 90% use cases that follow predictable patterns

---

## Test Files Created

- `/home/user/rustywind/tests/fuzz/test-none-patterns.mjs` - Initial exploration
- `/home/user/rustywind/tests/fuzz/test-none-detailed.mjs` - Comprehensive testing
- `/home/user/rustywind/tests/fuzz/test-none-summary.mjs` - Pattern classification
- `/home/user/rustywind/tests/fuzz/test-none-visualization.mjs` - Visual ordering maps

Run any of these with:

```bash
cd /home/user/rustywind/tests/fuzz
node test-none-patterns.mjs
```

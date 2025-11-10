# Quick Reference: -none/-0 Sorting Patterns

## TL;DR
**There is NO universal pattern!** Each utility has its own sorting logic.

---

## Quick Lookup Table

### ✅ SIMPLE PATTERNS (Easy to implement)

| Utility | Rule | Example Output |
|---------|------|----------------|
| `transition-none` | **Always LAST** | `transition-all transition-colors transition-none` |
| `brightness-0` | **Numeric (0 first)** | `brightness-0 brightness-50 brightness-100` |
| `contrast-0` | **Numeric (0 first)** | `contrast-0 contrast-50 contrast-100` |
| `saturate-0` | **Numeric (0 first)** | `saturate-0 saturate-50 saturate-100` |
| `scale-0` | **Numeric (0 first)** | `scale-0 scale-50 scale-100` |
| `rotate-0` | **Numeric (0 first)** | `rotate-0 rotate-45 rotate-90` |
| `duration-0` | **Numeric (0 first)** | `duration-0 duration-100 duration-500` |
| `animate-none` | **Alphabetical** | `animate-bounce animate-none animate-ping` |

---

### ⚠️ COMPLEX PATTERNS (Hard to implement)

| Utility | Rule | Position | What comes BEFORE | What comes AFTER |
|---------|------|----------|-------------------|------------------|
| `blur-none` | CSS-value-based | 6 of 8 | `blur-md`, `blur-lg`, `blur-2xl`, `blur-3xl` | `blur-sm`, `blur-xl` |
| `shadow-none` | CSS-value-based | 6 of 8 | `shadow-md`, `shadow-lg`, `shadow-2xl` | `shadow-sm`, `shadow-xl` |
| `rounded-none` | CSS-value-based | 7 of 9 | `rounded-md`, `rounded-lg`, `rounded-2xl`, `rounded-3xl` | `rounded-sm`, `rounded-xl` |
| `drop-shadow-none` | CSS-value-based | Last | All other values | (nothing) |

---

### 🔧 SPECIAL CASES

| Utility | Rule | Full Order |
|---------|------|------------|
| `border-0` | After base, before numeric | `border` → `border-0` → `border-2` → `border-4` → `border-8` |
| `grayscale-0` | After base | `grayscale` → `grayscale-0` |

---

## Visual Reference: Size-Based Utilities

### The Puzzle: Why does -sm come AFTER -none?

```
blur-md      ← comes BEFORE -none
blur-none    ← middle position
blur-sm      ← comes AFTER -none (weird!)
```

**Answer:** Prettier sorts by the actual CSS blur radius values:
- `blur-md` = 12px blur → higher value → comes first
- `blur-none` = 0px blur → zero value → middle
- `blur-sm` = 4px blur → small but not zero → comes after

This is NOT alphabetical or lexical sorting!

---

## Full Prettier Ordering (for reference)

```
blur: blur blur-2xl blur-3xl blur-lg blur-md blur-none blur-sm blur-xl
shadow: shadow shadow-2xl shadow-inner shadow-lg shadow-md shadow-none shadow-sm shadow-xl
rounded: rounded rounded-2xl rounded-3xl rounded-full rounded-lg rounded-md rounded-none rounded-sm rounded-xl
drop-shadow: drop-shadow-2xl drop-shadow-lg drop-shadow-md drop-shadow-sm drop-shadow-xl drop-shadow-none
```

Notice:
- NOT alphabetical (xl comes after sm)
- NOT by size name (2xl before lg)
- Based on actual CSS values

---

## Testing Commands

```bash
cd /home/user/rustywind/tests/fuzz

# Quick pattern overview
node test-none-patterns.mjs

# Detailed analysis
node test-none-detailed.mjs

# Classification summary
node test-none-summary.mjs

# Visual ordering maps
node test-none-visualization.mjs
```

---

## Implementation Recommendation

For RustyWind to match Prettier exactly:

1. **Easy wins:** Implement numeric sorting (`*-0` first) and `transition-none` last
2. **Special cases:** Hardcode `border-0` and `grayscale-0` positions
3. **Complex cases:** Either:
   - Import Tailwind's official sort order
   - Hardcode the exact sequences for blur/shadow/rounded/drop-shadow
   - Accept the sorting difference and document it

The complex cases are NOT solvable with regex or pattern matching alone!

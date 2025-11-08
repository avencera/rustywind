# Pattern-Based Static List - Context & Learnings

**Session:** 011CUviHN5e9Ta7ui77gQ2o8
**Last Updated:** 2025-11-08

## Purpose

This document captures important context, decisions, learnings, and questions for anyone (including future LLMs) picking up this implementation.

---

## The Problem We're Solving

RustyWind currently has a 5,032-line hardcoded static list in `rustywind-core/src/defaults.rs` that:
- Has ~80% accuracy matching Tailwind's canonical order
- Cannot handle arbitrary values (`bg-[#fff]`, `w-[100px]`)
- Cannot handle Tailwind v4 features
- Requires manual maintenance

---

## Critical Insights

### 1. Base Classes and Variants Are Separate in CSS

**This is the most important finding!**

Base classes (e.g., `flex`) and their variant versions (e.g., `sm:flex`) are NOT next to each other in Tailwind's CSS output. They're in completely different sections:

```
Section 1: All base classes (sorted by property order)
  - flex, grid, block, mx-0, mx-4, bg-red-500

Section 2: All variant classes (sorted by variant order, then property order)
  - hover:flex, md:flex, sm:mx-4, md:mx-8
```

This means we cannot simply extend the static list - we need algorithmic sorting.

### 2. Tailwind's Exact Sorting Algorithm

From `packages/tailwindcss/src/compile.ts:83-115`:

```
1. Variant Order (0 for base classes, then bit flags for variants)
   ↓
2. Property Index (from property-order.ts - position in 416-property array)
   ↓
3. Property Count (classes generating more properties come later)
   ↓
4. Alphabetical (final tiebreaker)
```

### 3. Classes Sort by CSS Properties, Not Names

Key principle: Classes are ordered based on **what CSS they generate**, not what they're called.

Example:
- `px-3` generates `padding-left` + `padding-right`
- Look up `padding-left` in property-order.ts → index 323
- That index determines sort position

---

## Architecture Overview

### Current RustyWind Structure

```
rustywind-core/src/
├── lib.rs              # Main library exports
├── app.rs              # Application logic
├── defaults.rs         # 5,032-line static list (THIS IS WHAT WE'RE REPLACING)
├── sorter.rs           # Sorter enum (DefaultSorter or CustomSorter)
├── class_wrapping.rs   # Class wrapping utilities
├── consts.rs           # Constants
└── parser/             # Parsing logic
```

### New Structure We'll Create

```
rustywind-core/src/
├── property_order.rs   # 416 CSS properties (Phase 1)
├── variant_order.rs    # Variant ordering (Phase 1)
├── utility_map.rs      # Utility → property mapping (Phase 2)
├── class_parser.rs     # Parse class strings (Phase 3)
├── pattern_sorter.rs   # Core sorting algorithm (Phase 4)
└── hybrid_sorter.rs    # Optimized with cache (Phase 5)
```

---

## Important Decisions

### Decision Log

#### 2025-11-08: Variant Order with u64 bitwise flags

**Decision:** Use u64 for variant order bitwise flags (instead of u128 or BigInt)

**Rationale:**
- We have 80 variants, but only 64 fit in u64
- First 64 variants cover all critical cases (pseudo-elements, interactive, responsive)
- Variants beyond index 63 (dark, motion-safe, etc.) will have order 0 if not in first 64
- This matches the pattern-based approach where unknown variants are handled gracefully
- Can upgrade to u128 later if needed without breaking changes

**Impact:** Minimal - the last 16 variants are edge cases and will still work, just won't be perfectly ordered relative to each other

#### 2025-11-08: Property count is 337, not 416

**Finding:** Tailwind v4 has 337 CSS properties, not 416 as mentioned in older research

**Action:** Updated documentation and code to reflect 337 properties

#### 2025-11-08: Multi-part utility base parsing

**Challenge:** Utilities like `min-w-0`, `border-t-2`, `rounded-tl-lg` have multi-part bases that need special handling

**Solution:** Implemented prefix matching before simple dash splitting
- Check known multi-part prefixes first (min-w, max-h, border-t, etc.)
- Falls back to simple dash splitting for single-part bases
- Handles ~30 common multi-part patterns

**Implementation:** parse_utility_parts() function in utility_map.rs

#### 2025-11-08: Utility map architecture - exact match + pattern matching

**Decision:** Two-tier lookup system for utility → property mapping

**Rationale:**
- Exact matches (HashMap): O(1) for static utilities like "flex", "block"
- Pattern matching: Algorithmic fallback for parameterized utilities like "m-4", "bg-red-500"
- Supports arbitrary values: bg-[#fff] detected by bracket notation
- Helper predicates determine value types (color, size, weight)

**Performance:** Fast path for common cases, flexible for edge cases

---

## Open Questions

### For User

1. **Caching Strategy:** The plan suggests a HashMap cache of ~300 most common classes. Should we use `quick_cache` or stick with standard `HashMap` + `once_cell::sync::Lazy`?

### For Implementation

(Will be added as questions arise)

---

## Key Files & References

### In This Repository

- `docs/planning/static_list_plan.md` - Complete implementation plan with code examples
- `rustywind-core/src/defaults.rs` - Current 5,032-line static list (to be replaced)
- `rustywind-core/src/sorter.rs` - Current sorter implementation

### External References

- Source commit: https://github.com/praveenperera/tailwindcss/commit/b2c5e50edf4f9c653d138870af0c550c3ff11e7e
- Tailwind property-order.ts: `packages/tailwindcss/src/property-order.ts`
- Tailwind compile.ts: `packages/tailwindcss/src/compile.ts`
- Tailwind sort.ts: `packages/tailwindcss/src/sort.ts`

---

## Data Structures

### ParsedClass

```rust
pub struct ParsedClass<'a> {
    pub original: &'a str,      // "md:hover:mx-4"
    pub variants: Vec<&'a str>, // ["md", "hover"]
    pub utility: &'a str,       // "mx"
    pub value: &'a str,         // "4"
    pub important: bool,        // false
}
```

### SortKey

```rust
pub struct SortKey {
    pub variant_order: u64,      // Bitwise flags for variants
    pub property_index: usize,   // Index from property-order.ts
    pub property_count: usize,   // How many properties generated
    pub class: String,           // Original class for alpha sort
}
```

---

## Success Metrics

Target improvements:
- Accuracy: 80% → 99%
- Code size: 5,032 lines → ~800 lines
- Arbitrary values: ❌ → ✅
- v4 features: ❌ → ✅
- Memory: ~200KB → ~50KB

---

## Notes for Future LLMs

If you're picking up this implementation:

1. **Read the plan first**: `docs/planning/static_list_plan.md` has detailed code examples
2. **Check progress**: `docs/planning/plan_progress.md` shows what's done
3. **Read this file**: You're doing it! This has the critical context
4. **The key insight**: Base classes and variants are separate in CSS (see Critical Insights #1)
5. **Follow the phases**: Don't skip ahead - each phase builds on the previous
6. **Ask before big decisions**: User wants to be consulted on architecture choices
7. **Push as you go**: Commit and push regularly

---

*Last updated: 2025-11-08 - Initial context document created*

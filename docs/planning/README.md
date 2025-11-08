# Pattern-Based Static List Implementation - Planning Documents

**Session:** 011CUviHN5e9Ta7ui77gQ2o8
**Date:** 2025-11-08
**Source Commit:** https://github.com/praveenperera/tailwindcss/commit/b2c5e50edf4f9c653d138870af0c550c3ff11e7e

## Overview

This directory contains comprehensive planning documents for implementing a pattern-based static list sorter in RustyWind that matches Tailwind CSS v4's canonical sorting algorithm.

## Current State

RustyWind currently has:
- **5,032-line** hardcoded static class list
- **~80% accuracy** in matching Tailwind's order
- **No support** for arbitrary values (e.g., `bg-[#fff]`, `w-[100px]`)
- **No support** for Tailwind v4 features

## Goal

Replace the static list with intelligent pattern-based sorting that achieves:
- **~99% accuracy** matching Tailwind's canonical order
- **~800 lines** of code (85% reduction)
- **Full support** for arbitrary values
- **Full support** for Tailwind v4 features
- **Better performance** through hybrid optimization

## Key Documents

### 1. [static_list_plan.md](./static_list_plan.md)
**Main Implementation Plan** - Complete 7-phase plan to build the pattern-based sorter

**Key Sections:**
- Critical finding: Variants are in completely different CSS sections than base classes
- Phase-by-phase implementation guide with actual Rust code
- Property order foundation (416 CSS properties)
- Utility pattern mapping
- Class parser implementation
- Pattern-based sorter matching Tailwind's algorithm
- Hybrid optimization with common class cache
- Testing and verification

**Timeline:** 3 weeks across 7 phases

### 2. [prettier_analysis.md](./prettier_analysis.md)
**How Prettier Plugin Works** - Research on how prettier-plugin-tailwindcss determines class order

**Key Findings:**
- NOT alphabetical or class-name-based
- Uses **property order** from 416 CSS properties
- Sorting algorithm: variants → property index → property count → alphabetical
- Classes sorted by **what CSS they generate**, not what they're called

### 3. [css_verification.md](./css_verification.md)
**CSS Output Order Verification** - Confirms Tailwind's CSS matches its sorting algorithm

**Key Finding:**
- Both `build()` and `getClassOrder()` use identical `compileCandidates()` function
- CSS file output represents the canonical sort order
- RustyWind's CSS-file mode achieves 100% accuracy

## The Sorting Algorithm

Tailwind CSS v4 sorts classes using this exact algorithm:

```
1. Variant Order (0 for base classes, then hover, focus, md, lg, etc.)
   ↓
2. Property Index (from property-order.ts - 416 properties)
   ↓
3. Property Count (more properties = later)
   ↓
4. Alphabetical (final tiebreaker)
```

**Example:**
```
Input:  px-3 focus:hover:p-3 hover:p-1 py-3
Output: px-3 py-3 hover:p-1 focus:hover:p-3
        ^^^^^^^^^ base classes first
                  ^^^^^^^^^^^^^^^^^^^^^^^ variants after
```

## Implementation Approach

### Pattern-Based Matching

Instead of listing every possible class, we:

1. **Parse** the class: `md:mx-4` → variant="md", utility="mx", value="4"
2. **Map** utility to property: "mx" → "margin-inline"
3. **Look up** property index: "margin-inline" → 38
4. **Look up** variant index: "md" → 2
5. **Compute** sort key: (variant_order: 2, property_index: 38)
6. **Compare** with other classes using same algorithm

### Hybrid Optimization

For performance, we maintain a fast-path cache of ~300 most common classes:
- `flex`, `grid`, `block`, `hidden`
- `m-0`, `m-1`, `m-2`, `m-4`, `mx-auto`
- `p-0`, `p-1`, `p-2`, `p-4`
- `bg-white`, `text-black`, etc.

This covers ~90% of real-world usage with O(1) HashMap lookup.

## Success Metrics

| Metric | Before (Current) | After (Pattern-Based) |
|--------|-----------------|----------------------|
| Accuracy | ~80% | ~99% |
| Code Size | 5,032 lines | ~800 lines |
| Arbitrary Values | ❌ No | ✅ Yes |
| v4 Features | ❌ No | ✅ Yes |
| Maintenance | Manual updates | Auto-generated |
| Memory | ~200KB | ~50KB |

## Next Steps

See [static_list_plan.md](./static_list_plan.md) for the complete implementation plan with:
- Detailed Rust code for each phase
- Testing strategies
- Migration path
- Risk mitigation
- Timeline breakdown

## Related Work

This plan builds on research from the Tailwind CSS v4 codebase analyzing:
- Property ordering system (`property-order.ts`)
- Sorting algorithm (`compile.ts`, `sort.ts`)
- Variant ordering system
- CSS generation pipeline

---

*Planning documents imported from commit b2c5e50 on 2025-11-08*

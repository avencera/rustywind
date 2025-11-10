# Rustywind vs Prettier Tailwind Plugin - Comparison Results

**Date:** November 10, 2025
**Rustywind Version:** v0.24.3 (locally built from `claude/fix-top-three-fuzz-issues-011CUyaP5RH5jAXudN4oRkMs`)
**Prettier Plugin:** prettier-plugin-tailwindcss (from tests/fuzz)

## Executive Summary

A comprehensive comparison of 50 real-world test files from various open-source projects reveals **significant differences** in how rustywind and Prettier's Tailwind CSS plugin sort classes.

### Key Findings

- **Files Tested:** 50
- **Identical Outputs:** 1 (2.00%)
- **Different Outputs:** 49 (98.00%)
- **Files with Class Differences:** 41
- **Total Class Changes:** 1,134
- **Pure Sorting Differences:** 1,044 (92.1%)
- **Other Differences:** 90 (7.9%)

## Compatibility Assessment

**❌ NOT COMPATIBLE for interchangeable use**

The tools produce different sorting orders in 98% of test cases, making them unsuitable for:
- Swapping between tools in the same project
- Team workflows where some use rustywind and others use prettier
- Migration between tools without expecting file changes

## Types of Differences Found

### 1. Pure Sorting Differences (92.1% of changes)

These are cases where both tools successfully identify and sort the same classes, but produce different orders.

#### Examples:

**Color Classes Positioning:**
```diff
- Rustywind: hidden text-sm font-semibold text-white uppercase lg:inline-block
+ Prettier:  text-white text-sm uppercase hidden lg:inline-block font-semibold
```

**Responsive Modifiers:**
```diff
- Rustywind: mr-3 hidden flex-row flex-wrap items-center md:flex lg:ml-auto
+ Prettier:  md:flex hidden flex-row flex-wrap items-center lg:ml-auto mr-3
```

**Typography and Spacing:**
```diff
- Rustywind: text-base leading-6 font-medium text-primary-500 hover:text-primary-600
+ Prettier:  text-primary-500 hover:text-primary-600 text-base leading-6 font-medium
```

**Layout Utilities:**
```diff
- Rustywind: relative mt-16 mb-6 flex w-full min-w-0 flex-col rounded-lg bg-white break-words shadow-xl
+ Prettier:  relative flex flex-col min-w-0 break-words bg-white w-full mb-6 shadow-xl rounded-lg mt-16
```

### 2. Other Differences (7.9% of changes)

These involve different class counts or additional formatting changes. Many appear to be related to:
- Prettier's general formatting affecting whitespace in className attributes
- Multi-line class attributes being reformatted
- Template literal expressions within classNames

## Detailed Analysis

### File Categories

| Category | Files | Files with Diffs | Match Rate |
|----------|-------|------------------|------------|
| **HTML Templates** | 4 | 4 | 0% |
| **React Components** | 24 | 23 | 4.2% |
| **Next.js Pages** | 2 | 1 | 50% |
| **Landing Pages** | 1 | 1 | 0% |
| **Complex Layouts** | 7 | 7 | 0% |
| **Vue Components** | 4 | 4 | 0% |
| **Overall** | **50** | **49** | **2%** |

### Only Matching File

**nextjs-pages/tailadmin-dashboard-page.tsx** - The only file where both tools produced identical output. This file is relatively simple with minimal Tailwind classes.

## Common Sorting Pattern Differences

### 1. Color Classes

**Prettier** tends to move color-related classes (like `text-white`, `bg-blue-500`) **earlier** in the sequence.

**Rustywind** keeps them interspersed with other utilities based on different categorization.

### 2. Responsive Modifiers

**Prettier** groups responsive variants differently than rustywind.

### 3. Layout Utilities

**Prettier** appears to prioritize display/flex properties earlier.

**Rustywind** follows a different categorization order.

### 4. Spacing Utilities

Different positioning of margin (`m-*`, `mx-*`, `mt-*`) and padding (`p-*`, `px-*`, `pt-*`) utilities.

### 5. Typography

Font weight (`font-bold`, `font-semibold`), size (`text-sm`, `text-xl`), and color are ordered differently.

## Test File Highlights

### Highest Complexity Files with Differences

1. **play-index.html** - 542 class attributes, all with differences
2. **notus-landing.js** - 185 className usages, all with differences
3. **tailadmin-notification-dropdown.tsx** - 92 className attributes, all with differences
4. **CardTable.js** - Extensive table styling with many variants
5. **FwbPagination.vue** - Complex Vue component with state variants

## Implications

### For Users

1. **Choose One Tool:** Projects should standardize on either rustywind OR prettier, not both
2. **Expect Changes:** Migrating between tools will cause mass file changes
3. **Git Conflicts:** Teams using different tools will have constant merge conflicts
4. **CI/CD:** Ensure the same tool is used in development and CI

### For Tool Developers

1. **No Standard Exists:** There's no official Tailwind CSS class sorting specification
2. **Prettier is Official:** The prettier plugin is maintained by Tailwind Labs
3. **Rustywind is Independent:** Follows its own sorting logic
4. **Room for Alignment:** Opportunity to converge on a common standard

## Recommendations

### For New Projects

- **Use Prettier Plugin** if already using Prettier
- **Use Rustywind** if you prefer a standalone CLI tool or need Rust performance

### For Existing Projects

- **Stick with current tool** unless there's a compelling reason to switch
- **If migrating:** Do it in a single, dedicated commit to isolate the formatting changes

### For Teams

- **Document the choice** in project README and contributing guidelines
- **Enforce with CI** to prevent mixing tools
- **Pre-commit hooks** to ensure consistency

## Test Infrastructure

All comparison scripts and results are available in the `tailwind-sorting-test-files/` directory:

- **compare-tools.js** - Main comparison script
- **analyze-class-diffs.js** - Class-specific difference analyzer
- **comparison-results/** - Full output from both tools
- **comparison-results/diffs/** - Diff files for each test case

## Reproducing Results

```bash
# Build rustywind
cargo build --release

# Run comparison
cd tailwind-sorting-test-files
node compare-tools.js

# Analyze class differences
node analyze-class-diffs.js
```

## Conclusion

While both rustywind and Prettier's Tailwind CSS plugin successfully sort Tailwind classes, they use **fundamentally different sorting algorithms** that produce incompatible results.

**The tools are NOT interchangeable.** Teams and projects must choose one and stick with it consistently.

Future work could investigate:
1. Whether one ordering is "more correct" according to Tailwind's recommendations
2. Opportunity for rustywind to offer a "prettier-compatible" mode
3. Whether a standard sorting specification should be proposed to the Tailwind community

---

**Test Collection:** 50 files from MIT-licensed open-source projects
**Source Projects:** Notus Next.js, TailAdmin, Tailwind Next.js Starter Blog, Play, Flowbite Vue
**Full Attribution:** See [sources.md](./sources.md)

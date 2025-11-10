# Tailwind CSS Class Sorting Test Files

This repository contains a collection of 50 diverse files from open source projects for testing and comparing Tailwind CSS class sorting tools, specifically **rustywind CLI** and the **Prettier Tailwind CSS plugin**.

## ⚠️ IMPORTANT: DO NOT MODIFY THESE FILES

**These test files are READ-ONLY reference data from real open-source projects.**

- ❌ **NEVER** modify these files to make tests pass
- ❌ **NEVER** change class orders or formatting
- ❌ **NEVER** add or remove classes
- ✅ **DO** use them as-is to identify sorting issues
- ✅ **DO** fix the sorting algorithm, not the test files

These files represent real-world usage patterns and must remain pristine to be valuable as test cases.

## Purpose

To verify that rustywind and Prettier's Tailwind plugin sort classes in the same way, providing confidence when choosing between these tools or using them interchangeably in projects.

## Contents

### Test Files (50 files total)

```
test-files/
├── html-templates/        # 4 HTML files with extensive Tailwind usage
├── react-components/      # 24 React/Next.js JSX/TSX components
├── nextjs-pages/         # 2 Next.js page components
├── landing-pages/        # 1 complete landing page
├── complex-layouts/      # 7 blog and content layouts
└── vue-components/       # 4 Vue 3 Single File Components
```

### Documentation Files

- **[sources.md](./sources.md)** - Complete list of source repositories with licenses and attribution
- **[test-categories.md](./test-categories.md)** - Categorization of files by complexity and features
- **[comparison-strategy.md](./comparison-strategy.md)** - Guide for running comparisons between tools
- **README.md** (this file) - Overview and quick start

## Quick Start

### 1. Install Tools

```bash
# Install rustywind
npm install -g rustywind
# or with cargo:
# cargo install rustywind

# Install Prettier + Tailwind plugin
npm install -D prettier prettier-plugin-tailwindcss
```

### 2. Run a Quick Test

Pick a test file and compare:

```bash
# Copy a test file
cp test-files/html-templates/play-index.html test-rustywind.html
cp test-files/html-templates/play-index.html test-prettier.html

# Run rustywind
rustywind test-rustywind.html --write

# Run prettier
prettier --write test-prettier.html

# Compare
diff test-rustywind.html test-prettier.html
```

### 3. Run Full Comparison

See [comparison-strategy.md](./comparison-strategy.md) for detailed testing methodologies including batch processing scripts.

## File Highlights

### Highest Class Density
- **play-index.html** - 542 class attributes (HTML landing page)
- **notus-landing.js** - 185 className usages (React landing page)
- **tailadmin-notification-dropdown.tsx** - 92 classNames (TypeScript component)

### Best for Edge Case Testing
- **Complex responsive combinations**: `play-index.html`, `notus-landing.js`
- **State modifiers** (hover, focus): `notus-sidebar.js`, navigation components
- **Dark mode classes**: `ThemeSwitch.tsx`, blog layouts
- **Multiple frameworks**: HTML, JSX, TSX, Vue files included

### Recommended Quick Test Set (5 files)
1. `html-templates/play-index.html` - High density HTML
2. `landing-pages/notus-landing.js` - High density JSX
3. `react-components/tailadmin-notification-dropdown.tsx` - High density TSX
4. `vue-components/FwbPagination.vue` - Vue component
5. `complex-layouts/PostLayout.tsx` - Medium complexity

## File Statistics

- **Total Files**: 50
- **File Types**: HTML (4), JS/JSX (14), TS/TSX (28), Vue (4)
- **Source Repositories**: 5 (all MIT licensed)
- **Total Lines**: ~10,000+ lines of code with Tailwind classes
- **Class Attributes**: 1000+ className/class usages across all files

## Features Tested

These test files cover:
- ✅ Responsive modifiers (sm:, md:, lg:, xl:, 2xl:)
- ✅ State variants (hover:, focus:, active:, disabled:)
- ✅ Dark mode (dark:)
- ✅ Group utilities (group-hover:, group-focus:)
- ✅ Arbitrary values ([value])
- ✅ Important modifier (!)
- ✅ Negative values (-m-4, -mt-2)
- ✅ Complex nested combinations
- ✅ Different file types (.html, .js, .jsx, .tsx, .vue)

## License & Attribution

All test files are sourced from MIT-licensed open source projects. See [sources.md](./sources.md) for complete attribution and links to original repositories.

**Source Projects:**
1. Notus Next.js (Creative Tim)
2. TailAdmin (TailAdmin)
3. Tailwind Next.js Starter Blog (timlrx)
4. Play (TailGrids)
5. Flowbite Vue (Themesberg)

These files are collected for testing and comparison purposes only.

## Research Context

This test file collection is part of a research project comparing Tailwind CSS class sorting tools. The goal is to ensure consistent behavior between rustywind and Prettier's Tailwind plugin, allowing developers to:

- Choose the right tool for their workflow
- Use both tools interchangeably across projects
- Migrate between tools with confidence
- Report and fix any sorting discrepancies

## Contributing

If you find additional edge cases or interesting files with complex Tailwind usage, contributions are welcome. Please ensure:
- Files are from MIT or similarly permissive licenses
- Proper attribution is provided
- Files add unique test value (new patterns, modifiers, or edge cases)

## Related Links

- **rustywind**: https://github.com/avencera/rustywind
- **prettier-plugin-tailwindcss**: https://github.com/tailwindlabs/prettier-plugin-tailwindcss
- **Tailwind CSS**: https://tailwindcss.com/

## Collection Date

Files collected: November 10, 2025

---

**Happy Testing!** 🎨✨

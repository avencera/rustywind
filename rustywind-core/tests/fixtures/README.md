# Test Fixtures for Tailwind CSS Class Extraction

This directory contains CSS fixtures for testing the Tailwind CSS class extractor.

## Files

### tailwind.css
- **Version**: Tailwind CSS v3.1.4
- **Size**: 2,266 lines
- **Classes**: 304 unique utility classes
- **Source**: Official Tailwind CSS v3.1.4 output

### tailwind-v4.css
- **Version**: Tailwind CSS v4.0 style
- **Size**: 759 lines
- **Classes**: 152 unique utility classes
- **Features**:
  - Responsive breakpoints (sm, md, lg, xl, 2xl)
  - State variants (hover, focus, active, disabled, checked)
  - Group variants (group-hover)
  - Dark mode support
  - Arbitrary value classes (v4 feature: `w-[500px]`, `bg-[#1da1f2]`)
  - Fractional widths (`w-1/2`, `w-1/3`, etc.)
  - Negative margins (`-mt-4`, `-ml-2`)
  - Complex utility classes with escaped characters

## Verifying Class Counts

The tests in `rustywind-core/src/sorter.rs` assert the counts above. The module doc of `rustywind-core/src/css_class_order.rs` states the rule
that picks one class from each selector.

## Test Coverage

The test suite (`src/sorter.rs`) verifies:

1. **Total class count** - Ensures all classes are extracted
2. **Escaped characters** - Verifies `\.` `\/` `\:` `\[` `\]` are unescaped correctly
3. **Order preservation** - First class should be index 0
4. **Specific classes** - Tests for:
   - Core utilities (container, flex, grid, hidden)
   - Responsive variants (sm:, md:, lg:, xl:, 2xl:)
   - State variants (hover:, focus:, active:, etc.)
   - Dark mode (dark:)
   - Arbitrary values (`w-[500px]`, etc.)
   - Fractional values (`w-1/2`, etc.)
   - Negative values (`-mt-4`, etc.)

## CSS Escape Sequences

Note: CSS escape sequences are handled by the extractor:
- `\32xl\:block` → `2xl:block` (CSS escape `\32` is the digit '2')
- `\.mr-0\.5` → `mr-0.5`

The extractor decodes the escapes as a CSS parser does. It does not only strip the backslashes.

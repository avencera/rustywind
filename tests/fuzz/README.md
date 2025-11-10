# RustyWind Fuzz Testing

This directory contains fuzz tests that compare RustyWind's class sorting output with Prettier's `prettier-plugin-tailwindcss` output.

## Running Tests

```bash
# Run all tests (fuzz + pattern-based + real-world)
npm test

# Individual test suites:
npm run test:fuzz          # Random class combinations
npm run test:patterns      # Pattern-based generation using real-world data
npm run test:real-world    # Actual classes from project files
npm run test:with-legacy   # Fuzz test without legacy v3 classes
```

## Test Results

TODO

**Recommendation**: Use default mode (with legacy) since:

1. Prettier still supports v3 legacy classes
2. RustyWind handles them correctly (better pass rate with them included)
3. Real-world projects may still use v3 classes during migration

## Legacy Classes

Legacy classes are Tailwind v3 utilities that have been replaced in v4:

### Color Opacity Utilities (Deprecated in v4)

- `bg-opacity-*` → Use `bg-color/opacity` syntax (e.g., `bg-blue-500/50`)
- `text-opacity-*` → Use `text-color/opacity` syntax
- `border-opacity-*` → Use `border-color/opacity` syntax
- `divide-opacity-*` → Use `divide-color/opacity` syntax
- `ring-opacity-*` → Use `ring-color/opacity` syntax
- `placeholder-opacity-*` → Use `placeholder-color/opacity` syntax

These are filtered when using `FILTER_LEGACY=true` environment variable.

## Configuration

Edit `compare.js` to adjust:

```javascript
const NUM_TESTS = 100; // Number of random test cases
const MIN_CLASSES = 5; // Minimum classes per test
const MAX_CLASSES = 30; // Maximum classes per test
const VARIANT_PROBABILITY = 0.3; // Chance of adding variants
const FILTER_LEGACY = process.env.FILTER_LEGACY !== "false";
```

## Real-World Class Test

In addition to fuzz testing with random combinations, we also test against real-world class strings extracted from actual open-source projects.

```bash
npm run test:real-world
```

This test:

1. Extracts all class/className attributes from 50 real project files in `../tailwind-sorting-test-files/`
2. Tests each unique class combination against both RustyWind and Prettier
3. Reports failures with specific examples from real code

**⚠️ IMPORTANT**: The test files in `tailwind-sorting-test-files/` are READ-ONLY reference data and should **NEVER** be modified to make tests pass. These files represent real-world usage patterns and must remain pristine.

### Current Real-World Test Results

- **814 unique class combinations** extracted from 50 files
- **~53% pass rate** (434/814)
- **365 unique failure patterns** identified

This test is valuable for catching issues that might not appear in random fuzz tests but occur in real-world usage.

## Failure-Focused Pattern Generation

The `test:patterns` suite uses a unique approach:

1. **Analyze Failures**: Runs real-world tests and extracts patterns from FAILING cases
2. **Generate from Failures**: Creates new test cases using:
   - Classes that appear in failures (70% probability)
   - Modifiers from failing cases (dark:, hover:, lg:, etc.)
   - Class pairs that fail together
   - Class counts typical of failures (avg 7.5 vs overall avg 5)
3. **Stress Test**: Targets problematic patterns - 85% failure rate is expected!

This is a **stress test by design**. It generates combinations that are known to be problematic in real-world code, helping catch issues early that would otherwise only surface when processing actual project files.

To regenerate failure patterns:

```bash
node extract-failure-patterns.mjs
```

## Files

- `compare.js` - Random fuzz test runner
- `compare-real-world-patterns.js` - Failure-focused fuzz test (generates from failing patterns)
- `test-real-world-classes.mjs` - Real-world class extraction and testing
- `extract-real-world-patterns.mjs` - Analyzes real files to extract common patterns
- `extract-failure-patterns.mjs` - Extracts patterns from FAILING test cases
- `common-patterns.json` - General patterns from real files (generated)
- `failure-patterns.json` - Patterns from failing cases (generated)
- `tailwind-classes.js` - Comprehensive list of Tailwind utilities
- `legacy-classes.js` - List of v3 legacy classes with filtering utilities
- `package.json` - Dependencies and scripts
- `../tailwind-sorting-test-files/` - Real-world test files (READ-ONLY)

## Understanding Failures

Failures indicate where RustyWind's sort order differs from Prettier's canonical Tailwind ordering. Common causes:

1. **Variant ordering** - Different priority for variants like `hover:`, `focus:`, etc.
2. **Value-based sorting** - Numeric values not sorted (e.g., `scale-50` vs `scale-110`)
3. **Property mapping** - Utilities mapping to incorrect CSS properties
4. **Edge cases** - Complex combinations or lesser-used utilities

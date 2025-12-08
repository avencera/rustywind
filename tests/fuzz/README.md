# Fuzz Testing Infrastructure

This directory contains the fuzz testing infrastructure for RustyWind, designed to validate class sorting compatibility with Prettier's Tailwind CSS plugin.

## Overview

RustyWind aims for high compatibility with Prettier's Tailwind CSS class sorting. The fuzz tests in this directory generate random combinations of Tailwind classes and compare RustyWind's sorting against Prettier's reference implementation.

**Current Pass Rate: ~96%** (see [docs/NEXT.md](docs/NEXT.md) for details)

## Directory Structure

```
tests/fuzz/
├── docs/              # Documentation about sorting behavior and analysis
├── tools/             # Utility scripts for analyzing test results
├── test-*.{js,mjs}    # Individual test files for specific scenarios
├── *.js               # Core test utilities and class definitions
└── package.json       # Node.js dependencies
```

## Core Files

### Class Definitions & Utilities

- **`tailwind-classes.js`** - Comprehensive list of Tailwind CSS utility classes organized by category
- **`legacy-classes.js`** - Legacy Tailwind classes for backwards compatibility testing
- **`compare.js`** - Core comparison logic between RustyWind and Prettier
- **`compare-real-world-patterns.js`** - Tests using real-world class combinations

### Test Runners

- **`run-multiple-seeds.js`** - Run fuzz tests with multiple random seeds and aggregate results
- **`run-multiple.mjs`** - Run multiple test rounds and collect statistics
- **`run-baseline-test.sh`** - Shell script for baseline testing

## Tools Directory

The `tools/` directory contains analysis utilities:

### Analysis Tools

- **`analyze_failures.py`** - Detailed categorization of test failures by utility type (shadow, ring, outline, etc.)
  - Reads from: `fuzz_failures_detailed.txt`
  - Outputs: Category pairs, specific class pairs, example failures

- **`analyze-failures.js`** - Analyzes failure patterns from multi-seed JSON results
  - Reads from: `multi-seed-results.json`
  - Outputs: `failure-analysis.json` with categorized patterns

- **`collect_failures.py`** - Runs multiple test rounds and collects aggregate failure statistics
  - Runs 20 test iterations by default
  - Outputs: `failure_analysis.txt` with category and specific pair frequencies

- **`test_many_rounds.py`** - Runs N rounds of fuzz tests and reports aggregate pass rates
  - Usage: `python test_many_rounds.py [num_rounds]`
  - Shows distribution of pass rates across rounds

## Test Files

### Specific Feature Tests

Individual test files focus on specific Tailwind features or edge cases:

#### Transform & Animation Tests
- `test-transforms.js` - Transform utility ordering (scale, rotate, skew, translate)
- `test_transform.js` - Skew transform specific tests
- `test-rotation-ordering.rs` - Rotation value ordering tests (in Rust test suite)

#### Border & Outline Tests
- `test-outline.mjs` - Outline utility tests
- `test-outline-transition.mjs` - Outline vs transition ordering
- `test-rounded-ordering.rs` - Border radius ordering (in Rust test suite)

#### Layout & Spacing Tests
- `test-spacing.js` - Spacing utility tests
- `test-space-debug.js` - Space utility debugging
- `test-snap-space.mjs` - Snap and space utility ordering
- `test-divide.mjs`, `test-divide-detailed.mjs` - Divide utility tests
- `test-self-divide.mjs` - Self vs divide ordering

#### Color & Opacity Tests
- `test-color-order.mjs` - Color utility ordering
- `test-opacity-recognition.js` - Opacity syntax recognition
- `test-opacity-slash.js` - Slash opacity syntax tests
- `test-bg-opacity.rs` - Background opacity tests (in Rust test suite)

#### Ring & Shadow Tests
- `test-ring-blur.mjs` - Ring and blur utility ordering
- `test-ring-shadow-ordering.rs` - Ring vs shadow ordering (in Rust test suite)

#### Variant Tests
- `test-variant.js` - Variant stacking and ordering
- `test-dark-placeholder.mjs` - Dark mode + placeholder variant combination
- `test-none-*.mjs` - Various "none" value tests (detailed, patterns, summary, visualization)

#### Utility Category Tests
- `test-ordering.js`, `test-ordering2.js`, `test_ordering.js` - General utility ordering
- `test-comprehensive-order.mjs` - Comprehensive ordering across all utility types
- `test-exact-position.mjs` - Exact position verification
- `test-property-mapping.mjs` - CSS property to utility mapping
- `test-reverse-order.mjs` - Reverse order testing
- `test-simple.js` - Simple baseline tests
- `test-size.js` - Size utility tests
- `test-specific.js` - Specific edge case tests
- `test-problematic.js` - Known problematic patterns

### Analysis & Extraction Tools

- `extract-failure-patterns.mjs` - Extract common failure patterns from test results
- `extract-real-world-patterns.mjs` - Extract patterns from real-world codebases
- `extract-variant-order-runtime.mjs` - Extract variant order from Tailwind runtime

### Verification Tools

- `verify-transforms.js` - Verify transform utility ordering
- `verify_prettier.mjs` - Verify Prettier plugin behavior
- `check-prettier.mjs` - Check Prettier formatting
- `check_all_combos.mjs` - Check all utility combinations

### Property Analysis

- `analyze-properties.mjs` - Analyze CSS property ordering
- `compare-properties.mjs` - Compare property ordering between tools

## Documentation Directory

The `docs/` directory contains in-depth analysis:

- **`NEXT.md`** - Current status, failure categorization, and next steps
  - Contains 100-round analysis showing 96.03% pass rate
  - Detailed failure category breakdown
  - Recommendations for reaching 97-98% pass rate

- **`HOW_TAILWIND_SORTS.md`** - Deep dive into Tailwind's sorting algorithm
  - Explains variant order calculation using bitwise OR
  - Details compound variant handling
  - Provides examples of multi-variant sorting

- **`TAILWIND_SOURCE_ANALYSIS.md`** - Analysis of Tailwind CSS source code
  - Documents variant order calculation from `compile.ts`
  - Explains variant comparison logic from `variants.ts`
  - Clarifies how variants are parsed and compared

## Running Tests

### Quick Test

Run the default fuzz test:

```bash
cd tests/fuzz
npm test
```

### Multiple Rounds

Run 100 rounds to get comprehensive statistics:

```bash
cd tests/fuzz
python tools/test_many_rounds.py 100
```

### Collect Failures

Run 20 rounds and analyze failure patterns:

```bash
cd tests/fuzz
python tools/collect_failures.py
```

### Analyze Multi-Seed Results

After running multi-seed tests:

```bash
cd tests/fuzz
node run-multiple-seeds.js
node tools/analyze-failures.js
```

## Understanding Results

### Pass Rate Metrics

- **90-100%**: Excellent - Normal range for random fuzz tests
- **80-89%**: Good - Some edge cases need attention
- **<80%**: Needs investigation - Systematic issues likely present

### Failure Categories

Failures are categorized by utility type pairs:

- **Property ordering** - General utility ordering edge cases
- **Filter vs Ring** - Filter utilities sorting against ring utilities
- **Arbitrary values** - Arbitrary value syntax (`[...]`) sorting issues
- **Opacity syntax** - Slash opacity (`/`) syntax issues
- **Ring vs Shadow** - Ring and shadow utility ordering
- **Others** - Various edge cases and one-off patterns

See [docs/NEXT.md](docs/NEXT.md) for detailed breakdown.

## Key Findings

From 100 rounds (10,000 tests):

1. **96.03% pass rate** - Highly compatible with Prettier
2. **Consistent results** - 99/100 rounds achieved 90%+ pass rate
3. **Diverse failures** - Remaining 4% spread across many edge cases, not systematic issues
4. **Main issues**:
   - Filter utility ordering relative to rings
   - Some arbitrary value edge cases with borders
   - Ring vs shadow ordering
   - Property order table gaps

## Next Steps

To improve pass rate further:

1. **Ring vs Shadow** - Ensure ring utilities sort after shadow utilities
2. **Filter utilities** - Add special handling for filter utilities relative to rings
3. **Arbitrary borders** - Fix `border-[1.5px]` vs `border-t-0` edge cases
4. **Property order** - Comprehensive property order table updates matching Tailwind v4

See [docs/NEXT.md](docs/NEXT.md) for detailed recommendations.

## Related Test Suites

RustyWind also has extensive Rust-based integration tests:

- `rustywind-core/tests/fuzz_regression_tests.rs` - Regression tests from fuzz findings
- `rustywind-core/tests/test_*.rs` - Category-specific integration tests
- `rustywind-core/tests/integration_tests.rs` - General integration tests

## Contributing

When adding new test cases:

1. Add specific test files in `tests/fuzz/test-*.{js,mjs}`
2. Update class lists in `tailwind-classes.js` if needed
3. Document findings in `docs/` directory
4. Add regression tests to `rustywind-core/tests/` for confirmed bugs

## References

- [Prettier Plugin Tailwind CSS](https://github.com/tailwindlabs/prettier-plugin-tailwindcss)
- [Tailwind CSS](https://tailwindcss.com)
- [RustyWind](https://github.com/avencera/rustywind)

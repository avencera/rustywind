# RustyWind Fuzz Testing

This directory contains fuzz tests that compare RustyWind's class sorting output with Prettier's `prettier-plugin-tailwindcss` output.

## Running Tests

```bash
# Default: Tests with all classes (including legacy v3 classes)
npm test

# Test without legacy v3 classes (v4-only)
npm run test:with-legacy
```

## Test Results

| Mode                  | Pass Rate | Notes                                                   |
| --------------------- | --------- | ------------------------------------------------------- |
| With Legacy (default) | 59%       | Includes Tailwind v3 legacy classes like `bg-opacity-*` |
| V4-only (filtered)    | 55%       | Excludes legacy classes, focuses on v4 utilities        |

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

## Files

- `compare.js` - Main fuzz test runner
- `tailwind-classes.js` - Comprehensive list of Tailwind utilities
- `legacy-classes.js` - List of v3 legacy classes with filtering utilities
- `package.json` - Dependencies and scripts

## Test Output

```
🧪 Starting fuzz test with 100 random class combinations...
📋 Class pool: 935 classes (including legacy classes)

.........F..........F....F.......F..F.... 50/100
.........F..........F....F.......F..F.... 100/100

================================================================================

📊 Results: 59 passed, 41 failed (59.0% pass rate)

❌ Failures:
[Detailed failure output with mismatched classes]
```

## Understanding Failures

Failures indicate where RustyWind's sort order differs from Prettier's canonical Tailwind ordering. Common causes:

1. **Variant ordering** - Different priority for variants like `hover:`, `focus:`, etc.
2. **Value-based sorting** - Numeric values not sorted (e.g., `scale-50` vs `scale-110`)
3. **Property mapping** - Utilities mapping to incorrect CSS properties
4. **Edge cases** - Complex combinations or lesser-used utilities

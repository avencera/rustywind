# CSS Class Count Verification

This document explains how to verify the class counts in the test fixtures using PostCSS.

## Verification Script

**Usage:**
```bash
node count-css-classes.mjs tailwind.css
node count-css-classes.mjs tailwind-v4.css
```

Or use npm scripts:
```bash
npm run count:v3    # Count classes in tailwind.css
npm run count:v4    # Count classes in tailwind-v4.css
npm run verify      # Count both
```

## How It Works

The script uses PostCSS to properly parse CSS:
- Parses CSS with `postcss`
- Extracts class selectors using `postcss-selector-parser`
- Removes backslashes from class names (to match Rust behavior)
- Returns unique class count

## Results

- **tailwind.css** (v3.1.4): **304 classes**
- **tailwind-v4.css**: **152 classes**

## Why PostCSS?

PostCSS provides accurate CSS parsing that:
- Correctly handles pseudo-selectors (`:hover`, `:focus`, etc.)
- Interprets CSS escape sequences properly (`\32` → '2')
- Validates against real CSS syntax

## Test Assertions

The Rust tests expect these exact counts:

```rust
// tailwind.css (v3.1.4)
assert_eq!(classes.len(), 305);

// tailwind-v4.css
assert_eq!(classes.len(), 152);
```

**Note:** The v3 test expects 305 classes because the Rust extractor uses a simple regex that includes pseudo-selectors in the class name (e.g., `.active\:bg-blue-700:active` becomes `active:bg-blue-700:active`). PostCSS correctly separates these into class + pseudo-selector, resulting in 304 unique classes.

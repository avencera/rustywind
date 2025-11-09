# RustyWind Fuzz Testing

This directory contains fuzz tests that compare RustyWind's class sorting with Prettier's `prettier-plugin-tailwindcss` to ensure 100% compatibility.

## Setup

```bash
cd tests/fuzz
npm install
```

## Running the Tests

Make sure RustyWind is compiled in release mode:

```bash
cd ../..
cargo build --release
```

Then run the fuzz test:

```bash
cd tests/fuzz
npm test
```

## How It Works

1. **Generate Random Classes**: Creates random combinations of Tailwind CSS classes (5-30 classes per test)
2. **Add Variants**: Randomly adds variants like `hover:`, `md:`, `dark:`, etc. (30% probability)
3. **Sort with Prettier**: Uses Prettier with `prettier-plugin-tailwindcss` to sort the classes
4. **Sort with RustyWind**: Uses RustyWind CLI to sort the same classes
5. **Compare**: Ensures both tools produce identical output

## Test Configuration

- **Number of tests**: 100 random class combinations
- **Classes per test**: 5-30 classes
- **Variant probability**: 30% chance of adding a variant
- **Double variant probability**: 10% chance of adding a second variant

## Comprehensive Class Coverage

The test includes all major Tailwind CSS utility categories:

- Layout (display, position, float, overflow, visibility)
- Flexbox & Grid
- Spacing (padding, margin, space-between)
- Sizing (width, height, min/max)
- Typography (font, text, line-height, list)
- Backgrounds (color, image, position, size)
- Borders (width, color, style, radius, divide, outline, ring)
- Effects (shadow, opacity, blend modes)
- Filters (blur, brightness, contrast, grayscale, etc.)
- Backdrop Filters
- Transforms (scale, rotate, translate, skew)
- Interactivity (cursor, pointer-events, user-select, scroll-snap)
- Transitions & Animations
- Additional (aspect-ratio, columns, accent, caret, etc.)

## Output

The test will show:
- `.` for each passing test
- `F` for each failing test
- `E` for each error

At the end, it displays:
- Pass rate percentage
- Detailed failure information including original classes and both sorted outputs

## Debugging Failures

If tests fail, the output will show:
1. Which test failed (test number)
2. The original unsorted classes
3. Prettier's sorted output
4. RustyWind's sorted output
5. The specific difference (position and class name)

Use this information to identify missing utilities or incorrect sorting behavior in RustyWind.

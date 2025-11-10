# Comparison Strategy: rustywind vs Prettier Tailwind CSS Plugin

This document provides a guide for comparing class sorting behavior between rustywind CLI and the Prettier Tailwind CSS plugin.

## Tools Being Compared

### rustywind
- **Description**: A CLI tool that sorts Tailwind CSS classes
- **Written in**: Rust
- **GitHub**: https://github.com/avencera/rustywind
- **Installation**: `cargo install rustywind` or `npm install -g rustywind`

### Prettier Plugin - Tailwind CSS
- **Description**: Official Prettier plugin for sorting Tailwind CSS classes
- **GitHub**: https://github.com/tailwindlabs/prettier-plugin-tailwindcss
- **Installation**: `npm install -D prettier prettier-plugin-tailwindcss`

## Setup Instructions

### 1. Install Both Tools

```bash
# Install rustywind
npm install -g rustywind
# or
cargo install rustywind

# Install Prettier + Tailwind plugin
npm install -D prettier prettier-plugin-tailwindcss
```

### 2. Configure Prettier

Create a `.prettierrc` file:

```json
{
  "plugins": ["prettier-plugin-tailwindcss"]
}
```

## Comparison Methodology

### Option 1: Manual File-by-File Comparison

For each test file:

```bash
# Create a copy for testing
cp original-file.tsx test-file-rustywind.tsx
cp original-file.tsx test-file-prettier.tsx

# Run rustywind
rustywind test-file-rustywind.tsx --write

# Run prettier
prettier --write test-file-prettier.tsx

# Compare the results
diff test-file-rustywind.tsx test-file-prettier.tsx
```

### Option 2: Batch Processing Script

Create a test script `compare-all.sh`:

```bash
#!/bin/bash

# Create output directories
mkdir -p ./results/rustywind
mkdir -p ./results/prettier
mkdir -p ./results/diffs

# Process all test files
find ./test-files -type f \( -name "*.html" -o -name "*.jsx" -o -name "*.js" -o -name "*.tsx" -o -name "*.vue" \) | while read file; do
    filename=$(basename "$file")

    # Copy for rustywind
    cp "$file" "./results/rustywind/$filename"

    # Copy for prettier
    cp "$file" "./results/prettier/$filename"

    # Run rustywind
    rustywind "./results/rustywind/$filename" --write

    # Run prettier
    prettier --write "./results/prettier/$filename"

    # Generate diff
    diff "./results/rustywind/$filename" "./results/prettier/$filename" > "./results/diffs/$filename.diff"

    if [ -s "./results/diffs/$filename.diff" ]; then
        echo "DIFFERENCE FOUND: $filename"
    else
        echo "MATCH: $filename"
    fi
done
```

### Option 3: Automated Test Suite

Create a Node.js test script `test-comparison.js`:

```javascript
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const testFilesDir = './test-files';
const resultsDir = './results';

// Get all test files
const testFiles = [];
const walk = (dir) => {
  const files = fs.readdirSync(dir);
  files.forEach(file => {
    const filePath = path.join(dir, file);
    if (fs.statSync(filePath).isDirectory()) {
      walk(filePath);
    } else if (/\.(html|jsx?|tsx?|vue)$/.test(file)) {
      testFiles.push(filePath);
    }
  });
};
walk(testFilesDir);

console.log(`Found ${testFiles.length} test files`);

const results = {
  identical: [],
  different: []
};

testFiles.forEach(file => {
  const filename = path.basename(file);
  const rustywinded = path.join(resultsDir, 'rustywind', filename);
  const prettified = path.join(resultsDir, 'prettier', filename);

  // Copy files
  fs.copyFileSync(file, rustywinded);
  fs.copyFileSync(file, prettified);

  // Run tools
  try {
    execSync(`rustywind ${rustywinded} --write`, { stdio: 'ignore' });
    execSync(`prettier --write ${prettified}`, { stdio: 'ignore' });

    // Compare
    const rustywinded_content = fs.readFileSync(rustywinded, 'utf8');
    const prettified_content = fs.readFileSync(prettified, 'utf8');

    if (rustywinded_content === prettified_content) {
      results.identical.push(filename);
      console.log(`✓ ${filename}`);
    } else {
      results.different.push(filename);
      console.log(`✗ ${filename} - DIFFERENCES FOUND`);
    }
  } catch (error) {
    console.error(`Error processing ${filename}:`, error.message);
  }
});

console.log('\n--- RESULTS ---');
console.log(`Identical: ${results.identical.length}/${testFiles.length}`);
console.log(`Different: ${results.different.length}/${testFiles.length}`);

if (results.different.length > 0) {
  console.log('\nFiles with differences:');
  results.different.forEach(f => console.log(`  - ${f}`));
}

// Write results to JSON
fs.writeFileSync(
  path.join(resultsDir, 'comparison-results.json'),
  JSON.stringify(results, null, 2)
);
```

## What to Look For

### Class Order Differences
Check if the tools sort classes in the same order:
- Layout (display, position, etc.)
- Sizing (width, height, etc.)
- Spacing (margin, padding)
- Typography
- Visual (background, border, etc.)
- Effects and transforms

### Responsive Modifier Handling
How do they handle:
- `sm:`, `md:`, `lg:`, `xl:`, `2xl:`
- Multiple responsive variants of the same utility

### State Modifier Handling
How do they handle:
- `hover:`, `focus:`, `active:`, `disabled:`
- `group-hover:`, `group-focus:`
- Combined modifiers like `hover:dark:bg-blue-500`

### Dark Mode Classes
Order of `dark:` prefixed classes relative to regular classes

### Arbitrary Values
Handling of `[arbitrary-value]` syntax

### Important Modifier
Placement of `!important` or `!` prefix

### Negative Values
Handling of negative utilities like `-m-4`, `-mt-2`

### Custom Classes
How they handle non-Tailwind classes (should remain in original position)

## Expected Differences

Based on the design of each tool, you might find differences in:

1. **Unknown class handling**: How each tool handles classes it doesn't recognize
2. **Sorting algorithms**: Subtle differences in the underlying sorting logic
3. **Edge cases**: Handling of malformed or unusual class combinations
4. **Version differences**: Different Tailwind CSS version support

## Reporting Issues

If you find differences:

1. **Document the file** where the difference occurs
2. **Extract the specific class string** that differs
3. **Show both outputs** side by side
4. **Determine which is correct** according to Tailwind CSS official docs
5. **File an issue** with the tool that has incorrect behavior

## Recommended Test Order

1. **Start with simple files** (Tag.tsx, simple components)
2. **Move to medium complexity** (PostLayout.tsx, card components)
3. **Test high-density files** (play-index.html, notus-landing.js)
4. **Test framework-specific files** (Vue components)
5. **Test edge cases** (files with arbitrary values, complex modifiers)

## Performance Comparison

Also consider measuring:
- **Speed**: Time to process all 50 files
- **Memory usage**: Resource consumption
- **File size impact**: Do sorted classes compress better?

```bash
# Measure rustywind speed
time rustywind ./test-files/**/*.{html,js,jsx,tsx,vue} --write

# Measure prettier speed
time prettier --write "./test-files/**/*.{html,js,jsx,tsx,vue}"
```

## Continuous Testing

Consider setting up a test suite that runs both tools on:
- New Tailwind CSS releases
- New tool versions
- Additional test files from real projects

This ensures ongoing compatibility and helps catch regressions.

# Test Fixtures for Tailwind CSS Class Extraction

This directory contains CSS fixtures for testing the Tailwind CSS class extractor.

## Files

### tailwind.css
- **Version**: Tailwind CSS v3.1.4
- **Size**: 2,266 lines
- **Classes**: 305 unique utility classes
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

To verify the number of classes extracted from each fixture, use this Python script:

```python
#!/usr/bin/env python3
import re
import sys

# Same regex pattern as Rust code: r"^\s*(\.[^\s]+)[ ]"
pattern = re.compile(r'^\s*(\.[^\s]+)[ ]')

classes = set()
with open(sys.argv[1], 'r') as f:
    for line in f:
        match = pattern.search(line)
        if match:
            # Extract class name, remove leading dot and backslashes (like Rust does)
            class_name = match.group(1)[1:].replace('\\', '')
            classes.add(class_name)

print(f"Total unique classes: {len(classes)}")
```

**Usage:**
```bash
python3 count_classes.py tailwind.css    # Output: 305
python3 count_classes.py tailwind-v4.css # Output: 152
```

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
- `\32xl\:block` → `32xl:block` (CSS escape `\32` for digit '2' becomes '32')
- `\.mr-0\.5` → `mr-0.5`
- `\:hover\:bg-blue` → `:hover:bg-blue` → `hover:bg-blue`

This is correct behavior - the backslashes are stripped during extraction.

#!/usr/bin/env node
/**
 * Extract Tailwind's variant order by testing with Prettier plugin
 */

import prettier from 'prettier';

// List of all known Tailwind variants to test
const KNOWN_VARIANTS = [
  // Pseudo-elements
  'before', 'after', 'first-line', 'first-letter', 'placeholder', 'file', 'marker', 'selection', 'backdrop',

  // Positional
  'first', 'last', 'only', 'odd', 'even', 'first-of-type', 'last-of-type', 'only-of-type',

  // State
  'visited', 'target', 'open', 'default', 'checked', 'indeterminate',
  'placeholder-shown', 'autofill', 'optional', 'required', 'valid', 'invalid',
  'in-range', 'out-of-range', 'read-only', 'read-write', 'empty',

  // Interactive
  'focus-within', 'hover', 'focus', 'focus-visible', 'active',

  // Enabled/Disabled
  'enabled', 'disabled',

  // Group/Peer
  'group', 'peer',

  // Breakpoints
  'sm', 'md', 'lg', 'xl', '2xl',

  // Print
  'print',

  // Dark mode
  'dark',

  // RTL/LTR
  'rtl', 'ltr',

  // Orientation
  'portrait', 'landscape',

  // Motion
  'motion-safe', 'motion-reduce',

  // Contrast
  'contrast-more', 'contrast-less',
];

async function testVariantOrder() {
  console.log('Testing variant order with Prettier...\n');

  // Create test HTML with all variants on the same base class
  // Prettier will sort them in the correct order
  const variantClasses = KNOWN_VARIANTS.map(v => `${v}:p-4`);
  const html = `<div class="${variantClasses.join(' ')}"></div>`;

  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });

  const match = formatted.match(/class="([^"]*)"/);
  if (!match) {
    console.error('Failed to parse Prettier output');
    return;
  }

  const sorted = match[1].split(' ');

  console.log('| Index | Variant |');
  console.log('|-------|---------|');
  sorted.forEach((cls, i) => {
    const variant = cls.split(':')[0];
    console.log(`| ${i} | \`${variant}\` |`);
  });

  console.log(`\n\nTotal: ${sorted.length} variants sorted`);

  // Extract just the variant names
  const variantOrder = sorted.map(cls => cls.split(':')[0]);

  console.log('\n\n// Rust array format:');
  console.log('pub const VARIANT_ORDER: &[&str] = &[');
  variantOrder.forEach((v, i) => {
    console.log(`    "${v}",${i % 5 === 4 ? ' //' + (i-3) + '-' + i : ''}`);
  });
  console.log('];');
}

testVariantOrder().catch(console.error);

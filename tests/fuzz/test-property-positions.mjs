#!/usr/bin/env node

/**
 * Test to find where Prettier places utilities with missing properties
 */

import prettier from 'prettier';
import { execSync } from 'child_process';

async function sortWithPrettier(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;

  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });

  const match = formatted.match(/class="([^"]*)"/);
  if (!match) {
    throw new Error('Could not extract classes');
  }

  return match[1].split(/\s+/).filter(c => c.length > 0);
}

function sortWithRustyWind(classes) {
  const input = classes.join(' ');
  try {
    const result = execSync(
      `echo '${input}' | /home/user/rustywind/target/release/rustywind --stdin`,
      { encoding: 'utf8' }
    ).trim();
    return result.split(/\s+/).filter(c => c.length > 0);
  } catch (error) {
    console.error('RustyWind error:', error.message);
    return [];
  }
}

const testCases = [
  {
    name: 'outline-style utilities (outline-solid, outline-dashed, outline-dotted)',
    classes: ['p-4', 'm-4', 'bg-blue-500', 'outline-solid', 'outline-dashed', 'outline-dotted', 'shadow-lg', 'ring-2', 'outline-1', 'outline-offset-2'],
  },
  {
    name: 'select utilities (user-select property)',
    classes: ['p-4', 'm-4', 'select-none', 'select-text', 'select-all', 'select-auto', 'opacity-50', 'cursor-pointer'],
  },
  {
    name: 'ring-inset (--tw-ring-inset property)',
    classes: ['p-4', 'm-4', 'ring-inset', 'ring-2', 'ring-blue-500', 'shadow-lg', 'outline-1', 'filter', 'blur-sm'],
  },
  {
    name: 'divide-x-reverse (--tw-divide-x-reverse property)',
    classes: ['p-4', 'm-4', 'divide-x-reverse', 'divide-x-2', 'space-x-4', 'gap-4', 'divide-y-reverse'],
  },
  {
    name: 'Mixed test with all problematic utilities',
    classes: [
      'pointer-events-auto',
      'mx-4',
      'z-10',
      'select-text',
      'outline-none',
      'ring-inset',
      'divide-x-reverse',
      'shadow-lg',
      'filter',
      'blur-sm',
    ],
  },
];

console.log('🔍 Testing Property Positions\n');
console.log('='.repeat(80));

for (const test of testCases) {
  console.log(`\nTest: ${test.name}`);
  console.log('-'.repeat(80));

  try {
    const prettierSorted = await sortWithPrettier(test.classes);
    const rustywindSorted = sortWithRustyWind(test.classes);

    console.log(`\nOriginal:  ${test.classes.join(' ')}`);
    console.log(`Prettier:  ${prettierSorted.join(' ')}`);
    console.log(`RustyWind: ${rustywindSorted.join(' ')}`);

    if (prettierSorted.join(' ') === rustywindSorted.join(' ')) {
      console.log('\n✅ MATCH');
    } else {
      console.log('\n❌ MISMATCH');

      // Find first difference
      for (let i = 0; i < Math.max(prettierSorted.length, rustywindSorted.length); i++) {
        const p = prettierSorted[i] || '(missing)';
        const r = rustywindSorted[i] || '(missing)';
        if (p !== r) {
          console.log(`\n  First difference at position ${i}:`);
          console.log(`    Prettier:  ${p}`);
          console.log(`    RustyWind: ${r}`);
          break;
        }
      }

      // Show position of problematic classes in prettier output
      const problematic = ['outline-solid', 'outline-dashed', 'outline-dotted', 'outline-none',
                          'select-none', 'select-text', 'select-all', 'select-auto',
                          'ring-inset', 'divide-x-reverse'];

      console.log('\n  Positions in Prettier output:');
      problematic.forEach(cls => {
        const idx = prettierSorted.indexOf(cls);
        if (idx !== -1) {
          const before = prettierSorted[idx - 1] || '(start)';
          const after = prettierSorted[idx + 1] || '(end)';
          console.log(`    ${cls}: position ${idx}, between [${before}] and [${after}]`);
        }
      });
    }

    console.log('\n' + '='.repeat(80));
  } catch (error) {
    console.error(`Error: ${error.message}`);
  }
}

console.log('\n✅ Done!');

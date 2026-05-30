#!/usr/bin/env node
import prettier from 'prettier';

const tests = [
  // Test 1: Same property, different variants
  ['hover:text-sm focus:text-lg', 'Should sort by variant order'],

  // Test 2: Different properties, same variant
  ['group:text-sm group:p-4', 'Should sort by property'],

  // Test 3: Different properties, different variants (Issue 3)
  ['peer:touch-none group:translate-y-4', 'Issue 3 - peer vs group, different props'],

  // Test 4: peer vs group with SAME property
  ['peer:text-sm group:text-lg', 'Same property (font-size)'],

  // Test 5: Compound vs simple variant
  ['even:group:overscroll-x-auto peer:ease-linear', 'Issue 4 - compound vs simple'],

  // Test 6: Both compound
  ['peer:hover:text-sm group:hover:text-lg', 'Both compound, same property'],
];

for (const [input, description] of tests) {
  const html = `<div class="${input}"></div>`;
  const result = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000
  });
  const output = result.match(/class="([^"]*)"/)[1];
  const changed = input !== output ? '  🔄 REORDERED' : '  ✓ unchanged';
  console.log(`${description}:`);
  console.log(`  Input:  ${input}`);
  console.log(`  Output: ${output}${changed}`);
  console.log();
}

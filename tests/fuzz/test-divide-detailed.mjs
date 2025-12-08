import prettier from 'prettier';
import * as prettierPlugin from 'prettier-plugin-tailwindcss';

async function test(classes) {
  const html = `<div class="${classes}"></div>`;
  const sorted = await prettier.format(html, {
    parser: 'html',
    plugins: [prettierPlugin],
  });
  const match = sorted.match(/class="([^"]*)"/);
  return match ? match[1] : classes;
}

// Test to find where divide-x-reverse should be in the property order
const tests = [
  // First, test divide-x-reverse against border radius (comes before border width)
  'divide-x-reverse rounded-lg',
  // Test against all border utilities
  'divide-x-reverse border-radius',
  'divide-x-reverse border-width',
  'divide-x-reverse border-style',
  'divide-x-reverse border-color',
  // Test divide utilities order
  'divide-x divide-y divide-style divide-color divide-x-reverse divide-y-reverse',
];

console.log('Testing divide-x-reverse position:\n');
for (const input of tests) {
  const result = await test(input);
  console.log(`Input:     ${input}`);
  console.log(`Prettier:  ${result}\n`);
}

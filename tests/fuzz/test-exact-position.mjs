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

// Test to find what comes RIGHT AFTER divide-x-reverse
const tests = [
  'divide-x-reverse background-color',
  'divide-x-reverse bg-red-500',
  'divide-x-reverse from-blue-500',
  'divide-x-reverse padding',
  'divide-x-reverse p-4',
  'divide-x-reverse text-left',
  // What comes before?
  'border-gray-500 divide-x-reverse',
  'divide-color divide-x-reverse',
];

console.log('Finding exact position of divide-x-reverse:\n');
for (const input of tests) {
  const result = await test(input);
  const reversed = result !== input ? '✓' : '✗';
  console.log(`${reversed} Input:     ${input}`);
  console.log(`  Prettier:  ${result}\n`);
}

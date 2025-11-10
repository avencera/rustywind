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

// Test the full order comprehensively
const tests = [
  // All elements that should come before divide-x-reverse
  'divide-x-reverse place-self-center align-self overflow-hidden border-radius border-2 border-solid border-gray-500 divide-x divide-y divide-solid divide-gray-500',
  
  // Simpler breakdown
  'overflow-hidden divide-x-reverse',
  'border-2 divide-x-reverse',
  'divide-solid divide-x-reverse',
  'divide-gray-500 divide-x-reverse',
  'divide-x divide-x-reverse',
  'place-self-center divide-x-reverse',
  'align-self divide-x-reverse',
];

console.log('Comprehensive order test:\n');
for (const input of tests) {
  const result = await test(input);
  console.log(`Input:     ${input}`);
  console.log(`Prettier:  ${result}\n`);
}

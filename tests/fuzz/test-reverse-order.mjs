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

// Test to find exact position of divide-x-reverse and divide-y-reverse
const tests = [
  'divide-y-reverse divide-x-reverse',
  'border-color divide-x-width divide-y-width divide-style divide-color divide-x-reverse divide-y-reverse',
  'self-start divide-x-reverse',
  'justify-self divide-x-reverse',
  'place-self divide-x-reverse',
];

console.log('Testing reverse utilities order:\n');
for (const input of tests) {
  const result = await test(input);
  console.log(`Input:     ${input}`);
  console.log(`Prettier:  ${result}\n`);
}

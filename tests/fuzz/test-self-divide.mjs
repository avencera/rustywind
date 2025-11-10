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

// Carefully test the order
const tests = [
  // From the failing test
  'divide-x-reverse self-start self-end self-center',
  // Reversed input
  'self-start self-end self-center divide-x-reverse',
  // Individual tests
  'divide-x-reverse self-start',
  'self-start divide-x-reverse',
  'divide-x-reverse place-self-center',
  'place-self-center divide-x-reverse',
];

console.log('Self vs divide-x-reverse order:\n');
for (const input of tests) {
  const result = await test(input);
  console.log(`Input:     ${input}`);
  console.log(`Result:    ${result}\n`);
}

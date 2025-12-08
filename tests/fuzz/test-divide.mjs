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

const tests = [
  'divide-x-reverse self-start overflow-hidden border-2 divide-solid divide-gray-500',
  'divide-x-reverse self-start self-end self-center',
  'divide-x-reverse overflow-hidden overflow-auto overflow-x-scroll',
  'divide-x-reverse divide-solid divide-dashed divide-dotted divide-double divide-none',
  'divide-x-reverse border border-2 border-t border-solid border-gray-500',
  'divide-x-reverse divide-x-2 divide-y-2 divide-gray-300',
];

console.log('Prettier divide-x-reverse sorting:\n');
for (const input of tests) {
  const result = await test(input);
  console.log(`Input:     ${input}`);
  console.log(`Prettier:  ${result}\n`);
}

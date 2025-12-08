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
  ['h-auto size-2', 'size vs height'],
  ['w-4 size-2', 'size vs width'],
  ['snap-y select-all', 'select vs snap'],
  ['columns-md select-auto', 'select vs columns'],
  ['rounded-br-none rounded-none', 'rounded-none vs rounded-br'],
  ['hue-rotate-30 outline-dashed', 'outline vs hue-rotate'],
  ['drop-shadow-none outline-dashed', 'outline vs drop-shadow'],
  ['sepia-0 delay-75', 'sepia vs delay'],
  ['select-all space-y-1', 'space-y vs select'],
  ['space-y-4 space-x-4', 'space-x vs space-y'],
  ['pt-2 py-0', 'py vs pt'],
  ['border-r-0 border-x-0', 'border-x vs border-r'],
  ['rounded-l-lg divide-x-reverse', 'divide-x-reverse vs rounded'],
  ['row-start-auto bg-opacity-50', 'bg-opacity first'],
];

console.log('Prettier sorting:\n');
for (const [input, name] of tests) {
  const result = await test(input);
  console.log(`${name}:`);
  console.log(`  Input:     ${input}`);
  console.log(`  Prettier:  ${result}`);
  console.log();
}

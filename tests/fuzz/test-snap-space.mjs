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
  ['snap-start space-y-1', 'snap vs space-y'],
  ['snap-x space-x-4', 'snap vs space-x'],
  ['snap-mandatory space-y-2', 'snap-mandatory vs space'],
  ['snap-start select-all', 'snap vs select'],
  ['space-y-1 select-all', 'space vs select'],
];

console.log('Prettier snap/space/select ordering:\n');
for (const [input, name] of tests) {
  const result = await test(input);
  console.log(`${name}:`);
  console.log(`  Input:     ${input}`);
  console.log(`  Prettier:  ${result}`);
  console.log();
}

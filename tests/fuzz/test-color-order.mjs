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
  ['bg-green-50 bg-blue-900', 'Test #27 (green vs blue)'],
  ['bg-red-500 bg-blue-500 bg-green-500', 'RGB colors'],
  ['bg-green-50 bg-blue-900 bg-red-100', 'Different shades'],
  ['text-red-500 text-amber-500 text-zinc-500 text-blue-500', 'Text colors'],
  ['bg-violet-500 bg-indigo-500 bg-blue-500 bg-cyan-500', 'Purple/blue spectrum'],
  ['bg-orange-500 bg-yellow-500 bg-lime-500 bg-teal-500', 'Warm to cool'],
  ['bg-zinc-500 bg-gray-500 bg-slate-500 bg-neutral-500', 'Grays'],
  ['bg-rose-500 bg-pink-500 bg-fuchsia-500 bg-purple-500', 'Pink/purple spectrum'],
  ['bg-red-100 bg-red-500 bg-red-900', 'Same color, different shades'],
  ['bg-blue-50 bg-blue-100 bg-blue-500 bg-blue-900', 'Blue shades'],
];

console.log('Color Ordering Tests:\n');
for (const [input, name] of tests) {
  const result = await test(input);
  const changed = input !== result ? '✓ CHANGED' : '  (unchanged)';
  console.log(`${name} ${changed}:`);
  console.log(`  Input:    ${input}`);
  console.log(`  Prettier: ${result}`);
  console.log();
}

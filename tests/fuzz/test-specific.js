/**
 * Test specific class orderings
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import prettier from 'prettier';

const execAsync = promisify(exec);

async function sortWithRustyWind(classes) {
  const { stdout } = await execAsync(
    `echo "${classes}" | /home/user/rustywind/target/release/rustywind --stdin`
  );
  return stdout.trim();
}

async function sortWithPrettier(classes) {
  const formatted = await prettier.format(`<div class="${classes}"></div>`, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
  });
  const match = formatted.match(/class="([^"]*)"/);
  return match ? match[1] : '';
}

async function test(name, classes) {
  console.log(`\n${name}:`);
  console.log(`Input:     ${classes}`);

  const prettier = await sortWithPrettier(classes);
  const rustywind = await sortWithRustyWind(classes);

  console.log(`Prettier:  ${prettier}`);
  console.log(`RustyWind: ${rustywind}`);
  console.log(`Match: ${prettier === rustywind ? '✓' : '✗'}`);
}

// Run tests
(async () => {
  await test('Transforms', '-translate-y-1 -skew-y-1 scale-y-50');
  await test('Width', 'min-w-min max-w-xl');
  await test('Break + Padding', 'break-all px-4');
  await test('Space + Touch', 'space-x-2 touch-pan-down');
})();

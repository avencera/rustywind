/**
 * Test variant ordering
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
  await test('Variant vs Base', 'divide-transparent ring-inset empty:rounded-full');
  await test('Another variant test', 'min-h-max max-w-0 empty:min-h-min');
})();

import { execSync } from 'child_process';
import prettier from 'prettier';

async function testWithBoth(classes) {
  // Test with Prettier
  const html = `<div class="${classes.join(' ')}"></div>`;
  const prettierResult = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const prettierOrder = prettierResult.match(/class="([^"]*)"/)[1];

  // Test with RustyWind
  const htmlInput = `<div class="${classes.join(' ')}"></div>`;
  const rustywindResult = execSync(
    `echo '${htmlInput}' | /home/user/rustywind/target/release/rustywind --stdin`,
    { encoding: 'utf-8' }
  ).trim();
  const rustywindOrder = rustywindResult.match(/class="([^"]*)"/)?.[1] || rustywindResult;

  const match = prettierOrder === rustywindOrder;
  console.log(`Input:     ${classes.join(' ')}`);
  console.log(`Prettier:  ${prettierOrder}`);
  console.log(`RustyWind: ${rustywindOrder}`);
  console.log(`Match: ${match ? '✓' : '✗'}`);
  console.log('');

  return match;
}

async function runTests() {
  console.log('Testing transform ordering:\n');

  const tests = [
    ['translate-x-0', '-rotate-1', 'skew-x-6', 'scale-x-100'],
    ['scale-100', 'rotate-0', 'skew-x-0', 'translate-x-0'],
    ['skew-y-3', 'scale-y-50', 'translate-y-2', 'rotate-45'],
  ];

  let passed = 0;
  for (const test of tests) {
    if (await testWithBoth(test)) passed++;
  }

  console.log(`\n${passed}/${tests.length} transform tests passed`);
}

runTests().catch(console.error);

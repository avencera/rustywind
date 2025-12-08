import prettier from 'prettier';

async function testOrder(utilities, label) {
  const html = `<div class="${utilities.join(' ')}"></div>`;
  const result = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const sorted = result.match(/class="([^"]*)"/)[1];
  console.log(`${label}:`);
  console.log(`  Input:  ${utilities.join(' ')}`);
  console.log(`  Output: ${sorted}`);
  const parts = sorted.split(' ');
  console.log(`  Order:  ${parts.join(' -> ')}`);
  console.log('');
}

async function runTests() {
  // Test all pairwise combinations of transforms
  await testOrder(['scale-100', 'rotate-0'], 'scale vs rotate');
  await testOrder(['rotate-0', 'skew-x-0'], 'rotate vs skew-x');
  await testOrder(['skew-x-0', 'scale-100'], 'skew-x vs scale');
  await testOrder(['translate-x-0', 'scale-100'], 'translate-x vs scale');
  await testOrder(['translate-x-0', 'rotate-0'], 'translate-x vs rotate');
  await testOrder(['translate-x-0', 'skew-x-0'], 'translate-x vs skew-x');

  // All four together
  await testOrder(['scale-100', 'rotate-0', 'skew-x-0', 'translate-x-0'], 'All transforms (scrambled)');
  await testOrder(['translate-x-0', 'rotate-0', 'skew-x-0', 'scale-100'], 'All transforms (T-R-Sk-Sc)');

  // Test padding sub-ordering more thoroughly
  await testOrder(['pr-0', 'pl-2'], 'pr vs pl');
  await testOrder(['pt-0', 'pb-2'], 'pt vs pb');
  await testOrder(['pl-2', 'pt-4'], 'pl vs pt');
  await testOrder(['pr-2', 'pb-4'], 'pr vs pb');

  // Test rounded sub-ordering
  await testOrder(['rounded-t-lg', 'rounded-r-lg'], 'rounded-t vs rounded-r');
  await testOrder(['rounded-b-lg', 'rounded-l-lg'], 'rounded-b vs rounded-l');

  // Clear sub-ordering
  await testOrder(['clear-left', 'clear-right', 'clear-none'], 'clear utilities');
}

runTests().catch(console.error);

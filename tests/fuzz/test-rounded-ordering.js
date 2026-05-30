/**
 * Test rounded utility ordering with Prettier
 */

import prettier from 'prettier';

async function sortWithPrettier(classes) {
  const formatted = await prettier.format(`<div class="${classes}"></div>`, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
  });
  const match = formatted.match(/class="([^"]*)"/);
  return match ? match[1] : '';
}

async function testClasses(name, classes) {
  const input = classes.join(' ');
  const result = await sortWithPrettier(input);

  console.log(`${name}:`);
  console.log(`   Input:    ${input}`);
  console.log(`   Prettier: ${result}`);
  console.log();

  return result.split(' ');
}

// Run tests
(async () => {
  console.log('Testing rounded utility ordering with Prettier + Tailwind plugin\n');
  console.log('='.repeat(70));
  console.log();

  // Test from test_rounded_cross_axis_b_vs_tl
  await testClasses('1. rounded-tl vs rounded-b', ['rounded-tl', 'rounded-b']);

  // Tests from test_rounded_all_cross_axis_cases
  await testClasses('2. rounded-tl vs rounded-b (reversed)', ['rounded-b', 'rounded-tl']);
  await testClasses('3. rounded-tr-lg vs rounded-b', ['rounded-tr-lg', 'rounded-b']);
  await testClasses('4. rounded-tl vs rounded-r-lg', ['rounded-tl', 'rounded-r-lg']);
  await testClasses('5. rounded-l-lg vs rounded-r', ['rounded-l-lg', 'rounded-r']);
  await testClasses('6. rounded-tl-none vs rounded-r', ['rounded-tl-none', 'rounded-r']);
  await testClasses('7. rounded-l vs rounded-b-none', ['rounded-l', 'rounded-b-none']);
  await testClasses('8. rounded-l-none vs rounded-b-lg', ['rounded-l-none', 'rounded-b-lg']);

  // Test from test_mixed_rounded_utilities
  await testClasses('9. Mixed rounded utilities', [
    'rounded-br-lg',
    'rounded-t-lg',
    'rounded-l-none',
    'rounded-tl-lg',
    'rounded-r',
    'rounded-tr-none',
  ]);

  console.log('='.repeat(70));
})();

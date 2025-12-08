/**
 * Test spacing utility ordering with Prettier
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
  console.log('Testing spacing utility ordering with Prettier + Tailwind plugin\n');
  console.log('='.repeat(70));
  console.log();

  // Test from test_space_x_vs_gap_x_same_axis
  await testClasses('1. space-x vs gap-x (same axis)', ['gap-x-0', 'space-x-1', 'space-x-2']);

  // Test from test_space_x_vs_space_y_ordering
  await testClasses('2. space-x vs space-y ordering', ['space-y-4', 'space-x-1', 'space-y-0', 'space-x-0', 'space-y-1', 'space-x-4']);

  // Additional tests to understand the pattern
  await testClasses('3. space-x-0 vs space-y-0', ['space-x-0', 'space-y-0']);
  await testClasses('4. space-x-1 vs gap-x-0', ['space-x-1', 'gap-x-0']);
  await testClasses('5. gap-x-0 vs gap-y-0', ['gap-x-0', 'gap-y-0']);

  console.log('='.repeat(70));
})();

/**
 * Test snap utility ordering with Prettier
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
  console.log('Testing snap utility ordering with Prettier + Tailwind plugin\n');
  console.log('='.repeat(70));
  console.log();

  // Test from test_snap_proximity_vs_snap_x
  await testClasses('1. snap-proximity vs snap-x', ['snap-x', 'snap-proximity']);
  await testClasses('2. snap-proximity vs snap-x (reversed)', ['snap-proximity', 'snap-x']);

  // Test from test_all_snap_type_utilities
  await testClasses('3. All snap-type utilities', ['snap-proximity', 'snap-none', 'snap-mandatory']);

  // Test from test_snap_utilities_mixed_with_scroll
  await testClasses('4. Snap utilities mixed with scroll', [
    'snap-x', 'overflow-scroll', 'snap-proximity', 'scroll-smooth', 'snap-mandatory', 'scroll-auto'
  ]);

  // Test from test_all_snap_utilities_comprehensive
  await testClasses('5. All snap utilities comprehensive', [
    'snap-y', 'snap-proximity', 'snap-x', 'snap-both', 'snap-start',
    'snap-mandatory', 'snap-center', 'snap-end', 'snap-none'
  ]);

  // Test from test_snap_utilities_alphabetical_pairs
  await testClasses('6. snap-both vs snap-center', ['snap-both', 'snap-center']);
  await testClasses('7. snap-center vs snap-end', ['snap-center', 'snap-end']);
  await testClasses('8. snap-end vs snap-mandatory', ['snap-end', 'snap-mandatory']);
  await testClasses('9. snap-mandatory vs snap-none', ['snap-mandatory', 'snap-none']);
  await testClasses('10. snap-none vs snap-proximity', ['snap-none', 'snap-proximity']);
  await testClasses('11. snap-proximity vs snap-start', ['snap-proximity', 'snap-start']);
  await testClasses('12. snap-start vs snap-x', ['snap-start', 'snap-x']);
  await testClasses('13. snap-x vs snap-y', ['snap-x', 'snap-y']);

  console.log('='.repeat(70));
})();

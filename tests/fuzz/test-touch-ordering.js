/**
 * Test touch utility ordering with Prettier
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
  console.log('Testing touch utility ordering with Prettier + Tailwind plugin\n');
  console.log('='.repeat(70));
  console.log();

  // Test from test_touch_manipulation_vs_touch_pan_left
  await testClasses('1. touch-manipulation vs touch-pan-left', ['touch-pan-left', 'touch-manipulation']);

  // Test from test_touch_pan_up_vs_touch_pan_x
  await testClasses('2. touch-pan-up vs touch-pan-x', ['touch-pan-x', 'touch-pan-up']);

  // Test from test_touch_none_vs_touch_pan_down
  await testClasses('3. touch-none vs touch-pan-down', ['touch-pan-down', 'touch-none']);

  // Test from test_touch_auto_vs_touch_manipulation
  await testClasses('4. touch-auto vs touch-manipulation', ['touch-manipulation', 'touch-auto']);

  // Test from test_multiple_touch_pan_utilities
  await testClasses('5. Multiple touch-pan utilities', [
    'touch-pan-x', 'touch-pan-left', 'touch-pan-up', 'touch-pan-down', 'touch-pan-right', 'touch-pan-y'
  ]);

  // Test from test_all_touch_utilities_alphabetically
  await testClasses('6. All touch utilities', [
    'touch-pinch-zoom', 'touch-pan-x', 'touch-manipulation', 'touch-auto',
    'touch-pan-up', 'touch-none', 'touch-pan-left', 'touch-pan-down',
    'touch-pan-right', 'touch-pan-y'
  ]);

  // Test from test_touch_utilities_mixed_with_other_utilities
  await testClasses('7. Touch utilities mixed with other utilities', [
    'touch-pan-x', 'pointer-events-none', 'touch-manipulation', 'cursor-pointer',
    'touch-pan-up', 'select-none', 'touch-auto', 'user-select-none'
  ]);

  console.log('='.repeat(70));
})();

/**
 * Test ring vs shadow utility ordering with Prettier
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
  console.log('Testing ring vs shadow ordering with Prettier + Tailwind plugin\n');
  console.log('='.repeat(70));
  console.log();

  // Test 1: Simple ring vs shadow
  await testClasses('1. ring-0 vs shadow-blue-500', ['shadow-blue-500', 'ring-0']);
  await testClasses('2. ring vs shadow-gray-500', ['shadow-gray-500', 'ring']);
  await testClasses('3. ring-2 vs shadow-gray-500', ['shadow-gray-500', 'ring-2']);

  // Test 2: Ring utilities vs shadow size utilities
  await testClasses('4. Multiple ring and shadow sizes', [
    'shadow-sm', 'ring-0', 'shadow-lg', 'ring', 'shadow-xl', 'ring-2', 'shadow', 'ring-1'
  ]);

  // Test 3: Mixed with other utilities
  await testClasses('5. Mixed with other utilities', [
    'shadow-blue-500', 'border-2', 'ring-0', 'bg-white', 'shadow-sm',
    'ring-2', 'p-4', 'shadow-gray-500', 'ring', 'text-gray-900', 'shadow-lg'
  ]);

  // Test 4: Ring colors vs shadow colors
  await testClasses('6. ring-blue-500 vs shadow-red-500', ['shadow-red-500', 'ring-blue-500']);
  await testClasses('7. ring-gray-200 vs shadow-gray-500', ['shadow-gray-500', 'ring-gray-200']);

  // Test 5: Ring inset
  await testClasses('8. ring-inset vs shadow', ['shadow', 'ring-inset']);

  console.log('='.repeat(70));
})();

/**
 * Test specific class pair orderings against prettier
 *
 * This script tests the 11 most common class pair mismatches found in fuzz testing
 * to determine the correct ordering according to Tailwind's Prettier plugin.
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

async function testPair(name, class1, class2) {
  const input = `${class1} ${class2}`;
  const reversed = `${class2} ${class1}`;

  const result1 = await sortWithPrettier(input);
  const result2 = await sortWithPrettier(reversed);

  // both should give the same result
  if (result1 !== result2) {
    console.log(`⚠️  ${name}: Inconsistent results!`);
    console.log(`   ${input} -> ${result1}`);
    console.log(`   ${reversed} -> ${result2}`);
    return;
  }

  const firstClass = result1.split(' ')[0];
  const secondClass = result1.split(' ')[1];

  console.log(`${name}:`);
  console.log(`   Input:    ${class1} vs ${class2}`);
  console.log(`   Prettier: ${firstClass} ${secondClass}`);
  console.log(`   First:    ${firstClass}`);
  console.log();
}

// run tests
(async () => {
  console.log('Testing class pair orderings with Prettier + Tailwind plugin\n');
  console.log('='.repeat(70));
  console.log();

  // Original 11 pairs
  await testPair('1. z-[-1] vs z-auto', 'z-[-1]', 'z-auto');
  await testPair('2. w-1 vs w-1/3', 'w-1', 'w-1/3');
  await testPair('3. w-2 vs w-2/3', 'w-2', 'w-2/3');
  await testPair('4. w-1 vs w-1/4', 'w-1', 'w-1/4');
  await testPair('5. w-2 vs w-3/4', 'w-2', 'w-3/4');
  await testPair('6. w-1/3 vs w-1/4', 'w-1/3', 'w-1/4');
  await testPair('7. w-1/2 vs w-1/3', 'w-1/2', 'w-1/3');
  await testPair('8. w-1 vs w-2/3', 'w-1', 'w-2/3');
  await testPair('9. w-1 vs w-1/2', 'w-1', 'w-1/2');
  await testPair('10. w-1/2 vs w-1/4', 'w-1/2', 'w-1/4');
  await testPair('11. w-1 vs w-3/4', 'w-1', 'w-3/4');

  console.log();
  console.log('NEW PAIRS FROM ADDITIONAL FUZZ TESTING:');
  console.log('-'.repeat(70));
  console.log();

  // New 17 pairs
  await testPair('12. w-1/3 vs w-2', 'w-1/3', 'w-2');
  await testPair('13. w-2/3 vs w-3/4', 'w-2/3', 'w-3/4');
  await testPair('14. w-1/2 vs w-2', 'w-1/2', 'w-2');
  await testPair('15. w-1/4 vs w-2', 'w-1/4', 'w-2');
  await testPair('16. w-1/4 vs w-2/3', 'w-1/4', 'w-2/3');
  await testPair('17. w-1/3 vs w-3/4', 'w-1/3', 'w-3/4');
  await testPair('18. w-1/2 vs w-4', 'w-1/2', 'w-4');
  await testPair('19. w-1/3 vs w-8', 'w-1/3', 'w-8');
  await testPair('20. w-1/3 vs w-4', 'w-1/3', 'w-4');
  await testPair('21. w-1/2 vs w-2/3', 'w-1/2', 'w-2/3');
  await testPair('22. w-1/2 vs w-8', 'w-1/2', 'w-8');
  await testPair('23. w-1/2 vs w-3/4', 'w-1/2', 'w-3/4');
  await testPair('24. w-1/4 vs w-8', 'w-1/4', 'w-8');
  await testPair('25. w-3/4 vs w-4', 'w-3/4', 'w-4');
  await testPair('26. w-3/4 vs w-8', 'w-3/4', 'w-8');
  await testPair('27. w-1/4 vs w-4', 'w-1/4', 'w-4');
  await testPair('28. w-1/3 vs w-2/3', 'w-1/3', 'w-2/3');

  console.log('='.repeat(70));
})();

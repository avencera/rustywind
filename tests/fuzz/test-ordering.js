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
  console.log('');
}

async function runTests() {
  // Test #1: Transform ordering - skew vs scale
  await testOrder(['skew-x-1', 'scale-150'], 'Transform: skew vs scale');
  await testOrder(['rotate-12', 'scale-x-150'], 'Transform: rotate vs scale');
  await testOrder(['-rotate-1', 'scale-x-100'], 'Transform: -rotate vs scale');
  await testOrder(['translate-x-0', '-rotate-1', 'skew-x-6', 'scale-x-100'], 'Transform: all together');

  // Test #2: Outline vs Ring
  await testOrder(['outline', 'ring-blue-500'], 'Outline vs Ring');
  await testOrder(['outline-offset-2', 'ring-offset-gray-500'], 'Outline-offset vs Ring-offset');

  // Test #3: Font vs Leading
  await testOrder(['font-extrabold', 'leading-snug'], 'Font vs Leading');

  // Test #4: Min-w vs Max-w
  await testOrder(['min-w-0', 'max-w-fit'], 'Min-w vs Max-w');

  // Test #5: Padding sub-ordering
  await testOrder(['pl-2', 'pr-0', 'p-4'], 'Padding: pl vs pr vs p');

  // Test #6: Grid-flow sub-ordering
  await testOrder(['grid-flow-row', 'grid-flow-dense'], 'Grid-flow: row vs dense');

  // Test #7: Border radius sub-ordering
  await testOrder(['rounded-r', 'rounded-t-none'], 'Border-radius: r vs t');
  await testOrder(['rounded-r-lg', 'rounded-t-lg'], 'Border-radius: r-lg vs t-lg');

  // Test #8: Divide placement
  await testOrder(['divide-none', 'self-baseline'], 'Divide vs Self');
  await testOrder(['divide-dashed', 'border-dotted'], 'Divide vs Border style');
  await testOrder(['divide-y', 'divide-dashed', 'border-dotted'], 'Divide-y vs divide-dashed vs border');

  // Test #9: Whitespace placement
  await testOrder(['whitespace-pre-line', 'pr-4'], 'Whitespace vs Padding');
  await testOrder(['whitespace-break-spaces', 'border-dashed'], 'Whitespace vs Border');

  // Test #10: Text transform vs text color
  await testOrder(['normal-case', 'text-black'], 'Normal-case vs Text-color');

  // Test #11: Size vs dimension utilities
  await testOrder(['size-auto', 'h-2', 'w-1/2'], 'Size vs H vs W');

  // Test #12: Clear positioning
  await testOrder(['clear-none', 'size-auto', 'h-2'], 'Clear vs Size vs H');

  // Test #13: Select placement
  await testOrder(['select-auto', 'cursor-zoom-in'], 'Select vs Cursor');

  // Test #14: Rotate vs Scale with skew
  await testOrder(['rotate-0', 'scale-x-0'], 'Rotate-0 vs Scale-x-0');

  // Test #15: Overscroll vs Divide
  await testOrder(['divide-transparent', 'overscroll-auto'], 'Divide vs Overscroll');
}

runTests().catch(console.error);

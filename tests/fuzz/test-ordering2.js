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
  // Test from failure #21
  await testOrder(['divide-none', 'self-baseline', 'rotate-3', 'gap-2', 'border-x-2'], 'Test #21');

  // Test from failure #30 - divide with borders
  await testOrder(['divide-y', 'divide-dashed', 'border-dotted', 'border-none'], 'Test #30: Divide with borders');

  // Test from failure #7 - whitespace with border
  await testOrder(['divide-y-reverse', 'whitespace-break-spaces', 'border-dashed'], 'Test #7: Whitespace with border');

  // Test from failure #11 - padding ordering
  await testOrder(['p-4', 'pl-2', 'pr-0', 'pr-2'], 'Test #11: Complex padding');

  // Test from failure #28 - min-w vs max-w in context
  await testOrder(['w-fit', 'min-w-0', 'max-w-fit'], 'Test #28: W/Min-w/Max-w');

  // Test from failure #22 - transform complete ordering
  await testOrder(['translate-x-0', '-rotate-1', 'skew-x-6', 'scale-x-100'], 'Test #22: Transform order');

  // Test from failure #29 - font vs leading
  await testOrder(['pr-0', 'font-extrabold', 'leading-snug'], 'Test #29: Font vs Leading with padding');

  // Test from failure #37 - text transform vs text color
  await testOrder(['normal-case', 'text-black', 'text-gray-500'], 'Test #37: Text utilities');

  // Test from failure #42 - select vs cursor
  await testOrder(['select-auto', 'cursor-zoom-in', 'empty:cursor-default'], 'Test #42: Select vs Cursor');

  // Test from failure #44 - divide vs overscroll
  await testOrder(['divide-transparent', 'overscroll-auto', 'space-y-1'], 'Test #44: Divide vs Overscroll');

  // Test from failure #46 - whitespace vs padding
  await testOrder(['whitespace-pre-line', 'pr-4', 'justify-normal'], 'Test #46: Whitespace vs Padding');

  // Test from failure #48 - transform with modifiers
  await testOrder(['visited:scale-x-100', 'visited:skew-x-6'], 'Test #48: Modifier transforms');

  // Clear property positioning
  await testOrder(['clear-none', 'clear-left', 'clear-right'], 'Clear utilities');

  // Invert positioning
  await testOrder(['cursor-ew-resize', 'invert-0', 'backdrop-hue-rotate-0'], 'Invert with other filters');
}

runTests().catch(console.error);

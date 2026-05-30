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
  // Test whitespace vs background
  await testOrder(['whitespace-nowrap', 'bg-no-repeat'], 'whitespace vs bg');
  await testOrder(['whitespace-pre-line', 'border-dashed'], 'whitespace vs border');
  await testOrder(['whitespace-normal', 'object-right-bottom'], 'whitespace vs object');

  // Test divide color utilities
  await testOrder(['divide-transparent', 'rounded-bl-none'], 'divide-transparent vs rounded');
  await testOrder(['divide-white', 'justify-self-end'], 'divide-white vs justify-self');
  await testOrder(['divide-gray-500', 'overflow-auto'], 'divide-gray vs overflow');

  // Test display utilities
  await testOrder(['inline', 'hidden'], 'inline vs hidden');
  await testOrder(['table-column-group', 'inline-grid'], 'table-column-group vs inline-grid');

  // Test text-clip vs border
  await testOrder(['text-clip', 'border-double'], 'text-clip vs border');

  // Test rounded vs border
  await testOrder(['rounded-none', 'border-y-2'], 'rounded vs border');

  // Test outline vs ring
  await testOrder(['outline-blue-500', 'ring-2'], 'outline-blue vs ring');
  await testOrder(['outline-dashed', 'ring-offset-white'], 'outline-dashed vs ring-offset');

  // Test snap utilities
  await testOrder(['snap-none', 'snap-mandatory'], 'snap-none vs snap-mandatory');
}

runTests().catch(console.error);

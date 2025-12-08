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
  // Padding combinations
  await testOrder(['p-4', 'pl-2', 'pr-4', 'pt-2', 'pb-2'], 'All padding');
  await testOrder(['px-4', 'py-2'], 'px vs py');
  await testOrder(['p-4', 'px-2'], 'p vs px');

  // Margin combinations
  await testOrder(['m-4', 'ml-2', 'mr-4', 'mt-2', 'mb-2'], 'All margin');
  await testOrder(['mx-4', 'my-2'], 'mx vs my');
  await testOrder(['m-4', 'mx-2'], 'm vs mx');

  // Mixed spacing
  await testOrder(['m-4', 'p-4', 'ml-2', 'pl-2'], 'Margin and padding mixed');
}

runTests().catch(console.error);

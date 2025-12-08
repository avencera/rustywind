import prettier from 'prettier';

async function testComparison(a, b) {
  const html = '<div class="' + a + ' ' + b + '"></div>';
  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match = formatted.match(/class="([^"]*)"/);
  const sorted = match ? match[1] : '';
  const input = a + ' ' + b;
  console.log(input.padEnd(50) + ' → ' + sorted);
}

console.log('Variant ordering tests:');
await testComparison('dark:md:z-10', 'md:dark:z-20');
await testComparison('focus:dark:p-4', 'dark:focus:p-8');
await testComparison('checked:checked:max-w-0', 'checked:max-w-4');
await testComparison('peer-hover:group-focus:ml-0', 'peer-focus:ml-4');

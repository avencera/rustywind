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
  console.log(a.padEnd(50) + ' vs ' + b.padEnd(50) + ' → ' + sorted);
}

console.log('Duplicate variant tests:');
await testComparison('hover:hover:caret-gray-500', 'hover:w-3/4');
await testComparison('hover:w-3/4', 'hover:hover:caret-gray-500');
await testComparison('focus-within:animate-ping', 'focus-within:focus-within:whitespace-nowrap');
await testComparison('peer-focus:text-left', 'peer-focus:peer-focus:bg-blend-screen');

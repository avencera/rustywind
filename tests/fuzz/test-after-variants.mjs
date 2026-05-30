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

console.log('After variant tests:');
await testComparison('after:outline-0', 'after:after:break-inside-avoid-page');
await testComparison('after:after:break-inside-avoid-page', 'after:outline-0');

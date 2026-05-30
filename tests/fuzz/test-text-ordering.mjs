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
  console.log(input.padEnd(40) + ' → ' + sorted);
}

console.log('Text utility tests:');
await testComparison('text-sm', 'text-[42px]');
await testComparison('text-lg', 'text-[42px]');
await testComparison('leading-6', 'leading-[40px]');
await testComparison('leading-tight', 'leading-[40px]');

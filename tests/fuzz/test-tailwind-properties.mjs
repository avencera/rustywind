// Test what properties Tailwind generates for different utilities
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

console.log('Comparison tests:');
await testComparison('rounded-lg', 'rounded-[14px]');
await testComparison('rounded', 'rounded-lg');
await testComparison('rounded', 'rounded-[14px]');
await testComparison('my-4', 'my-[6px]');
await testComparison('my-auto', 'my-[6px]');

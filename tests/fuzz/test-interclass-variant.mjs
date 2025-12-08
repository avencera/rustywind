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
  console.log(a.padEnd(30) + ' vs ' + b.padEnd(30) + ' → ' + sorted);
}

console.log('Inter-class variant ordering:');
await testComparison('dark:md:z-50', 'md:dark:resize-none');
await testComparison('dark:focus:border-x', 'focus:dark:ring-offset-white');
await testComparison('peer-hover:group-focus:ml-0', 'peer-focus:resize-x');

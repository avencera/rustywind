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
  console.log(a.padEnd(40) + ' vs ' + b.padEnd(40) + ' → ' + sorted);
}

console.log('Peer variant ordering:');
await testComparison('peer-hover:ml-0', 'peer-focus:ml-4');
await testComparison('peer-focus:ml-4', 'peer-hover:ml-0');
await testComparison('group-hover:ml-0', 'group-focus:ml-4');

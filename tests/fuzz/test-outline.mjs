import prettier from 'prettier';

async function testOutline() {
  const classes = ['outline-double', 'outline-offset-1'];
  const html = `<div class="${classes.join(' ')}"></div>`;

  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });

  const match = formatted.match(/class="([^"]*)"/);
  const sorted = match[1].split(/\s+/).filter(c => c.length > 0);

  console.log('Input:', classes);
  console.log('Prettier sorted:', sorted);
  console.log('');

  // Test will-change vs select
  const classes2 = ['select-none', 'will-change-scroll'];
  const html2 = `<div class="${classes2.join(' ')}"></div>`;

  const formatted2 = await prettier.format(html2, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });

  const match2 = formatted2.match(/class="([^"]*)"/);
  const sorted2 = match2[1].split(/\s+/).filter(c => c.length > 0);

  console.log('Input:', classes2);
  console.log('Prettier sorted:', sorted2);
  console.log('');

  // Test blur vs ring
  const classes3 = ['ring-inset', 'blur-lg'];
  const html3 = `<div class="${classes3.join(' ')}"></div>`;

  const formatted3 = await prettier.format(html3, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });

  const match3 = formatted3.match(/class="([^"]*)"/);
  const sorted3 = match3[1].split(/\s+/).filter(c => c.length > 0);

  console.log('Input:', classes3);
  console.log('Prettier sorted:', sorted3);
}

testOutline();

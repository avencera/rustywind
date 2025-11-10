import prettier from 'prettier';

async function testOutlineVsTransition() {
  // Test outline vs delay
  const test1 = ['outline-dotted', 'delay-100'];
  const html1 = `<div class="${test1.join(' ')}"></div>`;
  const formatted1 = await prettier.format(html1, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match1 = formatted1.match(/class="([^"]*)"/);
  const sorted1 = match1[1].split(/\s+/).filter(c => c.length > 0);
  console.log('Test 1: outline-dotted vs delay-100');
  console.log('Input:   ', test1);
  console.log('Prettier:', sorted1);
  console.log('');

  // Test outline vs duration
  const test2 = ['outline-none', 'duration-300'];
  const html2 = `<div class="${test2.join(' ')}"></div>`;
  const formatted2 = await prettier.format(html2, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match2 = formatted2.match(/class="([^"]*)"/);
  const sorted2 = match2[1].split(/\s+/).filter(c => c.length > 0);
  console.log('Test 2: outline-none vs duration-300');
  console.log('Input:   ', test2);
  console.log('Prettier:', sorted2);
  console.log('');

  // Test outline vs transition
  const test3 = ['outline-solid', 'transition-all'];
  const html3 = `<div class="${test3.join(' ')}"></div>`;
  const formatted3 = await prettier.format(html3, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match3 = formatted3.match(/class="([^"]*)"/);
  const sorted3 = match3[1].split(/\s+/).filter(c => c.length > 0);
  console.log('Test 3: outline-solid vs transition-all');
  console.log('Input:   ', test3);
  console.log('Prettier:', sorted3);
  console.log('');

  // Test outline vs will-change
  const test4 = ['outline-dotted', 'will-change-transform'];
  const html4 = `<div class="${test4.join(' ')}"></div>`;
  const formatted4 = await prettier.format(html4, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match4 = formatted4.match(/class="([^"]*)"/);
  const sorted4 = match4[1].split(/\s+/).filter(c => c.length > 0);
  console.log('Test 4: outline-dotted vs will-change-transform');
  console.log('Input:   ', test4);
  console.log('Prettier:', sorted4);
  console.log('');

  // Comprehensive test
  const test5 = ['outline-none', 'delay-100', 'outline-dotted', 'duration-300',
                 'outline-dashed', 'transition-all', 'outline-double',
                 'will-change-transform', 'outline-solid'];
  const html5 = `<div class="${test5.join(' ')}"></div>`;
  const formatted5 = await prettier.format(html5, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match5 = formatted5.match(/class="([^"]*)"/);
  const sorted5 = match5[1].split(/\s+/).filter(c => c.length > 0);
  console.log('Test 5: Comprehensive test');
  console.log('Input:   ', test5);
  console.log('Prettier:', sorted5);
}

testOutlineVsTransition();

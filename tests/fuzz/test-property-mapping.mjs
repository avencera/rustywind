import prettier from 'prettier';

async function testUtility(classes, description) {
  const html = `<div class="${classes.join(' ')}"></div>`;
  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match = formatted.match(/class="([^"]*)"/);
  const sorted = match[1].split(/\s+/).filter(c => c.length > 0);

  console.log(`${description}:`);
  console.log(`  Input:  [${classes.join(', ')}]`);
  console.log(`  Output: [${sorted.join(', ')}]`);
  console.log('');
}

// Test outline utilities
await testUtility(['outline-1', 'outline-double'], 'outline-1 vs outline-double');
await testUtility(['outline-2', 'outline-double'], 'outline-2 vs outline-double');
await testUtility(['outline', 'outline-double'], 'outline vs outline-double');
await testUtility(['outline-black', 'outline-double'], 'outline-black vs outline-double');
await testUtility(['outline-offset-0', 'outline-double'], 'outline-offset-0 vs outline-double');
await testUtility(['outline-offset-1', 'outline-1'], 'outline-offset-1 vs outline-1');
await testUtility(['outline-offset-1', 'outline-black'], 'outline-offset-1 vs outline-black');

// Test if outline-solid exists
await testUtility(['outline-solid', 'outline-offset-1'], 'outline-solid vs outline-offset-1');

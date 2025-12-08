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

// Test blur vs various shadow/ring utilities
await testUtility(['shadow', 'blur-lg'], 'shadow vs blur-lg');
await testUtility(['shadow-md', 'blur-lg'], 'shadow-md vs blur-lg');
await testUtility(['shadow-lg', 'blur-lg'], 'shadow-lg vs blur-lg');
await testUtility(['ring-1', 'blur-lg'], 'ring-1 vs blur-lg');
await testUtility(['ring-2', 'blur-lg'], 'ring-2 vs blur-lg');
await testUtility(['ring-inset', 'blur-lg'], 'ring-inset vs blur-lg');
await testUtility(['ring-offset-1', 'blur-lg'], 'ring-offset-1 vs blur-lg');

// Test blur vs outline to see the pattern
await testUtility(['outline-1', 'blur-lg'], 'outline-1 vs blur-lg');
await testUtility(['blur-lg', 'brightness-50'], 'blur-lg vs brightness-50');

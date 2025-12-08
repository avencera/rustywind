import prettier from 'prettier';

async function test() {
  console.log('=== Testing Shadow Utilities ===');
  const shadowTests = [
    'shadow',
    'shadow-sm', 
    'shadow-md',
    'shadow-lg',
    'shadow-xl',
    'shadow-2xl',
    'shadow-inner',
    'shadow-none',
  ];
  
  for (const shadow of shadowTests) {
    const html = `<div class="ring-2 ${shadow}"></div>`;
    const formatted = await prettier.format(html, {
      parser: 'html',
      plugins: ['prettier-plugin-tailwindcss'],
      printWidth: 10000,
    });
    const match = formatted.match(/class="([^"]*)"/);
    const sorted = match ? match[1] : '';
    console.log(`ring-2 ${shadow}`.padEnd(25), '→', sorted);
  }
  
  console.log('\n=== Testing Shadow Color Utilities ===');
  const colorTests = [
    'shadow-blue-500',
    'shadow-gray-500',
    'shadow-red-400',
  ];
  
  for (const shadowColor of colorTests) {
    const html = `<div class="ring-2 ${shadowColor}"></div>`;
    const formatted = await prettier.format(html, {
      parser: 'html',
      plugins: ['prettier-plugin-tailwindcss'],
      printWidth: 10000,
    });
    const match = formatted.match(/class="([^"]*)"/);
    const sorted = match ? match[1] : '';
    console.log(`ring-2 ${shadowColor}`.padEnd(25), '→', sorted);
  }
}

test();

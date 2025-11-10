import prettier from 'prettier';

async function test() {
  // Test the exact classes from the failing test
  const tests = [
    'shadow-blue-500 ring-0',
    'ring-0 shadow-blue-500',
    'shadow-gray-500 ring',
    'ring shadow-gray-500',
    'shadow-gray-500 ring-2',
    'ring-2 shadow-gray-500',
    'shadow-lg ring-2',
    'ring-2 shadow-lg',
  ];
  
  for (const classes of tests) {
    const html = '<div class="' + classes + '"></div>';
    const formatted = await prettier.format(html, {
      parser: 'html',
      plugins: ['prettier-plugin-tailwindcss'],
      printWidth: 10000,
    });
    
    const match = formatted.match(/class="([^"]*)"/);
    const sorted = match ? match[1] : '';
    
    console.log(classes.padEnd(30), '→', sorted);
  }
}

test();

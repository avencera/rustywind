import prettier from 'prettier';

async function test() {
  const tests = [
    'ring-gray-500 shadow-gray-500',
    'shadow-gray-500 ring-gray-500',
    'ring-blue-500 shadow-blue-500',
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
    
    console.log(classes.padEnd(40), '→', sorted);
  }
}

test();

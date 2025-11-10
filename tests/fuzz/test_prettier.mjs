import prettier from 'prettier';

async function test() {
  const tests = [
    'blur-lg ring-inset',
    'blur-lg ring-2 shadow-lg ring-inset',
    'ring-2 shadow-lg',
    'ring shadow',
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
    
    console.log('Input: ', classes);
    console.log('Output:', sorted);
    console.log('');
  }
}

test();

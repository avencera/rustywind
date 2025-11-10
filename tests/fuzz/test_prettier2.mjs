import prettier from 'prettier';

async function test() {
  const tests = [
    // Ring vs Shadow
    'ring-0 shadow-sm',
    'ring shadow',
    'ring-2 shadow-gray-500',
    'ring-inset shadow-lg',
    
    // Filter vs Ring
    'blur-lg ring-2',
    'blur-sm ring-inset',
    'brightness-50 ring',
    
    // Filter vs Shadow
    'blur-lg shadow-lg',
    'brightness-50 shadow',
    
    // All three
    'blur-lg ring-2 shadow-lg',
    'blur-sm shadow-lg ring-inset',
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

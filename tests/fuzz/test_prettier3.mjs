import prettier from 'prettier';

async function test() {
  const tests = [
    // Ring width vs Shadow size
    'ring-2 shadow-lg',
    'ring-1 shadow-sm',
    'ring-4 shadow-xl',
    
    // Ring width vs Shadow color
    'ring-2 shadow-blue-500',
    'ring-1 shadow-gray-500',
    
    // Ring color vs Shadow color
    'ring-blue-500 shadow-gray-500',
    
    // Ring-inset vs other rings
    'ring-inset ring-2',
    'ring-inset ring',
    
    // Ring-inset vs shadow-color
    'ring-inset shadow-blue-500',
    
    // Different filter utilities
    'blur-lg ring-2',
    'blur-sm ring-2',
    'saturate-150 ring-2',
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

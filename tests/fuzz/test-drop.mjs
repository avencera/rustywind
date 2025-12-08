import prettier from 'prettier';

async function test() {
  const tests = [
    'drop-shadow-sm drop-shadow-none',
    'drop-shadow-none drop-shadow-sm',
    'drop-shadow drop-shadow-none',
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

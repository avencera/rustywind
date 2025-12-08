import prettier from 'prettier';

async function test() {
  const tests = [
    'rounded rounded-[14px]',
    'rounded-[14px] rounded',
    'my-auto my-[6px]',
    'my-[6px] my-auto',
    'rounded-lg rounded-[14px]',
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

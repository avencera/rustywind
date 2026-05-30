const prettier = require('prettier');
const prettierPluginTailwind = require('prettier-plugin-tailwindcss');

async function test() {
  const classes = '-skew-x-12 -skew-x-3 -skew-x-1';
  
  const formatted = await prettier.format(`<div class="${classes}"></div>`, {
    parser: 'html',
    plugins: [prettierPluginTailwind],
  });
  
  console.log('Input:', classes);
  console.log('Output:', formatted.match(/class="([^"]*)"/)[1]);
}

test();

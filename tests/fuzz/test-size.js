import prettier from 'prettier';
import prettierPlugin from 'prettier-plugin-tailwindcss';

async function test() {
  const classes = 'size-2 h-auto w-4';
  const sorted = await prettier.format(`<div class="${classes}"></div>`, {
    parser: 'html',
    plugins: [prettierPlugin],
  });
  console.log('Input: ', classes);
  console.log('Sorted:', sorted.trim().match(/class="([^"]*)"/)[1]);
}

test();

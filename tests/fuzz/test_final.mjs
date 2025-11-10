import prettier from 'prettier';
import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function testBoth(classes) {
  // Test with Prettier
  const html = '<div class="' + classes + '"></div>';
  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match = formatted.match(/class="([^"]*)"/);
  const prettierResult = match ? match[1] : '';
  
  // Test with RustyWind
  const { stdout } = await execAsync(`echo '${html}' | cargo run --bin rustywind -- --stdin`, {
    cwd: '/home/user/rustywind'
  });
  const rustyMatch = stdout.match(/class="([^"]*)"/);
  const rustyResult = rustyMatch ? rustyMatch[1] : '';
  
  console.log('Input:     ', classes);
  console.log('Prettier:  ', prettierResult);
  console.log('RustyWind: ', rustyResult);
  console.log('Match:     ', prettierResult === rustyResult ? '✓' : '✗');
  console.log('');
}

async function test() {
  await testBoth('ring-0 shadow-blue-500');
  await testBoth('ring shadow-gray-500');
  await testBoth('ring-2 shadow-gray-500');
  await testBoth('blur-lg ring-inset');
  await testBoth('shadow-lg ring-2');
}

test();

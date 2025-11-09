import { exec } from 'child_process';
import { promisify } from 'util';
import prettier from 'prettier';

const execAsync = promisify(exec);

async function testSpacing() {
  const testCases = [
    // Test #92 pattern
    ['space-y-2', 'gap-y-4'],
    // Test #97 pattern
    ['space-y-1', 'space-x-reverse'],
    // Additional tests
    ['space-x-2', 'gap-x-4'],
    ['space-x-reverse', 'space-y-reverse'],
  ];

  for (const classes of testCases) {
    const html = `<div class="${classes.join(' ')}"></div>`;

    // Prettier
    const formatted = await prettier.format(html, {
      parser: 'html',
      plugins: ['prettier-plugin-tailwindcss'],
      printWidth: 10000,
    });
    const prettierMatch = formatted.match(/class="([^"]*)"/);
    const prettierSorted = prettierMatch[1].split(/\s+/);

    // RustyWind
    const rustywindBin = '../../target/release/rustywind';
    const { stdout } = await execAsync(`echo '${html}' | ${rustywindBin} --stdin`);
    const rustywindMatch = stdout.trim().match(/class="([^"]*)"/);
    const rustywindSorted = rustywindMatch[1].split(/\s+/);

    console.log(`\nTest: [${classes.join(', ')}]`);
    console.log(`Prettier:  [${prettierSorted.join(', ')}]`);
    console.log(`RustyWind: [${rustywindSorted.join(', ')}]`);
    console.log(`Match: ${prettierSorted.join(' ') === rustywindSorted.join(' ') ? '✅' : '❌'}`);
  }
}

testSpacing().catch(console.error);

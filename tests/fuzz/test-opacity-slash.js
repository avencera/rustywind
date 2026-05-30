#!/usr/bin/env node

import { exec } from 'child_process';
import { promisify } from 'util';
import prettier from 'prettier';

const execAsync = promisify(exec);

async function testSorting(classes) {
  // Test with RustyWind
  const rustyCmd = `echo 'class="${classes}"' | ../../target/release/rustywind --stdin`;
  const { stdout: rustyOut } = await execAsync(rustyCmd);
  // Extract classes from the output
  const rustyClasses = rustyOut.match(/class="([^"]+)"/)?.[1] || rustyOut.trim();

  // Test with Prettier
  const html = `<div class="${classes}"></div>`;
  const prettierOut = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
  });
  const prettierClasses = prettierOut.match(/class="([^"]+)"/)?.[1] || '';

  console.log(`Input:     ${classes}`);
  console.log(`RustyWind: ${rustyClasses}`);
  console.log(`Prettier:  ${prettierClasses}`);
  console.log(`Match:     ${rustyClasses === prettierClasses ? '✓' : '✗'}`);
  console.log('');
}

async function main() {
  console.log('Testing opacity slash syntax sorting:\n');

  // Test cases from the problem statement
  await testSorting('to-stroke/0 sticky');
  await testSorting('to-stroke/0 table-caption');
  await testSorting('text-white/60 flex');
  await testSorting('bg-black/25 sticky');
  await testSorting('bg-primary/20 flex');
  await testSorting('from-stroke/0 via-stroke to-stroke/0');

  // More complex cases
  await testSorting('flex text-white/60 bg-black/25 sticky');
  await testSorting('to-stroke/0 from-stroke/0 sticky table-caption');
}

main().catch(console.error);

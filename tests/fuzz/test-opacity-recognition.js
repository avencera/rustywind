#!/usr/bin/env node

import { exec } from 'child_process';
import { promisify } from 'util';
import prettier from 'prettier';

const execAsync = promisify(exec);

async function testRecognition(classes) {
  const rustyCmd = `echo '${classes}' | ../../target/release/rustywind --stdin`;
  const { stdout: rustyOut } = await execAsync(rustyCmd);

  const html = `<div class="${classes}"></div>`;
  const prettierOut = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
  });
  const prettierClasses = prettierOut.match(/class="([^"]+)"/)?.[1] || '';

  return {
    input: classes,
    rusty: rustyOut.trim(),
    prettier: prettierClasses,
    match: rustyOut.trim() === prettierClasses,
  };
}

async function main() {
  console.log('Testing which opacity classes are treated as known vs unknown:\n');

  // Test opacity with standard colors (should be KNOWN)
  console.log('=== Standard Colors with Opacity (should be KNOWN) ===');
  let result = await testRecognition('flex text-white/60');
  console.log(`text-white/60 + flex:`);
  console.log(`  Prettier: ${result.prettier} (${result.prettier.startsWith('flex') ? 'KNOWN' : 'UNKNOWN'})`);
  console.log(`  RustyWind: ${result.rusty} (${result.rusty.startsWith('text') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log('');

  result = await testRecognition('sticky bg-black/25');
  console.log(`bg-black/25 + sticky:`);
  console.log(`  Prettier: ${result.prettier} (${result.prettier.startsWith('sticky') ? 'KNOWN' : 'UNKNOWN'})`);
  console.log(`  RustyWind: ${result.rusty} (${result.rusty.startsWith('bg') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log('');

  result = await testRecognition('sticky bg-red-500/50');
  console.log(`bg-red-500/50 + sticky:`);
  console.log(`  Prettier: ${result.prettier} (${result.prettier.startsWith('sticky') ? 'KNOWN' : 'UNKNOWN'})`);
  console.log(`  RustyWind: ${result.rusty} (${result.rusty.startsWith('bg') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log('');

  // Test opacity with custom colors (should be UNKNOWN)
  console.log('=== Custom Colors with Opacity (should be UNKNOWN) ===');
  result = await testRecognition('flex to-stroke/0');
  console.log(`to-stroke/0 + flex:`);
  console.log(`  Prettier: ${result.prettier} (${result.prettier.startsWith('to-stroke') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log(`  RustyWind: ${result.rusty} (${result.rusty.startsWith('to-stroke') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log('');

  result = await testRecognition('sticky bg-primary/20');
  console.log(`bg-primary/20 + sticky:`);
  console.log(`  Prettier: ${result.prettier} (${result.prettier.startsWith('bg-primary') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log(`  RustyWind: ${result.rusty} (${result.rusty.startsWith('bg-primary') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log('');

  // Test border colors with opacity
  console.log('=== Border Colors with Opacity ===');
  result = await testRecognition('sticky border-gray-300/50');
  console.log(`border-gray-300/50 + sticky:`);
  console.log(`  Prettier: ${result.prettier} (${result.prettier.startsWith('sticky') ? 'KNOWN' : 'UNKNOWN'})`);
  console.log(`  RustyWind: ${result.rusty} (${result.rusty.startsWith('border') ? 'UNKNOWN' : 'KNOWN'})`);
  console.log('');
}

main().catch(console.error);

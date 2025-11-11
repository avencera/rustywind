#!/usr/bin/env node

import { execSync } from 'child_process';

const testCases = [
  {
    name: 'outline-style utilities',
    classes: 'p-4 m-4 bg-blue-500 outline-solid outline-dashed outline-dotted shadow-lg ring-2',
  },
  {
    name: 'select utilities',
    classes: 'p-4 m-4 select-none select-text select-all select-auto opacity-50',
  },
  {
    name: 'ring-inset',
    classes: 'p-4 m-4 ring-inset ring-2 ring-blue-500 shadow-lg outline-1',
  },
  {
    name: 'divide-x-reverse',
    classes: 'p-4 m-4 divide-x-reverse divide-x-2 space-x-4 gap-4',
  },
  {
    name: 'mixed critical utilities',
    classes: 'pointer-events-auto mx-4 z-10 select-text outline-none ring-inset divide-x-reverse',
  },
];

console.log('Testing where Prettier places problematic utilities:\n');

for (const test of testCases) {
  console.log(`\n${'='.repeat(80)}`);
  console.log(`Test: ${test.name}`);
  console.log(`${'='.repeat(80)}`);
  console.log(`Input:  ${test.classes}`);

  try {
    // Sort with Prettier
    const prettier = execSync(
      `echo 'class="${test.classes}"' | npx prettier-v4 --stdin-filepath=test.html --plugin=prettier-plugin-tailwindcss-v4 2>/dev/null`,
      { encoding: 'utf8', cwd: '/home/user/rustywind/tests/fuzz' }
    ).trim();

    // Sort with RustyWind
    const rustywind = execSync(
      `echo '${test.classes}' | /home/user/rustywind/target/release/rustywind --stdin`,
      { encoding: 'utf8' }
    ).trim();

    // Extract class list from prettier output
    const prettierMatch = prettier.match(/class="([^"]+)"/);
    const prettierClasses = prettierMatch ? prettierMatch[1] : prettier;

    console.log(`Prettier: ${prettierClasses}`);
    console.log(`RustyWind: ${rustywind}`);

    if (prettierClasses === rustywind) {
      console.log('✅ MATCH');
    } else {
      console.log('❌ MISMATCH');

      // Show differences
      const pClasses = prettierClasses.split(/\s+/);
      const rClasses = rustywind.split(/\s+/);

      console.log('\nDifferences:');
      const maxLen = Math.max(pClasses.length, rClasses.length);
      for (let i = 0; i < maxLen; i++) {
        const p = pClasses[i] || '(missing)';
        const r = rClasses[i] || '(missing)';
        if (p !== r) {
          console.log(`  [${i}] Prettier: ${p.padEnd(25)} RustyWind: ${r}`);
        }
      }
    }
  } catch (error) {
    console.error(`Error: ${error.message}`);
  }
}

console.log('\n' + '='.repeat(80));
console.log('Done!');
console.log('='.repeat(80));

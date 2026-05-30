#!/usr/bin/env node
/**
 * Test specific failure cases to understand root cause
 */

import prettier from 'prettier';
import { execSync } from 'child_process';

async function sortWithPrettier(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;
  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match = formatted.match(/class="([^"]*)"/);
  return match[1].split(/\s+/).filter(c => c.length > 0);
}

function sortWithRustyWind(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;
  const result = execSync(
    `echo '${html}' | /home/user/rustywind/target/release/rustywind --stdin`,
    { encoding: 'utf8' }
  ).trim();
  const match = result.match(/class="([^"]*)"/);
  return match[1].split(/\s+/).filter(c => c.length > 0);
}

const testCases = [
  {
    name: 'Issue 1: saturate vs ring-inset',
    classes: ['saturate-50', 'ring-inset', 'p-4'],
  },
  {
    name: 'Issue 2: backdrop-saturate vs ring-inset',
    classes: ['backdrop-saturate-150', 'ring-inset', 'p-4'],
  },
  {
    name: 'Issue 3: peer vs group variant ordering',
    classes: ['peer:touch-none', 'group:translate-y-4', 'p-4'],
  },
  {
    name: 'Issue 4: Complex variant ordering',
    classes: ['even:group:overscroll-x-auto', 'peer:ease-linear', 'p-4'],
  },
  {
    name: 'Issue 5: group:decoration vs from-gradient',
    classes: ['group:decoration-solid', 'from-stroke/0', 'p-4'],
  },
  {
    name: 'Issue 6: group:visited vs group:indent',
    classes: ['group:visited:pl-0', 'group:indent-0', 'p-4'],
  },
];

console.log('🔍 Testing Specific Failure Patterns\n');
console.log('='.repeat(80));

for (const test of testCases) {
  console.log(`\n${test.name}`);
  console.log('-'.repeat(80));

  const prettierSorted = await sortWithPrettier(test.classes);
  const rustywindSorted = sortWithRustyWind(test.classes);

  console.log(`Input:     ${test.classes.join(' ')}`);
  console.log(`Prettier:  ${prettierSorted.join(' ')}`);
  console.log(`RustyWind: ${rustywindSorted.join(' ')}`);

  if (prettierSorted.join(' ') === rustywindSorted.join(' ')) {
    console.log('✅ MATCH');
  } else {
    console.log('❌ MISMATCH');

    // Find the specific difference
    for (let i = 0; i < Math.max(prettierSorted.length, rustywindSorted.length); i++) {
      if (prettierSorted[i] !== rustywindSorted[i]) {
        console.log(`   Position ${i}: Prettier="${prettierSorted[i]}" vs RustyWind="${rustywindSorted[i]}"`);
        break;
      }
    }
  }
}

console.log('\n' + '='.repeat(80));
console.log('Done!');

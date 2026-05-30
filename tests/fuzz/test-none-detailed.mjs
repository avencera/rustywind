import prettier from 'prettier';
import * as prettierPlugin from 'prettier-plugin-tailwindcss';

async function test(classes) {
  const html = `<div class="${classes}"></div>`;
  const sorted = await prettier.format(html, {
    parser: 'html',
    plugins: [prettierPlugin],
  });
  const match = sorted.match(/class="([^"]*)"/);
  return match ? match[1] : classes;
}

console.log('DETAILED ANALYSIS OF -none UTILITY SORTING\n');
console.log('='.repeat(80));
console.log();

// Test each utility with comprehensive value combinations
const testGroups = [
  {
    category: 'FILTERS - blur',
    tests: [
      'blur-none blur-sm',
      'blur-sm blur-none',
      'blur-none blur-md',
      'blur-md blur-none',
      'blur-none blur-lg',
      'blur-lg blur-none',
      'blur-sm blur-md',
      'blur-sm blur-md blur-lg',
      'blur-none blur-sm blur-md blur-lg',
    ]
  },
  {
    category: 'FILTERS - drop-shadow',
    tests: [
      'drop-shadow-none drop-shadow-sm',
      'drop-shadow-sm drop-shadow-none',
      'drop-shadow-none drop-shadow-md',
      'drop-shadow-md drop-shadow-none',
      'drop-shadow-none drop-shadow-lg',
      'drop-shadow-lg drop-shadow-none',
      'drop-shadow-none drop-shadow-xl',
      'drop-shadow-xl drop-shadow-none',
      'drop-shadow-sm drop-shadow-md drop-shadow-lg drop-shadow-xl',
      'drop-shadow-none drop-shadow-sm drop-shadow-md drop-shadow-lg drop-shadow-xl',
    ]
  },
  {
    category: 'FILTERS - grayscale',
    tests: [
      'grayscale-0 grayscale',
      'grayscale grayscale-0',
    ]
  },
  {
    category: 'SHADOWS - shadow',
    tests: [
      'shadow-none shadow-sm',
      'shadow-sm shadow-none',
      'shadow-none shadow-md',
      'shadow-md shadow-none',
      'shadow-none shadow-lg',
      'shadow-lg shadow-none',
      'shadow-none shadow-xl',
      'shadow-xl shadow-none',
      'shadow-sm shadow-md shadow-lg shadow-xl',
      'shadow-none shadow-sm shadow-md shadow-lg shadow-xl',
    ]
  },
  {
    category: 'BORDERS - rounded',
    tests: [
      'rounded-none rounded-sm',
      'rounded-sm rounded-none',
      'rounded-none rounded-md',
      'rounded-md rounded-none',
      'rounded-none rounded-lg',
      'rounded-lg rounded-none',
      'rounded-none rounded-xl',
      'rounded-xl rounded-none',
      'rounded-sm rounded-md rounded-lg rounded-xl',
      'rounded-none rounded-sm rounded-md rounded-lg rounded-xl',
    ]
  },
  {
    category: 'BORDERS - border width',
    tests: [
      'border-0 border',
      'border border-0',
      'border-0 border-2',
      'border-2 border-0',
      'border-0 border-4',
      'border-4 border-0',
      'border-0 border-8',
      'border-8 border-0',
      'border border-2 border-4 border-8',
      'border-0 border border-2 border-4 border-8',
    ]
  },
  {
    category: 'TRANSITIONS - transition',
    tests: [
      'transition-none transition-all',
      'transition-all transition-none',
      'transition-none transition-colors',
      'transition-colors transition-none',
      'transition-none transition-opacity',
      'transition-opacity transition-none',
      'transition-all transition-colors transition-opacity',
      'transition-none transition-all transition-colors transition-opacity',
    ]
  },
  {
    category: 'ANIMATIONS - animate',
    tests: [
      'animate-none animate-spin',
      'animate-spin animate-none',
      'animate-none animate-ping',
      'animate-ping animate-none',
      'animate-none animate-pulse',
      'animate-pulse animate-none',
      'animate-none animate-bounce',
      'animate-bounce animate-none',
      'animate-spin animate-ping animate-pulse animate-bounce',
      'animate-none animate-spin animate-ping animate-pulse animate-bounce',
    ]
  },
  {
    category: 'NUMERIC - brightness',
    tests: [
      'brightness-0 brightness-50',
      'brightness-50 brightness-0',
      'brightness-0 brightness-100',
      'brightness-100 brightness-0',
      'brightness-0 brightness-150',
      'brightness-150 brightness-0',
      'brightness-50 brightness-100 brightness-150',
      'brightness-0 brightness-50 brightness-100 brightness-150',
    ]
  },
  {
    category: 'NUMERIC - scale',
    tests: [
      'scale-0 scale-50',
      'scale-50 scale-0',
      'scale-0 scale-100',
      'scale-100 scale-0',
      'scale-0 scale-150',
      'scale-150 scale-0',
      'scale-50 scale-100 scale-150',
      'scale-0 scale-50 scale-100 scale-150',
    ]
  },
];

const patterns = {};

for (const group of testGroups) {
  console.log(`\n${group.category}`);
  console.log('-'.repeat(80));

  for (const input of group.tests) {
    const output = await test(input);
    console.log(`  ${input}`);
    console.log(`  → ${output}`);

    // Track if none/0 moved
    const inputArr = input.split(' ');
    const outputArr = output.split(' ');
    const noneClass = inputArr.find(c => c.endsWith('-none') || c.endsWith('-0'));

    if (noneClass) {
      const noneInIdx = inputArr.indexOf(noneClass);
      const noneOutIdx = outputArr.indexOf(noneClass);

      if (noneInIdx !== noneOutIdx) {
        console.log(`     (${noneClass} moved from pos ${noneInIdx} to ${noneOutIdx})`);
      }
    }
  }
}

console.log('\n' + '='.repeat(80));
console.log('\n🔍 PATTERN ANALYSIS:\n');

// Now let's analyze the pattern more carefully
console.log('Testing specific hypotheses:\n');

// Hypothesis 1: -none always sorts after specific size values
console.log('H1: Does -none sort AFTER certain sizes (like -md, -lg, -xl)?');
const h1Tests = [
  ['blur-none blur-sm', 'If -none comes first, it might sort before -sm'],
  ['blur-none blur-md', 'If -none comes last, it sorts after -md'],
  ['shadow-none shadow-sm', 'Testing with shadow'],
  ['shadow-none shadow-md', 'Testing with shadow'],
  ['rounded-none rounded-sm', 'Testing with rounded'],
  ['rounded-none rounded-md', 'Testing with rounded'],
];

for (const [input, note] of h1Tests) {
  const output = await test(input);
  const noneFirst = output.startsWith(input.split(' ').find(c => c.includes('-none')));
  console.log(`  ${input} → ${output} ${noneFirst ? '(-none first ✗)' : '(-none last ✓)'}`);
}
console.log();

// Hypothesis 2: -0 always sorts alphabetically with numbers
console.log('H2: Does -0 sort alphabetically/numerically with other numbers?');
const h2Tests = [
  ['brightness-0 brightness-50', 'brightness'],
  ['scale-0 scale-50', 'scale'],
  ['rotate-0 rotate-45', 'rotate'],
  ['border-0 border-2', 'border (numeric)'],
  ['border-0 border', 'border (vs default)'],
];

for (const [input, note] of h2Tests) {
  const output = await test(input);
  const same = input === output;
  console.log(`  ${input} → ${output} ${same ? '(stays same ✓)' : '(reordered ✗)'} [${note}]`);
}
console.log();

// Hypothesis 3: Check if it's about size scale order
console.log('H3: What is the size scale ordering pattern?');
const h3Tests = [
  'blur-sm blur-md blur-lg blur-xl blur-2xl blur-3xl',
  'shadow-sm shadow-md shadow-lg shadow-xl shadow-2xl',
  'rounded-sm rounded-md rounded-lg rounded-xl rounded-2xl rounded-3xl',
];

for (const input of h3Tests) {
  const output = await test(input);
  console.log(`  ${input}`);
  console.log(`  → ${output}`);
  console.log(`     ${input === output ? '(no change - alphabetical)' : '(REORDERED!)'}`);
}
console.log();

// Hypothesis 4: Where does -none fit in the full scale?
console.log('H4: Where does -none fit in the complete size scale?');
const h4Tests = [
  'blur-none blur-sm blur-md blur-lg blur-xl blur-2xl blur-3xl',
  'shadow-none shadow-sm shadow-md shadow-lg shadow-xl shadow-2xl',
  'rounded-none rounded-sm rounded-md rounded-lg rounded-xl rounded-2xl rounded-3xl',
];

for (const input of h4Tests) {
  const output = await test(input);
  const outputArr = output.split(' ');
  const noneIdx = outputArr.findIndex(c => c.includes('-none'));
  console.log(`  ${output}`);
  console.log(`     -none is at position ${noneIdx} (0-indexed)`);
}
console.log();

console.log('='.repeat(80));

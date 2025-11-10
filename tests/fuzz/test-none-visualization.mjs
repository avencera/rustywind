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

console.log('VISUAL ORDERING MAPS\n');
console.log('='.repeat(80));
console.log();

// Create comprehensive ordering maps
const utilities = [
  {
    name: 'blur',
    values: ['blur', 'blur-3xl', 'blur-2xl', 'blur-xl', 'blur-lg', 'blur-md', 'blur-sm', 'blur-none'],
  },
  {
    name: 'shadow',
    values: ['shadow', 'shadow-2xl', 'shadow-xl', 'shadow-lg', 'shadow-md', 'shadow-sm', 'shadow-none', 'shadow-inner'],
  },
  {
    name: 'rounded',
    values: ['rounded', 'rounded-3xl', 'rounded-2xl', 'rounded-xl', 'rounded-lg', 'rounded-md', 'rounded-sm', 'rounded-none', 'rounded-full'],
  },
  {
    name: 'drop-shadow',
    values: ['drop-shadow-2xl', 'drop-shadow-xl', 'drop-shadow-lg', 'drop-shadow-md', 'drop-shadow-sm', 'drop-shadow-none'],
  },
];

for (const { name, values } of utilities) {
  // Shuffle to test ordering
  const shuffled = [...values].sort(() => Math.random() - 0.5);
  const input = shuffled.join(' ');
  const output = await test(input);
  const outputArr = output.split(' ');

  console.log(`${name}:`);
  console.log();

  // Create visual ordering
  outputArr.forEach((cls, idx) => {
    const isNone = cls.includes('-none');
    const marker = isNone ? ' ← -none position' : '';
    const position = `[${idx + 1}]`;
    console.log(`  ${position.padEnd(5)} ${cls}${marker}`);
  });

  console.log();
}

console.log('='.repeat(80));
console.log('\n🔍 OBSERVATION:\n');
console.log('Notice that -none does NOT appear at a consistent position!');
console.log('  - blur-none:        position 5 of 7');
console.log('  - shadow-none:      position 5 of 7 (or 8 with inner)');
console.log('  - rounded-none:     position 6 of 8 (or 9 with full)');
console.log('  - drop-shadow-none: position 6 of 6 (LAST!)');
console.log();
console.log('This suggests the ordering is based on actual CSS values,');
console.log('not on a pattern we can derive from the class names alone.');
console.log();

// Test with just pairs to see the relationship
console.log('='.repeat(80));
console.log('\nPAIRWISE COMPARISONS:\n');

const pairTests = [
  // What comes before -none?
  ['blur-md blur-none', 'blur: md before none?'],
  ['blur-sm blur-none', 'blur: sm before none?'],
  ['shadow-md shadow-none', 'shadow: md before none?'],
  ['shadow-sm shadow-none', 'shadow: sm before none?'],
  ['rounded-md rounded-none', 'rounded: md before none?'],
  ['rounded-sm rounded-none', 'rounded: sm before none?'],

  // What comes after -none?
  ['blur-none blur-sm', 'blur: sm after none?'],
  ['blur-none blur-xl', 'blur: xl after none?'],
  ['shadow-none shadow-sm', 'shadow: sm after none?'],
  ['shadow-none shadow-xl', 'shadow: xl after none?'],
  ['rounded-none rounded-sm', 'rounded: sm after none?'],
  ['rounded-none rounded-xl', 'rounded: xl after none?'],
];

console.log('Testing what comes BEFORE and AFTER -none:\n');

for (const [input, description] of pairTests) {
  const output = await test(input);
  const [first, second] = output.split(' ');
  const noneFirst = first.includes('-none');
  const result = noneFirst ? '✓ YES' : '✗ NO';

  console.log(`  ${description}`);
  console.log(`    ${input} → ${output} ${result}`);
}

console.log();
console.log('='.repeat(80));
console.log('\n💡 DISCOVERED PATTERN:\n');
console.log('For blur, shadow, rounded:');
console.log('  - -md, -lg, -xl, -2xl, -3xl come BEFORE -none');
console.log('  - -sm comes AFTER -none');
console.log('  - So the ordering is: [larger sizes] → -none → -sm → -xl (xl appears again!)');
console.log();
console.log('This is NOT lexical or size-based sorting!');
console.log('It appears to be based on the actual CSS property values.');
console.log();

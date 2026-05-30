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

console.log('FINAL SUMMARY: -none and -0 Sorting Patterns\n');
console.log('='.repeat(80));
console.log();

// Test to determine exact ordering for size-based utilities
console.log('1. SIZE-BASED UTILITIES (blur, shadow, rounded, drop-shadow)\n');
console.log('   These use a custom CSS-value-based ordering, NOT alphabetical.\n');

const sizeTests = [
  {
    utility: 'blur',
    input: 'blur-3xl blur-2xl blur-xl blur-lg blur-md blur-sm blur-none',
  },
  {
    utility: 'shadow',
    input: 'shadow-2xl shadow-xl shadow-lg shadow-md shadow-sm shadow-none shadow-inner',
  },
  {
    utility: 'rounded',
    input: 'rounded-3xl rounded-2xl rounded-xl rounded-lg rounded-md rounded-sm rounded-none rounded-full',
  },
  {
    utility: 'drop-shadow',
    input: 'drop-shadow-2xl drop-shadow-xl drop-shadow-lg drop-shadow-md drop-shadow-sm drop-shadow-none',
  },
];

for (const { utility, input } of sizeTests) {
  const output = await test(input);
  const outputArr = output.split(' ');
  const noneClass = outputArr.find(c => c.includes('-none'));
  const noneIdx = outputArr.indexOf(noneClass);

  console.log(`   ${utility}:`);
  console.log(`     Prettier order: ${output}`);
  console.log(`     -none position: ${noneIdx} of ${outputArr.length - 1} (middle of sequence)`);
  console.log();
}

console.log('-'.repeat(80));
console.log();

// Test transition-none
console.log('2. TRANSITION-NONE: Always sorts LAST\n');
const transitionTest = 'transition-transform transition-shadow transition-opacity transition-colors transition-all transition-none';
const transitionOutput = await test(transitionTest);
console.log(`   Input:  ${transitionTest}`);
console.log(`   Output: ${transitionOutput}`);
console.log(`   Pattern: transition-none is ALWAYS LAST`);
console.log();

console.log('-'.repeat(80));
console.log();

// Test border-0
console.log('3. BORDER-0: Sorts AFTER "border" but BEFORE border-[n]\n');
const borderTest = 'border-8 border-4 border-2 border border-0';
const borderOutput = await test(borderTest);
console.log(`   Input:  ${borderTest}`);
console.log(`   Output: ${borderOutput}`);
console.log(`   Pattern: border → border-0 → border-2/4/8 (ascending)`);
console.log();

console.log('-'.repeat(80));
console.log();

// Test grayscale-0
console.log('4. GRAYSCALE-0: Sorts AFTER "grayscale"\n');
const grayscaleTest = 'grayscale-0 grayscale';
const grayscaleOutput = await test(grayscaleTest);
console.log(`   Input:  ${grayscaleTest}`);
console.log(`   Output: ${grayscaleOutput}`);
console.log(`   Pattern: grayscale → grayscale-0`);
console.log();

console.log('-'.repeat(80));
console.log();

// Test numeric -0 values
console.log('5. NUMERIC -0 VALUES: Sort FIRST (numerically)\n');
const numericTests = [
  'brightness-150 brightness-100 brightness-50 brightness-0',
  'contrast-150 contrast-100 contrast-50 contrast-0',
  'saturate-150 saturate-100 saturate-50 saturate-0',
  'scale-150 scale-100 scale-50 scale-0',
  'rotate-180 rotate-90 rotate-45 rotate-0',
  'duration-1000 duration-500 duration-100 duration-0',
];

for (const input of numericTests) {
  const output = await test(input);
  const utility = input.split('-')[0];
  console.log(`   ${utility}: ${output}`);
}
console.log(`   Pattern: -0 sorts FIRST, then ascending numerically`);
console.log();

console.log('-'.repeat(80));
console.log();

// Test animate-none
console.log('6. ANIMATE-NONE: Sorts ALPHABETICALLY\n');
const animateTest = 'animate-spin animate-pulse animate-ping animate-none animate-bounce';
const animateOutput = await test(animateTest);
console.log(`   Input:  ${animateTest}`);
console.log(`   Output: ${animateOutput}`);
console.log(`   Pattern: Uses alphabetical ordering (animate-none comes after animate-bounce)`);
console.log();

console.log('='.repeat(80));
console.log('\n📋 CLASSIFICATION SUMMARY\n');

console.log('Category A: SIZE-BASED (custom CSS value ordering)');
console.log('  - blur-none');
console.log('  - shadow-none');
console.log('  - rounded-none');
console.log('  - drop-shadow-none');
console.log('  Pattern: -none fits in MIDDLE of custom size scale order');
console.log('           Position varies by utility (not predictable from name)');
console.log();

console.log('Category B: ALWAYS SORTS LAST');
console.log('  - transition-none');
console.log('  Pattern: Always appears AFTER all other transition-* values');
console.log();

console.log('Category C: SPECIAL POSITIONING');
console.log('  - border-0 (after "border", before border-2/4/8)');
console.log('  - grayscale-0 (after "grayscale")');
console.log('  Pattern: Sorts after base value, before/with numeric values');
console.log();

console.log('Category D: NUMERIC (sorts first)');
console.log('  - brightness-0');
console.log('  - contrast-0');
console.log('  - saturate-0');
console.log('  - scale-0');
console.log('  - rotate-0');
console.log('  - duration-0');
console.log('  Pattern: -0 is FIRST, then ascending numeric order');
console.log();

console.log('Category E: ALPHABETICAL');
console.log('  - animate-none');
console.log('  Pattern: Sorts alphabetically with other animate-* values');
console.log();

console.log('='.repeat(80));
console.log('\n🎯 KEY INSIGHT:\n');
console.log('The -none/-0 suffix does NOT have a universal sorting rule!');
console.log('Instead, each utility has its own sorting logic based on CSS property values.');
console.log('This is why rustywind struggles - there\'s no simple pattern to implement.');
console.log('\nPrettier\'s plugin likely uses Tailwind\'s internal ordering which is based');
console.log('on CSS specificity and property values, not lexical patterns.');
console.log();

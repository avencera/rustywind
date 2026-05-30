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

// Helper to determine if -none/-0 sorts last
function analyzeSorting(input, output) {
  const inputClasses = input.split(' ');
  const outputClasses = output.split(' ');

  // Find the -none or -0 class
  const noneClass = inputClasses.find(c => c.includes('-none') || c.includes('-0'));
  const noneIndexInInput = inputClasses.indexOf(noneClass);
  const noneIndexInOutput = outputClasses.indexOf(noneClass);

  // Check if it moved to the end
  const sortedLast = noneIndexInOutput === outputClasses.length - 1;
  const stayedInPlace = noneIndexInInput === noneIndexInOutput;
  const movedEarlier = noneIndexInOutput < noneIndexInInput;
  const movedLater = noneIndexInOutput > noneIndexInInput;

  return { sortedLast, stayedInPlace, movedEarlier, movedLater, noneClass, noneIndexInInput, noneIndexInOutput };
}

const tests = [
  // 1. Filters
  ['blur-none blur-sm', 'Filters: blur-none vs blur-sm'],
  ['blur-sm blur-none', 'Filters: blur-sm vs blur-none (reversed)'],
  ['blur-none blur-md', 'Filters: blur-none vs blur-md'],
  ['blur-none blur-sm blur-md', 'Filters: blur-none vs blur-sm vs blur-md'],

  ['brightness-0 brightness-50', 'Filters: brightness-0 vs brightness-50'],
  ['brightness-50 brightness-0', 'Filters: brightness-50 vs brightness-0 (reversed)'],
  ['brightness-0 brightness-50 brightness-100', 'Filters: brightness-0 vs brightness-50 vs brightness-100'],

  ['contrast-0 contrast-50', 'Filters: contrast-0 vs contrast-50'],
  ['contrast-50 contrast-0', 'Filters: contrast-50 vs contrast-0 (reversed)'],

  ['drop-shadow-none drop-shadow-xl', 'Filters: drop-shadow-none vs drop-shadow-xl'],
  ['drop-shadow-xl drop-shadow-none', 'Filters: drop-shadow-xl vs drop-shadow-none (reversed)'],

  ['grayscale-0 grayscale', 'Filters: grayscale-0 vs grayscale'],
  ['grayscale grayscale-0', 'Filters: grayscale vs grayscale-0 (reversed)'],

  ['saturate-0 saturate-50', 'Filters: saturate-0 vs saturate-50'],
  ['saturate-50 saturate-0', 'Filters: saturate-50 vs saturate-0 (reversed)'],

  // 2. Borders/Shadows
  ['shadow-none shadow-sm', 'Borders/Shadows: shadow-none vs shadow-sm'],
  ['shadow-sm shadow-none', 'Borders/Shadows: shadow-sm vs shadow-none (reversed)'],
  ['shadow-none shadow-md', 'Borders/Shadows: shadow-none vs shadow-md'],
  ['shadow-none shadow-sm shadow-md', 'Borders/Shadows: shadow-none vs shadow-sm vs shadow-md'],

  ['rounded-none rounded-sm', 'Borders/Shadows: rounded-none vs rounded-sm'],
  ['rounded-sm rounded-none', 'Borders/Shadows: rounded-sm vs rounded-none (reversed)'],
  ['rounded-none rounded-md', 'Borders/Shadows: rounded-none vs rounded-md'],
  ['rounded-none rounded-sm rounded-md', 'Borders/Shadows: rounded-none vs rounded-sm vs rounded-md'],

  ['border-0 border', 'Borders/Shadows: border-0 vs border'],
  ['border border-0', 'Borders/Shadows: border vs border-0 (reversed)'],
  ['border-0 border-2', 'Borders/Shadows: border-0 vs border-2'],
  ['border-0 border border-2', 'Borders/Shadows: border-0 vs border vs border-2'],

  // 3. Transitions/Animations
  ['transition-none transition-all', 'Transitions: transition-none vs transition-all'],
  ['transition-all transition-none', 'Transitions: transition-all vs transition-none (reversed)'],
  ['transition-none transition-colors', 'Transitions: transition-none vs transition-colors'],
  ['transition-none transition-all transition-colors', 'Transitions: transition-none vs transition-all vs transition-colors'],

  ['animate-none animate-spin', 'Animations: animate-none vs animate-spin'],
  ['animate-spin animate-none', 'Animations: animate-spin vs animate-none (reversed)'],

  ['duration-0 duration-100', 'Transitions: duration-0 vs duration-100'],
  ['duration-100 duration-0', 'Transitions: duration-100 vs duration-0 (reversed)'],

  // 4. Layout/Transforms
  ['scale-0 scale-50', 'Layout: scale-0 vs scale-50'],
  ['scale-50 scale-0', 'Layout: scale-50 vs scale-0 (reversed)'],
  ['scale-0 scale-50 scale-100', 'Layout: scale-0 vs scale-50 vs scale-100'],

  ['rotate-0 rotate-45', 'Layout: rotate-0 vs rotate-45'],
  ['rotate-45 rotate-0', 'Layout: rotate-45 vs rotate-0 (reversed)'],
];

console.log('Testing -none and -0 utility sorting patterns\n');
console.log('='.repeat(80));
console.log();

const results = {
  sortsLast: [],
  alphabetical: [],
  unclear: []
};

for (const [input, name] of tests) {
  const output = await test(input);
  const analysis = analyzeSorting(input, output);

  console.log(`${name}:`);
  console.log(`  Input:   ${input}`);
  console.log(`  Output:  ${output}`);
  console.log(`  Analysis: ${analysis.noneClass} at position ${analysis.noneIndexInInput} → ${analysis.noneIndexInOutput}`);

  if (analysis.sortedLast) {
    console.log(`  ✓ Pattern: SORTS LAST (moved to end)`);
    results.sortsLast.push({ name, input, output, analysis });
  } else if (analysis.stayedInPlace) {
    console.log(`  → Pattern: ALPHABETICAL (stayed in place)`);
    results.alphabetical.push({ name, input, output, analysis });
  } else if (analysis.movedEarlier) {
    console.log(`  ← Pattern: ALPHABETICAL (moved earlier)`);
    results.alphabetical.push({ name, input, output, analysis });
  } else if (analysis.movedLater) {
    console.log(`  → Pattern: UNCLEAR (moved later but not to end)`);
    results.unclear.push({ name, input, output, analysis });
  }

  console.log();
}

console.log('='.repeat(80));
console.log('\n📊 SUMMARY\n');

console.log(`✓ Utilities where -none/-0 SORTS LAST (${results.sortsLast.length}):`);
if (results.sortsLast.length > 0) {
  const utilities = [...new Set(results.sortsLast.map(r => r.analysis.noneClass.split('-')[0]))];
  console.log('  ' + utilities.join(', '));
  results.sortsLast.forEach(r => {
    console.log(`    - ${r.analysis.noneClass}`);
  });
} else {
  console.log('  (none)');
}
console.log();

console.log(`→ Utilities where -none/-0 sorts ALPHABETICALLY (${results.alphabetical.length}):`);
if (results.alphabetical.length > 0) {
  const utilities = [...new Set(results.alphabetical.map(r => r.analysis.noneClass.split('-')[0]))];
  console.log('  ' + utilities.join(', '));
  results.alphabetical.forEach(r => {
    console.log(`    - ${r.analysis.noneClass}`);
  });
} else {
  console.log('  (none)');
}
console.log();

console.log(`? Unclear patterns (${results.unclear.length}):`);
if (results.unclear.length > 0) {
  results.unclear.forEach(r => {
    console.log(`    - ${r.analysis.noneClass}: ${r.input} → ${r.output}`);
  });
} else {
  console.log('  (none)');
}
console.log();

console.log('='.repeat(80));
console.log('\n🔍 KEY FINDINGS:\n');

const groupedResults = {};
results.sortsLast.forEach(r => {
  const utility = r.analysis.noneClass;
  if (!groupedResults[utility]) groupedResults[utility] = 'SORTS_LAST';
});
results.alphabetical.forEach(r => {
  const utility = r.analysis.noneClass;
  if (!groupedResults[utility]) groupedResults[utility] = 'ALPHABETICAL';
});

Object.entries(groupedResults).forEach(([utility, pattern]) => {
  console.log(`  ${utility}: ${pattern}`);
});

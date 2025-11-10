/**
 * Fuzz test: Compare RustyWind's output with Prettier's Tailwind plugin
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { allClasses, variants } from './tailwind-classes.js';
import { filterLegacyClasses, isLegacyClass } from './legacy-classes.js';
import prettier from 'prettier';
import seedrandom from 'seedrandom';

const execAsync = promisify(exec);

// Configuration
const NUM_TESTS = 100; // Number of random class combinations to test
const MIN_CLASSES = 5;
const MAX_CLASSES = 30;
const VARIANT_PROBABILITY = 0.3; // 30% chance of adding a variant
const FILTER_LEGACY = process.env.FILTER_LEGACY !== 'false'; // Filter legacy classes by default

// Seed configuration for deterministic testing
const SEED = process.env.FUZZ_SEED || Math.random().toString(36).substring(2, 15);
const rng = seedrandom(SEED);

// Filter classes if needed
const classPool = FILTER_LEGACY ? filterLegacyClasses(allClasses) : allClasses;

/**
 * Generate a random integer between min and max (inclusive)
 */
function randomInt(min, max) {
  return Math.floor(rng() * (max - min + 1)) + min;
}

/**
 * Pick a random element from an array
 */
function randomPick(array) {
  return array[randomInt(0, array.length - 1)];
}

/**
 * Generate a random Tailwind class, possibly with variant(s)
 */
function generateRandomClass() {
  let className = randomPick(classPool);

  // Maybe add a variant (30% chance)
  if (rng() < VARIANT_PROBABILITY) {
    const variant = randomPick(variants);
    className = `${variant}:${className}`;

    // Small chance (10%) of adding a second variant
    if (rng() < 0.1) {
      const variant2 = randomPick(variants);
      className = `${variant2}:${className}`;
    }
  }

  return className;
}

/**
 * Generate a list of random Tailwind classes
 */
function generateRandomClasses(count) {
  const classes = [];
  for (let i = 0; i < count; i++) {
    classes.push(generateRandomClass());
  }
  return classes;
}

/**
 * Sort classes using Prettier with prettier-plugin-tailwindcss
 */
async function sortWithPrettier(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;

  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000, // Prevent line wrapping
  });

  // Extract the sorted classes from the formatted HTML
  const match = formatted.match(/class="([^"]*)"/);
  if (!match) {
    throw new Error('Could not extract classes from Prettier output');
  }

  return match[1].split(/\s+/).filter(c => c.length > 0);
}

/**
 * Sort classes using RustyWind
 */
async function sortWithRustyWind(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;

  // Run RustyWind with stdin
  const rustywindBin = '../../target/release/rustywind';
  const { stdout } = await execAsync(`echo '${html.replace(/'/g, "'\\''")}' | ${rustywindBin} --stdin`);

  // Extract sorted classes
  const match = stdout.trim().match(/class="([^"]*)"/);
  if (!match) {
    throw new Error('Could not extract classes from RustyWind output');
  }

  return match[1].split(/\s+/).filter(c => c.length > 0);
}

/**
 * Compare two arrays of classes
 */
function compareClasses(prettier, rustywind, original) {
  if (prettier.length !== rustywind.length) {
    return {
      match: false,
      reason: `Different lengths: Prettier=${prettier.length}, RustyWind=${rustywind.length}`,
      prettier,
      rustywind,
      original,
    };
  }

  for (let i = 0; i < prettier.length; i++) {
    if (prettier[i] !== rustywind[i]) {
      return {
        match: false,
        reason: `Mismatch at position ${i}: Prettier="${prettier[i]}", RustyWind="${rustywind[i]}"`,
        prettier,
        rustywind,
        original,
      };
    }
  }

  return { match: true };
}

/**
 * Run the fuzz test
 */
async function runFuzzTest() {
  console.log(`\n🧪 Starting fuzz test with ${NUM_TESTS} random class combinations...`);
  console.log(`🎲 Seed: ${SEED} (set FUZZ_SEED env var to reproduce)`);
  console.log(`📋 Class pool: ${classPool.length} classes (${FILTER_LEGACY ? 'legacy classes filtered' : 'including legacy classes'})\n`);

  let passed = 0;
  let failed = 0;
  const failures = [];

  for (let i = 0; i < NUM_TESTS; i++) {
    const numClasses = randomInt(MIN_CLASSES, MAX_CLASSES);
    const classes = generateRandomClasses(numClasses);

    try {
      const prettierSorted = await sortWithPrettier(classes);
      const rustywindSorted = await sortWithRustyWind(classes);

      const comparison = compareClasses(prettierSorted, rustywindSorted, classes);

      if (comparison.match) {
        passed++;
        process.stdout.write('.');
      } else {
        failed++;
        failures.push({ test: i + 1, ...comparison });
        process.stdout.write('F');
      }

      // Print progress every 10 tests
      if ((i + 1) % 10 === 0) {
        process.stdout.write(` ${i + 1}/${NUM_TESTS}\n`);
      }
    } catch (error) {
      failed++;
      failures.push({
        test: i + 1,
        error: error.message,
        original: classes,
      });
      process.stdout.write('E');
    }
  }

  console.log('\n');
  console.log('='.repeat(80));
  console.log(`\n📊 Results: ${passed} passed, ${failed} failed (${(passed / NUM_TESTS * 100).toFixed(1)}% pass rate)`);
  console.log(`🎲 Seed: ${SEED}\n`);

  if (failures.length > 0) {
    console.log('❌ Failures:\n');
    console.log(`To reproduce these failures, run: FUZZ_SEED=${SEED} npm test\n`);
    failures.forEach(({ test, reason, prettier, rustywind, original, error }) => {
      console.log(`Test #${test}:`);
      if (error) {
        console.log(`  Error: ${error}`);
        console.log(`  Original: ${original ? original.join(' ') : 'N/A'}`);
      } else {
        console.log(`  ${reason}`);
        console.log(`  Original:  [${original.join(', ')}]`);
        console.log(`  Prettier:  [${prettier.join(', ')}]`);
        console.log(`  RustyWind: [${rustywind.join(', ')}]`);
      }
      console.log('');
    });

    process.exit(1);
  } else {
    console.log('✅ All tests passed!');
    process.exit(0);
  }
}

// Run the test
runFuzzTest().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

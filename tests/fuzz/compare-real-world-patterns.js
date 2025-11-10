/**
 * Enhanced Fuzz test: Generate class combinations using real-world patterns
 *
 * This test uses patterns extracted from actual project files to generate
 * realistic class combinations, catching issues that pure random generation might miss.
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { allClasses, variants } from './tailwind-classes.js';
import { filterLegacyClasses } from './legacy-classes.js';
import { readFileSync } from 'fs';
import prettier from 'prettier';
import seedrandom from 'seedrandom';

const execAsync = promisify(exec);

// Load failure patterns
const failurePatterns = JSON.parse(readFileSync('./failure-patterns.json', 'utf8'));

// Configuration
const NUM_TESTS = 100; // Number of test cases
const FILTER_LEGACY = process.env.FILTER_LEGACY !== 'false';
const SEED = process.env.FUZZ_SEED || Math.random().toString(36).substring(2, 15);
const rng = seedrandom(SEED);

// Use classes that appear in FAILURES (70% of the time)
// And general class pool (30% of the time) for variety
const failingClasses = failurePatterns.failingClasses;
const classPool = FILTER_LEGACY ? filterLegacyClasses(allClasses) : allClasses;

// Extract modifiers that appear in failures
const failingModifiers = failurePatterns.failingModifiers;

// Extract class pairs that appear in failures
const failingPairs = failurePatterns.failingPairs;

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
 * Pick a random number of classes based on failure patterns
 * Failures tend to have more classes (avg 7.5 vs 5)
 */
function pickRealisticClassCount() {
  // Use failure average (7.5) with some variance
  const avg = failurePatterns.avgFailureClassCount;
  const variance = 4;
  return Math.max(3, Math.min(25, Math.round(avg + (rng() - 0.5) * variance * 2)));
}

/**
 * Generate classes using FAILURE patterns
 * This should produce more realistic failures
 */
function generateRealWorldClasses() {
  const count = pickRealisticClassCount();
  const classes = [];

  // 60% chance to start with a failing pair (higher than before)
  if (rng() < 0.6 && failingPairs.length > 0) {
    const pair = randomPick(failingPairs);
    classes.push(pair[0], pair[1]);
  }

  // Fill remaining with classes, preferring those that appear in failures
  while (classes.length < count) {
    let className;

    // 70% chance to pick from failing classes, 30% from general pool
    if (rng() < 0.7 && failingClasses.length > 0) {
      className = randomPick(failingClasses);
    } else {
      className = randomPick(classPool);

      // 50% chance of adding a failure-prone modifier
      if (rng() < 0.5 && failingModifiers.length > 0) {
        const modifier = randomPick(failingModifiers);
        className = `${modifier}:${className}`;

        // 20% chance of stacked modifiers (common in failures)
        if (rng() < 0.2 && failingModifiers.length > 0) {
          const modifier2 = randomPick(failingModifiers);
          className = `${modifier2}:${className}`;
        }
      }
    }

    classes.push(className);
  }

  return classes;
}

/**
 * Sort classes using Prettier
 */
async function sortWithPrettier(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;

  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });

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
  const rustywindBin = '../../target/release/rustywind';
  const { stdout } = await execAsync(`echo '${html.replace(/'/g, "'\\''")}' | ${rustywindBin} --stdin`);

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
 * Run the fuzz test with real-world patterns
 */
async function runFuzzTest() {
  console.log(`\n🎯 Failure-Focused Pattern Fuzz Test`);
  console.log('='.repeat(80));
  console.log(`Generating ${NUM_TESTS} test cases using patterns from FAILING real-world cases`);
  console.log(`🎲 Seed: ${SEED} (set FUZZ_SEED env var to reproduce)`);
  console.log(`📋 Failing classes pool: ${failingClasses.length} classes`);
  console.log(`📊 Using ${failingPairs.length} pairs from failing cases`);
  console.log(`🔍 Expected failure rate: ~${failurePatterns.failureRate}% (from real-world data)\n`);

  let passed = 0;
  let failed = 0;
  const failures = [];

  for (let i = 0; i < NUM_TESTS; i++) {
    const classes = generateRealWorldClasses();

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
    const samplesToShow = Math.min(5, failures.length);
    console.log(`❌ Sample Failures (showing first ${samplesToShow} of ${failures.length}):\n`);

    failures.slice(0, samplesToShow).forEach(({ test, reason, prettier, rustywind, original, error }) => {
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

    if (failures.length > samplesToShow) {
      console.log(`... and ${failures.length - samplesToShow} more failures\n`);
    }

    console.log(`💡 These failures were generated using real-world patterns:`);
    console.log(`   - Class count distribution from actual projects`);
    console.log(`   - Common class pairs that appear together`);
    console.log(`   - Real modifier usage patterns (dark:, hover:, etc.)\n`);

    process.exit(1);
  } else {
    console.log('✅ All real-world pattern tests passed!');
    process.exit(0);
  }
}

// Run the test
runFuzzTest().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

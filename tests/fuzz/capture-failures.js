/**
 * Capture and categorize fuzz test failures
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { allClasses, variants, variantStackingPatterns, opacityClasses, arbitraryValueClasses } from './tailwind-classes.js';
import { filterLegacyClasses, isLegacyClass } from './legacy-classes.js';
import prettier from 'prettier';
import seedrandom from 'seedrandom';
import fs from 'fs';

const execAsync = promisify(exec);

// Configuration
const NUM_TESTS = 100;
const MIN_CLASSES = 5;
const MAX_CLASSES = 30;
const VARIANT_PROBABILITY = 0.3;
const FILTER_LEGACY = process.env.FILTER_LEGACY !== 'false';
const NUM_ROUNDS = parseInt(process.env.ROUNDS || '25');

// Seed configuration for deterministic testing
const BASE_SEED = process.env.BASE_SEED || 'failure-analysis';

const baseClasses = FILTER_LEGACY ? filterLegacyClasses(allClasses) : allClasses;
const classPool = [...baseClasses, ...opacityClasses, ...arbitraryValueClasses];

function randomInt(rng, min, max) {
  return Math.floor(rng() * (max - min + 1)) + min;
}

function randomPick(rng, array) {
  return array[randomInt(rng, 0, array.length - 1)];
}

function generateRandomClass(rng) {
  let className = randomPick(rng, classPool);

  if (rng() < VARIANT_PROBABILITY) {
    if (rng() < 0.4 && variantStackingPatterns.length > 0) {
      const pattern = randomPick(rng, variantStackingPatterns);
      className = `${pattern[0]}:${pattern[1]}:${className}`;
    } else {
      const variant = randomPick(rng, variants);
      className = `${variant}:${className}`;

      if (rng() < 0.2) {
        const variant2 = randomPick(rng, variants);
        className = `${variant2}:${className}`;
      }
    }
  }

  return className;
}

function generateRandomClasses(rng, count) {
  const classes = [];
  for (let i = 0; i < count; i++) {
    classes.push(generateRandomClass(rng));
  }
  return classes;
}

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

function analyzeFailure(prettier, rustywind, original) {
  // Find the first mismatch
  for (let i = 0; i < Math.max(prettier.length, rustywind.length); i++) {
    if (prettier[i] !== rustywind[i]) {
      return {
        position: i,
        prettier: prettier[i],
        rustywind: rustywind[i],
        prettierClasses: prettier,
        rustywindClasses: rustywind,
        original: original
      };
    }
  }

  // Length mismatch
  return {
    position: -1,
    prettier: prettier.length,
    rustywind: rustywind.length,
    prettierClasses: prettier,
    rustywindClasses: rustywind,
    original: original
  };
}

async function runFailureCapture() {
  console.log(`\n🔍 Capturing failures across ${NUM_ROUNDS} rounds...`);
  console.log(`🎲 Base Seed: ${BASE_SEED}\n`);

  const allFailures = [];
  let totalPassed = 0;
  let totalFailed = 0;

  for (let round = 0; round < NUM_ROUNDS; round++) {
    const seed = `${BASE_SEED}-round-${round}`;
    const rng = seedrandom(seed);

    console.log(`Round ${round + 1}/${NUM_ROUNDS}...`);

    for (let i = 0; i < NUM_TESTS; i++) {
      const numClasses = randomInt(rng, MIN_CLASSES, MAX_CLASSES);
      const classes = generateRandomClasses(rng, numClasses);

      try {
        const prettierSorted = await sortWithPrettier(classes);
        const rustywindSorted = await sortWithRustyWind(classes);

        if (JSON.stringify(prettierSorted) === JSON.stringify(rustywindSorted)) {
          totalPassed++;
        } else {
          totalFailed++;
          allFailures.push({
            round: round + 1,
            test: i + 1,
            seed: seed,
            ...analyzeFailure(prettierSorted, rustywindSorted, classes)
          });
        }
      } catch (error) {
        totalFailed++;
        allFailures.push({
          round: round + 1,
          test: i + 1,
          seed: seed,
          error: error.message,
          original: classes
        });
      }
    }
  }

  console.log(`\nTotal: ${totalPassed} passed, ${totalFailed} failed (${(totalPassed / (totalPassed + totalFailed) * 100).toFixed(2)}%)\n`);

  // Save failures to JSON
  fs.writeFileSync('failure-analysis.json', JSON.stringify(allFailures, null, 2));
  console.log(`✅ Saved ${allFailures.length} failures to failure-analysis.json\n`);

  return allFailures;
}

// Run the capture
runFailureCapture().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

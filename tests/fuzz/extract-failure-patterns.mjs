#!/usr/bin/env node

/**
 * Extract patterns from FAILING real-world tests
 *
 * This analyzes the failures to find common patterns that cause issues,
 * then uses those patterns to generate more targeted fuzz tests.
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import { join } from 'path';
import prettier from 'prettier';

const execAsync = promisify(exec);
const TEST_FILES_DIR = '../tailwind-sorting-test-files/test-files';
const RUSTYWIND_BIN = '../../target/release/rustywind';

/**
 * Extract all class/className attributes from a file
 */
function extractClasses(content) {
  const classes = [];
  const patterns = [
    /\bclass(?:Name)?=["']([^"']+)["']/g,
    /\bclass(?:Name)?=\{["']([^"']+)["']\}/g,
    /\bclass(?:Name)?=\{`([^`]+)`\}/g,
  ];

  patterns.forEach(pattern => {
    let match;
    while ((match = pattern.exec(content)) !== null) {
      const classString = match[1];
      if (classString && !classString.includes('${') && classString.trim().length > 0) {
        const classList = classString.trim().split(/\s+/).filter(c => c.length > 0);
        if (classList.length > 0) {
          classes.push(classList);
        }
      }
    }
  });

  return classes;
}

/**
 * Walk directory and get all test files
 */
function getTestFiles(dir) {
  const files = [];

  function walk(currentDir) {
    const entries = readdirSync(currentDir);
    entries.forEach(entry => {
      const fullPath = join(currentDir, entry);
      const stat = statSync(fullPath);
      if (stat.isDirectory()) {
        walk(fullPath);
      } else if (/\.(html|jsx?|tsx?|vue)$/.test(entry)) {
        files.push(fullPath);
      }
    });
  }

  walk(dir);
  return files;
}

async function sortWithPrettier(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;
  const formatted = await prettier.format(html, {
    parser: 'html',
    plugins: ['prettier-plugin-tailwindcss'],
    printWidth: 10000,
  });
  const match = formatted.match(/class="([^"]*)"/);
  if (!match) throw new Error('Could not extract classes from Prettier');
  return match[1].split(/\s+/).filter(c => c.length > 0);
}

async function sortWithRustyWind(classes) {
  const html = `<div class="${classes.join(' ')}"></div>`;
  const { stdout } = await execAsync(`echo '${html.replace(/'/g, "'\\''")}' | ${RUSTYWIND_BIN} --stdin`);
  const match = stdout.trim().match(/class="([^"]*)"/);
  if (!match) throw new Error('Could not extract classes from RustyWind');
  return match[1].split(/\s+/).filter(c => c.length > 0);
}

/**
 * Test and collect failures
 */
async function extractFailurePatterns() {
  console.log('🔍 Analyzing Real-World Failures to Extract Patterns\n');

  const testFiles = getTestFiles(TEST_FILES_DIR);
  console.log(`Found ${testFiles.length} test files\n`);

  let allClassLists = [];
  testFiles.forEach(file => {
    const content = readFileSync(file, 'utf8');
    const classLists = extractClasses(content);
    allClassLists = allClassLists.concat(classLists.map(cl => ({ classes: cl, file })));
  });

  // Remove duplicates
  const uniqueClassLists = [];
  const seen = new Set();
  allClassLists.forEach(({ classes }) => {
    const key = classes.join('|');
    if (!seen.has(key)) {
      seen.add(key);
      uniqueClassLists.push(classes);
    }
  });

  console.log(`Testing ${uniqueClassLists.length} unique class combinations...\n`);

  const failures = [];
  const failurePatterns = {
    classesInFailures: new Set(),
    modifiersInFailures: {},
    pairsInFailures: {},
    classCountsInFailures: [],
  };

  let tested = 0;
  for (const classes of uniqueClassLists) {
    tested++;
    if (tested % 100 === 0) {
      process.stdout.write(`\rTested: ${tested}/${uniqueClassLists.length}`);
    }

    try {
      const prettierSorted = await sortWithPrettier(classes);
      const rustywindSorted = await sortWithRustyWind(classes);

      // Check if they differ
      const differs = prettierSorted.length !== rustywindSorted.length ||
                      prettierSorted.some((c, i) => c !== rustywindSorted[i]);

      if (differs) {
        failures.push({ classes, prettierSorted, rustywindSorted });

        // Extract patterns from this failure
        failurePatterns.classCountsInFailures.push(classes.length);

        classes.forEach(cls => {
          failurePatterns.classesInFailures.add(cls);

          // Track modifiers
          if (cls.includes(':')) {
            const modifiers = cls.split(':').slice(0, -1);
            modifiers.forEach(mod => {
              failurePatterns.modifiersInFailures[mod] = (failurePatterns.modifiersInFailures[mod] || 0) + 1;
            });
          }

          // Track pairs
          classes.forEach(otherCls => {
            if (cls !== otherCls) {
              const key = `${cls}|${otherCls}`;
              failurePatterns.pairsInFailures[key] = (failurePatterns.pairsInFailures[key] || 0) + 1;
            }
          });
        });
      }
    } catch (error) {
      // Skip errors
    }
  }

  console.log(`\n\n📊 Failure Analysis Results\n`);
  console.log(`Total tested: ${tested}`);
  console.log(`Failures: ${failures.length} (${(failures.length / tested * 100).toFixed(1)}%)\n`);

  // Analyze failure patterns
  const topFailureModifiers = Object.entries(failurePatterns.modifiersInFailures)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 20);

  const topFailurePairs = Object.entries(failurePatterns.pairsInFailures)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 100);

  const avgFailureClassCount = failurePatterns.classCountsInFailures.reduce((a, b) => a + b, 0) /
                                failurePatterns.classCountsInFailures.length;

  console.log(`Classes that appear in failures: ${failurePatterns.classesInFailures.size}`);
  console.log(`Average class count in failures: ${avgFailureClassCount.toFixed(1)}\n`);

  console.log('Top 10 Modifiers in Failing Cases:');
  topFailureModifiers.slice(0, 10).forEach(([mod, count], idx) => {
    console.log(`  ${idx + 1}. ${mod}: ${count} times`);
  });

  console.log('\nTop 10 Class Pairs in Failing Cases:');
  topFailurePairs.slice(0, 10).forEach(([pair, count], idx) => {
    const [cls1, cls2] = pair.split('|');
    console.log(`  ${idx + 1}. "${cls1}" + "${cls2}": ${count} times`);
  });

  // Create failure-focused patterns
  const failureFocusedPatterns = {
    failingClasses: Array.from(failurePatterns.classesInFailures),
    failingModifiers: topFailureModifiers.slice(0, 15).map(([mod]) => mod),
    failingPairs: topFailurePairs.slice(0, 50).map(([pair]) => {
      const [cls1, cls2] = pair.split('|');
      return [cls1, cls2];
    }),
    avgFailureClassCount: Math.round(avgFailureClassCount),
    failureRate: (failures.length / tested * 100).toFixed(1),
  };

  const outputPath = './failure-patterns.json';
  writeFileSync(outputPath, JSON.stringify(failureFocusedPatterns, null, 2));
  console.log(`\n💾 Failure patterns saved to: ${outputPath}\n`);

  return failureFocusedPatterns;
}

// Run analysis
extractFailurePatterns().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

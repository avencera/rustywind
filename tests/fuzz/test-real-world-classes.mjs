/**
 * Real-world class test: Extract classes from actual project files and test against Prettier
 *
 * IMPORTANT: This test uses files from ../tailwind-sorting-test-files/
 * These files are READ-ONLY references from real open-source projects.
 * NEVER modify these files to make tests pass!
 */

import { exec } from 'child_process';
import { promisify } from 'util';
import { readFileSync, readdirSync, statSync } from 'fs';
import { join } from 'path';
import prettier from 'prettier';

const execAsync = promisify(exec);

const TEST_FILES_DIR = '../tailwind-sorting-test-files/test-files';
const RUSTYWIND_BIN = '../../target/release/rustywind';

/**
 * Extract all class/className attributes from a file
 */
function extractClasses(content, filePath) {
  const classes = [];

  // Match class="..." and className="..." and className={...}
  const patterns = [
    /\bclass(?:Name)?=["']([^"']+)["']/g,
    /\bclass(?:Name)?=\{["']([^"']+)["']\}/g,
    /\bclass(?:Name)?=\{`([^`]+)`\}/g,
  ];

  patterns.forEach(pattern => {
    let match;
    while ((match = pattern.exec(content)) !== null) {
      const classString = match[1];
      // Skip empty or template expressions
      if (classString && !classString.includes('${') && classString.trim().length > 0) {
        // Split by whitespace and filter valid tailwind classes
        const classList = classString.trim().split(/\s+/).filter(c => c.length > 0);
        if (classList.length > 0) {
          classes.push({
            raw: classString.trim(),
            classes: classList,
            location: filePath,
          });
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
  const { stdout } = await execAsync(`echo '${html.replace(/'/g, "'\\''")}' | ${RUSTYWIND_BIN} --stdin`);

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
 * Run the real-world class test
 */
async function runRealWorldTest() {
  console.log('\n🌍 Real-World Class Test');
  console.log('=' .repeat(80));
  console.log('Testing classes extracted from actual project files');
  console.log('Source: ../tailwind-sorting-test-files/test-files/\n');
  console.log('⚠️  NOTE: Test files are READ-ONLY and should NEVER be modified!\n');

  // Get all test files
  const testFiles = getTestFiles(TEST_FILES_DIR);
  console.log(`📁 Found ${testFiles.length} test files\n`);

  // Extract all classes from all files
  console.log('📝 Extracting classes from files...');
  let allClassSets = [];
  let totalClassAttributes = 0;

  testFiles.forEach(file => {
    const content = readFileSync(file, 'utf8');
    const classSets = extractClasses(content, file);
    allClassSets = allClassSets.concat(classSets);
    totalClassAttributes += classSets.length;
  });

  console.log(`   Found ${totalClassAttributes} class attributes\n`);

  // Remove duplicates based on raw class string
  const uniqueClassSets = [];
  const seen = new Set();
  allClassSets.forEach(classSet => {
    if (!seen.has(classSet.raw)) {
      seen.add(classSet.raw);
      uniqueClassSets.push(classSet);
    }
  });

  console.log(`   ${uniqueClassSets.length} unique class combinations\n`);
  console.log('🧪 Testing each class combination...\n');

  let passed = 0;
  let failed = 0;
  const failures = [];
  const uniqueFailurePatterns = new Set();

  for (let i = 0; i < uniqueClassSets.length; i++) {
    const { classes, raw, location } = uniqueClassSets[i];

    try {
      const prettierSorted = await sortWithPrettier(classes);
      const rustywindSorted = await sortWithRustyWind(classes);

      const comparison = compareClasses(prettierSorted, rustywindSorted, classes);

      if (comparison.match) {
        passed++;
        process.stdout.write('.');
      } else {
        failed++;

        // Track unique failure patterns
        const failurePattern = `${comparison.prettier.join(' ')} | ${comparison.rustywind.join(' ')}`;
        uniqueFailurePatterns.add(failurePattern);

        failures.push({
          test: i + 1,
          location: location.replace(TEST_FILES_DIR + '/', ''),
          ...comparison,
        });
        process.stdout.write('F');
      }

      // Print progress every 50 tests
      if ((i + 1) % 50 === 0) {
        process.stdout.write(` ${i + 1}/${uniqueClassSets.length}\n`);
      }
    } catch (error) {
      failed++;
      failures.push({
        test: i + 1,
        location: location.replace(TEST_FILES_DIR + '/', ''),
        error: error.message,
        original: classes,
      });
      process.stdout.write('E');
    }
  }

  console.log('\n');
  console.log('='.repeat(80));
  console.log('\n📊 RESULTS');
  console.log('='.repeat(80));
  console.log(`\nTotal class combinations tested: ${uniqueClassSets.length}`);
  console.log(`✅ Passed: ${passed} (${(passed / uniqueClassSets.length * 100).toFixed(1)}%)`);
  console.log(`❌ Failed: ${failed} (${(failed / uniqueClassSets.length * 100).toFixed(1)}%)`);
  console.log(`🔍 Unique failure patterns: ${uniqueFailurePatterns.size}\n`);

  if (failures.length > 0) {
    // Show first 10 failures
    const samplesToShow = Math.min(10, failures.length);
    console.log(`❌ Sample Failures (showing first ${samplesToShow} of ${failures.length}):\n`);

    failures.slice(0, samplesToShow).forEach(({ test, reason, prettier, rustywind, original, location, error }) => {
      console.log(`[${test}] ${location}`);
      if (error) {
        console.log(`  Error: ${error}`);
        console.log(`  Classes: ${original ? original.join(' ') : 'N/A'}`);
      } else {
        console.log(`  ${reason}`);
        console.log(`  Original:  ${original.join(' ')}`);
        console.log(`  Prettier:  ${prettier.join(' ')}`);
        console.log(`  RustyWind: ${rustywind.join(' ')}`);
      }
      console.log('');
    });

    if (failures.length > samplesToShow) {
      console.log(`... and ${failures.length - samplesToShow} more failures\n`);
    }

    console.log('💡 TIP: These failures represent real-world usage patterns.');
    console.log('   Fix the sorting algorithm to handle these cases correctly.\n');
    console.log('⚠️  REMEMBER: Do NOT modify the test files to make tests pass!');
    console.log('   The files in tailwind-sorting-test-files/ are reference data.\n');

    process.exit(1);
  } else {
    console.log('✅ All real-world class combinations match!');
    console.log('   RustyWind and Prettier produce identical sorting.\n');
    process.exit(0);
  }
}

// Run the test
runRealWorldTest().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

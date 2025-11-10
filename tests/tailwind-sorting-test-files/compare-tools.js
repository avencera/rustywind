#!/usr/bin/env node

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configuration
const RUSTYWIND_BIN = '../../target/release/rustywind';
const RESULTS_DIR = './comparison-results';
const TEST_FILES_DIR = './test-files';

// Colors for console output
const colors = {
  reset: '\x1b[0m',
  green: '\x1b[32m',
  red: '\x1b[31m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  cyan: '\x1b[36m',
};

function log(message, color = colors.reset) {
  console.log(`${color}${message}${colors.reset}`);
}

// Create results directories
function setupDirectories() {
  const dirs = [
    RESULTS_DIR,
    path.join(RESULTS_DIR, 'rustywind'),
    path.join(RESULTS_DIR, 'prettier'),
    path.join(RESULTS_DIR, 'diffs'),
  ];

  dirs.forEach(dir => {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  });
}

// Get all test files
function getTestFiles() {
  const testFiles = [];

  function walk(dir) {
    const files = fs.readdirSync(dir);
    files.forEach(file => {
      const filePath = path.join(dir, file);
      const stat = fs.statSync(filePath);

      if (stat.isDirectory()) {
        walk(filePath);
      } else if (/\.(html|jsx?|tsx?|vue)$/.test(file)) {
        testFiles.push(filePath);
      }
    });
  }

  walk(TEST_FILES_DIR);
  return testFiles;
}

// Run rustywind on a file
function runRustywind(filePath) {
  try {
    execSync(`${RUSTYWIND_BIN} "${filePath}" --write`, {
      stdio: 'pipe',
      encoding: 'utf8'
    });
    return true;
  } catch (error) {
    log(`  Error running rustywind: ${error.message}`, colors.red);
    return false;
  }
}

// Run prettier on a file
function runPrettier(filePath) {
  try {
    const absolutePath = path.resolve(filePath);
    execSync(`npx prettier --write "${absolutePath}"`, {
      stdio: 'pipe',
      encoding: 'utf8',
      cwd: path.join(__dirname, '../fuzz')
    });
    return true;
  } catch (error) {
    log(`  Error running prettier: ${error.message}`, colors.red);
    return false;
  }
}

// Compare two files
function compareFiles(file1, file2) {
  const content1 = fs.readFileSync(file1, 'utf8');
  const content2 = fs.readFileSync(file2, 'utf8');

  return {
    identical: content1 === content2,
    content1,
    content2,
  };
}

// Generate a diff
function generateDiff(file1, file2, outputPath) {
  try {
    const diff = execSync(`diff -u "${file1}" "${file2}"`, {
      encoding: 'utf8',
      stdio: 'pipe'
    });
    fs.writeFileSync(outputPath, diff);
    return diff;
  } catch (error) {
    // diff returns non-zero exit code when files differ
    if (error.stdout) {
      fs.writeFileSync(outputPath, error.stdout);
      return error.stdout;
    }
    return '';
  }
}

// Main comparison function
async function runComparison() {
  log('\n🔍 Tailwind CSS Class Sorter Comparison', colors.cyan);
  log('=' .repeat(60), colors.cyan);
  log(`Comparing: rustywind vs prettier-plugin-tailwindcss\n`, colors.cyan);

  setupDirectories();

  const testFiles = getTestFiles();
  log(`📁 Found ${testFiles.length} test files\n`, colors.blue);

  const results = {
    identical: [],
    different: [],
    errors: [],
    totalFiles: testFiles.length,
  };

  let processedCount = 0;

  for (const testFile of testFiles) {
    const filename = path.basename(testFile);
    const relPath = path.relative(TEST_FILES_DIR, testFile);

    processedCount++;
    process.stdout.write(`[${processedCount}/${testFiles.length}] Processing: ${relPath}... `);

    const rustywinded = path.join(RESULTS_DIR, 'rustywind', filename);
    const prettified = path.join(RESULTS_DIR, 'prettier', filename);
    const diffPath = path.join(RESULTS_DIR, 'diffs', `${filename}.diff`);

    try {
      // Copy original files
      fs.copyFileSync(testFile, rustywinded);
      fs.copyFileSync(testFile, prettified);

      // Run tools
      const rustywindindSuccess = runRustywind(rustywinded);
      const prettierSuccess = runPrettier(prettified);

      if (!rustywindindSuccess || !prettierSuccess) {
        results.errors.push({
          file: relPath,
          rustywindindSuccess,
          prettierSuccess,
        });
        log('ERROR', colors.red);
        continue;
      }

      // Compare results
      const comparison = compareFiles(rustywinded, prettified);

      if (comparison.identical) {
        results.identical.push(relPath);
        log('✓ MATCH', colors.green);
        // Remove empty diff file
        if (fs.existsSync(diffPath)) {
          fs.unlinkSync(diffPath);
        }
      } else {
        results.different.push(relPath);
        log('✗ DIFF', colors.red);

        // Generate diff
        generateDiff(rustywinded, prettified, diffPath);
      }

    } catch (error) {
      results.errors.push({
        file: relPath,
        error: error.message,
      });
      log(`ERROR: ${error.message}`, colors.red);
    }
  }

  // Print summary
  log('\n' + '='.repeat(60), colors.cyan);
  log('📊 RESULTS SUMMARY', colors.cyan);
  log('='.repeat(60), colors.cyan);

  const matchPercentage = ((results.identical.length / results.totalFiles) * 100).toFixed(2);

  log(`\nTotal files tested: ${results.totalFiles}`, colors.blue);
  log(`✓ Identical outputs: ${results.identical.length} (${matchPercentage}%)`, colors.green);
  log(`✗ Different outputs: ${results.different.length}`, results.different.length > 0 ? colors.red : colors.green);
  log(`⚠ Errors: ${results.errors.length}`, results.errors.length > 0 ? colors.yellow : colors.green);

  if (results.different.length > 0) {
    log('\n📝 Files with differences:', colors.yellow);
    results.different.forEach(file => {
      log(`  - ${file}`, colors.yellow);
    });
    log(`\nDiff files saved to: ${path.join(RESULTS_DIR, 'diffs')}`, colors.cyan);
  }

  if (results.errors.length > 0) {
    log('\n⚠️  Files with errors:', colors.yellow);
    results.errors.forEach(item => {
      log(`  - ${item.file}`, colors.yellow);
      if (item.error) {
        log(`    Error: ${item.error}`, colors.red);
      }
    });
  }

  // Save detailed results
  const resultsFile = path.join(RESULTS_DIR, 'comparison-results.json');
  fs.writeFileSync(resultsFile, JSON.stringify(results, null, 2));
  log(`\n💾 Detailed results saved to: ${resultsFile}`, colors.cyan);

  log('\n' + '='.repeat(60) + '\n', colors.cyan);

  // Exit with appropriate code
  process.exit(results.different.length > 0 || results.errors.length > 0 ? 1 : 0);
}

// Run the comparison
runComparison().catch(error => {
  log(`\n❌ Fatal error: ${error.message}`, colors.red);
  console.error(error);
  process.exit(1);
});

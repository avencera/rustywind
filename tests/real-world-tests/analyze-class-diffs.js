#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const RESULTS_DIR = './comparison-results';
const DIFFS_DIR = path.join(RESULTS_DIR, 'diffs');

// Extract className/class differences from diff output
function extractClassDiffs(diffContent) {
  const lines = diffContent.split('\n');
  const classDiffs = [];

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Look for lines with className or class
    if (line.match(/^[-+].*\b(className|class)=/)) {
      const isRemoved = line.startsWith('-');
      const isAdded = line.startsWith('+');

      if (isRemoved || isAdded) {
        // Extract the class value
        const classMatch = line.match(/\b(?:className|class)=["']([^"']+)["']/);
        if (classMatch) {
          classDiffs.push({
            type: isRemoved ? 'removed' : 'added',
            line: line.substring(1).trim(),
            classes: classMatch[1],
          });
        }
      }
    }
  }

  return classDiffs;
}

// Find paired class changes (rustywind vs prettier)
function findClassChanges(classDiffs) {
  const changes = [];

  for (let i = 0; i < classDiffs.length - 1; i++) {
    const current = classDiffs[i];
    const next = classDiffs[i + 1];

    if (current.type === 'removed' && next.type === 'added') {
      // Check if the classes are different but have similar content
      const rustywindindClasses = current.classes.split(/\s+/).sort();
      const prettierClasses = next.classes.split(/\s+/).sort();

      // If they have the same classes but different order, this is a sorting difference
      if (JSON.stringify(rustywindindClasses) === JSON.stringify(prettierClasses)) {
        changes.push({
          rustywind: current.classes,
          prettier: next.classes,
          isSortingOnly: true,
        });
        i++; // Skip the next one as we've processed it
      } else {
        // Different classes entirely (might be formatting or both)
        changes.push({
          rustywind: current.classes,
          prettier: next.classes,
          isSortingOnly: false,
        });
        i++;
      }
    }
  }

  return changes;
}

// Analyze all diff files
function analyzeAllDiffs() {
  const diffFiles = fs.readdirSync(DIFFS_DIR)
    .filter(f => f.endsWith('.diff'))
    .sort();

  console.log('\n🔬 Analyzing Class Sorting Differences');
  console.log('='.repeat(80));
  console.log(`Found ${diffFiles.length} diff files\n`);

  const allChanges = {};
  let totalClassChanges = 0;
  let totalSortingOnlyChanges = 0;

  diffFiles.forEach(diffFile => {
    const diffPath = path.join(DIFFS_DIR, diffFile);
    const diffContent = fs.readFileSync(diffPath, 'utf8');
    const classDiffs = extractClassDiffs(diffContent);
    const classChanges = findClassChanges(classDiffs);

    if (classChanges.length > 0) {
      allChanges[diffFile] = classChanges;
      totalClassChanges += classChanges.length;
      totalSortingOnlyChanges += classChanges.filter(c => c.isSortingOnly).length;
    }
  });

  console.log(`📊 Summary:`);
  console.log(`  Files with class differences: ${Object.keys(allChanges).length}`);
  console.log(`  Total class attribute changes: ${totalClassChanges}`);
  console.log(`  Pure sorting differences: ${totalSortingOnlyChanges}`);
  console.log(`  Changes with other differences: ${totalClassChanges - totalSortingOnlyChanges}\n`);

  // Show examples of sorting-only differences
  console.log('🎯 Examples of Pure Sorting Differences:\n');

  let exampleCount = 0;
  const maxExamples = 10;

  for (const [file, changes] of Object.entries(allChanges)) {
    const sortingOnlyChanges = changes.filter(c => c.isSortingOnly);

    if (sortingOnlyChanges.length > 0 && exampleCount < maxExamples) {
      console.log(`File: ${file.replace('.diff', '')}`);

      sortingOnlyChanges.slice(0, 2).forEach(change => {
        console.log(`  Rustywind: ${change.rustywind}`);
        console.log(`  Prettier:  ${change.prettier}`);
        console.log();
      });

      exampleCount++;

      if (exampleCount >= maxExamples) break;
    }
  }

  // Show examples with different classes
  console.log('\n⚠️  Examples of Changes Beyond Sorting:\n');

  exampleCount = 0;
  for (const [file, changes] of Object.entries(allChanges)) {
    const nonSortingChanges = changes.filter(c => !c.isSortingOnly);

    if (nonSortingChanges.length > 0 && exampleCount < maxExamples) {
      console.log(`File: ${file.replace('.diff', '')}`);

      nonSortingChanges.slice(0, 2).forEach(change => {
        const rustywindindClasses = change.rustywind.split(/\s+/);
        const prettierClasses = change.prettier.split(/\s+/);

        console.log(`  Rustywind (${rustywindindClasses.length} classes): ${change.rustywind}`);
        console.log(`  Prettier  (${prettierClasses.length} classes): ${change.prettier}`);
        console.log();
      });

      exampleCount++;

      if (exampleCount >= maxExamples) break;
    }
  }

  // Save detailed analysis
  const analysisFile = path.join(RESULTS_DIR, 'class-differences-analysis.json');
  fs.writeFileSync(analysisFile, JSON.stringify(allChanges, null, 2));
  console.log(`\n💾 Detailed analysis saved to: ${analysisFile}\n`);
}

analyzeAllDiffs();

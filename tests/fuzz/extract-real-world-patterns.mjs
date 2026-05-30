#!/usr/bin/env node

/**
 * Extract patterns from real-world test files to inform fuzz test generation
 *
 * This script analyzes the real project files and extracts:
 * - Common class combinations
 * - Class count distribution
 * - Common modifiers
 * - Utility patterns
 */

import { readFileSync, readdirSync, statSync, writeFileSync } from 'fs';
import { join } from 'path';

const TEST_FILES_DIR = '../tailwind-sorting-test-files/test-files';

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

/**
 * Analyze patterns in class lists
 */
function analyzePatterns() {
  console.log('🔍 Analyzing Real-World Class Patterns\n');

  const testFiles = getTestFiles(TEST_FILES_DIR);
  console.log(`Found ${testFiles.length} test files\n`);

  let allClassLists = [];
  let allClasses = new Set();
  const modifierCounts = {};
  const classCooccurrence = {}; // Which classes appear together
  const classCounts = []; // Distribution of class counts

  // Extract all class lists
  testFiles.forEach(file => {
    const content = readFileSync(file, 'utf8');
    const classLists = extractClasses(content);
    allClassLists = allClassLists.concat(classLists);

    classLists.forEach(classList => {
      classCounts.push(classList.length);

      classList.forEach(cls => {
        allClasses.add(cls);

        // Track modifiers
        if (cls.includes(':')) {
          const modifiers = cls.split(':').slice(0, -1);
          modifiers.forEach(mod => {
            modifierCounts[mod] = (modifierCounts[mod] || 0) + 1;
          });
        }

        // Track co-occurrence
        classList.forEach(otherCls => {
          if (cls !== otherCls) {
            const key = `${cls}|${otherCls}`;
            classCooccurrence[key] = (classCooccurrence[key] || 0) + 1;
          }
        });
      });
    });
  });

  // Analyze class count distribution
  const classCountStats = {
    min: Math.min(...classCounts),
    max: Math.max(...classCounts),
    avg: classCounts.reduce((a, b) => a + b, 0) / classCounts.length,
    median: classCounts.sort((a, b) => a - b)[Math.floor(classCounts.length / 2)],
  };

  // Find most common modifiers
  const topModifiers = Object.entries(modifierCounts)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 20);

  // Find most common co-occurrences
  const topCooccurrences = Object.entries(classCooccurrence)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 100)
    .map(([key, count]) => {
      const [cls1, cls2] = key.split('|');
      return { cls1, cls2, count };
    });

  // Create patterns object
  const patterns = {
    classCountStats,
    topModifiers: topModifiers.map(([mod, count]) => ({ modifier: mod, count })),
    topCooccurrences,
    totalClassLists: allClassLists.length,
    totalUniqueClasses: allClasses.size,
  };

  // Print summary
  console.log('📊 Pattern Analysis Results\n');
  console.log(`Total class lists: ${patterns.totalClassLists}`);
  console.log(`Total unique classes: ${patterns.totalUniqueClasses}\n`);

  console.log('Class Count Distribution:');
  console.log(`  Min: ${classCountStats.min}`);
  console.log(`  Max: ${classCountStats.max}`);
  console.log(`  Average: ${classCountStats.avg.toFixed(1)}`);
  console.log(`  Median: ${classCountStats.median}\n`);

  console.log('Top 20 Most Common Modifiers:');
  topModifiers.forEach(({ modifier, count }, idx) => {
    console.log(`  ${idx + 1}. ${modifier}: ${count} occurrences`);
  });

  console.log('\nTop 20 Class Co-occurrences:');
  topCooccurrences.slice(0, 20).forEach(({ cls1, cls2, count }, idx) => {
    console.log(`  ${idx + 1}. "${cls1}" + "${cls2}": ${count} times`);
  });

  // Save patterns to file
  const outputPath = './real-world-patterns.json';
  writeFileSync(outputPath, JSON.stringify(patterns, null, 2));
  console.log(`\n💾 Patterns saved to: ${outputPath}`);

  // Also create a more consumable version with just the common patterns
  const commonPatterns = {
    // Realistic class counts based on real data
    classCountDistribution: {
      min: classCountStats.min,
      max: classCountStats.max,
      avg: Math.round(classCountStats.avg),
      median: classCountStats.median,
      // Ranges with probabilities (estimated from median/avg)
      ranges: [
        { min: 1, max: 5, probability: 0.3 },  // Simple elements
        { min: 6, max: 10, probability: 0.4 }, // Common case
        { min: 11, max: 20, probability: 0.2 }, // Complex elements
        { min: 21, max: 30, probability: 0.1 }, // Very complex
      ]
    },

    // Most common modifiers to use in generation
    commonModifiers: topModifiers.slice(0, 10).map(m => m.modifier),

    // Common class pairs that often appear together
    commonPairs: topCooccurrences.slice(0, 50).map(({ cls1, cls2 }) => [cls1, cls2]),
  };

  const commonPatternsPath = './common-patterns.json';
  writeFileSync(commonPatternsPath, JSON.stringify(commonPatterns, null, 2));
  console.log(`💾 Common patterns saved to: ${commonPatternsPath}\n`);

  return commonPatterns;
}

// Run analysis
analyzePatterns();

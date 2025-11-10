/**
 * Analyze failure patterns from multi-seed results
 */
import { readFile, writeFile } from 'fs/promises';

const RESULTS_FILE = 'multi-seed-results.json';

/**
 * Categorize a failure pattern by the types of classes involved
 */
function categorizeFailure(pattern) {
  const categories = [];

  // Extract the class names from the mismatch message
  const match = pattern.match(/Prettier="([^"]+)", RustyWind="([^"]+)"/);
  if (!match) return ['unknown'];

  const prettierClass = match[1];
  const rustywindClass = match[2];

  // Categorize by class type patterns
  if (prettierClass.includes('outline') || rustywindClass.includes('outline')) {
    categories.push('outline');
  }
  if (prettierClass.includes('transition') || rustywindClass.includes('transition')) {
    categories.push('transition');
  }
  if (prettierClass.includes('duration') || rustywindClass.includes('duration')) {
    categories.push('duration');
  }
  if (prettierClass.includes('delay') || rustywindClass.includes('delay')) {
    categories.push('delay');
  }
  if (prettierClass.includes('rounded') || rustywindClass.includes('rounded')) {
    categories.push('rounded');
  }
  if (prettierClass.includes('bg-') || rustywindClass.includes('bg-')) {
    categories.push('background');
  }
  if (prettierClass.includes('space-') || rustywindClass.includes('space-')) {
    categories.push('spacing');
  }
  if (prettierClass.includes('gap-') || rustywindClass.includes('gap-')) {
    categories.push('gap');
  }
  if (prettierClass.includes('divide-') || rustywindClass.includes('divide-')) {
    categories.push('divide');
  }
  if (prettierClass.includes('ring-') || rustywindClass.includes('ring-')) {
    categories.push('ring');
  }
  if (prettierClass.includes('shadow') || rustywindClass.includes('shadow')) {
    categories.push('shadow');
  }
  if (prettierClass.includes('border-') || rustywindClass.includes('border-')) {
    categories.push('border');
  }
  if (prettierClass.includes('-rotate') || rustywindClass.includes('-rotate')) {
    categories.push('rotation');
  }
  if (prettierClass.includes('will-change') || rustywindClass.includes('will-change')) {
    categories.push('will-change');
  }

  // If no specific category found, try to identify generic category
  if (categories.length === 0) {
    categories.push('other');
  }

  return categories;
}

/**
 * Extract the actual classes from failure details
 */
function extractClasses(example) {
  const details = example.details;
  if (!details || !details.lines) return null;

  const original = details.lines.find(l => l.includes('Original:'));
  const prettier = details.lines.find(l => l.includes('Prettier:'));
  const rustywind = details.lines.find(l => l.includes('RustyWind:'));

  if (!original || !prettier || !rustywind) return null;

  // Extract arrays from lines like "  Original:  [class1, class2, ...]"
  const extractArray = (line) => {
    const match = line.match(/\[(.*)\]/);
    if (!match) return [];
    return match[1].split(',').map(s => s.trim());
  };

  return {
    original: extractArray(original),
    prettier: extractArray(prettier),
    rustywind: extractArray(rustywind)
  };
}

/**
 * Main analysis function
 */
async function analyzeFailures() {
  console.log('📊 Analyzing failure patterns...\n');

  const data = JSON.parse(await readFile(RESULTS_FILE, 'utf-8'));

  console.log('Summary from results:');
  console.log(`  Total rounds: ${data.summary.totalRounds}`);
  console.log(`  Total tests: ${data.summary.totalTests}`);
  console.log(`  Total passed: ${data.summary.totalPassed}`);
  console.log(`  Total failed: ${data.summary.totalFailed}`);
  console.log(`  Unique patterns: ${data.summary.uniqueFailurePatterns}\n`);

  // Categorize all failures
  const categoryMap = new Map(); // category -> list of patterns
  const categoryExamples = new Map(); // category -> example with classes

  for (const failurePattern of data.failurePatterns) {
    const categories = categorizeFailure(failurePattern.pattern);

    // Try to get a good example with full class details
    const exampleWithClasses = failurePattern.examples.find(ex => {
      const classes = extractClasses(ex);
      return classes !== null;
    });

    for (const category of categories) {
      if (!categoryMap.has(category)) {
        categoryMap.set(category, []);
      }
      categoryMap.get(category).push({
        pattern: failurePattern.pattern,
        occurrences: failurePattern.occurrenceCount,
        example: failurePattern.examples[0],
        exampleWithClasses
      });

      // Store first good example for each category
      if (!categoryExamples.has(category) && exampleWithClasses) {
        categoryExamples.set(category, exampleWithClasses);
      }
    }
  }

  // Sort categories by frequency
  const sortedCategories = Array.from(categoryMap.entries())
    .map(([category, patterns]) => ({
      category,
      patternCount: patterns.length,
      totalOccurrences: patterns.reduce((sum, p) => sum + p.occurrences, 0),
      patterns
    }))
    .sort((a, b) => b.totalOccurrences - a.totalOccurrences);

  console.log('📋 Failure Categories (sorted by frequency):\n');

  for (const { category, patternCount, totalOccurrences, patterns } of sortedCategories) {
    console.log(`${category.toUpperCase()}:`);
    console.log(`  Unique patterns: ${patternCount}`);
    console.log(`  Total occurrences: ${totalOccurrences}`);
    console.log(`  Top 3 patterns:`);

    const topPatterns = patterns
      .sort((a, b) => b.occurrences - a.occurrences)
      .slice(0, 3);

    for (const p of topPatterns) {
      console.log(`    - ${p.pattern} (${p.occurrences}x)`);
    }
    console.log('');
  }

  // Generate detailed category report
  const report = {
    summary: data.summary,
    categories: sortedCategories.map(({ category, patternCount, totalOccurrences, patterns }) => ({
      category,
      patternCount,
      totalOccurrences,
      topPatterns: patterns
        .sort((a, b) => b.occurrences - a.occurrences)
        .slice(0, 5)
        .map(p => ({
          pattern: p.pattern,
          occurrences: p.occurrences,
          seed: p.example.seed,
          testNumber: p.example.testNumber,
          classes: extractClasses(p.exampleWithClasses || p.example)
        }))
    }))
  };

  await writeFile('failure-analysis.json', JSON.stringify(report, null, 2));
  console.log('💾 Detailed analysis saved to: failure-analysis.json\n');

  // Print recommendations
  console.log('📝 Recommendations for static tests:\n');
  console.log('Based on the analysis, we should create static tests for these categories:');
  for (const { category, totalOccurrences } of sortedCategories.slice(0, 10)) {
    console.log(`  - ${category} (${totalOccurrences} failures)`);
  }
}

analyzeFailures().catch(error => {
  console.error('Error:', error);
  process.exit(1);
});

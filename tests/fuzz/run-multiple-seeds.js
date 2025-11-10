/**
 * Run fuzz tests with multiple random seeds and collect all failures
 */
import { exec } from 'child_process';
import { promisify } from 'util';
import { writeFile } from 'fs/promises';

const execAsync = promisify(exec);

const NUM_ROUNDS = 100;
const RESULTS_FILE = 'multi-seed-results.json';

/**
 * Generate a random seed
 */
function generateRandomSeed() {
  return Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);
}

/**
 * Parse failure output from npm test
 */
function parseFailures(output) {
  const failures = [];
  const lines = output.split('\n');
  let currentTest = null;
  let capturingFailure = false;

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];

    // Check for test number
    const testMatch = line.match(/^Test #(\d+):/);
    if (testMatch) {
      currentTest = {
        testNumber: parseInt(testMatch[1]),
        lines: [line]
      };
      capturingFailure = true;
      continue;
    }

    // Capture failure details
    if (capturingFailure) {
      // Empty line marks end of this failure
      if (line.trim() === '') {
        if (currentTest) {
          failures.push(currentTest);
          currentTest = null;
        }
        capturingFailure = false;
      } else {
        currentTest?.lines.push(line);
      }
    }
  }

  // Don't forget the last failure if file ends without empty line
  if (currentTest) {
    failures.push(currentTest);
  }

  return failures;
}

/**
 * Extract pass/fail stats from output
 */
function extractStats(output) {
  const match = output.match(/(\d+) passed, (\d+) failed \((\d+\.\d+)% pass rate\)/);
  if (match) {
    return {
      passed: parseInt(match[1]),
      failed: parseInt(match[2]),
      passRate: parseFloat(match[3])
    };
  }
  return null;
}

/**
 * Run a single fuzz test with a specific seed
 */
async function runTestWithSeed(seed, roundNumber) {
  try {
    process.stdout.write(`Round ${roundNumber}/${NUM_ROUNDS} (seed: ${seed.substring(0, 8)}...) `);

    const { stdout, stderr } = await execAsync(`FUZZ_SEED=${seed} npm test`, {
      maxBuffer: 10 * 1024 * 1024, // 10MB buffer
      env: { ...process.env, FUZZ_SEED: seed }
    });

    // If we get here, all tests passed
    process.stdout.write('✓ All passed\n');
    return {
      seed,
      round: roundNumber,
      stats: { passed: 100, failed: 0, passRate: 100.0 },
      failures: [],
      success: true
    };
  } catch (error) {
    // Non-zero exit means there were failures
    const output = error.stdout || '';
    const stats = extractStats(output);
    const failures = parseFailures(output);

    process.stdout.write(`✗ ${stats?.failed || 0} failures\n`);

    return {
      seed,
      round: roundNumber,
      stats,
      failures,
      success: false
    };
  }
}

/**
 * Main function
 */
async function main() {
  console.log(`\n🎲 Running ${NUM_ROUNDS} rounds of fuzz tests with random seeds...\n`);

  const allResults = [];
  const uniqueFailures = new Map(); // Map of failure pattern -> list of seeds

  let totalPassed = 0;
  let totalFailed = 0;
  let roundsWithFailures = 0;

  for (let i = 1; i <= NUM_ROUNDS; i++) {
    const seed = generateRandomSeed();
    const result = await runTestWithSeed(seed, i);
    allResults.push(result);

    if (result.stats) {
      totalPassed += result.stats.passed;
      totalFailed += result.stats.failed;
    }

    if (!result.success && result.failures.length > 0) {
      roundsWithFailures++;

      // Categorize failures by their error message
      for (const failure of result.failures) {
        const reason = failure.lines.find(l => l.includes('Mismatch at position'));
        if (reason) {
          const key = reason.trim();
          if (!uniqueFailures.has(key)) {
            uniqueFailures.set(key, []);
          }
          uniqueFailures.get(key).push({
            seed: result.seed,
            round: result.round,
            testNumber: failure.testNumber,
            fullFailure: failure
          });
        }
      }
    }
  }

  console.log('\n' + '='.repeat(80));
  console.log('\n📊 Multi-Seed Fuzz Test Results:\n');
  console.log(`Total rounds: ${NUM_ROUNDS}`);
  console.log(`Rounds with failures: ${roundsWithFailures} (${(roundsWithFailures / NUM_ROUNDS * 100).toFixed(1)}%)`);
  console.log(`Total tests run: ${NUM_ROUNDS * 100}`);
  console.log(`Total passed: ${totalPassed} (${(totalPassed / (NUM_ROUNDS * 100) * 100).toFixed(1)}%)`);
  console.log(`Total failed: ${totalFailed} (${(totalFailed / (NUM_ROUNDS * 100) * 100).toFixed(1)}%)`);
  console.log(`\nUnique failure patterns found: ${uniqueFailures.size}\n`);

  // Display unique failure patterns
  if (uniqueFailures.size > 0) {
    console.log('❌ Unique Failure Patterns:\n');

    let patternNum = 1;
    for (const [pattern, occurrences] of uniqueFailures.entries()) {
      console.log(`\nPattern #${patternNum}: (${occurrences.length} occurrences)`);
      console.log(`  ${pattern}`);
      console.log(`  Example from: Round ${occurrences[0].round}, Test #${occurrences[0].testNumber}, Seed: ${occurrences[0].seed}`);

      // Show first full failure as example
      if (occurrences[0].fullFailure) {
        console.log(`  Full details:`);
        occurrences[0].fullFailure.lines.forEach(line => {
          console.log(`    ${line}`);
        });
      }

      patternNum++;
    }
  }

  // Save detailed results to file
  const detailedResults = {
    summary: {
      totalRounds: NUM_ROUNDS,
      roundsWithFailures,
      totalTests: NUM_ROUNDS * 100,
      totalPassed,
      totalFailed,
      uniqueFailurePatterns: uniqueFailures.size
    },
    failurePatterns: Array.from(uniqueFailures.entries()).map(([pattern, occurrences]) => ({
      pattern,
      occurrenceCount: occurrences.length,
      examples: occurrences.map(occ => ({
        seed: occ.seed,
        round: occ.round,
        testNumber: occ.testNumber,
        details: occ.fullFailure
      }))
    })),
    allResults
  };

  await writeFile(RESULTS_FILE, JSON.stringify(detailedResults, null, 2));
  console.log(`\n\n💾 Detailed results saved to: ${RESULTS_FILE}`);

  console.log('\n' + '='.repeat(80) + '\n');

  process.exit(uniqueFailures.size > 0 ? 1 : 0);
}

main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

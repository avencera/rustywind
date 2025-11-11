#!/usr/bin/env node
/**
 * Run fuzz tests multiple times and collect statistics
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

const ROUNDS = 25;
const results = [];

console.log(`Running ${ROUNDS} rounds of fuzz tests to establish baseline...\n`);
console.log('='.repeat(80));
console.log('');

for (let round = 1; round <= ROUNDS; round++) {
  try {
    process.stdout.write(`Round ${round}/${ROUNDS}... `);

    // Note: npm test will exit with code 1 if there are failures, but we still want to parse the output
    const { stdout, stderr } = await execAsync('npm test', {
      cwd: process.cwd(),
      maxBuffer: 10 * 1024 * 1024
    }).catch(e => ({
      stdout: e.stdout || '',
      stderr: e.stderr || '',
      error: e
    }));

    // Parse results from output
    const output = (stdout || '') + (stderr || '');
    const passedMatch = output.match(/(\d+) passed/);
    const failedMatch = output.match(/(\d+) failed/);
    const passRateMatch = output.match(/([\d.]+)% pass rate/);

    if (passedMatch && failedMatch && passRateMatch) {
      const passed = parseInt(passedMatch[1]);
      const failed = parseInt(failedMatch[1]);
      const passRate = parseFloat(passRateMatch[1]);

      results.push({ round, passed, failed, passRate });
      console.log(`✓ ${passed} passed, ${failed} failed (${passRate}%)`);
    } else {
      console.log('✗ Failed to parse results');
      // console.log('Output:', output.slice(-500));
    }
  } catch (error) {
    console.log(`✗ Unexpected error: ${error.message}`);
  }
}

console.log('');
console.log('='.repeat(80));
console.log('BASELINE RESULTS SUMMARY');
console.log('='.repeat(80));
console.log('');

if (results.length > 0) {
  const totalPassed = results.reduce((sum, r) => sum + r.passed, 0);
  const totalFailed = results.reduce((sum, r) => sum + r.failed, 0);
  const totalTests = totalPassed + totalFailed;
  const avgPassRate = results.reduce((sum, r) => sum + r.passRate, 0) / results.length;
  const minPassRate = Math.min(...results.map(r => r.passRate));
  const maxPassRate = Math.max(...results.map(r => r.passRate));

  console.log(`Total rounds completed: ${results.length}/${ROUNDS}`);
  console.log(`Total tests run: ${totalTests}`);
  console.log(`Total passed: ${totalPassed}`);
  console.log(`Total failed: ${totalFailed}`);
  console.log(`Overall pass rate: ${(totalPassed * 100 / totalTests).toFixed(2)}%`);
  console.log('');
  console.log(`Average pass rate: ${avgPassRate.toFixed(2)}%`);
  console.log(`Min pass rate: ${minPassRate.toFixed(2)}%`);
  console.log(`Max pass rate: ${maxPassRate.toFixed(2)}%`);
  console.log('');

  // Show distribution
  console.log('Pass rate distribution:');
  const distribution = {};
  results.forEach(r => {
    const bucket = Math.floor(r.passRate / 5) * 5;
    distribution[bucket] = (distribution[bucket] || 0) + 1;
  });

  Object.keys(distribution).sort((a, b) => b - a).forEach(bucket => {
    const bar = '█'.repeat(distribution[bucket]);
    console.log(`  ${bucket}-${parseInt(bucket) + 4}%: ${bar} (${distribution[bucket]} rounds)`);
  });

  console.log('');
  console.log('Individual round results:');
  results.forEach(r => {
    console.log(`  Round ${r.round.toString().padStart(2)}: ${r.passRate.toFixed(1)}% (${r.passed}/${r.passed + r.failed})`);
  });
} else {
  console.log('❌ No successful test runs');
}

console.log('');
console.log('='.repeat(80));

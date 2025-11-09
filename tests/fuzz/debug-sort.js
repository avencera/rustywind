/**
 * Debug sorting for specific classes
 */

import { exec } from 'child_process';
import { promisify } from 'util';

const execAsync = promisify(exec);

async function sortWithRustyWind(classes) {
  const html = `<div class="${classes}"></div>`;
  const rustywindBin = '../../target/release/rustywind';
  const { stdout } = await execAsync(`echo '${html}' | ${rustywindBin} --stdin`);

  const match = stdout.trim().match(/class="([^"]*)"/);
  return match ? match[1] : '';
}

async function test() {
  console.log('Testing space-x-2 and touch-pan-down ordering\n');

  // Test both orders
  const result1 = await sortWithRustyWind('space-x-2 touch-pan-down');
  console.log('Input:  space-x-2 touch-pan-down');
  console.log('Output:', result1);
  console.log('');

  const result2 = await sortWithRustyWind('touch-pan-down space-x-2');
  console.log('Input:  touch-pan-down space-x-2');
  console.log('Output:', result2);
  console.log('');

  // Check which binary is being used
  const { stdout: version } = await execAsync(`${rustywindBin} --version`);
  console.log('Binary version:', version.trim());

  // Check the binary path
  const { stdout: which } = await execAsync(`ls -lh ${rustywindBin}`);
  console.log('Binary info:', which.trim());
}

test();

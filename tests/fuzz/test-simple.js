import { execSync } from 'child_process';

// Test cases focusing on specific property issues
const tests = [
  { name: 'size vs height', classes: ['h-auto', 'size-2'] },
  { name: 'size vs width', classes: ['w-4', 'size-2'] },
  { name: 'select vs snap', classes: ['snap-y', 'select-all'] },
  { name: 'select vs columns', classes: ['columns-md', 'select-auto'] },
  { name: 'rounded-none vs rounded-br', classes: ['rounded-br-none', 'rounded-none'] },
  { name: 'outline vs hue-rotate', classes: ['hue-rotate-30', 'outline-dashed'] },
  { name: 'outline vs drop-shadow', classes: ['drop-shadow-none', 'outline-dashed'] },
  { name: 'sepia vs delay', classes: ['sepia-0', 'delay-75'] },
  { name: 'space-y vs select', classes: ['select-all', 'space-y-1'] },
  { name: 'space-x vs space-y', classes: ['space-y-4', 'space-x-4'] },
  { name: 'py vs pt', classes: ['pt-2', 'py-0'] },
  { name: 'border-x vs border-r', classes: ['border-r-0', 'border-x-0'] },
  { name: 'divide-x-reverse vs rounded', classes: ['rounded-l-lg', 'divide-x-reverse'] },
  { name: 'bg-opacity first', classes: ['row-start-auto', 'bg-opacity-50'] },
];

console.log('Testing RustyWind sorting:\n');

for (const test of tests) {
  const input = test.classes.join(' ');
  try {
    const result = execSync(`echo "${input}" | cargo run --release --manifest-path ../../rustywind-cli/Cargo.toml --bin rustywind --quiet -- --stdin`, { encoding: 'utf-8' }).trim();
    console.log(`${test.name}:`);
    console.log(`  Input:      [${test.classes.join(', ')}]`);
    console.log(`  RustyWind:  [${result.split(' ').join(', ')}]`);
    console.log();
  } catch (error) {
    console.error(`Error testing ${test.name}:`, error.message);
  }
}

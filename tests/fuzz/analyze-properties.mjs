import fs from 'fs';

// Read the property-order.ts file
const content = fs.readFileSync('/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts', 'utf-8');

// Extract the array
const arrayMatch = content.match(/export default \[([\s\S]+)\]/);
if (!arrayMatch) {
  console.error('Could not find array');
  process.exit(1);
}

const arrayContent = arrayMatch[1];

// Parse the properties
const properties = [];
const lines = arrayContent.split('\n');
for (const line of lines) {
  const match = line.match(/^\s*'([^']+)'/);
  if (match) {
    properties.push(match[1]);
  }
}

console.log(`Total properties: ${properties.length}\n`);

// Find specific properties
const targets = [
  'outline',
  'outline-width',
  'outline-offset',
  'outline-color',
  '--tw-blur',
  '--tw-inset-ring-shadow',
  '--tw-ring-inset',
  'will-change',
  'user-select',
  'contain',
  'box-shadow',
  '--tw-shadow',
  'filter',
];

for (const prop of targets) {
  const index = properties.indexOf(prop);
  if (index >= 0) {
    console.log(`${index.toString().padStart(3, ' ')}: ${prop}`);
  } else {
    console.log(`  -: ${prop} (NOT FOUND)`);
  }
}

console.log('\nAnalysis:');
console.log('outline vs outline-offset:',
  properties.indexOf('outline'), '<', properties.indexOf('outline-offset'),
  '→', properties.indexOf('outline') < properties.indexOf('outline-offset'));

console.log('will-change vs user-select:',
  properties.indexOf('will-change'), '<', properties.indexOf('user-select'),
  '→', properties.indexOf('will-change') < properties.indexOf('user-select'));

const blurIdx = properties.indexOf('--tw-blur');
const ringIdx = properties.indexOf('--tw-inset-ring-shadow');
console.log('--tw-blur vs --tw-inset-ring-shadow:',
  blurIdx, '<', ringIdx,
  '→', blurIdx < ringIdx);

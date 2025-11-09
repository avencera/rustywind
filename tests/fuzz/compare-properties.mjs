import fs from 'fs';

// Read Tailwind v4 property-order.ts
const tw4Content = fs.readFileSync('/home/user/rustywind/tmp/tailwindcss/packages/tailwindcss/src/property-order.ts', 'utf-8');
const tw4ArrayMatch = tw4Content.match(/export default \[([\s\S]+)\]/);
const tw4Lines = tw4ArrayMatch[1].split('\n');
const tw4Properties = [];
for (const line of tw4Lines) {
  const match = line.match(/^\s*'([^']+)'/);
  if (match) tw4Properties.push(match[1]);
}

// Read our property_order.rs
const ourContent = fs.readFileSync('/home/user/rustywind/rustywind-core/src/property_order.rs', 'utf-8');
const ourArrayMatch = ourContent.match(/pub const PROPERTY_ORDER: &\[&str\] = &\[([\s\S]+?)\];/);
const ourLines = ourArrayMatch[1].split('\n');
const ourProperties = [];
for (const line of ourLines) {
  const match = line.match(/^\s*"([^"]+)"/);
  if (match) ourProperties.push(match[1]);
}

console.log(`Tailwind v4: ${tw4Properties.length} properties`);
console.log(`Our impl:    ${ourProperties.length} properties`);
console.log('');

// Find properties in ours but not in TW4
const extraInOurs = ourProperties.filter(p => !tw4Properties.includes(p));
if (extraInOurs.length > 0) {
  console.log(`Extra in ours (${extraInOurs.length}):`);
  extraInOurs.forEach((p, i) => {
    const idx = ourProperties.indexOf(p);
    console.log(`  ${idx.toString().padStart(3, ' ')}: ${p}`);
  });
  console.log('');
}

// Find properties in TW4 but not in ours
const missingInOurs = tw4Properties.filter(p => !ourProperties.includes(p));
if (missingInOurs.length > 0) {
  console.log(`Missing in ours (${missingInOurs.length}):`);
  missingInOurs.forEach((p, i) => {
    const idx = tw4Properties.indexOf(p);
    console.log(`  ${idx.toString().padStart(3, ' ')}: ${p}`);
  });
  console.log('');
}

// Find properties with different indices
console.log('Properties with different relative order:');
let foundDiff = false;
for (let i = 0; i < Math.min(tw4Properties.length, ourProperties.length); i++) {
  if (tw4Properties[i] !== ourProperties[i]) {
    console.log(`  Position ${i}:`);
    console.log(`    TW4: ${tw4Properties[i]}`);
    console.log(`    Ours: ${ourProperties[i]}`);
    foundDiff = true;
    if (i > 10) break; // Only show first few differences
  }
}
if (!foundDiff && extraInOurs.length === 0 && missingInOurs.length === 0) {
  console.log('  None! Orders match perfectly.');
}

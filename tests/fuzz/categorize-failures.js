/**
 * Categorize captured failures to identify patterns
 */

import fs from 'fs';

// Read the failures
const failures = JSON.parse(fs.readFileSync('failure-analysis.json', 'utf8'));

console.log(`\n📊 Analyzing ${failures.length} failures...\n`);

// Category definitions
const categories = {
  'group-peer-ordering': {
    name: 'Group/Peer Variant Ordering',
    description: 'group: vs peer: or compound variants with group/peer',
    matches: [],
    detect: (f) => {
      if (!f.prettier || !f.rustywind) return false;
      const p = f.prettier || '';
      const r = f.rustywind || '';
      return (p.includes('group:') || p.includes('peer:')) &&
             (r.includes('group:') || r.includes('peer:')) &&
             p !== r;
    }
  },
  'color-opacity-shade': {
    name: 'Color Opacity vs Shade Ordering',
    description: 'Classes like bg-gray-500 vs bg-white/20',
    matches: [],
    detect: (f) => {
      if (!f.prettier || !f.rustywind) return false;
      const p = f.prettier || '';
      const r = f.rustywind || '';
      // Check if one has opacity (/) and other has shade (-\d+)
      const hasOpacity = p.includes('/') || r.includes('/');
      const hasShade = /-\d{2,3}(?:\s|$)/.test(p) || /-\d{2,3}(?:\s|$)/.test(r);
      return hasOpacity && hasShade && p !== r;
    }
  },
  'numeric-comparison': {
    name: 'Numeric Value Comparison',
    description: 'Numeric values sorting incorrectly (w-2 vs w-4, max-w-xl vs max-w-2xl)',
    matches: [],
    detect: (f) => {
      if (!f.prettier || !f.rustywind) return false;
      const p = f.prettier || '';
      const r = f.rustywind || '';
      // Look for numeric differences in similar utilities
      const pMatch = p.match(/^([a-z-]+)-(\d+|[\d.]+xl)/);
      const rMatch = r.match(/^([a-z-]+)-(\d+|[\d.]+xl)/);
      return pMatch && rMatch && pMatch[1] === rMatch[1] && pMatch[2] !== rMatch[2];
    }
  },
  'arbitrary-keyword': {
    name: 'Arbitrary vs Keyword Ordering',
    description: 'Arbitrary values [..] vs keyword values',
    matches: [],
    detect: (f) => {
      if (!f.prettier || !f.rustywind) return false;
      const p = f.prettier || '';
      const r = f.rustywind || '';
      const pHasArbitrary = p.includes('[') && p.includes(']');
      const rHasArbitrary = r.includes('[') && r.includes(']');
      return (pHasArbitrary || rHasArbitrary) && p !== r;
    }
  },
  'duplicate-classes': {
    name: 'Duplicate Class Handling',
    description: 'Same class appears multiple times',
    matches: [],
    detect: (f) => {
      if (!f.prettierClasses || !f.rustywindClasses) return false;
      const pDupes = f.prettierClasses.length !== new Set(f.prettierClasses).size;
      const rDupes = f.rustywindClasses.length !== new Set(f.rustywindClasses).size;
      const origDupes = f.original && f.original.length !== new Set(f.original).size;
      return origDupes && (pDupes || rDupes);
    }
  },
  'length-mismatch': {
    name: 'Length Mismatch',
    description: 'Different number of classes in output',
    matches: [],
    detect: (f) => {
      if (!f.prettierClasses || !f.rustywindClasses) return false;
      return f.prettierClasses.length !== f.rustywindClasses.length;
    }
  },
  'variant-stacking': {
    name: 'Variant Stacking Order',
    description: 'Multiple variants like dark:md:, xl:hover:, etc.',
    matches: [],
    detect: (f) => {
      if (!f.prettier || !f.rustywind) return false;
      const p = f.prettier || '';
      const r = f.rustywind || '';
      const pVariants = (p.match(/:/g) || []).length;
      const rVariants = (r.match(/:/g) || []).length;
      return (pVariants >= 2 || rVariants >= 2) && p !== r;
    }
  },
  'property-order': {
    name: 'Property Index Ordering',
    description: 'Different CSS properties sorting incorrectly',
    matches: [],
    detect: (f) => {
      // This is a catch-all for failures that don't match other patterns
      return true;
    }
  }
};

// Categorize each failure
for (const failure of failures) {
  let categorized = false;

  for (const [key, category] of Object.entries(categories)) {
    // Skip property-order for now (catch-all)
    if (key === 'property-order') continue;

    if (category.detect(failure)) {
      category.matches.push(failure);
      categorized = true;
      break; // Only assign to first matching category
    }
  }

  // If not categorized, add to property-order
  if (!categorized) {
    categories['property-order'].matches.push(failure);
  }
}

// Print summary
console.log('=' .repeat(80));
console.log('FAILURE CATEGORY SUMMARY');
console.log('='.repeat(80));
console.log('');

const sortedCategories = Object.entries(categories)
  .sort((a, b) => b[1].matches.length - a[1].matches.length)
  .filter(([_, cat]) => cat.matches.length > 0);

for (const [key, category] of sortedCategories) {
  const percentage = (category.matches.length / failures.length * 100).toFixed(1);
  console.log(`📌 ${category.name}`);
  console.log(`   ${category.description}`);
  console.log(`   Count: ${category.matches.length} (${percentage}%)`);
  console.log('');
}

// Print detailed examples for each category
console.log('');
console.log('='.repeat(80));
console.log('DETAILED EXAMPLES (up to 3 per category)');
console.log('='.repeat(80));
console.log('');

for (const [key, category] of sortedCategories) {
  console.log(`\n## ${category.name} (${category.matches.length} failures)\n`);

  const examples = category.matches.slice(0, 3);
  for (let i = 0; i < examples.length; i++) {
    const f = examples[i];
    console.log(`Example ${i + 1}:`);
    console.log(`  Mismatch at position: ${f.position}`);
    console.log(`  Prettier:  "${f.prettier || 'N/A'}"`);
    console.log(`  RustyWind: "${f.rustywind || 'N/A'}"`);

    if (f.prettierClasses && f.rustywindClasses) {
      // Show context around mismatch
      const start = Math.max(0, f.position - 2);
      const end = Math.min(f.prettierClasses.length, f.position + 3);
      console.log(`  Context (Prettier):  [${f.prettierClasses.slice(start, end).join(', ')}]`);
      console.log(`  Context (RustyWind): [${f.rustywindClasses.slice(start, end).join(', ')}]`);
    }
    console.log('');
  }
}

// Save categorized results
const categorizedResults = {
  summary: Object.entries(categories).map(([key, cat]) => ({
    category: cat.name,
    description: cat.description,
    count: cat.matches.length,
    percentage: (cat.matches.length / failures.length * 100).toFixed(1)
  })).filter(c => c.count > 0).sort((a, b) => b.count - a.count),
  categories: Object.fromEntries(
    Object.entries(categories)
      .filter(([_, cat]) => cat.matches.length > 0)
      .map(([key, cat]) => [key, {
        name: cat.name,
        description: cat.description,
        count: cat.matches.length,
        examples: cat.matches.slice(0, 5)
      }])
  )
};

fs.writeFileSync('failure-categories.json', JSON.stringify(categorizedResults, null, 2));
console.log('\n✅ Saved categorization to failure-categories.json\n');

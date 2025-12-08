#!/usr/bin/env node
import fs from 'node:fs/promises';
import process from 'node:process';
import postcss from 'postcss';
import selectorParser from 'postcss-selector-parser';

const countClassesInCss = async (filePath) => {
  const classSet = new Set();
  const cssContent = await fs.readFile(filePath, 'utf8');

  try {
    const root = postcss.parse(cssContent, { from: filePath });

    root.walkRules(rule => {
      const transformer = selectorParser(selectorsAst => {
        selectorsAst.walkClasses(node => {
          if (node && node.value) {
            // Remove backslashes to match Rust behavior: .replace('\\', '')
            const className = node.value.replace(/\\/g, '');
            classSet.add(className);
          }
        });
      });

      try {
        transformer.processSync(rule.selector);
      } catch (err) {
        // Skip malformed selectors
        process.stderr.write(`Warning: Could not parse selector: ${rule.selector}\n`);
      }
    });
  } catch (err) {
    process.stderr.write(`Error parsing CSS: ${err.message}\n`);
    process.exit(1);
  }

  return classSet;
};

const run = async () => {
  const filePath = process.argv[2];

  if (!filePath) {
    process.stderr.write('Usage: node count-css-classes.mjs <css-file>\n');
    process.exit(1);
  }

  const classes = await countClassesInCss(filePath);
  const sortedClasses = [...classes].sort();

  process.stdout.write(`Total unique classes: ${classes.size}\n\n`);
  process.stdout.write('First 20 classes:\n');
  for (const [i, className] of sortedClasses.slice(0, 20).entries()) {
    process.stdout.write(`  ${i}: ${className}\n`);
  }

  if (sortedClasses.length > 20) {
    process.stdout.write(`  ... and ${sortedClasses.length - 20} more\n`);
  }
};

run().catch(err => {
  process.stderr.write(`${err}\n`);
  process.exit(1);
});

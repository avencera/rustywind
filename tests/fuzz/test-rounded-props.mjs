// Test to understand Tailwind's rounded utility behavior
import { sortClasses } from 'prettier-plugin-tailwindcss';

const tests = [
  ['rounded', 'rounded-[14px]', 'rounded-lg'],
  ['rounded-none', 'rounded', 'rounded-[14px]'],
  ['rounded-t', 'rounded-[14px]'],
  ['rounded-tl', 'rounded-[14px]'],
];

for (const classes of tests) {
  const sorted = sortClasses(classes.join(' '));
  console.log(classes.join(', ').padEnd(50), '→', sorted);
}

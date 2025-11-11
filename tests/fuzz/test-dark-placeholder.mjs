import prettier from 'prettier';

async function test() {
  const tests = [
    // Test dark:placeholder: vs single variants
    '<div class="dark:placeholder:columns-4 peer-focus:text-2xl"></div>',
    '<div class="peer-focus:text-2xl dark:placeholder:columns-4"></div>',

    // Test dark:placeholder: vs base classes
    '<div class="dark:placeholder:columns-4 p-4"></div>',
    '<div class="p-4 dark:placeholder:columns-4"></div>',

    // Test dark:placeholder: vs other double-stacks
    '<div class="dark:placeholder:columns-4 dark:hover:bg-white"></div>',
    '<div class="dark:hover:bg-white dark:placeholder:columns-4"></div>',

    // Test dark:placeholder: vs focus-within:
    '<div class="focus-within:brightness-150 dark:placeholder:space-x-4"></div>',
    '<div class="dark:placeholder:space-x-4 focus-within:brightness-150"></div>',

    // Test the variant order itself
    '<div class="dark:p-4 placeholder:p-4 dark:placeholder:p-4"></div>',
  ];

  console.log('Testing dark:placeholder: sorting with Prettier:\n');
  console.log('='.repeat(80));

  for (const html of tests) {
    const formatted = await prettier.format(html, {
      parser: 'html',
      plugins: ['prettier-plugin-tailwindcss'],
      printWidth: 10000,
    });
    console.log(`\nInput:  ${html}`);
    console.log(`Output: ${formatted.trim()}`);
  }
}

test();

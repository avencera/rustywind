import prettier from 'prettier';

async function testOrdering(classes) {
    const html = `<div class="${classes}"></div>`;
    const result = await prettier.format(html, {
        parser: 'html',
        plugins: ['prettier-plugin-tailwindcss'],
        printWidth: 10000,
    });
    const match = result.match(/class="([^"]*)"/);
    return match ? match[1] : '';
}

async function main() {
    const tests = [
        // Test all border properties together
        'border-solid border-t-0 border-r-0 border-b-0 border-l-0 border-red-500',

        // Test all rounded properties together
        'rounded-lg rounded-t rounded-r rounded-b rounded-l rounded-tl rounded-tr rounded-br rounded-bl',

        // Test mix-blend modes
        'mix-blend-normal mix-blend-multiply mix-blend-screen mix-blend-overlay mix-blend-darken',

        // Test filters
        'blur grayscale-0 backdrop-blur backdrop-grayscale backdrop-sepia sepia',

        // Test outline and ring
        'outline outline-offset-0 ring-0 ring-offset-0',

        // Test bg vs object
        'bg-none bg-red-500 object-contain object-left-top',

        // Test display ordering
        'block inline inline-block flex inline-flex grid inline-grid hidden contents',

        // Test break vs rounded
        'break-normal break-words rounded-lg rounded-t',
    ];

    for (const test of tests) {
        const result = await testOrdering(test);
        console.log(`Input:  ${test}`);
        console.log(`Output: ${result}`);
        console.log('');
    }
}

main().catch(console.error);

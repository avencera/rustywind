# Fuzz Testing Infrastructure

This directory compares RustyWind class ordering against `prettier-plugin-tailwindcss`.

## What Runs

- `compare.js` generates random Tailwind class lists and compares RustyWind output to Prettier output.
- `compare-real-world-patterns.js` generates class lists biased toward real-world failure patterns. If `failure-patterns.json` has been generated locally, it uses that data; otherwise it falls back to checked-in representative patterns.
- `tailwind-classes.js` and `legacy-classes.js` provide the class and variant pools used by the generators.

The default npm test runs both comparison suites:

```bash
cd tests/fuzz
npm ci
npm test
```

The test scripts expect a release RustyWind binary at `../../target/release/rustywind`.

## Preferred Runner

Use the Rust `xtask` wrapper for repeatable multi-round runs and failure analysis:

```bash
cargo build --release
cd tests/fuzz && npm ci
cd ../..
cargo xtask fuzz run 25
```

The runner saves detailed failures and category summaries under `tests/fuzz/tools/output/`, which is ignored by git.

## Useful Commands

```bash
# one npm pass: 100 random cases + 100 pattern-focused cases
cd tests/fuzz && npm test

# include legacy Tailwind classes in the random fuzz pool
cd tests/fuzz && npm run test:with-legacy

# run 100 xtask rounds with four workers
cargo xtask fuzz run 100 --workers 4
```

## Pattern Data

`extract-failure-patterns.mjs` and `extract-real-world-patterns.mjs` are optional local analysis tools. They expect a local `../tailwind-sorting-test-files/test-files` dataset and can generate `failure-patterns.json`, `real-world-patterns.json`, and `common-patterns.json`.

Those generated JSON files are intentionally not required for `npm test`; the checked-in fallback keeps the test suite runnable from a clean checkout.

## Upgrade Notes

When Tailwind or `prettier-plugin-tailwindcss` changes, update the pinned npm dependencies, rerun the fuzz suite, and follow [docs/UPGRADE_CHECKLIST.md](docs/UPGRADE_CHECKLIST.md).

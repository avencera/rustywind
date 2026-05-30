# RustyWind Fuzz Status

This folder tracks high-level notes for keeping RustyWind close to `prettier-plugin-tailwindcss`.

## Current Baseline

- The tracked Rust tests cover parser behavior, property ordering, variant ordering, arbitrary values, and regressions from fuzz failures.
- `tests/fuzz` provides direct runtime comparisons against Prettier.
- Run `cargo xtask fuzz run 25` before release to refresh the pass-rate baseline and write failure summaries to `tests/fuzz/tools/output/`.

## Known Risk Areas

1. Variant canonicalization
   - Compound variants such as `group-*` and `peer-*`
   - Mixed responsive, dark-mode, pseudo-class, and arbitrary variants
   - Duplicate or repeated variants

2. Property order and declaration counts
   - Ring, shadow, drop-shadow, and outline utilities
   - Multi-declaration utilities such as rounded corners and text sizes
   - Synthetic Tailwind variables such as `--tw-ring-shadow`

3. Arbitrary values
   - Border, rounded, color opacity, and transform values
   - Interactions between arbitrary values and keyword utilities

## Release Gate

Do not treat the fuzz docs as proof by themselves. Before release, run:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release
cd tests/fuzz && npm ci && npm test
cd ../..
cargo xtask fuzz run 25
```

For dependency upgrades, use [UPGRADE_CHECKLIST.md](UPGRADE_CHECKLIST.md).

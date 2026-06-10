# Changelog

## [Unreleased]

## [0.25.0-rc.1] - 2026-06-10

### Added

- Add `--tailwind-prefix` support for Tailwind v3 (`tw-`) and v4 (`tw:`)
  prefix styles while preserving original class strings in output

### Breaking changes

- `RustyWind` now includes a `tailwind_prefix` option. Code constructing
  `RustyWind` with a struct literal must set `tailwind_prefix: None` or use
  `RustyWind::new`.
- `PatternSorter` is no longer a unit struct. Use `PatternSorter::new()` or
  `PatternSorter::default()`.
- For prefix-aware sorters, `SortKey.class` stores the normalized class used for
  sorting rather than the original prefixed input.

## [0.25.0-alpha.1] - 2026-05-30

### Tailwind v4 support

This release aligns rustywind's class sorting with Tailwind CSS v4 and the
latest prettier-plugin-tailwindcss, adding support for new utilities, variants,
and sorting behavior.

- Update class extraction regex to handle arbitrary values with whitespace
  (e.g. `max-w-[min(100%, 500px)]`)
- Add bracket-depth-aware token splitting
- Handle ellipsis placeholders (`...` and `…`) in dedup and sorting
- Support prefix important modifier (`!bg-white`) and arbitrary CSS properties
  (`[--foo:bar]`)
- Support CSS variable shorthand values (`text-(--color)`)
- Expand variant list from 58 to 82, adding `*`, `**`, `not-*`, `has`, `in`,
  `aria`, `data`, `nth-*`, `max-*`, `min-*`
- Add selector-dynamic and variant-mask comparison for proper data-/aria-/nth-*
  variant ordering
- Classify arbitrary variant selectors by type (combinators, pseudo-classes,
  attribute selectors)
- Update property order (342 → 351) with mask, shadow, perspective, and
  text-shadow properties
- Expand utility mapping with mask, gradient, drop-shadow, text-shadow, and
  many other Tailwind v4 utilities
- Improve color recognition with explicit core color name and shade checks
- Add npm dist-tag support for prerelease publishing

## [0.24.3] - 2025-08-07

- Fix version number mismatch

## [0.24.2] - 2025-08-07

- Release includes builds for MacOS

## [0.24.1] - 2025-08-07

- Fix class extraction regex, [#119](https://github.com/avencera/rustywind/pull/119), thanks [@5need](https://github.com/5need)

## [0.24.0] - 2025-02-27

- In MacOS limit number of threads to 4 for up to a 400% performance boost
  - performance improvement is higher on macs with more cores, saw 4x speed up on my M4 Pro with 14 cores

## [0.23.1] - 2024-10-21

- Fix default regex

## [0.23.0] - 2024-10-21

- Add options to handle wrapped classes to extend the set of use cases [#109](https://github.com/avencera/rustywind/pull/109), thanks [@dikkadev](https://github.com/dikkadev])

## [0.22.1] - 2024-10-21

- Fix regex for parsing css classes, [#99](https://github.com/avencera/rustywind/pull/99), thanks [@DanikVitek](https://github.com/DanikVitek)
- Update cargo deps

## [0.22.0] - 2024-04-12

- Split rustywind into multiple crates, [#100](https://github.com/avencera/rustywind/pull/100), thanks [@Rolv-Apneseth](https://github.com/Rolv-Apneseth), and [@bram209](https://github.com/bram209/)

## [0.21.1] - 2024-03-20

- Switch up std HashMap for faster hasher ahash
- Update rustls to 0.22
- Update cargo deps

## [0.21.0] - 2023-12-20

- Prevent writing to file if the contents hasn't changed. Fixes: [#88](https://github.com/avencera/rustywind/issues/88)
- Show UI difference for files that were/will be changed vs ones that won't be touched
- Upgraded deps

## [0.20.0] - 2023-10-15

- Add new `--quiet` flag to prevent log messages [#86](https://github.com/avencera/rustywind/pull/86), thanks [@azzamsa](https://github.com/azzamsa)
- Updated dependencies

## [0.19.0] - 2023-08-13

- Add new `--skip-ssl-verification`flag for use with `https` in dev for `--vite-css` [#78](https://github.com/avencera/rustywind/pull/78), thanks [@praveenperera](https://github.com/praveenperera)

## [0.18.0] - 2023-08-11

- Add expiremental support for getting CSS order from a vite URL [#77](https://github.com/avencera/rustywind/pull/77), thanks [@praveenperera](https://github.com/praveenperera)

## [0.17.0] - 2023-08-01

- Match sorting rules to tailwind prettier plugin [#76](https://github.com/avencera/rustywind/pull/76), thanks [@praveenperera](https://github.com/praveenperera)

## [0.16.0] - 2023-04-23

- Updated all dependencies, including `aho-corasick` [#73](https://github.com/avencera/rustywind/pull/73), thanks [@dnaka91](https://github.com/dnaka91)
- Change regex to work with colors with opacity [#72](https://github.com/avencera/rustywind/pull/72), thanks [@dnaka91](https://github.com/dnaka91)

## [0.15.4] - 2023-02-24

- Update cargo.lock file to new version [#70](https://github.com/avencera/rustywind/issues/70)
- Fix Regex to only match the characters selected (and work with conditionals), thanks [@rubas](https://github.com/rubas) [#66](https://github.com/avencera/rustywind/pull/66)

## [0.15.3] - 2023-01-03

- Fix npm publishing thanks [@adamdicarlo0](https://github.com/adamdicarlo0) [#69](https://github.com/avencera/rustywind/pull/69)

## [0.15.2] - 2023-01-03

- Fix download rate limited by github thanks [@adamdicarlo0](https://github.com/adamdicarlo0) [#68](https://github.com/avencera/rustywind/pull/68)

## [0.15.1] - 2022-09-06

- Fixed removing `\n` newline when formatting from STDIN
- Updated dependencies

## [0.15.0] - 2022-04-25

- Add `--config-file` option, thanks [@mweiss-carerev](https://github.com/mweiss-carerev) [#58](https://github.com/avencera/rustywind/pull/58)

## [0.14.0] - 2022-03-14

- `--ignore-files` option, thanks [@ftonato](https://github.com/neonowy) [#55](https://github.com/avencera/rustywind/pull/55)

- Improve error message on custom regex

### Internal

- Refactored, using Clap3 derive macros to parse args

## [0.13.0] - 2022-02-09

### Added

- `--custom-regex` option, thanks [@neonowy](https://github.com/neonowy) [#39](https://github.com/avencera/rustywind/pull/39)

- JIT classes support, thanks [@royduin](https://github.com/royduin) [#42](https://github.com/avencera/rustywind/pull/42)

- New `--check-formatted` option, thanks [@praveenperera](https://github.com/praveenperera) [#45](https://github.com/avencera/rustywind/pull/45)

### Internal

- Replace `lazy_static` crate with `once_cell`, thanks [@praveenperera](https://github.com/praveenperera) [#46](https://github.com/avencera/rustywind/pull/46)

- Update `clap` crate to 3.0, thanks [@praveenperera](https://github.com/praveenperera) [#47](https://github.com/avencera/rustywind/pull/47)

### Fixed

- No longer exit with error if no classes are found, thanks [@shackra](https://github.com/shackra) [#50](https://github.com/avencera/rustywind/pull/50)

## [0.12.2] - 2021-07-07

- Create dockerized version thanks [@JeroenG](https://github.com/Jeroen-G) [#36](https://github.com/avencera/rustywind/pull/36)

## [0.12.1] - 2021-06-13

- Prevent panic if class name same as variant is used

## [0.12.0] - 2021-06-13

- Fix not sorting half classes properly `ex: mt-0.5`
- Sort all variant classes

## [0.11.0] - 2021-06-11

- Sorts responsive classes with the same default sorter [#28](https://github.com/avencera/rustywind/issues/28)

## [0.10.0] - 2021-06-10

- Run on multiple files or folders

```shell
rustywind --write abc.js efg.js

rustywind --write abc/templates efg/templates
```

## [0.9.1] – 2021-06-09

- Report correct version number

## [0.9.0] – 2021-06-02

- Split classes by all ASCII whitespace characters (will now separate classes by spaces or new lines) - thanks [@mklein994](https://github.com/mklein994)

## [0.8.1] – 2021-03-26

- Support for M1 macs

## [0.8.0] – 2021-01-09

### Added

- New `--stdin` flag sort contents in STDIN and print out sorted contents to STDOUT

## [0.7.1] – 2021-01-09

### Fixed

- Fixed windows releases not being created

## [0.7.0] – 2020-12-06

### Changed

- Changed default sorter to match headwind, now works with Tailwind 2.0 (by [@dhrubabasu](https://github.com/dhrubabasu))

## [0.6.7] – 2020-09-05

### Changed

- Changed default sorter to match headwind

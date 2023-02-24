# Changelog

## [Unreleased]

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

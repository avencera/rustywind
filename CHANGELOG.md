# Changelog

## [Unreleased]

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

- Changed default sorter to match headwind, now works with Tailwind 2.0 (by @dhrubabasu)

## [0.6.7] – 2020-09-05

### Changed

- Changed default sorter to match headwind

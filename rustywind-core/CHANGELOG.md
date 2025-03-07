# Changelog

## [Unreleased]

### Performance

- In MacOS limit number of threads to 4 for up to a 400% performance boost

### Refactor

- Completely refactored the public API, now all the functionality is in the `RustyWind` struct

### Changed

- Changed `HowClassesAreWrapped` to `ClassWrapping`
- Fixed some clippy warnings
- Implemented `Default` for `Options`

## [0.2.0] - 2024-10-21

- Add options to handle wrapped classes to extend the set of use cases [#109](https://github.com/avencera/rustywind/pull/109), thanks [@dikkadev]](https://github.com/dikkadev])

## [0.1.3] - 2024-10-21

- Fix regex for parsing css classes, [#99](https://github.com/avencera/rustywind/pull/99), thanks [@DanikVitek](https://github.com/DanikVitek)

## [0.1.2] - 2024-05-27

- Made `sort_classes` function public, thanks [@Rolv-Apneseth](https://github.com/Rolv-Apneseth), in [#104](https://github.com/avencera/rustywind/pull/104)

## [0.1.1] - 2024-04-12

- Improve docs

## [0.1.0] - 2024-04-12

- Initial release of RustyWind functionality split into multiple crates, thanks [@Rolv-Apneseth](https://github.com/Rolv-Apneseth) and [@bram209](https://github.com/bram209), in [#100](https://github.com/avencera/rustywind/pull/100)

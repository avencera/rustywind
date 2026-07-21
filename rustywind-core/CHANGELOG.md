# Changelog

## Unreleased

## [0.5.0] - 2026-07-21

### Added

- Add first-class Astro parsing for static quoted class attributes while preserving frontmatter,
  expressions, `class:list`, raw content, and component semantics
- Add first-class Winnow parsing for directly quoted JSX and TSX `class` and `className`
  attributes while preserving host-language program text and dynamic attributes
- Add language-aware class extraction for HTML, Svelte, Astro, Django, Jinja, Twig, Liquid,
  Handlebars, ERB, EJS, PHP, Blade, Lit, Ruby, JSX, and TSX
- Preserve template expressions as opaque boundaries while independently sorting adjacent static
  class runs

### Changed

- Parse known markup languages with Winnow and extract typed class-attribute spans directly instead
  of scanning every regular-expression match against every tag
- Parse template expressions, arbitrary-value brackets, comments, strings, and regular expressions
  with a shared fail-closed Winnow grammar
- Order container-query variants—including default, named, and arbitrary breakpoints—by direction
  and resolved size to match Tailwind / Prettier
- Sort parenthesized ring color utilities and typed ring length custom properties with Tailwind
  ring ordering (for example `ring-(--color)` and `ring-(length:--width)`)

### Fixed

- Preserve Django and Jinja output expressions and control tags, including expressions attached to
  class names, [#138](https://github.com/avencera/rustywind/issues/138)
- Prevent ERB ternaries and quoted expressions from being corrupted,
  [#140](https://github.com/avencera/rustywind/issues/140)
- Prevent Svelte inline conditionals from being reordered as static classes,
  [#142](https://github.com/avencera/rustywind/issues/142)
- Keep ambiguous template expressions in plain `.html` files unchanged unless an explicit
  template-language profile is selected
- Sort valid Tailwind punctuation and quoted arbitrary values in files with unrecognized extensions
  while continuing to reject template expressions
- Preserve nested class attributes inside capitalized Svelte components
- Continue accepting the first positional capture in custom regex extractors, as in RustyWind 0.24
  and earlier, while also supporting a named `classes` capture

### Performance

- Index markup tag spans so class extraction no longer scans every tag for every class attribute in
  large documents

### Breaking changes

- `RustyWind::has_classes` now accepts a `SourceDocument`, `RustyWind::sort_file_contents` is
  replaced by `sort_document`, and `RustyWind::sort_classes` is replaced by `sort_class_list`, which
  requires a validated `PlainClassList`
- `FinderRegex::CustomRegex` now contains a `CustomClassExtractor`; library callers constructing this
  variant must validate their `Regex` with `CustomClassExtractor::new`
- `SourceLanguage` now includes `Astro` and `Jsx` (shared by `.jsx` and `.tsx` sources). Downstream
  exhaustive matches must handle both variants
- The default extractor for known markup languages only sorts actual quoted `class` and `className`
  attributes. Class-looking text in comments, raw elements, non-markup program expressions, and
  other attributes is no longer rewritten

## [0.4.1] - 2026-07-08

### Fixed

- Skip class strings containing template syntax to avoid corrupting
  template/interpolation class attributes, [#141](https://github.com/avencera/rustywind/pull/141), thanks [@brianfoshee](https://github.com/brianfoshee)

## [0.4.0] - 2026-07-06

### Added

- Add Tailwind prefix-aware sorting for Tailwind v3 (`tw-`) and v4 (`tw:`)
  prefix styles while preserving original class strings in output

### Breaking changes

- `RustyWind` now includes a `tailwind_prefix` option. Code constructing
  `RustyWind` with a struct literal must set `tailwind_prefix: None` or use
  `RustyWind::new`.
- `PatternSorter` is no longer a unit struct. Use `PatternSorter::new()` or
  `PatternSorter::default()`.
- For prefix-aware sorters, `SortKey.class` stores the normalized class used for
  sorting rather than the original prefixed input.

## [0.4.0-rc.1] - 2026-06-10

### Added

- Add Tailwind prefix-aware sorting for Tailwind v3 (`tw-`) and v4 (`tw:`)
  prefix styles while preserving original class strings in output

### Breaking changes

- `RustyWind` now includes a `tailwind_prefix` option. Code constructing
  `RustyWind` with a struct literal must set `tailwind_prefix: None` or use
  `RustyWind::new`.
- `PatternSorter` is no longer a unit struct. Use `PatternSorter::new()` or
  `PatternSorter::default()`.
- For prefix-aware sorters, `SortKey.class` stores the normalized class used for
  sorting rather than the original prefixed input.

## [0.3.1] - 2025-08-07

- Fix class extraction regex, [#119](https://github.com/avencera/rustywind/pull/119), thanks [@5need](https://github.com/5need)

## [0.3.0] - 2025-02-27

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

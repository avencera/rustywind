# Tailwind comparison engine

This package compares RustyWind's ordering of static quoted class attributes
with Prettier's Tailwind CSS plugin across pinned external corpora. It supports
JSX, TSX, Svelte, and Astro sources. The engine reverses eligible class lists in
memory, formats both the original and scrambled source with Prettier, and runs
RustyWind on the scrambled source. This avoids false agreement from
already-sorted source while detecting formatter output that depends on input
order.

Install the exact toolchain and run the focused unit tests with:

```bash
cd tests/tailwind-compare
npm ci
npm test
```

The repository's `xtask` command is the intended orchestration layer. It checks
out the corpus and supplies every input explicitly:

```bash
node compare.mjs \
  --corpus example/project \
  --repo /path/to/project \
  --revision 0123456789abcdef0123456789abcdef01234567 \
  --scope src \
  --kind tsx \
  --limit 30 \
  --rustywind ../../target/release/rustywind \
  --stylesheet tailwind.css \
  --output /path/to/report.json
```

Candidates are ranked by descending static attribute count, then by bytewise
UTF-8 relative path. The report fingerprint is SHA-256 over those ordered
selected records: relative path, NUL, scrambled source, NUL. Reports contain
only logical corpus paths, not checkout or executable paths.

The process exits nonzero for invalid arguments, missing inputs, and a failed
Tailwind classification probe. Source parse errors and RustyWind failures are
recorded in the report for the caller to interpret alongside ordering and
extraction findings.

RustyWind may only change bytes inside real quoted `class` and `className`
attribute values. Any change to program text, comments, dynamic attributes, or
other source bytes is recorded as a failing source-preservation error.

Schema version 2 classifies each attribute independently. Attribute-count and
token-multiset mismatches take precedence. When both Prettier runs preserve the
tokens but produce different orders, the attribute is reported as the
non-failing `prettier-nonconvergent` classification and Prettier is not used as
an ordering oracle for it. Only convergent Prettier output can produce `exact`,
`custom-only`, or `known-order` classifications. Detail records use the
explicit fields `original`, `scrambled`, `prettierOriginal`,
`prettierScrambled`, and `rustywind`.

The `prettierNonconvergent` summary field counts attributes for which the two
Prettier runs disagree. `prettierChanged` continues to count changes from the
scrambled input to the scrambled Prettier run, including nonconvergent
attributes. Known-order and extraction mismatches remain actionable failures;
Prettier nonconvergence and custom-only differences do not fail a corpus.

Only quoted `class` and `className` literals without template delimiters are in
scope. Dynamic expressions, `class:list`, helper calls such as `cn` and `cva`,
and project-specific Tailwind configuration are outside this comparison; the
shared shadcn-style semantic palette is the one deliberate exception. A token
is Tailwind-known when the pinned Prettier plugin moves it past two unknown
sentinels using a Tailwind 4 stylesheet that layers a shadcn-style semantic
`@theme` palette (background/foreground, card, muted, `sidebar-*`, `chart-1`…
`chart-5`, and others) over the defaults so design-system color tokens are
graded as known utilities. Differences that preserve the order of known tokens
are reported separately as `custom-only`.

RustyWind's named-color fallback is configuration-blind, which carries two
known limitations. A project theme key in another namespace that shares a
color-capable prefix (for example `--text-display`, making `text-display` a
font-size utility) is still sorted as a color by default; `--no-named-colors`
keeps ambiguous names unknown, while deriving order from the project's real CSS
via `--output-css-file` or `--vite-css` provides exact project-specific order.
Invalid opacity modifiers such as `bg-muted/foo` are sorted as their base
color even though Tailwind rejects the candidate; modifier validation is
per-utility (`text-sm/tight` and `group/name` are valid) and is out of scope
for the comparison engine.

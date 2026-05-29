# Upgrade Checklist (Tailwind / Prettier Plugin)

Use this file whenever a new version of Tailwind CSS or `prettier-plugin-tailwindcss` ships. It highlights the areas we rely on most so we can confirm whether the upstream behavior still matches our mirror.

## 1. Variant Ordering & Canonicalization
- **Files to review:**
  - `tailwindcss/packages/tailwindcss/src/canonicalize-candidates.ts`
  - `tailwindcss/packages/tailwindcss/src/variants.ts`
  - `prettier-plugin-tailwindcss/src/sorting.ts`
- **What to verify:**
  - Has the canonicalization pipeline changed (e.g., new variant types, different stacking rules)?
  - Did `Variants.compare` add/remove recursion or alter compound handling?
  - Did the plugin adopt any new special cases (e.g., for arbitrary variants) that we must mirror in `variant_order.rs` / `pattern_sorter.rs`?

## 2. Property Order & Declaration Counts
- **Files to review:**
  - `tailwindcss/packages/tailwindcss/src/property-order.ts`
  - `tailwindcss/packages/tailwindcss/src/utilities/*` (for multi-declaration utilities like ring/shadow, drop-shadow, rounded corners)
  - Bundled output in `prettier-plugin-tailwindcss/dist/index.mjs` (search for the property-order array and comment block describing comparison tiers)
- **What to verify:**
  - Has the property order array changed length or position for critical entries (`box-shadow`, `--tw-ring-shadow`, `outline-style`, etc.)?
  - Did any utilities gain or lose CSS declarations (affects our property-count map)?
  - Are there new synthetic properties (e.g., `--tw-*`) that we need to add to `utility_map.rs`?

## 3. Utility Canonicalization & Arbitrary Values
- **Files to review:**
  - `tailwindcss/packages/tailwindcss/src/canonicalize-candidates.ts` (especially the sections that normalize arbitrary values and map stacked utilities)
  - `tailwindcss/packages/tailwindcss/src/utilities/index.ts`
- **What to verify:**
  - Any changes in how Tailwind canonicalizes arbitrary values (`rounded-[...]`, `bg-[...]/opacity`, etc.)?
  - New heuristics for ordering arbitrary vs keyword values (if so, migrate them into our property-count logic instead of ad-hoc rules).

## 4. Prettier Plugin Sorting Contract
- **Files to review:**
  - `prettier-plugin-tailwindcss/src/sorting.ts`
  - `prettier-plugin-tailwindcss/src/types.ts` (for changes to `TransformerEnv.context.getClassOrder` expectations)
- **What to verify:**
  - Is the plugin still delegating entirely to Tailwind’s `getClassOrder`, or did it add its own post-processing (e.g., special handling for `...` placeholders, whitespace preservation)?
  - Any new plugin options that influence sorting (e.g., `tailwindPreserveWhitespace`, `tailwindPreserveDuplicates` semantics)?

## 5. Regression Guardrails
Whenever you spot a change upstream:
1. Run focused property probes such as `tests/fuzz/compare-properties.mjs`, `test-property-mapping.mjs`, and the relevant `test-*.mjs` files against the updated packages.
2. Capture fresh fuzz baselines with `cargo xtask fuzz run 25` and save any notable failure summaries from `tests/fuzz/tools/output/`.
3. Update this checklist or `docs/README.md` only if upstream behavior actually changed; otherwise, document the validation in the release notes.

> **Reminder:** upstream Tailwind and `prettier-plugin-tailwindcss` source should be the source of truth. Mirror that logic before trusting internal guesses.

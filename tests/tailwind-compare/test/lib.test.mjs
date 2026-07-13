import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";
import test from "node:test";

import * as prettier from "prettier";
import * as sveltePlugin from "prettier-plugin-svelte";
import * as tailwindPlugin from "prettier-plugin-tailwindcss";

import {
  classifyDifference,
  corpusFingerprint,
  extractAttributes,
  orderCandidates,
  scrambleAttributes,
  splitClassTokens,
} from "../lib.mjs";

test("extracts only static quoted class attributes", () => {
  const source = `
    <div class="flex p-4" className = 'grid gap-2'></div>
    <div className={styles.root} class:list={["p-4"]}></div>
    <div class="before:content-['{']"></div>
    <div data-class="ignored"></div>
  `;

  assert.deepEqual(extractAttributes(source, "astro"), [
    "flex p-4",
    "grid gap-2",
  ]);
});

test("ignores markup text in source comments and string literals", () => {
  const fixtures = [
    {
      kind: "tsx",
      source: `
        const sample = '<div class="fake string">';
        // <div className="fake comment" />
        export const view = <div className="real p-4" />;
      `,
    },
    {
      kind: "svelte",
      source: `
        <script>
          const sample = '<div class="fake string">';
          // <div class="fake comment">
        </script>
        <!-- <div class="fake markup-comment"> -->
        <div class="real p-4"></div>
      `,
    },
    {
      kind: "astro",
      source: `---
const sample = '<div class="fake string">';
// <div class="fake comment">
---
<!-- <div class="fake markup-comment"> -->
<div class="real p-4"></div>
`,
    },
  ];

  for (const { kind, source } of fixtures) {
    assert.deepEqual(extractAttributes(source, kind), ["real p-4"]);
    const scrambled = scrambleAttributes(source, kind);
    assert.match(scrambled, /class(?:Name)?="p-4 real"/u);
    assert.match(scrambled, /class="fake string"/u);
    assert.match(scrambled, /class(?:Name)?="fake comment"/u);
  }
});

test("splits whitespace outside arbitrary-value brackets", () => {
  const value = String.raw`sm:hover:flex [grid-template-columns:1fr_2fr] content-['hello world'] [--value:'a \' b'] p-4`;

  assert.deepEqual(splitClassTokens(value), [
    "sm:hover:flex",
    "[grid-template-columns:1fr_2fr]",
    "content-['hello world']",
    String.raw`[--value:'a \' b']`,
    "p-4",
  ]);
});

test("tracks nested brackets and quoted closing brackets", () => {
  assert.deepEqual(
    splitClassTokens("[&:has([data-x='[ ]'])]:block  mx-2\tflex"),
    ["[&:has([data-x='[ ]'])]:block", "mx-2", "flex"],
  );
});

test("scrambles eligible attributes without touching dynamic attributes", () => {
  const source = `<div class="flex  p-4\tmt-2" className={value}><span class='block'></span></div>`;

  assert.equal(
    scrambleAttributes(source, "tsx"),
    `<div class="mt-2 p-4 flex" className={value}><span class='block'></span></div>`,
  );
});

test("orders candidates by attribute count then UTF-8 path bytes", () => {
  const candidates = [
    { path: "z.tsx", attributes: 3 },
    { path: "é.tsx", attributes: 3 },
    { path: "a.tsx", attributes: 3 },
    { path: "many.tsx", attributes: 4 },
    { path: "empty.tsx", attributes: 0 },
  ];

  assert.deepEqual(
    orderCandidates(candidates, 3).map(({ path }) => path),
    ["many.tsx", "a.tsx", "z.tsx"],
  );
});

test("classifies convergent exact, custom-only, and known-order differences", () => {
  const known = new Map([
    ["flex", true],
    ["p-4", true],
    ["brand-card", false],
  ]);

  assert.equal(
    classifyDifference(
      comparison({
        original: ["flex", "p-4"],
        scrambled: ["p-4", "flex"],
        prettier: ["flex", "p-4"],
        rustywind: ["flex", "p-4"],
      }),
      known,
    ),
    "exact",
  );
  assert.equal(
    classifyDifference(
      comparison({
        original: ["brand-card", "flex", "p-4"],
        scrambled: ["p-4", "flex", "brand-card"],
        prettier: ["brand-card", "flex", "p-4"],
        rustywind: ["flex", "p-4", "brand-card"],
      }),
      known,
    ),
    "custom-only",
  );
  assert.equal(
    classifyDifference(
      comparison({
        original: ["p-4", "flex"],
        scrambled: ["flex", "p-4"],
        prettier: ["p-4", "flex"],
        rustywind: ["flex", "p-4"],
      }),
      known,
    ),
    "known-order",
  );
});

test("classifies token extraction before Prettier convergence", () => {
  assert.equal(
    classifyDifference({
      original: ["flex", "p-4"],
      scrambled: ["p-4", "flex"],
      prettierOriginal: ["flex", "p-4"],
      prettierScrambled: ["p-4", "flex"],
      rustywind: ["flex", "flex"],
    }),
    "token-multiset",
  );
});

test("rejects order classification without a result for every token", () => {
  assert.throws(
    () =>
      classifyDifference(
        comparison({
          original: ["unclassified", "flex"],
          scrambled: ["flex", "unclassified"],
          prettier: ["unclassified", "flex"],
          rustywind: ["flex", "unclassified"],
        }),
        new Map([["flex", true]]),
      ),
    /missing Tailwind classification for token: "unclassified"/u,
  );
});

test("allows convergent duplicate removal by both formatters", () => {
  assert.equal(
    classifyDifference({
      original: ["rtl:mr-0", "flex", "rtl:mr-0"],
      scrambled: ["rtl:mr-0", "flex", "rtl:mr-0"],
      prettierOriginal: ["flex", "rtl:mr-0"],
      prettierScrambled: ["flex", "rtl:mr-0"],
      rustywind: ["flex", "rtl:mr-0"],
    }),
    "exact",
  );
});

test("classifies nonconvergent Prettier output before RustyWind ordering", () => {
  assert.equal(
    classifyDifference({
      original: ["grid", "m-2"],
      scrambled: ["m-2", "grid"],
      prettierOriginal: ["grid", "m-2"],
      prettierScrambled: ["m-2", "grid"],
      rustywind: ["grid", "m-2"],
    }),
    "prettier-nonconvergent",
  );
});

test("pins per-attribute Svelte each-else Prettier convergence", async () => {
  const source =
    '{#each items as item}<div class="flex p-4">{item}</div>{:else}<div class="grid m-2">empty</div>{/each}';
  const scrambledSource = scrambleAttributes(source, "svelte");
  const options = {
    filepath: "regression.svelte",
    parser: "svelte",
    plugins: [sveltePlugin, tailwindPlugin],
    tailwindStylesheet: fileURLToPath(
      new URL("../tailwind.css", import.meta.url),
    ),
  };

  const prettierOriginal = extractAttributes(
    await prettier.format(source, options),
    "svelte",
  ).map(splitClassTokens);
  const prettierScrambled = extractAttributes(
    await prettier.format(scrambledSource, options),
    "svelte",
  ).map(splitClassTokens);

  assert.deepEqual(prettierOriginal, [
    ["flex", "p-4"],
    ["grid", "m-2"],
  ]);
  assert.deepEqual(prettierScrambled, [
    ["flex", "p-4"],
    ["m-2", "grid"],
  ]);
  assert.equal(
    classifyDifference({
      original: ["grid", "m-2"],
      scrambled: ["m-2", "grid"],
      prettierOriginal: prettierOriginal[1],
      prettierScrambled: prettierScrambled[1],
      rustywind: ["grid", "m-2"],
    }),
    "prettier-nonconvergent",
  );
});

test("fingerprints ordered path and scrambled source records with NUL separators", () => {
  const records = [
    { path: "a.tsx", scrambledSource: '<div class="p-4 flex" />' },
    { path: "nested/b.tsx", scrambledSource: '<p class="mt-2 block">x</p>\n' },
  ];
  const expected = createHash("sha256")
    .update('a.tsx\0<div class="p-4 flex" />\0', "utf8")
    .update('nested/b.tsx\0<p class="mt-2 block">x</p>\n\0', "utf8")
    .digest("hex");

  assert.equal(corpusFingerprint(records), expected);
  assert.notEqual(corpusFingerprint([...records].reverse()), expected);
});

function comparison({ original, scrambled, prettier, rustywind }) {
  return {
    original,
    scrambled,
    prettierOriginal: prettier,
    prettierScrambled: prettier,
    rustywind,
  };
}

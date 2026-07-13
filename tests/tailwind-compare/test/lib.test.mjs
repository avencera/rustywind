import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import test from "node:test";

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

  assert.deepEqual(extractAttributes(source), ["flex p-4", "grid gap-2"]);
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
    scrambleAttributes(source),
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

test("classifies exact, custom-only, known-order, and multiset differences", () => {
  const known = new Map([
    ["flex", true],
    ["p-4", true],
    ["brand-card", false],
  ]);

  assert.equal(
    classifyDifference(["flex", "p-4"], ["flex", "p-4"], known),
    "exact",
  );
  assert.equal(
    classifyDifference(
      ["brand-card", "flex", "p-4"],
      ["flex", "p-4", "brand-card"],
      known,
    ),
    "custom-only",
  );
  assert.equal(
    classifyDifference(["p-4", "flex"], ["flex", "p-4"], known),
    "known-order",
  );
  assert.equal(
    classifyDifference(["flex", "p-4"], ["flex", "flex"], known),
    "token-multiset",
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

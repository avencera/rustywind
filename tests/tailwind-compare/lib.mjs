import { createHash } from "node:crypto";

import { parse as parseAstro } from "@astrojs/compiler/sync";
import { parsers as typescriptParsers } from "prettier/plugins/typescript";
import { parse as parseSvelte } from "svelte/compiler";

export const kinds = Object.freeze({
  astro: { extension: ".astro", parser: "astro" },
  svelte: { extension: ".svelte", parser: "svelte" },
  tsx: { extension: ".tsx", parser: "typescript" },
});

export function isStaticCandidate(value) {
  return !/[{}]/u.test(value) && !value.includes("<%") && !value.includes("<?");
}

function walk(root, visit) {
  const pending = [root];
  const seen = new WeakSet();
  while (pending.length > 0) {
    const value = pending.pop();
    if (value === null || typeof value !== "object" || seen.has(value)) {
      continue;
    }
    seen.add(value);
    visit(value);
    for (const child of Object.values(value)) {
      if (child !== null && typeof child === "object") pending.push(child);
    }
  }
}

function isClassAttributeName(name) {
  return name === "class" || name === "className";
}

function findTsxAttributes(source) {
  const ast = typescriptParsers.typescript.parse(source, {
    filepath: "source.tsx",
  });
  const attributes = [];
  walk(ast, (node) => {
    if (
      node.type !== "JSXAttribute" ||
      !isClassAttributeName(node.name?.name) ||
      node.value?.type !== "Literal" ||
      typeof node.value.value !== "string"
    ) {
      return;
    }
    const [start, end] = node.value.range;
    const quote = source[start];
    if ((quote !== '"' && quote !== "'") || source[end - 1] !== quote) return;
    attributes.push({ end: end - 1, start: start + 1 });
  });
  return attributes;
}

function findSvelteAttributes(source) {
  const ast = parseSvelte(source, { modern: true });
  const attributes = [];
  walk(ast.fragment, (node) => {
    if (
      node.type !== "Attribute" ||
      !isClassAttributeName(node.name) ||
      node.value?.length !== 1 ||
      node.value[0].type !== "Text"
    ) {
      return;
    }
    const { start, end } = node.value[0];
    const quote = source[start - 1];
    if ((quote !== '"' && quote !== "'") || source[end] !== quote) return;
    attributes.push({ end, start });
  });
  return attributes;
}

function findAstroAttributes(source) {
  const { ast } = parseAstro(source, { position: true });
  const attributes = [];
  walk(ast, (node) => {
    if (
      node.type !== "attribute" ||
      node.kind !== "quoted" ||
      !isClassAttributeName(node.name)
    ) {
      return;
    }
    const quote = node.raw[0];
    if ((quote !== '"' && quote !== "'") || node.raw.at(-1) !== quote) {
      return;
    }
    const rawStart = source.indexOf(
      node.raw,
      node.position.start.offset + node.name.length,
    );
    if (rawStart === -1) return;
    attributes.push({
      end: rawStart + node.raw.length - 1,
      start: rawStart + 1,
    });
  });
  return attributes;
}

function findStaticAttributes(source, kind) {
  let attributes;
  if (kind === "tsx") {
    attributes = findTsxAttributes(source);
  } else if (kind === "svelte") {
    attributes = findSvelteAttributes(source);
  } else if (kind === "astro") {
    attributes = findAstroAttributes(source);
  } else {
    throw new Error(`Unsupported source kind: ${kind}`);
  }
  return attributes
    .filter(({ start, end }) => isStaticCandidate(source.slice(start, end)))
    .sort((left, right) => left.start - right.start);
}

export function extractAttributes(source, kind) {
  return findStaticAttributes(source, kind).map(({ start, end }) =>
    source.slice(start, end),
  );
}

export function splitClassTokens(value) {
  const tokens = [];
  let start;
  let bracketDepth = 0;
  let bracketQuote;
  let escaped = false;

  for (let index = 0; index < value.length; index += 1) {
    const character = value[index];
    if (bracketDepth > 0) {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (bracketQuote) {
        if (character === bracketQuote) bracketQuote = undefined;
      } else if (["'", '"', "`"].includes(character)) {
        bracketQuote = character;
      } else if (character === "[") {
        bracketDepth += 1;
      } else if (character === "]") {
        bracketDepth -= 1;
      }
      if (start === undefined) start = index;
      continue;
    }

    if (character === "[") {
      bracketDepth = 1;
      if (start === undefined) start = index;
    } else if (/\s/u.test(character)) {
      if (start !== undefined) {
        tokens.push(value.slice(start, index));
        start = undefined;
      }
    } else if (start === undefined) {
      start = index;
    }
  }

  if (start !== undefined) tokens.push(value.slice(start));
  return tokens;
}

export function scrambleAttributes(source, kind) {
  let scrambled = source;
  const attributes = findStaticAttributes(source, kind);
  for (const { start, end } of attributes.reverse()) {
    const tokens = splitClassTokens(source.slice(start, end));
    if (tokens.length < 2) continue;
    scrambled = `${scrambled.slice(0, start)}${tokens.reverse().join(" ")}${scrambled.slice(end)}`;
  }
  return scrambled;
}

export function same(left, right) {
  return (
    left.length === right.length &&
    left.every((value, index) => value === right[index])
  );
}

export function sameMultiset(left, right) {
  return same([...left].sort(compareUtf8), [...right].sort(compareUtf8));
}

export function compareUtf8(left, right) {
  return Buffer.compare(Buffer.from(left, "utf8"), Buffer.from(right, "utf8"));
}

export function orderCandidates(candidates, limit) {
  return [...candidates]
    .filter(({ attributes }) => attributes > 0)
    .sort(
      (left, right) =>
        right.attributes - left.attributes ||
        compareUtf8(left.path, right.path),
    )
    .slice(0, limit);
}

export function classifyDifference(comparison, known) {
  const {
    original,
    scrambled,
    prettierOriginal,
    prettierScrambled,
    rustywind,
  } = comparison;
  if (
    !sameMultiset(original, scrambled) ||
    ![prettierScrambled, rustywind].every((tokens) =>
      sameMultiset(prettierOriginal, tokens),
    )
  ) {
    return "token-multiset";
  }
  if (!same(prettierOriginal, prettierScrambled)) {
    return "prettier-nonconvergent";
  }
  if (same(prettierScrambled, rustywind)) return "exact";
  if (known === undefined) return "order-difference";

  const isKnown = (token) => {
    const value = known.get(token);
    if (typeof value !== "boolean") {
      throw new Error(
        `missing Tailwind classification for token: ${JSON.stringify(token)}`,
      );
    }
    return value;
  };
  const prettierKnown = prettierScrambled.filter(isKnown);
  const rustywindKnown = rustywind.filter(isKnown);
  return same(prettierKnown, rustywindKnown) ? "custom-only" : "known-order";
}

export function corpusFingerprint(candidates) {
  const hash = createHash("sha256");
  for (const { path, scrambledSource } of candidates) {
    hash.update(path, "utf8");
    hash.update("\0", "utf8");
    hash.update(scrambledSource, "utf8");
    hash.update("\0", "utf8");
  }
  return hash.digest("hex");
}

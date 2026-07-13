import { createHash } from "node:crypto";

const attributePattern = /(^|[\s<])(class(?:Name)?\s*=\s*)(["'])([\s\S]*?)\3/gm;

export const kinds = Object.freeze({
  astro: { extension: ".astro", parser: "astro" },
  svelte: { extension: ".svelte", parser: "svelte" },
  tsx: { extension: ".tsx", parser: "typescript" },
});

export function isStaticCandidate(value) {
  return !/[{}]/u.test(value) && !value.includes("<%") && !value.includes("<?");
}

export function extractAttributes(source) {
  return [...source.matchAll(attributePattern)]
    .map((match) => match[4])
    .filter(isStaticCandidate);
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

export function scrambleAttributes(source) {
  return source.replace(
    attributePattern,
    (match, prefix, assignment, quote, value) => {
      if (!isStaticCandidate(value)) return match;
      const tokens = splitClassTokens(value);
      if (tokens.length < 2) return match;
      return `${prefix}${assignment}${quote}${tokens.reverse().join(" ")}${quote}`;
    },
  );
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

export function classifyDifference(prettierTokens, rustywindTokens, known) {
  if (!sameMultiset(prettierTokens, rustywindTokens)) return "token-multiset";
  if (same(prettierTokens, rustywindTokens)) return "exact";

  const prettierKnown = prettierTokens.filter(
    (token) => known.get(token) === true,
  );
  const rustywindKnown = rustywindTokens.filter(
    (token) => known.get(token) === true,
  );
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

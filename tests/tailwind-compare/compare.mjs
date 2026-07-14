#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import * as prettier from "prettier";
import * as astroPlugin from "prettier-plugin-astro";
import * as sveltePlugin from "prettier-plugin-svelte";
import * as tailwindPlugin from "prettier-plugin-tailwindcss";
import { createSorter } from "prettier-plugin-tailwindcss/sorter";

import {
  classifyDifference,
  compareUtf8,
  corpusFingerprint,
  extractAttributes,
  kinds,
  orderCandidates,
  same,
  scrambleAttributes,
  splitClassTokens,
} from "./lib.mjs";

const engineDirectory = dirname(fileURLToPath(import.meta.url));
const rustywindTimeout = 30_000;
const ignoredDirectories = new Set([
  ".git",
  ".svelte-kit",
  "build",
  "dist",
  "node_modules",
]);
const syntaxPlugins = {
  astro: [astroPlugin],
  svelte: [sveltePlugin],
  tsx: [],
};

function usage() {
  return [
    "usage: node compare.mjs",
    "  --corpus NAME --repo PATH --revision REVISION --scope PATH",
    "  --kind tsx|svelte|astro --limit COUNT --rustywind PATH",
    "  --stylesheet PATH --output PATH",
  ].join("\n");
}

function parseArguments(argv) {
  if (argv.includes("--help")) {
    process.stdout.write(`${usage()}\n`);
    process.exit(0);
  }

  const options = new Map();
  for (let index = 0; index < argv.length; index += 2) {
    const name = argv[index];
    const value = argv[index + 1];
    if (
      !name?.startsWith("--") ||
      value === undefined ||
      value.startsWith("--")
    ) {
      throw new Error(usage());
    }
    if (options.has(name)) throw new Error(`duplicate argument: ${name}`);
    options.set(name, value);
  }

  const knownArguments = new Set([
    "--corpus",
    "--kind",
    "--limit",
    "--output",
    "--repo",
    "--revision",
    "--rustywind",
    "--scope",
    "--stylesheet",
  ]);
  for (const name of options.keys()) {
    if (!knownArguments.has(name)) throw new Error(`unknown argument: ${name}`);
  }
  for (const name of knownArguments) {
    if (!options.has(name))
      throw new Error(`missing argument: ${name}\n${usage()}`);
  }

  const limitText = options.get("--limit");
  if (!/^\d+$/u.test(limitText))
    throw new Error("--limit must be a positive integer");
  const limit = Number.parseInt(limitText, 10);
  if (!Number.isSafeInteger(limit) || limit < 1) {
    throw new Error("--limit must be a positive integer");
  }

  const kind = options.get("--kind");
  if (!Object.hasOwn(kinds, kind)) {
    throw new Error("--kind must be one of: tsx, svelte, astro");
  }

  const corpus = options.get("--corpus");
  const revision = options.get("--revision");
  if (corpus.length === 0) throw new Error("--corpus must not be empty");
  if (revision.length === 0) throw new Error("--revision must not be empty");

  return {
    corpus,
    kind,
    limit,
    output: resolve(options.get("--output")),
    repo: resolve(options.get("--repo")),
    revision,
    rustywind: resolve(options.get("--rustywind")),
    scopeArgument: options.get("--scope"),
    stylesheet: resolve(options.get("--stylesheet")),
  };
}

function ensureFile(path, argument) {
  if (!existsSync(path) || !statSync(path).isFile()) {
    throw new Error(`${argument} must name an existing file`);
  }
}

function ensureDirectory(path, argument) {
  if (!existsSync(path) || !statSync(path).isDirectory()) {
    throw new Error(`${argument} must name an existing directory`);
  }
}

function within(parent, child) {
  const path = relative(parent, child);
  return (
    path === "" ||
    (!path.startsWith(`..${sep}`) && path !== ".." && !isAbsolute(path))
  );
}

function normalizeRelativePath(path) {
  return path.split(sep).join("/");
}

function resolveConfiguration(arguments_) {
  ensureDirectory(arguments_.repo, "--repo");
  ensureFile(arguments_.rustywind, "--rustywind");
  ensureFile(arguments_.stylesheet, "--stylesheet");

  const scope = resolve(arguments_.repo, arguments_.scopeArgument);
  if (!within(arguments_.repo, scope))
    throw new Error("--scope must be inside --repo");
  ensureDirectory(scope, "--scope");

  return {
    ...arguments_,
    scope,
    logicalScope:
      normalizeRelativePath(relative(arguments_.repo, scope)) || ".",
  };
}

function validateRustywind(configuration) {
  const result = spawnSync(configuration.rustywind, ["--version"], {
    encoding: "utf8",
    killSignal: "SIGKILL",
    timeout: rustywindTimeout,
  });
  if (result.error) {
    throw new Error(
      `could not invoke --rustywind: ${scrubError(result.error, configuration)}`,
    );
  }
  if (result.status !== 0) {
    throw new Error(
      `--rustywind --version exited with status ${result.status}: ${scrubError(
        result.stderr,
        configuration,
      )}`,
    );
  }
}

function walk(directory, extension) {
  const files = [];
  const entries = readdirSync(directory, { withFileTypes: true }).sort(
    (left, right) => compareUtf8(left.name, right.name),
  );
  for (const entry of entries) {
    if (entry.isDirectory() && ignoredDirectories.has(entry.name)) continue;
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...walk(path, extension));
    } else if (entry.isFile() && path.endsWith(extension)) {
      files.push(path);
    }
  }
  return files;
}

function selectCandidates(configuration) {
  const candidates = walk(
    configuration.scope,
    kinds[configuration.kind].extension,
  ).map((file) => {
    const source = readFileSync(file, "utf8");
    const path = normalizeRelativePath(relative(configuration.repo, file));
    return {
      attributes: extractAttributes(source, configuration.kind).length,
      file,
      path,
      source,
      scrambledSource: scrambleAttributes(source, configuration.kind),
    };
  });
  return orderCandidates(candidates, configuration.limit);
}

function prettierOptions(configuration, file) {
  return {
    filepath: file,
    parser: kinds[configuration.kind].parser,
    plugins: [...syntaxPlugins[configuration.kind], tailwindPlugin],
    tailwindStylesheet: configuration.stylesheet,
  };
}

function scrubError(error, configuration) {
  let message = String(error?.stderr || error?.message || error).trim();
  const paths = [
    [configuration.repo, "<repo>"],
    [configuration.rustywind, "<rustywind>"],
    [configuration.stylesheet, "<stylesheet>"],
    [engineDirectory, "<engine>"],
  ].sort((left, right) => right[0].length - left[0].length);
  for (const [path, replacement] of paths) {
    message = message.split(path).join(replacement);
  }
  return message || "unknown error";
}

function runRustywind(configuration, candidate) {
  const result = spawnSync(
    configuration.rustywind,
    ["--stdin", "--stdin-filename", candidate.file, "--quiet"],
    {
      encoding: "utf8",
      input: candidate.scrambledSource,
      killSignal: "SIGKILL",
      maxBuffer: 20 * 1024 * 1024,
      timeout: rustywindTimeout,
    },
  );
  if (result.error) {
    result.error.invocation = true;
    throw result.error;
  }
  if (result.status !== 0) {
    const error = new Error(`RustyWind exited with status ${result.status}`);
    error.stderr = result.stderr;
    throw error;
  }
  return result.stdout;
}

function unique(tokens) {
  return [...new Set(tokens)];
}

export async function probeKnownClasses(tokens, stylesheet) {
  const probes = unique(tokens).map((token, index) => {
    let suffix = 0;
    let left;
    let right;
    do {
      // NUL makes markers invalid Tailwind candidates under any stylesheet
      left = `\0__rustywind_unknown_${index}_${suffix}_a`;
      right = `\0__rustywind_unknown_${index}_${suffix}_b`;
      suffix += 1;
    } while (token === left || token === right);
    return { left, right, token };
  });
  if (probes.length === 0) return new Map();

  const sorter = await createSorter({
    preserveDuplicates: true,
    stylesheetPath: stylesheet,
  });
  const results = sorter.sortClassLists(
    probes.map(({ left, right, token }) => [left, token, right]),
  );

  return new Map(
    probes.map(({ left, right, token }, index) => {
      const result = results[index] ?? [];
      if (same(result, [left, right, token])) return [token, true];
      if (same(result, [left, token, right])) return [token, false];
      throw new Error(
        `Tailwind sorter returned an unexpected probe result for ${JSON.stringify(token)}`,
      );
    }),
  );
}

async function validatePrettier(configuration) {
  const fixtures = {
    astro: '---\n---\n<div class="p-4 flex"></div>\n',
    svelte: '<div class="p-4 flex"></div>\n',
    tsx: '<div className="p-4 flex" />;\n',
  };
  try {
    await prettier.format(
      fixtures[configuration.kind],
      prettierOptions(
        configuration,
        join(configuration.scope, `__rustywind_probe.${configuration.kind}`),
      ),
    );
    const known = await probeKnownClasses(["flex"], configuration.stylesheet);
    if (known.get("flex") !== true) {
      throw new Error("Tailwind did not recognize the built-in flex utility");
    }
  } catch (error) {
    throw new Error(
      `Prettier configuration probe failed: ${scrubError(error, configuration)}`,
    );
  }
}

function packageVersion(name) {
  return JSON.parse(
    readFileSync(join(engineDirectory, "node_modules", name, "package.json")),
  ).version;
}

function compareAttribute(
  summary,
  pending,
  candidate,
  attribute,
  comparison,
) {
  summary.prettierChanged += Number(
    !same(comparison.scrambled, comparison.prettierScrambled),
  );
  summary.rustywindChanged += Number(
    !same(comparison.scrambled, comparison.rustywind),
  );

  const kind = classifyDifference(comparison);
  if (kind === "token-multiset") {
    summary.extractionMismatch += 1;
  } else if (kind === "prettier-nonconvergent") {
    summary.prettierNonconvergent += 1;
  } else if (kind === "exact") {
    summary.exact += 1;
    return;
  } else if (kind !== "order-difference") {
    throw new Error(`Unexpected comparison kind: ${kind}`);
  }

  pending.push({
    attribute,
    ...comparison,
    kind,
    path: candidate.path,
  });
}

async function compareCandidates(configuration, candidates) {
  const summary = {
    prettierChanged: 0,
    rustywindChanged: 0,
    exact: 0,
    customOnly: 0,
    prettierNonconvergent: 0,
    knownOrderMismatch: 0,
    extractionMismatch: 0,
    failures: 0,
  };
  const pending = [];
  const failures = [];

  for (const candidate of candidates) {
    let prettierOriginalOutput;
    let prettierScrambledOutput;
    let rustywindOutput;
    try {
      prettierOriginalOutput = await prettier.format(
        candidate.source,
        prettierOptions(configuration, candidate.file),
      );
    } catch (error) {
      failures.push({
        path: candidate.path,
        stage: "prettier-original",
        error: scrubError(error, configuration),
      });
      continue;
    }
    try {
      prettierScrambledOutput = await prettier.format(
        candidate.scrambledSource,
        prettierOptions(configuration, candidate.file),
      );
    } catch (error) {
      failures.push({
        path: candidate.path,
        stage: "prettier-scrambled",
        error: scrubError(error, configuration),
      });
      continue;
    }
    try {
      rustywindOutput = runRustywind(configuration, candidate);
    } catch (error) {
      if (error.invocation) throw error;
      failures.push({
        path: candidate.path,
        stage: "rustywind",
        error: scrubError(error, configuration),
      });
      continue;
    }

    const originalAttributes = extractAttributes(
      candidate.source,
      configuration.kind,
    );
    const scrambledAttributes = extractAttributes(
      candidate.scrambledSource,
      configuration.kind,
    );
    const prettierOriginalAttributes = extractAttributes(
      prettierOriginalOutput,
      configuration.kind,
    );
    const prettierScrambledAttributes = extractAttributes(
      prettierScrambledOutput,
      configuration.kind,
    );
    const rustywindAttributes = extractAttributes(
      rustywindOutput,
      configuration.kind,
    );
    if (
      originalAttributes.length !== candidate.attributes ||
      scrambledAttributes.length !== candidate.attributes ||
      prettierOriginalAttributes.length !== candidate.attributes ||
      prettierScrambledAttributes.length !== candidate.attributes ||
      rustywindAttributes.length !== candidate.attributes
    ) {
      summary.extractionMismatch += candidate.attributes;
      pending.push({
        kind: "attribute-count",
        expected: candidate.attributes,
        original: originalAttributes.length,
        scrambled: scrambledAttributes.length,
        path: candidate.path,
        prettierOriginal: prettierOriginalAttributes.length,
        prettierScrambled: prettierScrambledAttributes.length,
        rustywind: rustywindAttributes.length,
      });
      continue;
    }

    for (let index = 0; index < candidate.attributes; index += 1) {
      compareAttribute(
        summary,
        pending,
        candidate,
        index,
        {
          original: splitClassTokens(originalAttributes[index]),
          scrambled: splitClassTokens(scrambledAttributes[index]),
          prettierOriginal: splitClassTokens(
            prettierOriginalAttributes[index],
          ),
          prettierScrambled: splitClassTokens(
            prettierScrambledAttributes[index],
          ),
          rustywind: splitClassTokens(rustywindAttributes[index]),
        },
      );
    }
  }

  const orderDifferences = pending.filter(
    (detail) => detail.kind === "order-difference",
  );
  let known;
  try {
    known = await probeKnownClasses(
      orderDifferences.flatMap(
        ({ prettierScrambled, rustywind }) => [
          ...prettierScrambled,
          ...rustywind,
        ],
      ),
      configuration.stylesheet,
    );
  } catch (error) {
    throw new Error(
      `Tailwind classification probe failed: ${scrubError(error, configuration)}`,
    );
  }

  for (const detail of orderDifferences) {
    detail.kind = classifyDifference(detail, known);
    detail.prettierUnknown = unique(
      detail.rustywind.filter((token) => known.get(token) !== true),
    );
    if (detail.kind === "custom-only") summary.customOnly += 1;
    if (detail.kind === "known-order") summary.knownOrderMismatch += 1;
  }

  summary.failures = failures.length;
  return { details: pending, failures, summary };
}

async function runComparison(configuration) {
  validateRustywind(configuration);
  await validatePrettier(configuration);
  const candidates = selectCandidates(configuration);
  const comparison = await compareCandidates(configuration, candidates);
  const attributes = candidates.reduce(
    (total, candidate) => total + candidate.attributes,
    0,
  );
  const result = {
    schemaVersion: 2,
    corpus: {
      name: configuration.corpus,
      revision: configuration.revision,
      scope: configuration.logicalScope,
      kind: configuration.kind,
      limit: configuration.limit,
      files: candidates.length,
      attributes,
      fingerprint: corpusFingerprint(candidates),
    },
    versions: {
      prettier: prettier.version,
      prettierPluginTailwindcss: packageVersion("prettier-plugin-tailwindcss"),
      tailwindcss: packageVersion("tailwindcss"),
      prettierPluginAstro: packageVersion("prettier-plugin-astro"),
      prettierPluginSvelte: packageVersion("prettier-plugin-svelte"),
    },
    summary: comparison.summary,
    candidates: candidates.map(({ path, attributes: count }) => ({
      path,
      attributes: count,
    })),
    details: comparison.details,
    failures: comparison.failures,
  };

  mkdirSync(dirname(configuration.output), { recursive: true });
  writeFileSync(configuration.output, `${JSON.stringify(result, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(result.summary)}\n`);
}

async function main() {
  const configuration = resolveConfiguration(
    parseArguments(process.argv.slice(2)),
  );
  try {
    await runComparison(configuration);
  } catch (error) {
    throw new Error(scrubError(error, configuration), { cause: error });
  }
}

if (
  process.argv[1] !== undefined &&
  resolve(process.argv[1]) === fileURLToPath(import.meta.url)
) {
  main().catch((error) => {
    process.stderr.write(`${error.message || error}\n`);
    process.exitCode = 1;
  });
}

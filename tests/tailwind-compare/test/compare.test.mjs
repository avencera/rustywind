import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { probeKnownClasses } from "../compare.mjs";
import { classifyDifference } from "../lib.mjs";

const engineDirectory = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const comparePath = resolve(engineDirectory, "compare.mjs");

test("classifies known-order differences for utilities containing both quotes", async () => {
  const quotedUtility = `before:content-['hello_"world"']`;
  const known = await probeKnownClasses(
    [quotedUtility, "flex"],
    resolve(engineDirectory, "tailwind.css"),
  );

  assert.equal(known.get(quotedUtility), true);
  assert.equal(
    classifyDifference(
      {
        original: [quotedUtility, "flex"],
        scrambled: ["flex", quotedUtility],
        prettierOriginal: [quotedUtility, "flex"],
        prettierScrambled: [quotedUtility, "flex"],
        rustywind: ["flex", quotedUtility],
      },
      known,
    ),
    "known-order",
  );
});

test("scrubs configured paths from invocation errors", (context) => {
  const directory = mkdtempSync(join(tmpdir(), "rustywind-compare-"));
  context.after(() => rmSync(directory, { force: true, recursive: true }));
  const repo = join(directory, "repo");
  const rustywind = join(directory, "rustywind");
  const output = join(directory, "result.json");
  mkdirSync(repo);
  writeFileSync(join(repo, "input.tsx"), '<div className="p-4 flex" />;\n');
  writeFileSync(
    rustywind,
    `#!/bin/sh
if [ "$1" = "--version" ]; then
  rm -- "$0"
  echo "rustywind test"
fi
`,
  );
  chmodSync(rustywind, 0o755);

  const result = spawnSync(
    process.execPath,
    [
      comparePath,
      "--corpus",
      "test",
      "--repo",
      repo,
      "--revision",
      "test",
      "--scope",
      ".",
      "--kind",
      "tsx",
      "--limit",
      "1",
      "--rustywind",
      rustywind,
      "--stylesheet",
      resolve(engineDirectory, "tailwind.css"),
      "--output",
      output,
    ],
    { encoding: "utf8" },
  );

  assert.equal(result.status, 1);
  assert.match(result.stderr, /spawnSync <rustywind> ENOENT/u);
  assert.equal(result.stderr.includes(directory), false);
});

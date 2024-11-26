// @ts-check
"use strict";

// Imports
const os = require("os");
const fs = require("fs");
const path = require("path");
const util = require("util");

const download = require("./download");

const fsExists = util.promisify(fs.exists);
const mkdir = util.promisify(fs.mkdir);

const forceInstall = process.argv.includes("--force");
if (forceInstall) {
  console.log("--force, ignoring caches");
}

const { VERSION } = require("./constants");
const BIN_PATH = path.join(__dirname, "../bin");


////////////////////////////////////////////////////////////////////////////////
const APP_NAME = "rustywind";
const REPO = "avencera/rustywind";
const GITHUB_REPO = `https://github.com/${REPO}`;
////////////////////////////////////////////////////////////////////////////////

process.on("unhandledRejection", (reason, promise) => {
  console.log("Unhandled rejection: ", promise, "reason:", reason);
});

function getTarget() {
  const arch = os.arch();
  const platform = os.platform();

  console.log(`Downloading: ${APP_NAME}`);
  console.log(` from: ${GITHUB_REPO}`);
  console.log(` for platform: ${arch}-${platform}\n`);

  switch (platform) {
    case "darwin":
      return arch == "x64" ? "x86_64-apple-darwin" : "aarch64-apple-darwin";
    case "win32":
      return arch === "x64" ? "x86_64-pc-windows-msvc" : "i686-pc-windows-msvc";
    case "linux":
      return arch === "x64"
        ? "x86_64-unknown-linux-musl"
        : arch === "arm"
          ? "arm-unknown-linux-gnueabihf"
          : arch === "arm64"
            ? "aarch64-unknown-linux-gnu"
            : arch === "ppc64"
              ? "powerpc64le-unknown-linux-gnu"
              : "i686-unknown-linux-musl";
    default:
      throw new Error("Unknown platform: " + os.platform());
  }
}

async function main() {
  const binExists = await fsExists(BIN_PATH);
  if (!binExists) {
    await mkdir(BIN_PATH);
  }

  const opts = {
    version: VERSION,
    token: process.env["GITHUB_TOKEN"],
    target: getTarget(),
    destDir: BIN_PATH,
    force: forceInstall,
  };

  try {
    await download(opts);
  } catch (err) {
    console.error(`Downloading rustywind failed: ${err.stack}`);
    process.exit(1);
  }
}

main();

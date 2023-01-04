// @ts-check
"use strict";

const fs = require("fs");

const { VERSION } = require("./constants");
const downloadRelease = require("./download-release");

process.on("unhandledRejection", (reason, promise) => {
  console.log("Unhandled rejection: ", promise, "reason:", reason);
});

async function main() {
  const opts = {
    version: VERSION,
    token: process.env["GITHUB_TOKEN"],
  };
  try {
    const release = await downloadRelease(opts);
    release.assets = release.assets.map((asset) => ({
      ...asset,
      download_count: undefined,
    }));
    fs.writeFileSync("release.json", JSON.stringify(release, undefined, 2), {
      encoding: "utf8",
    });
  } catch (err) {
    console.error(`Downloading rustywind metadata failed: ${err.stack}`);
    process.exit(1);
  }
}

main();

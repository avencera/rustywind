// @ts-check
"use strict";

const { REPO } = require("./constants");
const get = require("./get");

/**
 * @param {string} repo
 * @param {string} tag
 */
function getApiUrl(repo, tag) {
  return `https://api.github.com/repos/${repo}/releases/tags/${tag}`;
}

/**
 * @param {{ token?: string; version: string; }} opts
 */
async function getReleaseFromGitHubApi(opts) {
  const downloadOpts = {
    headers: {
      "user-agent": "rustywind",
    },
  };

  if (opts.token) {
    downloadOpts.headers.authorization = `token ${opts.token}`;
  }

  console.log(`Finding rustywind ${opts.version} release`);
  const release = await get(getApiUrl(REPO, opts.version), downloadOpts);
  let jsonRelease;
  try {
    jsonRelease = JSON.parse(release);
  } catch (e) {
    throw new Error("Malformed API response: " + e.stack);
  }

  if (!jsonRelease.assets) {
    throw new Error("Bad API response: " + JSON.stringify(release));
  }

  return jsonRelease;
}

/**
 * @param {{ token?: string; version: string; }} opts
 */
module.exports = async (opts) => {
  if (!opts.version) {
    return Promise.reject(new Error("Missing version"));
  }

  return getReleaseFromGitHubApi(opts);
};

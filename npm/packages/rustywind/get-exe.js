// @ts-check
"use strict";

const path = require("path");
const fs = require("fs");

/**
 * Detect if running on musl libc (Alpine, etc.)
 * @returns {boolean}
 */
function isMusl() {
  // Use detect-libc if available (more reliable)
  try {
    const { MUSL, familySync } = require("detect-libc");
    return familySync() === MUSL;
  } catch {
    // Fallback: check ldd output
    try {
      const { execSync } = require("child_process");
      const output = execSync("ldd --version 2>&1 || true", { encoding: "utf8" });
      return output.toLowerCase().includes("musl");
    } catch {
      return false;
    }
  }
}

/**
 * Get the package name for the current platform
 * @returns {string}
 */
function getPackageName() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "darwin") {
    return arch === "arm64"
      ? "rustywind-darwin-arm64"
      : "rustywind-darwin-x64";
  }

  if (platform === "win32") {
    return arch === "x64"
      ? "rustywind-win32-x64-msvc"
      : "rustywind-win32-ia32-msvc";
  }

  if (platform === "linux") {
    if (arch === "x64") {
      return "rustywind-linux-x64-musl";
    }
    if (arch === "arm64") {
      return isMusl()
        ? "rustywind-linux-arm64-musl"
        : "rustywind-linux-arm64-gnu";
    }
    if (arch === "arm") {
      return "rustywind-linux-arm-gnueabihf";
    }
  }

  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

/**
 * Get the binary name for the current platform
 * @returns {string}
 */
function getBinaryName() {
  return process.platform === "win32" ? "rustywind.exe" : "rustywind";
}

/**
 * Get the path to the binary in the platform package
 * @returns {string}
 */
function getExePath() {
  const packageName = getPackageName();
  const binaryName = getBinaryName();

  try {
    const pkgPath = path.dirname(require.resolve(`${packageName}/package.json`));
    return path.join(pkgPath, binaryName);
  } catch {
    throw new Error(
      `Could not find package "${packageName}". ` +
      `Make sure optional dependencies are installed.`
    );
  }
}

module.exports = {
  getExePath,
  getPackageName,
  getBinaryName,
  isMusl,
};

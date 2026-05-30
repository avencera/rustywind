// @ts-check
"use strict";

const fs = require("fs");
const path = require("path");

const { getExePath, getBinaryName } = require("./get-exe");

const binDir = path.join(__dirname, "bin");
const binaryName = getBinaryName();
const destPath = path.join(binDir, binaryName);

// Ensure bin directory exists
if (!fs.existsSync(binDir)) {
  fs.mkdirSync(binDir, { recursive: true });
}

// Skip if binary already exists
if (fs.existsSync(destPath)) {
  process.exit(0);
}

let sourcePath;
try {
  sourcePath = getExePath();
} catch (err) {
  // Platform package not found - this is okay, the JS wrapper will handle it
  console.warn(`[rustywind] ${err.message}`);
  console.warn("[rustywind] Falling back to JS wrapper.");
  process.exit(0);
}

// Try to hard-link, fall back to copy
try {
  fs.linkSync(sourcePath, destPath);
} catch {
  try {
    fs.copyFileSync(sourcePath, destPath);
  } catch (err) {
    console.error("[rustywind] Failed to copy binary:", err.message);
    console.warn("[rustywind] Falling back to JS wrapper.");
    process.exit(0);
  }
}

// Set executable permissions on non-Windows
if (process.platform !== "win32") {
  try {
    fs.chmodSync(destPath, 0o755);
  } catch {
    // Ignore permission errors
  }
}

// Remove the placeholder wrapper if it exists and we have the real binary
const wrapperPath = path.join(binDir, "rustywind.js");
if (fs.existsSync(wrapperPath) && fs.existsSync(destPath)) {
  try {
    fs.unlinkSync(wrapperPath);
  } catch {
    // Ignore
  }
}

"use strict";

const path = require("path");

module.exports.rustyWindPath = path.join(
  __dirname,
  `../bin/rustywind${process.platform === "win32" ? ".exe" : ""}`
);

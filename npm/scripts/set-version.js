#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");

const v = process.argv[2];
if (!v || !/^\d+\.\d+\.\d+/.test(v)) {
  console.error("Usage: node set-version.js <semver>");
  process.exit(1);
}

const npmRoot = path.join(__dirname, "..");
const wrapperPath = path.join(npmRoot, "wrapper", "package.json");
const wrapper = JSON.parse(fs.readFileSync(wrapperPath, "utf8"));
wrapper.version = v;
for (const k of Object.keys(wrapper.optionalDependencies || {})) {
  wrapper.optionalDependencies[k] = v;
}
fs.writeFileSync(wrapperPath, JSON.stringify(wrapper, null, 2) + "\n");

const packagesDir = path.join(npmRoot, "packages");
for (const dir of fs.readdirSync(packagesDir)) {
  const pkgPath = path.join(packagesDir, dir, "package.json");
  if (!fs.existsSync(pkgPath)) continue;
  const pkg = JSON.parse(fs.readFileSync(pkgPath, "utf8"));
  pkg.version = v;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + "\n");
}

console.log(`Set npm package versions to ${v}`);

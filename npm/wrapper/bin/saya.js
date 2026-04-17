#!/usr/bin/env node
"use strict";

const { spawnSync } = require("child_process");
const fs = require("fs");
const path = require("path");

const key = `${process.platform}-${process.arch}`;
const map = {
  "darwin-arm64": "@sozlabs/saya-cli-darwin-arm64",
  "darwin-x64": "@sozlabs/saya-cli-darwin-x64",
  "linux-arm64": "@sozlabs/saya-cli-linux-arm64",
  "linux-x64": "@sozlabs/saya-cli-linux-x64",
  "win32-x64": "@sozlabs/saya-cli-win32-x64",
};

const pkg = map[key];
if (!pkg) {
  console.error(
    `saya: unsupported platform ${key}. Install Rust and build from source (see https://github.com/sozlabs/saya-cli#build).`
  );
  process.exit(1);
}

let pkgRoot;
try {
  pkgRoot = path.dirname(require.resolve(`${pkg}/package.json`));
} catch {
  console.error(
    `saya: native package missing (${pkg}). Re-run: npm install -g @sozlabs/saya-cli`
  );
  process.exit(1);
}

const exe = process.platform === "win32" ? "saya.exe" : "saya";
const binPath = path.join(pkgRoot, "bin", exe);
if (!fs.existsSync(binPath)) {
  console.error(`saya: binary not found at ${binPath}`);
  process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), { stdio: "inherit" });
process.exit(result.status === null ? 1 : result.status);

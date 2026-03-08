#!/usr/bin/env node
"use strict";

const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

const ext = process.platform === "win32" ? ".exe" : "";
const bin = path.join(__dirname, `gdcli${ext}`);

if (!fs.existsSync(bin)) {
  process.stderr.write(
    `gdcli binary not found at ${bin}\n` +
    `Try reinstalling: npm install -g gdcli-godot\n` +
    `Or install from source: cargo install gdcli\n`
  );
  process.exit(1);
}

const child = spawn(bin, process.argv.slice(2), { stdio: "inherit" });

child.on("error", (err) => {
  process.stderr.write(`Failed to run gdcli: ${err.message}\n`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
  } else {
    process.exit(code ?? 1);
  }
});

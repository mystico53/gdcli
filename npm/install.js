#!/usr/bin/env node
"use strict";

const fs = require("fs");
const path = require("path");
const https = require("https");
const { execSync } = require("child_process");

const REPO = "mystico53/gdcli";
const BIN_DIR = path.join(__dirname, "bin");

function getPlatformAsset() {
  const platform = process.platform;
  const arch = process.arch;

  if (platform === "win32" && (arch === "x64" || arch === "ia32")) {
    return { asset: "gdcli-windows-x86_64.zip", binary: "gdcli.exe" };
  }
  if (platform === "darwin") {
    return { asset: "gdcli-macos-universal.tar.gz", binary: "gdcli" };
  }
  if (platform === "linux" && arch === "x64") {
    return { asset: "gdcli-linux-x86_64.tar.gz", binary: "gdcli" };
  }

  console.error(`Unsupported platform: ${platform}-${arch}`);
  console.error("Please install from source: cargo install gdcli");
  process.exit(1);
}

function fetchRedirect(url) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { "User-Agent": "gdcli-npm" } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        resolve(res.headers.location);
      } else if (res.statusCode === 200) {
        resolve(url);
      } else {
        reject(new Error(`HTTP ${res.statusCode} for ${url}`));
      }
    }).on("error", reject);
  });
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https.get(url, { headers: { "User-Agent": "gdcli-npm" } }, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return download(res.headers.location, dest).then(resolve).catch(reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
      }
      const file = fs.createWriteStream(dest);
      res.pipe(file);
      file.on("finish", () => file.close(resolve));
      file.on("error", reject);
    }).on("error", reject);
  });
}

async function main() {
  const { asset, binary } = getPlatformAsset();
  const url = `https://github.com/${REPO}/releases/latest/download/${asset}`;

  console.log(`Downloading gdcli from ${url}...`);

  const tmpDir = path.join(__dirname, ".tmp");
  fs.mkdirSync(tmpDir, { recursive: true });
  fs.mkdirSync(BIN_DIR, { recursive: true });

  const archivePath = path.join(tmpDir, asset);
  await download(url, archivePath);

  // Extract
  if (asset.endsWith(".zip")) {
    // Windows: use PowerShell to unzip
    execSync(
      `powershell -Command "Expand-Archive -Force '${archivePath}' '${tmpDir}'"`,
      { stdio: "pipe" }
    );
  } else {
    execSync(`tar xzf "${archivePath}" -C "${tmpDir}"`, { stdio: "pipe" });
  }

  // Move binary to bin/
  const src = path.join(tmpDir, binary);
  const dest = path.join(BIN_DIR, binary);
  fs.copyFileSync(src, dest);

  if (process.platform !== "win32") {
    fs.chmodSync(dest, 0o755);
  }

  // Clean up
  fs.rmSync(tmpDir, { recursive: true, force: true });

  console.log(`gdcli installed to ${dest}`);
}

main().catch((err) => {
  console.error("Failed to install gdcli:", err.message);
  console.error("You can install manually: cargo install gdcli");
  process.exit(1);
});

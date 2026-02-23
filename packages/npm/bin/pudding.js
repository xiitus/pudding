#!/usr/bin/env node

const { spawn } = require("child_process");
const { existsSync } = require("fs");
const path = require("path");

const binaryName = process.platform === "win32" ? "pudding.exe" : "pudding";
const packageRoot = path.resolve(__dirname, "..");
const workspaceRoot = path.resolve(packageRoot, "..", "..");
const cargoTargetDir = process.env.CARGO_TARGET_DIR;
const puddingBinPath = process.env.PUDDING_BIN_PATH;

if (puddingBinPath && !path.isAbsolute(puddingBinPath)) {
  console.error(
    `PUDDING_BIN_PATH には絶対パスを指定してください: ${puddingBinPath}`,
  );
  process.exit(1);
}

const candidates = [
  puddingBinPath,
  cargoTargetDir ? path.join(cargoTargetDir, "release", binaryName) : null,
  path.join(workspaceRoot, "target", "release", binaryName),
  path.join(workspaceRoot, "crates", "pudding", "target", "release", binaryName),
].filter(Boolean);

const uniqueCandidates = [...new Set(candidates)];
const binPath = uniqueCandidates.find((candidate) => existsSync(candidate));

if (!binPath) {
  console.error(
    "pudding のバイナリが見つかりません。`cargo build -p pudding --release` を実行するか、絶対パスの `PUDDING_BIN_PATH` を指定してください。",
  );
  console.error("探索したパス（workspace 起点のみ）:");
  for (const candidate of uniqueCandidates) {
    console.error(`- ${candidate}`);
  }
  process.exit(1);
}

const args = process.argv.slice(2);
const child = spawn(binPath, args, { stdio: "inherit" });
child.on("exit", (code) => process.exit(code ?? 0));

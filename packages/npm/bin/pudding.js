#!/usr/bin/env node

const { spawn } = require("child_process");
const { existsSync } = require("fs");
const path = require("path");

const root = path.resolve(__dirname, "..", "..", "..");
const binPath = path.join(root, "crates", "pudding", "target", "release", "pudding");

if (!existsSync(binPath)) {
  console.error("puddingのバイナリが見つかりません。先に `cargo build -p pudding --release` を実行してください。");
  process.exit(1);
}

const args = process.argv.slice(2);
const child = spawn(binPath, args, { stdio: "inherit" });
child.on("exit", (code) => process.exit(code ?? 0));

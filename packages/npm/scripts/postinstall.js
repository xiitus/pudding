#!/usr/bin/env node

const { spawnSync } = require("child_process");
const path = require("path");

const root = path.resolve(__dirname, "..", "..", "..");
const manifest = path.join(root, "crates", "pudding", "Cargo.toml");

const result = spawnSync("cargo", ["build", "-p", "pudding", "--release", "--manifest-path", manifest], {
  stdio: "inherit",
});

if (result.status !== 0) {
  console.error("puddingのビルドに失敗しました。Rustがインストールされていることを確認してください。");
  process.exit(result.status || 1);
}

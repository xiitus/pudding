#!/usr/bin/env node

const { spawnSync } = require("child_process");
const path = require("path");

const root = path.resolve(__dirname, "..", "..", "..");
const manifest = path.join(root, "crates", "pudding", "Cargo.toml");
const cargoEnv = process.env.CARGO;
const cargoCommand = cargoEnv && path.isAbsolute(cargoEnv) ? cargoEnv : "cargo";
const commandArgs = ["build", "-p", "pudding", "--release", "--manifest-path", manifest];

console.log(`[pudding:postinstall] 実行コマンド: ${cargoCommand} ${commandArgs.join(" ")}`);

const result = spawnSync(cargoCommand, commandArgs, {
  stdio: "inherit",
});

if (result.status !== 0) {
  if (result.error) {
    console.error(
      `[pudding:postinstall] puddingのビルドに失敗しました。実行コマンド: ${cargoCommand}。詳細: ${result.error.message}`
    );
  } else {
    console.error(
      `[pudding:postinstall] puddingのビルドに失敗しました。終了コード: ${result.status}. 実行コマンド: ${cargoCommand}`
    );
  }
  console.error(
    "[pudding:postinstall] Rust/Cargoのインストール、または CARGO 環境変数に cargo 実行ファイルの絶対パスが設定されているか確認してください。"
  );
  process.exit(result.status || 1);
}

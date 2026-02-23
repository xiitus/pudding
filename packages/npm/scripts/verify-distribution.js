#!/usr/bin/env node

const { spawnSync } = require("child_process");
const { existsSync } = require("fs");
const path = require("path");

const repoRoot = path.resolve(__dirname, "..", "..", "..");
const binScript = path.join(repoRoot, "packages", "npm", "bin", "pudding.js");
const postinstallScript = path.join(
  repoRoot,
  "packages",
  "npm",
  "scripts",
  "postinstall.js",
);
const puddingBinName = process.platform === "win32" ? "pudding.exe" : "pudding";
const puddingBin = path.join(repoRoot, "target", "release", puddingBinName);

if (!existsSync(puddingBin)) {
  console.error(
    `[verify:distribution] バイナリが見つかりません: ${puddingBin}. cargo build -p pudding --release を先に実行してください。`,
  );
  process.exit(1);
}

function runNode(script, args, env) {
  return spawnSync(process.execPath, [script, ...args], {
    env: { ...process.env, ...env },
    encoding: "utf8",
  });
}

function expect(condition, message, details) {
  if (condition) {
    return;
  }
  console.error(`[verify:distribution] ${message}`);
  if (details) {
    console.error(details);
  }
  process.exit(1);
}

const relativePathFailure = runNode(binScript, ["--help"], {
  PUDDING_BIN_PATH: "relative/path",
});
expect(relativePathFailure.status === 1, "相対パス拒否テストに失敗", {
  status: relativePathFailure.status,
  stdout: relativePathFailure.stdout,
  stderr: relativePathFailure.stderr,
});
expect(
  relativePathFailure.stderr.includes("絶対パス"),
  "相対パス拒否時のエラーメッセージが不足",
  relativePathFailure.stderr,
);

const launchSuccess = runNode(binScript, ["--help"], {
  PUDDING_BIN_PATH: puddingBin,
});
expect(launchSuccess.status === 0, "絶対パス指定での起動テストに失敗", {
  status: launchSuccess.status,
  stdout: launchSuccess.stdout,
  stderr: launchSuccess.stderr,
});

const postinstallFailure = runNode(postinstallScript, [], {
  CARGO: "/definitely/missing/cargo",
});
expect(
  postinstallFailure.status !== 0,
  "postinstall 失敗時のエラー伝播テストに失敗",
  {
    status: postinstallFailure.status,
    stdout: postinstallFailure.stdout,
    stderr: postinstallFailure.stderr,
  },
);
expect(
  `${postinstallFailure.stdout}${postinstallFailure.stderr}`.includes(
    "puddingのビルドに失敗しました",
  ),
  "postinstall 失敗時のエラーメッセージが不足",
  `${postinstallFailure.stdout}${postinstallFailure.stderr}`,
);

console.log("[verify:distribution] すべての配布導線テストが成功しました。");

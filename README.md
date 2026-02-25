# pudding

`pudding` は zellij の KDL レイアウトを TUI で編集する CLI ツールです。

旧バージョンの「ペインマルチプレクサ」実行機能は廃止され、v2 では「レイアウト編集と保存」に特化しています。

## 前提

- Rust / Cargo
- macOS または Linux
- zellij（保存時に自動反映を使う場合）

## すぐ使う

```bash
# ビルド
cargo build -p pudding --release

# デフォルトテンプレート(default)を開く
cargo run -p pudding --
```

名前付きテンプレートを開く:

```bash
cargo run -p pudding -- --name dev
```

KDL を標準出力に出す（ファイル保存なし）:

```bash
cargo run -p pudding -- --name dev --dry-run
```

## CLI

```bash
pudding [--name <name>] [--dry-run]
```

- 引数なし: `default` テンプレートを開く
- `--name <name>`: 指定テンプレートを開く（なければ単一ペインの初期レイアウトを生成）
- `--dry-run`: TUI を起動せず、KDL を stdout に出力して終了
- 実行エラー時: メッセージを stderr に出力し、終了コードは非 0

## TUI 操作

- カーソル移動: `← ↑ ↓ →`
- 分割: `v`（縦）/ `h`（横）
- ペインコマンド編集: `c`（Enter 確定 / Esc キャンセル / Backspace 削除）
- ペイン削除: `d`（確認 `y/n`、最後の 1 ペインは削除不可）
- タブ切替: `Tab` / `Shift+Tab`
- タブ追加: `T`
- タブ名変更: `n`
- 保存: `s`
- 終了: `q`（未保存時は確認ダイアログ）

## テンプレート（KDL）

保存先:

- `~/.config/pudding/templates/<name>.kdl`
- `XDG_CONFIG_HOME` がある場合: `<XDG_CONFIG_HOME>/pudding/templates/<name>.kdl`

テンプレート名制約:

- 使用可能文字: `A-Z a-z 0-9 _ -`
- 長さ: 1〜64 文字

KDL の最小例:

```kdl
layout name="dev" {
  tab name="main" active=true {
    pane split_direction="vertical" {
      pane command="bash"
      pane command="htop"
    }
  }
}
```

## v1 からの移行

- v1 の JSON テンプレート（`~/.config/pudding/templates/*.json`）は v2 では読み込めません
- v2 は `*.kdl` のみ対応です（保存先: `~/.config/pudding/templates/<name>.kdl`）
- 既存 JSON を使っていた場合は、v2 で同名テンプレートを開いて再作成し、`s` で保存してください

## zellij 連携

- `s` で保存時、`ZELLIJ_SESSION_NAME` が非空なら `zellij action apply-layout <absolute_path>` を実行
- `--dry-run` 時は `apply-layout` を実行しない
- 反映失敗時はクラッシュせず、TUI のステータスバーにエラーを表示

## 設定ファイル

- `~/.config/pudding/config.json`（または `XDG_CONFIG_HOME` 配下）
- `default_command` を保持
- 旧形式の `keybinds` は読み込み互換あり（v2 の主要操作は TUI 側で固定キーを使用）

## npm / pnpm / bun でのローカル利用

```bash
pnpm add -g ./packages/npm
bun add -g ./packages/npm
```

`packages/npm/bin/pudding.js` のバイナリ探索順:

1. `PUDDING_BIN_PATH`（絶対パスのみ）
2. `CARGO_TARGET_DIR/release/pudding`
3. ワークスペース直下 `target/release/pudding`
4. `crates/pudding/target/release/pudding`

見つからない場合:

```bash
cargo build -p pudding --release
```

配布導線チェック:

```bash
pnpm --dir packages/npm run verify:distribution
bun run --cwd packages/npm verify:distribution
```

## トラブルシュート

- `pudding: command not found`
  - `cargo install --path crates/pudding` か `target/release/pudding` を PATH に追加
- `failed to parse KDL template`
  - 対象 `*.kdl` の構文（`layout -> tab -> pane`）を確認
- `name supports only [A-Za-z0-9_-]`
  - テンプレート名が制約違反
- `zellij action apply-layout ... failed`
  - zellij がインストール済みか、セッション内実行か確認
- `PUDDING_BIN_PATH には絶対パスを指定してください`
  - `PUDDING_BIN_PATH` を絶対パスに修正

## ライセンス

MIT

# pudding

zellij を参考に、機能を最小限まで削ったペイン・マルチプレクサです。分割、サイズ変更、隣接ペイン交換、状態保存/復元、テンプレート作成に絞っています。

## 機能
- 画面分割（縦/横）
- サイズ変更（分割線の移動）
- 縦幅が同じで隣接するペインの交換
- 横幅が同じで隣接するペインの交換
- 状態保存 / 復元
- AAミニチュアによるテンプレート作成
- ペインは `bite`、分割線は `spoon` と呼称

## インストール（開発用）
Rust が必要です。

```bash
cargo build -p pudding
```

## pnpm/bun での利用（ローカル）
```bash
pnpm add -g ./packages/npm
bun add -g ./packages/npm
```

`pudding` コマンドは次の順で実行バイナリを探索します。
- `PUDDING_BIN_PATH` で指定した**絶対パス**
- `CARGO_TARGET_DIR/release/pudding`
- ワークスペース配下の `target/release/pudding`

新方針:
- `PUDDING_BIN_PATH` が相対パスの場合は起動せず明示的にエラー終了します。
- `cwd`（カレントディレクトリ）配下の探索は行いません。

見つからない場合は、`cargo build -p pudding --release` を実行してください。

配布導線のセルフチェック:
```bash
pnpm --dir packages/npm run verify:distribution
bun run --cwd packages/npm verify:distribution
```

### 権限ハードニング方針（npm ランチャー）
- 実行対象は通常ファイルかつ実行可能ビットがあるもののみ許可します。
- シンボリックリンクや world-writable なバイナリは拒否し、fail-closed で終了します。

## 使い方
### テンプレート編集
```bash
cargo run -p pudding -- template edit --name default
```

- 矢印キーでカーソル移動
- `v` で縦分割、`h` で横分割
- `n` でペイン名、`c` で初期コマンドを設定
- `s` で保存、`q` で終了

### テンプレート適用
```bash
cargo run -p pudding -- run --template default
```

## 設定
`~/.config/pudding/config.json`

```json
{
  "default_command": "bash",
  "keybinds": {
    "split_vertical": "v",
    "split_horizontal": "h",
    "resize_left": "H",
    "resize_right": "L",
    "resize_up": "K",
    "resize_down": "J",
    "swap_vertical": "S",
    "swap_horizontal": "s",
    "save_state": "Ctrl+S",
    "restore_state": "Ctrl+R",
    "focus_next": "Tab",
    "quit": "Ctrl+C"
  }
}
```

## テンプレート/状態の保存先
- テンプレート: `~/.config/pudding/templates/*.json`
- 状態: `~/.config/pudding/states/*.json`

## ライセンス
MIT

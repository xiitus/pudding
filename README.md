# pudding

`pudding` は、最低限の操作に絞ったペイン・マルチプレクサです。

- 画面分割（縦/横）
- サイズ変更（分割線の移動）
- 隣接ペイン交換（縦方向/横方向）
- 状態保存 / 復元
- AAミニチュアによるテンプレート編集

## まず5分で使う

### 1. 前提
- Rust と Cargo が使えること
- macOS / Linux で動作確認済み

### 2. ビルド
```bash
cargo build -p pudding --release
```

### 3. テンプレートを作る
```bash
cargo run -p pudding -- template edit --name default
```

テンプレートエディタの基本キー:
- 矢印キー: カーソル移動
- `v`: 縦分割
- `h`: 横分割
- `n`: ペイン名編集
- `c`: 初期コマンド編集
- `s`: 保存
- `q`: 終了

### 4. 実行する
```bash
cargo run -p pudding -- run --template default
```

`ghostty` / `cmux` 上でも通常のTUIアプリとして起動できます。

## コマンド一覧

```bash
pudding --help
```

主なサブコマンド:
- `pudding run --template <name>`: テンプレートで起動
- `pudding template edit --name <name>`: テンプレート編集
- `pudding template apply --name <name>`: テンプレート適用で起動

## ランタイムの基本キー（デフォルト）

- `v` / `h`: 縦分割 / 横分割
- `H` / `L` / `K` / `J`: リサイズ（左 / 右 / 上 / 下, 1回あたり20%）
- `S` / `s`: 隣接交換（縦方向 / 横方向）
- `Ctrl+S`: 現在状態を保存
- `Ctrl+R`: 保存状態を復元
- `Tab`: フォーカス移動
- `Ctrl+C`: 終了

## 設定ファイル

場所: `~/.config/pudding/config.json`

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

## 保存先

- テンプレート: `~/.config/pudding/templates/*.json`
- 状態: `~/.config/pudding/states/*.json`

テンプレート名/保存名の制約:
- 使用可能文字: `A-Z a-z 0-9 _ -`
- 文字数: 1〜64

## pnpm / bun でのローカル利用

```bash
pnpm add -g ./packages/npm
bun add -g ./packages/npm
```

npm ラッパーは次の順で実行バイナリを探索します:
- `PUDDING_BIN_PATH`（絶対パスのみ許可）
- `CARGO_TARGET_DIR/release/pudding`
- ワークスペース配下の `target/release/pudding`

見つからない場合:
```bash
cargo build -p pudding --release
```

配布導線のセルフチェック:
```bash
pnpm --dir packages/npm run verify:distribution
bun run --cwd packages/npm verify:distribution
```

## トラブルシュート

- `pudding: command not found` が出る:
  - PATH に入っていません。次のどちらかを実行してください。
  - `cargo install --path crates/pudding`
  - `alias pudding="<project-root>/target/release/pudding"`
- `invalid config file` が出る:
  - `~/.config/pudding/config.json` のJSONが壊れています。修正するか削除して再生成してください。
- テンプレート読み込みエラーが出る:
  - 名前制約違反、ID重複、`ratio` 範囲外（0と1を含まない）を確認してください。
- npm ラッパーで起動できない:
  - `PUDDING_BIN_PATH` が相対パスだと失敗します。絶対パスを指定してください。

## ライセンス

MIT

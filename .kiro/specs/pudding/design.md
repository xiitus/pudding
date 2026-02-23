# 設計

## 概要
- Rust + TUI（ratatui/crossterm）で実装
- レイアウトは二分木で表現（`bite` と `spoon`）
- PTY は portable-pty を使用
- テンプレート/状態は JSON で保存
- `editor`（テンプレート編集）と `runtime`（実行制御）を分離し、共通モデルは `core` に集約する

## データモデル
```json
{
  "name": "default",
  "layout": {
    "type": "bite",
    "id": 1,
    "name": "main",
    "command": "bash"
  }
}
```

`spoon` は以下の形:
```json
{
  "type": "spoon",
  "id": 3,
  "orientation": "vertical",
  "ratio": 0.5,
  "first": { "type": "bite", "id": 1, "name": "left", "command": "bash" },
  "second": { "type": "bite", "id": 2, "name": "right", "command": "htop" }
}
```

## UI
### テンプレート編集
- 画面全体をミニチュア表示し、カーソル位置で `v/h` 分割
- `n` で名前、`c` でコマンド設定
- `s` 保存、`q` 終了

### 実行画面
- ペインにプロセス出力を表示
- キーバインドで分割/リサイズ/交換/保存/復元
- `Tab` でフォーカス移動

## モジュール分割
- `core`: `bite/spoon` モデル、検証、JSON シリアライズ、操作ロジック（純粋関数）
- `editor`: AA ミニチュア編集 UI、入力イベント解釈、テンプレート保存ユースケース
- `runtime`: PTY 管理、描画対象バッファ、実行時操作（分割/交換/保存/復元）
- 依存方向は `editor|runtime -> core` のみとし、`editor <-> runtime` の直接依存を禁止

## npm 配布探索方針
- 配布方式は 2 系統を比較する
- 1) npm ラッパーが Rust バイナリを同梱し、`postinstall` で配置する方式
- 2) ユーザー環境で `cargo install` を誘導する薄い CLI 方式
- 評価観点は `pnpm/bun` 互換性、OS 別配布コスト、初回インストール時間、失敗時の明示的エラー表示
- 採用案は検証結果を `README` とリリース手順へ反映する

## 設定
- `~/.config/pudding/config.json`
- キーバインドとデフォルトコマンドを定義

## 検証/失敗時ポリシー
- `template/state` のロード時に構造検証を行い、失敗時は即時エラー
- 名前入力は `[A-Za-z0-9_-]{1,64}` のみ許可
- `Config::load` は壊れた JSON を許容せず、起動を失敗させる
- 保存/復元に失敗した場合は UI ステータスへ原因を表示する

## セキュリティ設計
- コマンド実行はテンプレート由来でも空文字を禁止
- テンプレート名/状態名のパストラバーサルを禁止
- 暗黙フォールバックを禁止し、fail-closed で停止する

## テスト設計（追加）
- `core` のプロパティテスト: ratio 境界、ID 重複、交換可能条件、保存/復元の可逆性
- `editor` の入力テスト: 分割キー、名前/コマンド入力、無効入力拒否、保存導線
- `runtime` の統合テスト: PTY 起動失敗、レイアウト変更後の描画更新、状態保存/復元整合
- npm 配布検証: `pnpm`/`bun` でのインストール・起動・エラー伝播を CI マトリクスで確認

# 設計

## 概要
- Rust + TUI（ratatui/crossterm）で実装
- レイアウトは二分木で表現（`bite` と `spoon`）
- PTY は portable-pty を使用
- テンプレート/状態は JSON で保存

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

## 設定
- `~/.config/pudding/config.json`
- キーバインドとデフォルトコマンドを定義


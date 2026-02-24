# 実装タスク — pudding-v2

## 実装計画

- [x] 1. 既存コードの移行と依存関係の更新
- [x] 1.1 RuntimeApp・PaneProcess・PTY 関連コードを削除する
  - `RuntimeApp`・`PaneProcess` 構造体とその実装を `runtime.rs` ごと削除する
  - `runtime_*.rs` ヘルパーファイル群をすべて削除する
  - `portable-pty` への依存を使用箇所ごと除去する
  - _Requirements: 6.1_

- [x] 1.2 Cargo.toml の依存関係を更新する
  - `portable-pty` を `[dependencies]` から削除する
  - `kdl` crate（6.x）を新規依存として追加する
  - ビルドが通ることを確認する（`cargo build` で警告・エラーなし）
  - _Requirements: 6.4, 6.5_

- [x] 1.3 既存の JSON テンプレートシステムとデータモデルを撤去する
  - `model.rs` の `Node::Bite` / `Node::Spoon` / `Template` 定義を削除する
  - `template.rs` の JSON 読み書き・`serde_json` 依存部分を除去する
  - `editor.rs` の旧 EditorApp 実装を削除（ファイルは残す）する
  - `action.rs` / `keybind.rs` は構造を維持しつつ不要なアクション定義を整理する
  - _Requirements: 6.3_

- [x] 2. ドメインモデルとペインツリー操作の実装
- [x] 2.1 KDL レイアウト用のデータモデルを定義する
  - `Layout`（name, tabs, active_tab）、`Tab`（name, root）、`Node`（Pane / Split）を定義する
  - `Direction`（Vertical / Horizontal）と `ratio` の有効範囲（0.1〜0.9）を定義する
  - 全ノードで `id` が一意であることを不変条件として文書化する
  - _Requirements: 6.3_

- [x] 2.2 ペインツリーの操作関数を実装する
  - `next_id`（最大 id+1）、`collect_panes`、`find_node`、`find_node_mut` を実装する
  - `split_node`（指定ノードを Split に変換し新 Pane を追加）を実装する
  - `delete_node`（兄弟ノードで親 Split を置き換える、最後の 1 ペインは禁止）を実装する
  - `layout_rects`（ratatui `Rect` への変換）と `find_pane_at`（座標からペイン検索）を実装する
  - _Requirements: 3.2, 3.3, 3.5_

- [x] 2.3 LayoutModel のユニットテストを実装する
  - `split_node`・`delete_node` の正常系・異常系（1 ペイン削除禁止・ID 重複なし）を確認する
  - `find_pane_at` がカーソル座標を正しくペインにマッピングすることを確認する
  - `next_id` が最大 ID+1 を返すことを確認する
  - _Requirements: 3.3, 3.5_

- [x] 3. KDL 変換とテンプレート永続化の実装
- [x] 3.1 Layout → KDL へのシリアライズを実装する
  - `Layout` を `kdl::KdlDocument` に変換する `to_kdl_document` 関数を実装する
  - ルートは `layout { }`、タブは `tab name="..." { }`、Split は `pane split_direction="..." { }`、Pane は `pane` または `pane command="..."` として出力する
  - 単一タブでも `tab` ノードを明示的に出力する（zellij 互換）
  - _Requirements: 2.4_

- [x] 3.2 KDL → Layout へのパースを実装する
  - `kdl::KdlDocument` を受け取り `Layout` を返す `from_kdl_document` 関数を実装する
  - `layout { }` ルートがない場合はエラーを返す
  - ネストした `pane` ノードを再帰的に走査して `Node::Split` / `Node::Pane` ツリーを構築する
  - パース後に `id` 重複・ratio 範囲外をバリデーションする
  - _Requirements: 2.3, 2.4, 2.5_

- [x] 3.3 テンプレート名バリデーションとファイル読み書きを実装する
  - `validate_name`（`[A-Za-z0-9_-]`・1〜64 文字）を実装する
  - `load(name)`：KDL ファイルが存在すればパースして返す。存在しなければデフォルト（単一 Pane）の `Layout` を返す
  - `save(name, layout)`：`~/.config/pudding/templates/<name>.kdl` に書き込む（パーミッション 0o600・親ディレクトリ 0o700）。パスを `PathBuf` で返す
  - `save_dry_run(layout)`：ファイルに書かず KDL テキストを `String` で返す
  - _Requirements: 2.1, 2.2, 2.3, 2.5_

- [x] 3.4 KdlConverter と TemplateStore のユニットテスト・統合テストを実装する
  - `to_kdl_document` → `from_kdl_document` のラウンドトリップが等価な `Layout` を返すことを確認する（単一ペイン・縦分割・タブ複数）
  - `validate_name` の有効・無効ケースを確認する（パストラバーサル含む）
  - `save` → `load` のファイルシステムラウンドトリップを一時ディレクトリで確認する
  - _Requirements: 2.1, 2.2, 2.4, 2.5_

- [x] 4. (P) zellij セッション連携の実装
  - `$ZELLIJ_SESSION_NAME` 環境変数の非空チェックでセッション内判定する `is_in_zellij_session` を実装する
  - `apply_layout(path)`：`zellij action apply-layout <absolute_path>` を実行し、非 0 終了時は `Err` を返す
  - `apply_layout_if_in_session(path)`：セッション外・dry-run 時は `None`。失敗時はエラーメッセージを `Some(String)` で返す（クラッシュしない）
  - `is_in_zellij_session` の環境変数あり・なしのテストを実装する
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [x] 5. CLI エントリポイントの実装
- [x] 5.1 clap による CLI 引数を定義する
  - `pudding`（引数なし）・`--name <name>`・`--dry-run` オプションを clap derive で定義する
  - `--help` が引数一覧を表示することを確認する
  - _Requirements: 1.1, 1.2, 1.3, 1.5_

- [x] 5.2 起動フローを組み立てる
  - 設定ファイルをロードし、テンプレート名（デフォルト: `"default"`）で `TemplateStore::load` を呼ぶ
  - `--dry-run` 時は `save_dry_run` の結果を stdout に出力して終了する
  - 通常時は `EditorApp::run` を呼び出し、エラーを stderr に出力して非 0 で終了する
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [x] 6. TUI エディタの骨格と描画基盤の実装
- [x] 6.1 画面レイアウト（タブバー・メインエリア・ステータスバー）を構築する
  - ratatui の `Layout` で `[タブバー(1行)] [メインエリア(fill)] [ステータスバー(2行)]` に分割する
  - タブバーにタブ名を横並びで表示し、アクティブタブを強調（Yellow）表示する
  - ステータスバーに「[pudding] アクティブペイン名・保存状態・主要キーヒント」を表示する
  - `EditorApp` 構造体（layout, name, cursor, input_mode, dirty, status_msg）を定義する
  - _Requirements: 3.1, 3.8, 4.1, 6.2_

- [x] 6.2 カーソル移動とアクティブペイン検出を実装する
  - 矢印キー（Left/Right/Up/Down）でカーソルをメインエリア内で移動させる
  - カーソル位置（`x` マーカー、Cyan）を `f.buffer_mut().get_mut` で描画する
  - `find_pane_at(cursor)` でアクティブペインを特定し、ボーダーを Yellow でハイライトする
  - _Requirements: 3.2_

- [x] 7. ペイン操作（分割・コマンド設定・削除）の実装
- [x] 7.1 縦横分割操作を実装する
  - `v` キーでカーソル位置の縦分割（`Direction::Vertical`）を `LayoutModel::split_node` で実行する
  - `h` キーで横分割（`Direction::Horizontal`）を実行する
  - 分割後は `dirty = true` にセットし、ステータスバーに「分割しました」を表示する
  - _Requirements: 3.3_

- [x] 7.2 コマンド設定インライン入力を実装する
  - `c` キーでアクティブペインのコマンド入力モード（`InputMode::PaneCommand`）に入る
  - 中央モーダルダイアログ（`centered_rect`）に入力プロンプトを表示する
  - Enter で確定、Esc でキャンセル、Backspace で 1 文字削除の入力ハンドリングを実装する
  - _Requirements: 3.4_

- [x] 7.3 ペイン削除操作を実装する
  - `d` キーで削除確認プロンプト（`InputMode::ConfirmDelete`）を表示する
  - ペインが 1 つのみの場合は「削除できません（最後のペイン）」をステータスバーに表示して操作を禁止する
  - 確認後に `LayoutModel::delete_node` を呼び、`dirty = true` にセットする
  - _Requirements: 3.5_

- [x] 8. タブ管理の実装
- [x] 8.1 タブバー描画とタブ切り替えを実装する
  - 現在アクティブな `Layout.active_tab` インデックスに基づきタブバーをレンダリングする
  - `Tab` / `Shift+Tab` キーで `active_tab` をインクリメント／デクリメント（循環）する
  - タブ切り替え時にカーソル位置をリセットする
  - _Requirements: 4.1, 4.5_

- [x] 8.2 タブ追加と名前変更を実装する
  - `T` キーで新規タブ（デフォルト名: `tab-N`、単一 Pane）を `Layout.tabs` に追加し、新タブをアクティブにする
  - `n` キーでアクティブタブの名前変更モード（`InputMode::TabName`）に入り、Enter で確定する
  - 追加・変更後は `dirty = true` にセットする
  - _Requirements: 4.2, 4.3_

- [x] 9. 保存・終了フローの統合
- [x] 9.1 保存と zellij 連携を統合する
  - `s` キーで `TemplateStore::save` を呼び出す。成功したパスを `ZellijBridge::apply_layout_if_in_session` に渡す
  - apply-layout 成功時はステータスバーに「保存しました（zellij に反映）」を表示する
  - apply-layout 失敗時はステータスバーにエラーメッセージを表示するがクラッシュしない
  - 保存成功後は `dirty = false` にリセットする
  - _Requirements: 3.6, 5.1, 5.3, 5.4_

- [x] 9.2 TUI 終了と未保存確認を実装する
  - `q` キーで `dirty == false` の場合は即座に終了する
  - `dirty == true` の場合は `InputMode::ConfirmQuit` を表示し「保存せず終了しますか？(y/n)」を確認する
  - `y` で終了、`n` で TUI に戻る
  - _Requirements: 3.7_

- [x] 9.3 dry-run モードのフローを実装する
  - CLI で `--dry-run` フラグが立っている場合、`EditorApp` を起動せず `TemplateStore::save_dry_run` の結果を stdout に出力して終了する
  - dry-run 時は apply-layout を実行しない
  - _Requirements: 1.3, 5.2_

- [x] 10. 統合テストと最終検証
- [x] 10.1 KDL ラウンドトリップ統合テストを実装する
  - タブ複数・縦横分割・コマンド設定を含む `Layout` を KDL に変換し、再パースして等価性を確認する
  - 生成された KDL が有効な zellij レイアウト構文であることをコメントで記録する
  - _Requirements: 2.3, 2.4_

- [x] 10.2 TemplateStore のファイルシステム統合テストを実装する
  - 一時ディレクトリを使って `save` → `load` → `save_dry_run` のラウンドトリップを確認する
  - パーミッション（0o600）が正しく設定されることを確認する（Unix のみ）
  - _Requirements: 2.1, 2.2, 2.5_

---

> **注意（4.4 の延期）**
> 要件 4.4（タブ削除）は MVP 外として本タスクリストから除外する。タブ削除は「最後のタブの扱い」「含まれるペインの後処理」を考慮した後続スペックで対応する。

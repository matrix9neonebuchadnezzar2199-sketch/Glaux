# Glaux Phase Plan

| Phase | 名称 | 状態 | 完了条件 |
|-------|------|------|----------|
| 0 | ドキュメント土台 | completed | docs/ から設計・PHASE・マイルストーンが追える |
| 1 | Rust 土台 | completed | `cargo build --release` 成功、空ウィンドウ表示 |
| 2 | ランタイム管理 | completed | 自己展開・SHA-256・子プロセス起動/停止 |
| 3 | API クライアント | completed | `/v1/chat/completions` stream 対応 |
| 4 | チャット UI | completed | Message list、送信、クリーン、コンテキスト表示 |
| 5 | モデル制御 UI | completed | 状態表示、起動/停止/解説 |
| 6 | 要約 | completed | Drawer、プロンプト切替 |
| 7 | 設定 | completed | テーマ、文字サイズ、推論パラメータ |
| 8 | ヘルプ | completed | 本アイコン Help、履歴なし説明 |
| 9 | 単独 EXE | completed | アセット埋め込みビルド手順と検証 |
| 10 | All_Status | completed | user-stories マトリクスとランナー |

## 進行ルール

- Phase 開始時: 本表の `状態` を `in_progress` に更新
- Phase 終了時: `milestones.md` と `decision-log.md` を更新し、本表を `completed` にする

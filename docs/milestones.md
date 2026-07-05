# Glaux Milestones

| ID | 名称 | 状態 | 日付 | 成果物 | 検証 |
|----|------|------|------|--------|------|
| M0 | ドキュメント土台 | completed | 2026-06-30 | `docs/` 一式 | 00-index から全設計へ辿れる |
| M1 | Rust 土台 | completed | 2026-06-30 | `Cargo.toml`, eframe/egui | `cargo build --release` OK |
| M2 | ランタイム | completed | 2026-06-30 | `runtime/` モジュール | 展開・SHA-256・ヘルスチェック実装 |
| M3 | API | completed | 2026-06-30 | `api/chat.rs` | stream / blocking 実装 |
| M4 | チャット UI | completed | 2026-06-30 | chat view | 送信・コピー・再生成・クリーン |
| M5 | モデル制御 | completed | 2026-06-30 | model bar | 起動/停止/解説 |
| M6 | 要約 | completed | 2026-06-30 | summary drawer | 3 モード |
| M7 | 設定 | completed | 2026-06-30 | settings + config.json | 保存/読込 |
| M8 | ヘルプ | completed | 2026-06-30 | help window | オフライン説明 |
| M9 | 単独 EXE | completed | 2026-06-30 | `build.rs`, `scripts/build-release.ps1` | cfg 埋め込み + dist 手順 |
| M10 | All_Status | completed | 2026-06-30 | user-stories + scripts | matrix 生成 OK |

## 残課題

- `artifacts/llama-server.exe` と `artifacts/model/*.gguf` は配布ビルド前に手動配置が必要
- 実機 8GB RAM での Gemma 4 E2B 起動・応答速度はマスター環境で手動検証（`manual` ストーリー）

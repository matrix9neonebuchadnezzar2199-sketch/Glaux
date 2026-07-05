# Glaux Architecture

## 概要

```
[User] → Glaux.exe (eframe/egui)
           ├─ UI Layer (chat, settings, help, model info)
           ├─ Runtime Manager (extract, spawn, health)
           └─ API Client → llama-server @ 127.0.0.1:PORT
                              └─ Gemma 4 E2B GGUF
```

## レイヤー責務

| レイヤー | モジュール | 責務 |
|----------|------------|------|
| UI | `ui::*` | 描画、入力、Drawer。重い処理は同期しない |
| Runtime | `runtime::*` | 自己展開、SHA-256、子プロセス、メモリチェック |
| API | `api::chat` | OpenAI 互換 `/v1/chat/completions`、stream |
| Config | `config` | `config.json` のみ永続化 |
| Context | `context` | メモリ上メッセージ、トークン概算 |

## プロセスライフサイクル

1. アプリ起動 → 設定読込
2. 「起動」または自動起動 → extracting → starting → ready
3. チャット/要約 → generating → ready
4. 「停止」または終了 → stopping → stopped

## UI スレッド規約

- モデル展開・サーバー起動・HTTP stream はバックグラウンドスレッド
- 結果は `std::sync::mpsc` または `crossbeam-channel` で UI に通知
- PDF-SCAN 教訓: UI スレッドでブロックしない

## API 境界

- Request: `POST /v1/chat/completions` with `stream: true`
- Messages: `system`（Glaux 人格・要約指示）+ `user`/`assistant` 履歴（メモリのみ）
- Gemma 4: `temperature=1.0`, `top_p=0.95` を初期値（設定で上書き可）

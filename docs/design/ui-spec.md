# Glaux UI Spec

Component Gallery 名で統一。アプリ名: **Glaux -OFFLINE AI Chat-**

## Chat view

- **Header**: タイトル、要約、設定、Help
- **Model status bar**: `{モデル名} — {状態}`
- **Model control buttons**: 起動、停止、解説
- **Message list**: Bot / User バブル（テーマ連動）
- **Progress indicator**: `Context: 約 N / 4000 tokens`
- **入力欄**: Textarea + 送信 + クリーン + **プロンプト支援**（ホバーメニュー、紺背景）

## 状態

| 状態 | UI |
|------|-----|
| Loading | 起動中 Alert |
| Empty | 初回案内（runtime/model 配置手順） |
| Error | Alert + 配置フォルダを開く |
| Toast | コピー完了、設定保存 |

## Settings（Modal）

- **外観**: Segmented control — Glaux（ダーク）/ ライトモード
- **文字サイズ**: Segmented control — 小/中/大/特大
- **使用モデル**: 表示のみ — Google Gemma 4 E2B
- **Toggle**: 自動起動
- **Footer**: 保存 / 閉じる（右寄せ）

## テーマ

| プリセット | 説明 |
|------------|------|
| Glaux（ダーク） | 黒緑 + 金アクセント、明文字 |
| ライトモード | 明背景 + 黒文字 + 抑えた金 |

ライト時は `override_text_color`、入力欄、バブル文字色をすべて暗色系に統一。

## アクセシビリティ

- ボタン最小 44px タッチ相当
- 色だけに依存しない状態ラベル（「起動完了」等のテキスト）

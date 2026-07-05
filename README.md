# Glaux -OFFLINE AI Chat-

ローカルオフラインの Gemma 4 チャットボット。配布は **`Glaux.exe` + `model/`** のみ。llama-server と DLL は EXE に内蔵され、初回起動時に `%LOCALAPPDATA%\Glaux\runtime\` へ自動展開されます。

## ドキュメント

設計・PHASE・マイルストーンの正本は [`docs/00-index.md`](docs/00-index.md)。

## 開発ビルド

```powershell
cd F:\Cursor\Glaux
cargo build
cargo run
```

アーティファクト無しでも UI は起動する。モデル起動には次のいずれかが必要:

- `artifacts/llama-server.exe`（+ DLL）と `artifacts/model/*.gguf`
- 環境変数 `GLAUX_DEV_SERVER` / `GLAUX_DEV_MODEL`（フルパス）

## ランタイム取得

```powershell
.\scripts\bootstrap-runtime.ps1
cargo build --release
.\target\release\glaux.exe
```

`bootstrap-runtime.ps1` は以下を `artifacts/` に配置します:

- llama.cpp Windows CPU runtime（`llama-server.exe` + DLL）
- `model/gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf`

## ポータブル配布ビルド

```powershell
.\scripts\bootstrap-runtime.ps1   # 初回のみ
.\scripts\build-release.ps1
# → dist/Glaux/
```

配布フォルダ構成:

```
Glaux/
  Glaux.exe
  model/
    gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf
  README.txt
```

## モデル

- **同梱**: Google Gemma 4 E2B Unsloth QAT Q2（約 2.2GB、空きメモリ 3GB 以上推奨）

## テストマトリクス

```powershell
node scripts/run-user-story-tests.mjs --matrix-only
node scripts/run-user-story-tests.mjs
```

## ライセンス

Gemma モデルは [Google Gemma Terms of Use](https://ai.google.dev/gemma/terms) に従います。

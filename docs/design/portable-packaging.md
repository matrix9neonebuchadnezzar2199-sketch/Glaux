# Glaux Portable Folder Packaging

## 配布形態

- **配布物**: `dist/Glaux/` フォルダ一式（`Glaux.exe` + `model/`）
- **ランタイム**: EXE 内蔵。初回起動時に `%LOCALAPPDATA%\Glaux\runtime\` へ展開

```
Glaux/
  Glaux.exe
  model/
    gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf
  README.txt
```

## ビルド前準備

```
Glaux/artifacts/
  llama-server.exe
  *.dll
  model/
    gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf
```

`.\scripts\bootstrap-runtime.ps1` で取得可能。

## 開発時フォールバック

ポータブル layout が無い場合（`cargo run` 等）は `artifacts/` を参照する。

環境変数:

- `GLAUX_DEV_SERVER` — llama-server.exe のフルパス
- `GLAUX_DEV_MODEL` — GGUF のフルパス
- `GLAUX_DEV_MODEL_DIR` — GGUF ディレクトリ

旧構成 `models/` は配布フォルダ側のみ読み取り互換あり。

## Git 除外

```
artifacts/
target/
dist/
```

## リスク

- 配布フォルダ全体 約 2.5GB 級（E2B Q2 のみ）
- AV 誤検知 → コード署名推奨

## ビルド手順

```powershell
cd F:\Cursor\Glaux
.\scripts\bootstrap-runtime.ps1
.\scripts\build-release.ps1
```

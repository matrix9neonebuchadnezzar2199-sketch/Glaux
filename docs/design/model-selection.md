# Glaux Model Selection

## 同梱モデル（v0.2）

| 項目 | E2B | E4B（任意） |
|------|-----|-------------|
| モデル | Google Gemma 4 E2B IT | Google Gemma 4 E4B IT (Unsloth QAT) |
| 形式 | GGUF | GGUF |
| 量子化 | UD-Q2_K_XL (Unsloth QAT mobile) | UD-Q4_K_XL |
| ファイル名 | `gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf` | `gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf` |
| 空きメモリ推奨 | 3072 MB | 6000 MB |
| context 固定値 | 1000 | 1000 |
| ライセンス | Gemma Terms of Use | Gemma Terms of Use |

## ポリシー

- 既定は **E2B**。設定画面で **E4B** を手動選択可能（`model/` にファイルがある場合）
- `build-release.ps1` は `artifacts/model/` に E4B があるときのみ dist へ同梱
- コンテキスト長は **1000** 固定（低メモリ向け。設定 UI なし）
- llama-server は `-t 2`、`-b 512`、`-ub 128`、`-ctk/-ctv q8_0`、`--cache-ram 0`、`--no-warmup`、`--no-webui`、`-np 1`、CPU のみ（SWA checkpoint は llama-server 既定 32）

## 採用理由

- オンデバイス向け小型 Dense、system role 対応
- Glaux はテキストチャット専用（マルチモーダル未使用）
- 8GB RAM 環境でも E2B + 長コンテキスト運用が現実的

## サンプリング（Gemma 4 推奨）

- `temperature`: 1.0
- `top_p`: 0.95
- `top_k`: 64

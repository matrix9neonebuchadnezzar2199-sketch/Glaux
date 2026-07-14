# Glaux Model Selection

## 同梱モデル（v0.2）

| 項目 | E2B | E4B（任意） |
|------|-----|-------------|
| モデル | Google Gemma 4 E2B IT | Google Gemma 4 E4B IT (Unsloth QAT) |
| 形式 | GGUF | GGUF |
| 量子化 | UD-Q2_K_XL (Unsloth QAT mobile) | UD-Q4_K_XL |
| ファイル名 | `gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf` | `gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf` |
| 空きメモリ推奨 | 3072 MB | 6000 MB |
| context 固定値 | 3000 | 3000 |
| ライセンス | Gemma Terms of Use | Gemma Terms of Use |

## 選択可能な追加プリセット

`model/` に該当 GGUF があれば設定画面で選択できる。

| モデル | ファイル名 | 量子化 | サイズ | 空きメモリ推奨 |
|--------|-----------|--------|--------|----------------|
| MiniCPM5 1B | `minicpm5-1b-Q8_0.gguf` | Q8_0 | 約 1.1GB | 1792 MB | ChatML |
| MiniCPM5 1B | `minicpm5-1b-f16.gguf` | F16 | 約 2.1GB | 2560 MB | ChatML |
| Rakuten AI 2.0 mini | `RakutenAI-2.0-mini-instruct-Q4_K_M.gguf` | Q4_K_M | 約 0.9GB | 1536 MB | Rakuten |
| Rakuten AI 2.0 mini | `RakutenAI-2.0-mini-instruct-Q5_K_M.gguf` | Q5_K_M | 約 1.0GB | 1792 MB | Rakuten |
| Rakuten AI 2.0 mini | `RakutenAI-2.0-mini-instruct-Q8_0.gguf` | Q8_0 | 約 1.5GB | 2048 MB | Rakuten |
| Qwen 2.5 3B | `qwen2.5-3b-instruct-q2_k.gguf` | Q2_K | 約 1.3GB | 2560 MB | Qwen |
| Qwen 2.5 3B | `qwen2.5-3b-instruct-q3_k_m.gguf` | Q3_K_M | 約 1.6GB | 3072 MB | Qwen |
| Gemma 2 2B JPN-IT | `gemma-2-2b-jpn-it-Q4_K_M.gguf` | Q4_K_M | 約 1.6GB | 2560 MB | Gemma2 |
| Llama 3.2 1B (Unsloth) | `unsloth.Q4_K_M.gguf` | Q4_K_M | 約 0.9GB | 1536 MB | Llama3 |

MiniCPM5 1B の取得: `hf download Abiray/MiniCPM5-1B-GGUF minicpm5-1b-Q8_0.gguf` / `minicpm5-1b-f16.gguf`（または `scripts/bootstrap-runtime.ps1`）。

## ポリシー

- 既定は **E2B**。設定画面で **E4B** を手動選択可能（`model/` にファイルがある場合）
- `build-release.ps1` は `artifacts/model/` に E4B があるときのみ dist へ同梱
- コンテキスト長は **3000** 固定（設定 UI なし）
- llama-server は `-t 2`、`-b 512`、`-ub 128`、`-ctk/-ctv q8_0`、`--cache-ram 0`、`--no-warmup`、`--no-webui`、`-np 1`、CPU のみ（SWA checkpoint は llama-server 既定 32）

## 採用理由

- オンデバイス向け小型 Dense、system role 対応
- Glaux はテキストチャット専用（マルチモーダル未使用）
- 8GB RAM 環境でも E2B + 長コンテキスト運用が現実的

## サンプリング（Gemma 4 推奨）

- `temperature`: 1.0
- `top_p`: 0.95
- `top_k`: 64

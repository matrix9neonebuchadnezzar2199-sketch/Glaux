# Glaux release build — Glaux.exe + model/ のみ配布（ランタイムは EXE 内蔵）
# 前提: artifacts/ に llama-server + DLL、artifacts/model/ に E2B GGUF を配置済み

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$DistName = "Glaux"
$Server = Join-Path $Root "artifacts\llama-server.exe"
$ModelQwen3BQ3 = Join-Path $Root "artifacts\model\qwen2.5-3b-instruct-q3_k_m.gguf"
$ModelQwen3B = Join-Path $Root "artifacts\model\qwen2.5-3b-instruct-q2_k.gguf"
$ModelMini = Join-Path $Root "artifacts\model\RakutenAI-2.0-mini-instruct-Q4_K_M.gguf"
$ModelE2B = Join-Path $Root "artifacts\model\gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf"
$ModelE4B = Join-Path $Root "artifacts\model\gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf"

if (-not (Test-Path $Server)) {
    Write-Error "Missing $Server — .\scripts\bootstrap-runtime.ps1 を実行してください"
}
if (-not (Test-Path $ModelE2B) -and -not (Test-Path $ModelMini)) {
    Write-Error "Missing model GGUF — E2B または Rakuten mini を artifacts/model/ に配置してください"
}

Write-Host "Building Glaux release (embedded runtime)..."
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$exe = Join-Path $Root "target\release\glaux.exe"
if (-not (Test-Path $exe)) {
    Write-Error "Build succeeded but glaux.exe not found"
}

$dist = Join-Path $Root "dist\$DistName"
$modelDest = Join-Path $dist "model"

# 配布フォルダは丸ごと削除せず上書き更新（Glaux 起動中でも model/ を消さない）
New-Item -ItemType Directory -Force -Path $modelDest | Out-Null

Copy-Item -Force $exe (Join-Path $dist "Glaux.exe")

Write-Host "Copying models..."
# 旧 E2B（Q4）が残っていると配布サイズが膨らむため削除
$legacyE2B = Join-Path $modelDest "gemma-4-e2b-it.gguf"
if (Test-Path $legacyE2B) {
    Write-Host "  - removing legacy gemma-4-e2b-it.gguf"
    Remove-Item -Force $legacyE2B
}
if (Test-Path $ModelE2B) {
    Copy-Item -Force $ModelE2B (Join-Path $modelDest "gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf")
} else {
    Write-Host "  (E2B not found in artifacts/model — skipped)"
}
if (Test-Path $ModelQwen3BQ3) {
    Write-Host "  + Qwen 2.5 3B Q3"
    Copy-Item -Force $ModelQwen3BQ3 (Join-Path $modelDest "qwen2.5-3b-instruct-q3_k_m.gguf")
} else {
    Write-Host "  (Qwen 2.5 3B Q3 not found in artifacts/model — skipped)"
}
if (Test-Path $ModelQwen3B) {
    Write-Host "  + Qwen 2.5 3B"
    Copy-Item -Force $ModelQwen3B (Join-Path $modelDest "qwen2.5-3b-instruct-q2_k.gguf")
} else {
    Write-Host "  (Qwen 2.5 3B not found in artifacts/model — skipped)"
}
if (Test-Path $ModelMini) {
    Write-Host "  + Rakuten mini"
    Copy-Item -Force $ModelMini (Join-Path $modelDest "RakutenAI-2.0-mini-instruct-Q4_K_M.gguf")
} else {
    Write-Host "  (Rakuten mini not found in artifacts/model — skipped)"
}
if (Test-Path $ModelE4B) {
    Write-Host "  + E4B (optional)"
    Copy-Item -Force $ModelE4B (Join-Path $modelDest "gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf")
} else {
    Write-Host "  (E4B not found in artifacts/model — skipped)"
}

$readme = @"
Glaux -OFFLINE AI Chat-
=======================

ローカル完結型オフライン AI チャットです。

## 起動

Glaux.exe をダブルクリックしてください。

## フォルダ構成

Glaux.exe
model/       各種 GGUF（E4B は任意同梱）

llama-server と DLL は Glaux.exe に内蔵されています。
初回起動時に %LOCALAPPDATA%\Glaux\runtime\ へ自動展開されます（ユーザー操作不要）。

## モデル

- model/qwen2.5-3b-instruct-q3_k_m.gguf — Qwen 2.5 3B Instruct Q3（約 1.6GB、空きメモリ 3GB 以上推奨）
- model/qwen2.5-3b-instruct-q2_k.gguf — Qwen 2.5 3B Instruct Q2（約 1.3GB、空きメモリ 2.5GB 以上推奨）
- model/RakutenAI-2.0-mini-instruct-Q4_K_M.gguf — Rakuten AI 2.0 mini（約 1GB、空きメモリ 1.5GB 以上推奨）
- model/gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf — Gemma 4 E2B Unsloth QAT Q2（空きメモリ 3GB 以上推奨）
- model/gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf — Gemma 4 E4B Unsloth QAT（任意、6GB 以上推奨）

## 設定保存先

%LOCALAPPDATA%\Glaux\config.json
（チャット本文は保存しません）

## ライセンス

Gemma モデルは Google Gemma Terms of Use に従います。
"@
Set-Content -Path (Join-Path $dist "README.txt") -Value $readme -Encoding UTF8

$sizeMb = [math]::Round((Get-ChildItem -Recurse $dist | Measure-Object -Property Length -Sum).Sum / 1MB, 1)
Write-Host ""
Write-Host "OK: $dist ($sizeMb MB)"
Write-Host ""
Write-Host "検証手順:"
Write-Host "  1. dist\$DistName を別フォルダへコピー"
Write-Host "  2. Glaux.exe を起動（初回はランタイム展開あり）"
Write-Host "  3. チャット送信"
Write-Host "  4. 終了後 tasklist | findstr llama-server で子プロセス残存なし"

# Glaux runtime bootstrap
# Downloads llama.cpp Windows CPU binaries (win-cpu-x64 ONLY) and Gemma GGUF into artifacts/.
$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$Root = Split-Path -Parent $PSScriptRoot
$Artifacts = Join-Path $Root "artifacts"
$ModelDir = Join-Path $Artifacts "model"
$Cache = Join-Path $Root ".cache\bootstrap"

$LlamaZipUrl = "https://github.com/ggml-org/llama.cpp/releases/download/b9760/llama-b9760-bin-win-cpu-x64.zip"
$ModelMiniCpmUrl = "https://huggingface.co/Abiray/MiniCPM5-1B-GGUF/resolve/main/minicpm5-1b-Q8_0.gguf"
$ModelMiniCpmF16Url = "https://huggingface.co/Abiray/MiniCPM5-1B-GGUF/resolve/main/minicpm5-1b-f16.gguf"
$ModelQwen3BQ3Url = "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q3_k_m.gguf"
$ModelQwen3BUrl = "https://huggingface.co/Qwen/Qwen2.5-3B-Instruct-GGUF/resolve/main/qwen2.5-3b-instruct-q2_k.gguf"
$ModelMiniUrl = "https://huggingface.co/mmnga/RakutenAI-2.0-mini-instruct-gguf/resolve/main/RakutenAI-2.0-mini-instruct-Q4_K_M.gguf"
$ModelE2BUrl = "https://huggingface.co/erenyeager-1/gemma-4-E2B-it-qat-mobile-GGUF/resolve/main/gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf"
$ModelE4BUrl = "https://huggingface.co/unsloth/gemma-4-E4B-it-qat-GGUF/resolve/main/gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf"

New-Item -ItemType Directory -Force -Path $Artifacts, $ModelDir, $Cache | Out-Null

$LlamaZip = Join-Path $Cache "llama-b9760-bin-win-cpu-x64.zip"
$ExtractDir = Join-Path $Cache "llama-b9760-bin-win-cpu-x64"
$ModelMiniCpmOut = Join-Path $ModelDir "minicpm5-1b-Q8_0.gguf"
$ModelMiniCpmF16Out = Join-Path $ModelDir "minicpm5-1b-f16.gguf"
$ModelQwen3BQ3Out = Join-Path $ModelDir "qwen2.5-3b-instruct-q3_k_m.gguf"
$ModelQwen3BOut = Join-Path $ModelDir "qwen2.5-3b-instruct-q2_k.gguf"
$ModelMiniOut = Join-Path $ModelDir "RakutenAI-2.0-mini-instruct-Q4_K_M.gguf"
$ModelE2BOut = Join-Path $ModelDir "gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf"
$ModelE4BOut = Join-Path $ModelDir "gemma-4-E4B-it-qat-UD-Q4_K_XL.gguf"

if (-not (Test-Path $LlamaZip)) {
    Write-Host "Downloading llama.cpp CPU runtime..."
    curl.exe -L --fail --output $LlamaZip $LlamaZipUrl
}

if (-not (Test-Path $ExtractDir)) {
    Write-Host "Extracting llama.cpp runtime..."
    Expand-Archive -Force -Path $LlamaZip -DestinationPath $ExtractDir
}

$Server = Get-ChildItem -Path $ExtractDir -Recurse -Filter "llama-server.exe" | Select-Object -First 1
if (-not $Server) {
    throw "llama-server.exe was not found in $ExtractDir"
}

$RuntimeDir = Split-Path -Parent $Server.FullName
Write-Host "Copying llama.cpp runtime files to artifacts/..."
Get-ChildItem -Path $RuntimeDir -File | ForEach-Object {
    Copy-Item -Force $_.FullName (Join-Path $Artifacts $_.Name)
}

# llama-server.exe は VCRUNTIME140.dll / MSVCP140.dll に依存。未導入端末では stderr なしで 0xc0000005 即死する
$Sys32 = Join-Path $env:SystemRoot "System32"
$VcRedistDlls = @(
    "vcruntime140.dll",
    "vcruntime140_1.dll",
    "msvcp140.dll",
    "msvcp140_1.dll",
    "msvcp140_2.dll",
    "concrt140.dll"
)
Write-Host "Bundling MSVC runtime DLLs (portable)..."
foreach ($dll in $VcRedistDlls) {
    $src = Join-Path $Sys32 $dll
    if (Test-Path $src) {
        Copy-Item -Force $src (Join-Path $Artifacts $dll)
        Write-Host "  + $dll"
    } else {
        Write-Host "  (skip missing $dll)"
    }
}

if (-not (Test-Path $ModelMiniCpmOut)) {
    Write-Host "Downloading MiniCPM5 1B Q8_0 GGUF (~1.1 GB)..."
    curl.exe -L --fail --output $ModelMiniCpmOut $ModelMiniCpmUrl
}

if (-not (Test-Path $ModelMiniCpmF16Out)) {
    Write-Host "Downloading MiniCPM5 1B F16 GGUF (~2.1 GB)..."
    curl.exe -L --fail --output $ModelMiniCpmF16Out $ModelMiniCpmF16Url
}

if (-not (Test-Path $ModelQwen3BQ3Out)) {
    Write-Host "Downloading Qwen 2.5 3B Instruct Q3_K_M GGUF (~0.9 GB)..."
    curl.exe -L --fail --output $ModelQwen3BQ3Out $ModelQwen3BQ3Url
}

if (-not (Test-Path $ModelQwen3BOut)) {
    Write-Host "Downloading Qwen 2.5 3B Instruct Q2_K GGUF (~1.3 GB)..."
    curl.exe -L --fail --output $ModelQwen3BOut $ModelQwen3BUrl
}

if (-not (Test-Path $ModelMiniOut)) {
    Write-Host "Downloading Rakuten AI 2.0 mini Q4_K_M GGUF (~936 MB)..."
    curl.exe -L --fail --output $ModelMiniOut $ModelMiniUrl
}

if (-not (Test-Path $ModelE2BOut)) {
    Write-Host "Downloading Gemma 4 E2B QAT Q2 GGUF (~2.2 GB)..."
    curl.exe -L --fail --output $ModelE2BOut $ModelE2BUrl
}

if (-not (Test-Path $ModelE4BOut)) {
    Write-Host "Downloading Gemma 4 E4B GGUF (optional, ~4GB)..."
    curl.exe -L --fail --output $ModelE4BOut $ModelE4BUrl
}

Write-Host ""
Write-Host "Runtime ready:"
Write-Host "  $Artifacts\llama-server.exe (+ DLLs)"
if (Test-Path $ModelMiniCpmOut) {
    Write-Host "  $ModelMiniCpmOut"
}
if (Test-Path $ModelMiniCpmF16Out) {
    Write-Host "  $ModelMiniCpmF16Out"
}
if (Test-Path $ModelQwen3BQ3Out) {
    Write-Host "  $ModelQwen3BQ3Out"
}
if (Test-Path $ModelQwen3BOut) {
    Write-Host "  $ModelQwen3BOut"
}
if (Test-Path $ModelMiniOut) {
    Write-Host "  $ModelMiniOut"
}
Write-Host "  $ModelE2BOut"
if (Test-Path $ModelE4BOut) {
    Write-Host "  $ModelE4BOut"
}
Write-Host ""
Write-Host "Next:"
Write-Host "  cargo build --release"
Write-Host "  .\scripts\build-release.ps1"

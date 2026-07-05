# Glaux Runtime Notes

## パス解決（ポータブル配布）

```
{Glaux.exe のフォルダ}/
  runtime/
    llama-server.exe
    *.dll
  model/
    gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf
```

開発時は `artifacts/` にフォールバック。

## 設定保存先

```
%LOCALAPPDATA%\Glaux\config.json
```

チャット本文は保存しない。設定（テーマ、文字サイズ、モデル、temperature 等）のみ。

## ログ

```
%LOCALAPPDATA%\Glaux\logs\glaux-YYYYMMDD.log
```

## llama-server 起動

- **CPU 専用**: `-ngl 0` / `--no-op-offload` / `--device none`（GPU オフロード禁止）
- 同梱ランタイムは `llama-*-bin-win-cpu-x64.zip` のみ（`build.rs` が GPU DLL を拒否）
- バインド: `127.0.0.1` の空きポートのみ
- ヘルスチェック: `GET /health` または models エンドポイント
- 終了: アプリ終了時と「停止」ボタンで子プロセス kill
- コンテキスト長: 4000 固定を `-c` に渡す

## トラブルシュート

| 症状 | 確認 |
|------|------|
| 起動失敗 | `runtime/llama-server.exe` と DLL が同梱されているか |
| メモリ不足 | 他アプリ終了、コンテキスト長を下げる |
| モデル未配置 | `model/` に選択中 GGUF があるか |
| AV 隔離 | runtime DLL が削除されていないか |

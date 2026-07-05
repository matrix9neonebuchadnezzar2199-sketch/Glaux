# Glaux ドキュメント索引

ローカル完結型チャットボット **Glaux -OFFLINE AI Chat-**（Gemma 4 E2B 同梱・ポータブルフォルダ配布）の設計・実装・検証の正本入口。

## 設計

| 文書 | 内容 |
|------|------|
| [architecture.md](design/architecture.md) | 全体アーキテクチャ、UI/Runtime/llama-server 境界 |
| [model-selection.md](design/model-selection.md) | E2B 採用理由、量子化、メモリ前提 |
| [portable-packaging.md](design/portable-packaging.md) | ポータブルフォルダ配布、runtime/model 構成 |
| [single-exe-packaging.md](design/single-exe-packaging.md) | レガシー（v0.1 単独 EXE）参照 |
| [ui-spec.md](design/ui-spec.md) | 画面・Component Gallery 名・状態 |
| [context-policy.md](design/context-policy.md) | コンテキスト概算、クリーン、履歴非保存 |

## 運用

| 文書 | 内容 |
|------|------|
| [phase-plan.md](phase-plan.md) | Phase 0〜10 の目的・完了条件 |
| [milestones.md](milestones.md) | M0〜M10 マイルストーン記録 |
| [decision-log.md](decision-log.md) | 設計判断ログ |
| [runtime-notes.md](runtime-notes.md) | パス解決、設定、ログ、トラブルシュート |

## 検証

| 文書 | 内容 |
|------|------|
| [user-stories/README.md](user-stories/README.md) | All_Status 列定義・再実行手順 |
| [user-stories/user-stories-matrix.csv](user-stories/user-stories-matrix.csv) | 全機能ストーリー × ステータス |

## ビルド

```powershell
.\scripts\bootstrap-runtime.ps1
.\scripts\build-release.ps1
```

詳細は [portable-packaging.md](design/portable-packaging.md) 参照。

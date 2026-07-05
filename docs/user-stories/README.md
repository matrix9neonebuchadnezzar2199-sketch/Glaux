# Glaux User Stories (All_Status)

## 列定義 (`user-stories-matrix.csv`)

| 列 | 説明 |
|----|------|
| `story_id` | `{EPIC}-{NNN}` 形式（例: `CHT-001`） |
| `epic` | 機能グループ |
| `feature` | 機能名 |
| `route` | 画面・操作箇所 |
| `persona` | 利用者種別 |
| `user_story` | ユーザーストーリー本文 |
| `precondition` | 前提条件 |
| `steps` | 操作手順（番号付き） |
| `expected_behavior` | 検証可能な期待結果 |
| `apis` | 関連 API（該当時） |
| `impl_status` | `implemented` / `partial` / `planned` / `n/a` |
| `test_method` | `api` / `unit` / `build` / `docker` / `manual` |
| `milestone` | 対応マイルストーン |
| `notes` | 補足 |
| `last_run` | 最終テスト実行時刻 |
| `test_result` | `pass` / `fail` / `skip` / `pending` |
| `evidence` | ログ・コマンド出力 |

## 正本

- ストーリー定義: `scripts/user-story-catalog.mjs`
- マトリクス生成: `scripts/run-user-story-tests.mjs`

## 再実行手順

```powershell
cd F:\Cursor\Glaux

# マトリクスのみ再生成
node scripts/run-user-story-tests.mjs --matrix-only

# build テスト込み（fail > 0 で exit 1）
node scripts/run-user-story-tests.mjs
```

手動ストーリー（`manual`）は実機で `dist/Glaux -OFFLINE AI Chat-/` を使い検証する。artifacts 未配置の開発ビルドでは RUN/MDL/CHT の多くは skip となる。

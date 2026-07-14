/**
 * Glaux 全機能ユーザーストーリー定義（正本）。
 * `node scripts/run-user-story-tests.mjs` が matrix / test-results を docs/user-stories/ に出力する。
 */

/** @typedef {'implemented'|'partial'|'planned'|'n/a'} ImplStatus */
/** @typedef {'api'|'unit'|'build'|'docker'|'manual'} TestMethod */

/**
 * @typedef {object} UserStory
 * @property {string} id
 * @property {string} epic
 * @property {string} feature
 * @property {string} route
 * @property {string} persona
 * @property {string} userStory
 * @property {string} precondition
 * @property {string} steps
 * @property {string} expected
 * @property {string} apis
 * @property {ImplStatus} implStatus
 * @property {TestMethod} testMethod
 * @property {string} [milestone]
 * @property {string} [notes]
 */

/** @type {UserStory[]} */
export const USER_STORIES = [
  {
    id: 'RUN-001',
    epic: 'ランタイム',
    feature: 'ポータブル配布起動',
    route: '(startup)',
    persona: 'user',
    userStory: '利用者として Glaux.exe 起動後に選択中モデルが自動で読み込まれることを期待する',
    precondition: 'dist/Glaux/ 配置済み',
    steps: '1. 配布フォルダをコピー 2. Glaux.exe 起動 3. 起動中→起動完了を待つ',
    expected: 'runtime/llama-server.exe と選択中 model/*.gguf で推論が自動開始される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M2'
  },
  {
    id: 'RUN-002',
    epic: 'ランタイム',
    feature: 'ポータブルパス解決',
    route: '(startup)',
    persona: 'user',
    userStory: '利用者として Glaux.exe と model/ だけの配布フォルダで起動したい',
    precondition: 'dist/Glaux/ 配置済み',
    steps: '1. Glaux.exe 起動 2. model/ の存在を確認 3. 起動',
    expected: 'EXE 内蔵ランタイムが展開され、選択中 model/*.gguf で推論が開始される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M2'
  },
  {
    id: 'RUN-003',
    epic: 'ランタイム',
    feature: '子プロセス終了',
    route: '(exit)',
    persona: 'user',
    userStory: '利用者としてアプリ終了後に llama-server が残らないことを期待する',
    precondition: 'モデル起動済み',
    steps: '1. モデル起動 2. Glaux を終了 3. tasklist で llama-server を確認',
    expected: 'llama-server プロセスが残存しない',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M2'
  },
  {
    id: 'MDL-001',
    epic: 'モデル',
    feature: '起動ボタン',
    route: '(model bar)',
    persona: 'user',
    userStory: '利用者として停止状態から選択中モデルを起動したい',
    precondition: 'アセット展開可能',
    steps: '1. 起動ボタン 2. 状態が起動中→起動完了へ遷移',
    expected: '選択モデル名 — 起動完了 と表示',
    apis: 'GET /health or /v1/models',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M5'
  },
  {
    id: 'MDL-002',
    epic: 'モデル',
    feature: '停止ボタン',
    route: '(model bar)',
    persona: 'user',
    userStory: '利用者としてモデルを停止したい',
    precondition: '起動完了',
    steps: '1. 停止ボタン 2. 状態が停止へ',
    expected: '送信・要約が無効化される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M5'
  },
  {
    id: 'MDL-003',
    epic: 'モデル',
    feature: '解説ウィンドウ',
    route: '(model bar)',
    persona: 'user',
    userStory: '利用者として選択中モデルのオフライン説明を読みたい',
    precondition: 'なし',
    steps: '1. 解説ボタン 2. ウィンドウ表示',
    expected: '外部リンクなしで日本語説明が表示される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M5'
  },
  {
    id: 'MDL-004',
    epic: 'モデル',
    feature: 'モデル選択',
    route: '(settings)',
    persona: 'user',
    userStory: '利用者として設定画面で同梱モデル情報を確認したい',
    precondition: 'model/ に E2B GGUF 配置済み',
    steps: '1. 設定を開く 2. 使用モデルに E2B が表示される 3. ファイル名を確認',
    expected: 'Google Gemma 4 E2B (Unsloth QAT Q2) と gemma-4-E2B-it-qat-UD-Q2_K_XL.gguf が表示される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M5'
  },
  {
    id: 'CHT-001',
    epic: 'チャット',
    feature: '送信とストリーム',
    route: '(chat)',
    persona: 'user',
    userStory: '利用者としてメッセージを送信しストリーミング応答を見たい',
    precondition: 'モデル起動完了',
    steps: '1. 入力 2. 送信 3. 応答が逐次表示',
    expected: 'assistant メッセージが stream で更新される',
    apis: 'POST /v1/chat/completions',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M4'
  },
  {
    id: 'CHT-002',
    epic: 'チャット',
    feature: '履歴非保存',
    route: '(chat)',
    persona: 'user',
    userStory: '利用者として会話がディスクに保存されないことを期待する',
    precondition: 'チャット送信済み',
    steps: '1. 送信 2. AppData/Glaux を検索 3. 再起動',
    expected: '会話 DB/履歴ファイルが無く、再起動で会話が復元されない',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M4'
  },
  {
    id: 'CHT-003',
    epic: 'チャット',
    feature: 'クリーン',
    route: '(chat)',
    persona: 'user',
    userStory: '利用者としてメモリ上の会話を初期化したい',
    precondition: '会話あり',
    steps: '1. クリーンボタン',
    expected: 'Message list が空になり Context 概算が 0 付近',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M4'
  },
  {
    id: 'CHT-004',
    epic: 'チャット',
    feature: 'コピー',
    route: '(chat)',
    persona: 'user',
    userStory: '利用者として応答をクリップボードにコピーしたい',
    precondition: 'assistant 応答あり',
    steps: '1. コピーをクリック',
    expected: 'クリップボードに応答本文が入りトーストが表示される',
    apis: 'POST /v1/chat/completions',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M4'
  },
  {
    id: 'CTX-001',
    epic: 'コンテキスト',
    feature: '概算表示',
    route: '(model bar)',
    persona: 'user',
    userStory: '利用者として現在のコンテキスト使用量の概算を見たい',
    precondition: '会話あり',
    steps: '1. メッセージ追加 2. Context 表示を確認',
    expected: '約 N / 3000 · メモリ 約 N MB 形式で更新される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M4'
  },
  {
    id: 'SET-001',
    epic: '設定',
    feature: '設定保存',
    route: '(settings)',
    persona: 'user',
    userStory: '利用者としてテーマ・文字サイズを保存したい',
    precondition: 'なし',
    steps: '1. 設定を開く 2. テーマ・文字サイズを変更 3. 保存',
    expected: '%LOCALAPPDATA%/Glaux/config.json に反映（チャット本文は含まない）',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M7'
  },
  {
    id: 'SET-002',
    epic: '設定',
    feature: '設定 UI',
    route: '(settings)',
    persona: 'user',
    userStory: '利用者として外観を設定画面から変更したい',
    precondition: 'なし',
    steps: '1. 設定を開く 2. テーマ・文字サイズを変更 3. 保存',
    expected: '整列された設定 UI で theme/font_size が保存される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M7'
  },
  {
    id: 'HLP-001',
    epic: 'ヘルプ',
    feature: 'Help Drawer',
    route: '(help)',
    persona: 'user',
    userStory: '利用者として履歴非保存ポリシーをオフラインで読みたい',
    precondition: 'なし',
    steps: '1. 本アイコン 2. Help 表示',
    expected: 'メモリ上のみ・クリーン・概算の説明が表示',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'manual',
    milestone: 'M8'
  },
  {
    id: 'BLD-001',
    epic: 'ビルド',
    feature: 'release ビルド',
    route: 'n/a',
    persona: 'developer',
    userStory: '開発者として release ビルドが通ることを確認したい',
    precondition: 'Rust toolchain',
    steps: '1. cargo build --release',
    expected: 'target/release/glaux.exe が生成される',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'build',
    milestone: 'M1'
  },
  {
    id: 'BLD-002',
    epic: 'ビルド',
    feature: 'ポータブル release',
    route: 'n/a',
    persona: 'developer',
    userStory: '開発者として artifacts 配置後にポータブル配布フォルダを dist へ出力したい',
    precondition: 'artifacts 配置済み（E2B）',
    steps: '1. scripts/build-release.ps1',
    expected: 'dist/Glaux/ に Glaux.exe と model/ が生成される（runtime は EXE 内蔵）',
    apis: 'n/a',
    implStatus: 'implemented',
    testMethod: 'build',
    milestone: 'M9'
  }
]

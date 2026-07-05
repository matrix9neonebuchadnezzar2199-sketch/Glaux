#!/usr/bin/env node
/**
 * Glaux ユーザーストーリー matrix 生成 + build テスト
 *
 * Usage:
 *   node scripts/run-user-story-tests.mjs
 *   node scripts/run-user-story-tests.mjs --matrix-only
 */
import { spawnSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { USER_STORIES } from './user-story-catalog.mjs'

const __dirname = path.dirname(fileURLToPath(import.meta.url))
const ROOT = path.join(__dirname, '..')
const OUT_DIR = path.join(ROOT, 'docs', 'user-stories')

const args = new Set(process.argv.slice(2))
const matrixOnly = args.has('--matrix-only')

const RUN_ID = new Date().toISOString().slice(0, 19).replace('T', ' ')
const RUN_DATE = new Date().toISOString().slice(0, 10)

/** @type {Map<string, {result: string, method: string, evidence: string, notes: string}>} */
const results = new Map()

function csvEscape (value) {
  const s = String(value ?? '')
  if (/[",\n\r]/.test(s)) return `"${s.replace(/"/g, '""')}"`
  return s
}

function writeCsv (filename, headers, rows) {
  const lines = [
    headers.map(csvEscape).join(','),
    ...rows.map((row) => row.map(csvEscape).join(','))
  ]
  const bom = '\uFEFF'
  fs.writeFileSync(path.join(OUT_DIR, filename), bom + lines.join('\r\n') + '\r\n', 'utf8')
}

function setResult (id, result, method, evidence, notes = '') {
  results.set(id, { result, method, evidence, notes })
}

function runBuild () {
  const r = spawnSync('cargo', ['build', '--release'], {
    cwd: ROOT,
    encoding: 'utf8',
    shell: true
  })
  const ok = r.status === 0
  const evidence = ok ? 'cargo build --release exit 0' : (r.stderr || r.stdout || '').slice(0, 500)
  return { ok, evidence }
}

function main () {
  fs.mkdirSync(OUT_DIR, { recursive: true })

  if (!matrixOnly) {
    const { ok, evidence } = runBuild()
    for (const s of USER_STORIES) {
      if (s.testMethod === 'build') {
        setResult(s.id, ok ? 'pass' : 'fail', 'build', evidence)
      } else if (s.testMethod === 'manual') {
        setResult(s.id, 'skip', 'manual', 'requires GUI / runtime assets', '手動検証')
      } else {
        setResult(s.id, 'skip', s.testMethod, 'not automated in v1', '')
      }
    }
  }

  const matrixHeaders = [
    'story_id', 'epic', 'feature', 'route', 'persona', 'user_story',
    'precondition', 'steps', 'expected_behavior', 'apis',
    'impl_status', 'test_method', 'milestone', 'notes',
    'last_run', 'test_result', 'evidence'
  ]

  const matrixRows = USER_STORIES.map((s) => {
    const r = results.get(s.id)
    return [
      s.id, s.epic, s.feature, s.route, s.persona, s.userStory,
      s.precondition, s.steps, s.expected, s.apis,
      s.implStatus, s.testMethod, s.milestone ?? '', s.notes ?? '',
      matrixOnly ? '' : RUN_ID,
      r?.result ?? (matrixOnly ? '' : 'pending'),
      r?.evidence ?? ''
    ]
  })

  writeCsv('user-stories-matrix.csv', matrixHeaders, matrixRows)

  const epicCounts = {}
  let pass = 0
  let fail = 0
  let skip = 0
  for (const s of USER_STORIES) {
    epicCounts[s.epic] = (epicCounts[s.epic] ?? 0) + 1
    const r = results.get(s.id)?.result
    if (r === 'pass') pass++
    else if (r === 'fail') fail++
    else skip++
  }

  const summary = [
    `# Glaux User Story Test Summary`,
    ``,
    `- Run: ${RUN_ID}`,
    `- Date: ${RUN_DATE}`,
    `- Total: ${USER_STORIES.length}`,
    `- pass: ${pass} / fail: ${fail} / skip: ${skip}`,
    ``,
    `## Epic counts`,
    ...Object.entries(epicCounts).map(([k, v]) => `- ${k}: ${v}`),
    ``
  ].join('\n')

  fs.writeFileSync(path.join(OUT_DIR, 'summary.md'), summary, 'utf8')
  console.log(summary)

  if (fail > 0) process.exit(1)
}

main()

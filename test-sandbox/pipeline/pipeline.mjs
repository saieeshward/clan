#!/usr/bin/env node
// CLAN progress pipeline — one entry point for every claim we make.
//
//   node pipeline.mjs --clan <binary> [--stages 0,1,3] [--metrics <metrics-lite.json>]
//
// Stages:
//   0  cargo workspace tests            (deterministic, ~minutes, free)
//   1  conformance.mjs                  (deterministic, ~seconds, free)
//   2  agentic lite benchmark           (NOT run by this script — run the
//      run-h1-h2.workflow.js flows with real subagents, produce
//      metrics-lite.json, then pass it via --metrics)
//   3  scorecard: claims.yaml + stage outputs -> scorecard.{json,md},
//      append history.jsonl, diff vs previous run
//
// Default stages: 0,1,3.

import { spawnSync } from 'node:child_process'
import { readFileSync, writeFileSync, existsSync, mkdirSync, appendFileSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
const REPO = join(HERE, '..', '..')
const args = process.argv.slice(2)
const argOf = (f, d) => { const i = args.indexOf(f); return i >= 0 ? args[i + 1] : d }
const CLAN = argOf('--clan', process.platform === 'win32' ? join(REPO, 'target', 'release', 'clan.exe') : join(REPO, 'target', 'release', 'clan'))
const STAGES = argOf('--stages', '0,1,3').split(',').map(s => s.trim())
const METRICS = argOf('--metrics', join(HERE, '..', 'happy', 'lite', 'metrics-lite.json'))
const RUN_ID = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
const OUT = join(HERE, 'results', RUN_ID)
mkdirSync(OUT, { recursive: true })

const stageResults = { run_id: RUN_ID }

// ---------- stage 0: cargo tests ----------
if (STAGES.includes('0')) {
  console.log('\n=== stage 0: cargo test ===')
  const r = spawnSync('cargo', ['test', '--workspace', '--quiet'], { cwd: REPO, encoding: 'utf8', timeout: 600000 })
  const out = (r.stdout || '') + (r.stderr || '')
  const m = [...out.matchAll(/(\d+) passed; (\d+) failed/g)]
  const passed = m.reduce((a, x) => a + +x[1], 0), failed = m.reduce((a, x) => a + +x[2], 0)
  stageResults.cargo = { exit: r.status, passed, failed }
  writeFileSync(join(OUT, 'cargo.log'), out)
  console.log(`cargo: ${passed} passed, ${failed} failed (exit ${r.status})`)
}

// ---------- stage 1: conformance ----------
if (STAGES.includes('1')) {
  console.log('\n=== stage 1: conformance ===')
  const r = spawnSync(process.execPath, [join(HERE, 'conformance.mjs'), '--clan', CLAN, '--out', OUT], { encoding: 'utf8', stdio: 'inherit', timeout: 300000 })
  stageResults.conformance = JSON.parse(readFileSync(join(OUT, 'conformance.json'), 'utf8'))
}

// ---------- stage 2 ingest: agentic metrics ----------
function computeAgenticMetrics(m) {
  const sum = a => a.reduce((x, y) => x + y, 0)
  const out = {}
  try {
    const clanChars = sum(m.h1.clan.map(e => e.authored_chars))
    const adhocChars = sum(m.h1.adhoc.map(e => e.authored_chars))
    out.h1_output_ratio = +(clanChars / adhocChars).toFixed(3)
    out.h1_clan_chars = clanChars; out.h1_adhoc_chars = adhocChars
  } catch { out.h1_output_ratio = null }
  try {
    const hops = m.h1.clan.length
    out.h1_chain_entries_vs_hops = +((m.h1_clan_final.n_decisions || 0) / hops).toFixed(3)
    out.h1_validates = !!m.h1_clan_final.validates
  } catch { out.h1_chain_entries_vs_hops = null }
  try {
    const c = m.h2.clan, a = m.h2.adhoc
    let cross = null
    // exclude the final synthesis hop — CLAN's win there is C-SYNTH-WIN, not the crossover claim
    for (let i = 0; i < Math.min(c.length, a.length) - 1; i++) {
      const ci = c[i].injected_skipguide ?? c[i].injected_full
      if (ci < a[i].injected_full) { cross = i; break }
    }
    out.h2_crossover_hop = cross
    const cl = c[c.length - 1], al = a[a.length - 1]
    out.h2_synth_ratio = +(((cl.injected_skipguide ?? cl.injected_full)) / al.injected_full).toFixed(3)
  } catch { out.h2_crossover_hop = null; out.h2_synth_ratio = null }
  try {
    out.unrecovered_failures = m.receipts.filter(r => r.n_errors > 0 && /unrecovered|gave up|failed permanently/i.test(r.friction || '')).length
  } catch { out.unrecovered_failures = null }
  return out
}
if (existsSync(METRICS)) {
  const raw = readFileSync(METRICS)
  const m = JSON.parse(raw.toString('utf8').replace(/^﻿/, ''))
  stageResults.agentic = computeAgenticMetrics(m)
  stageResults.agentic_source = METRICS
  console.log('\n=== stage 2 (ingested): agentic metrics ===\n', stageResults.agentic)
} else {
  console.log(`\n=== stage 2: no metrics file at ${METRICS} — agentic claims will be marked 'stale/missing' ===`)
}

// ---------- stage 3: scorecard ----------
if (STAGES.includes('3')) {
  console.log('\n=== stage 3: scorecard ===')
  // minimal YAML reader for claims.yaml (flat list-of-maps subset)
  const yaml = readFileSync(join(HERE, 'claims.yaml'), 'utf8')
  const claims = []
  let cur = null
  for (const line of yaml.split('\n')) {
    if (/^\s{2}- id:/.test(line)) { cur = { id: line.split(':')[1].trim() }; claims.push(cur); continue }
    const m = line.match(/^\s{4}(\w+):\s*(.+?)\s*(#.*)?$/)
    if (cur && m) cur[m[1]] = m[2].replace(/^["']|["']$/g, '')
  }

  const conf = stageResults.conformance
  const ag = stageResults.agentic
  const testPass = id => conf?.tests?.find(t => t.id.startsWith(id))?.pass ?? conf?.tests?.filter(t => t.id.startsWith(id)).every(t => t.pass)
  const evalThreshold = (value, threshold) => {
    if (value === null || value === undefined) return 'missing'
    const m = threshold.match(/(<=|>=|<|>|==)\s*([\d.]+)/)
    if (!m) return 'unknown'
    const [, op, n] = m; const v = +value, t = +n
    const ok = op === '<=' ? v <= t : op === '>=' ? v >= t : op === '<' ? v < t : op === '>' ? v > t : v === t
    return ok ? 'pass' : 'fail'
  }

  const rows = claims.map(c => {
    let status = 'manual', value = ''
    if (c.stage === 'conformance' && conf) {
      const ids = (c.tests || '').replace(/[[\]]/g, '').split(',').map(s => s.trim()).filter(Boolean)
      const fails = ids.filter(id => conf.tests.filter(t => t.id.startsWith(id)).some(t => !t.pass))
      status = fails.length === 0 ? 'pass' : 'fail'
      value = fails.length ? `failing: ${fails.join(' ')}` : `${ids.length} tests green`
    } else if (c.stage === 'agentic') {
      if (!ag) { status = 'missing'; value = 'no metrics-lite.json' }
      else {
        value = ag[c.metric]
        if (c.metric === 'h1_fidelity') { value = ag.h1_validates ? 1.0 : 0.0 } // edit-presence check lives in the workflow's Verify phase
        status = evalThreshold(value, c.threshold || '')
      }
    } else if (c.stage === 'planned') { status = 'planned' }
    if (String(c.expect || '').startsWith('red') && status === 'fail') status = 'known-red'
    return { id: c.id, claim: c.claim, stage: c.stage, value: String(value ?? ''), threshold: c.threshold || '', status }
  })

  const scorecard = {
    run_id: RUN_ID, ran_at: new Date().toISOString(),
    clan_version: conf?.clan_version || '',
    cargo: stageResults.cargo || null,
    conformance: conf ? { passed: conf.passed, total: conf.total, hard_failures: conf.hard_failures, unexpected_green: conf.unexpected_green } : null,
    agentic: ag || null,
    claims: rows,
  }
  writeFileSync(join(OUT, 'scorecard.json'), JSON.stringify(scorecard, null, 2))

  // history + regression diff
  const histPath = join(HERE, 'history.jsonl')
  let prev = null
  if (existsSync(histPath)) {
    const lines = readFileSync(histPath, 'utf8').trim().split('\n')
    if (lines.length) prev = JSON.parse(lines[lines.length - 1])
  }
  const regressions = prev ? rows.filter(r => {
    const p = prev.claims?.find(x => x.id === r.id)
    return p && p.status === 'pass' && (r.status === 'fail' || r.status === 'known-red')
  }).map(r => r.id) : []
  scorecard.regressions_vs_previous = regressions
  appendFileSync(histPath, JSON.stringify(scorecard) + '\n')

  // human-readable scorecard
  const icon = s => ({ pass: 'PASS', fail: 'FAIL', 'known-red': 'KNOWN-RED', missing: 'NO-DATA', planned: 'PLANNED', manual: 'MANUAL' }[s] || s)
  const md = [
    `# CLAN scorecard — ${RUN_ID}`, '',
    `CLI: ${scorecard.clan_version} · cargo: ${stageResults.cargo ? `${stageResults.cargo.passed}p/${stageResults.cargo.failed}f` : 'skipped'} · conformance: ${conf ? `${conf.passed}/${conf.total}` : 'skipped'}`, '',
    '| Claim | Status | Value | Threshold |', '|---|---|---|---|',
    ...rows.map(r => `| **${r.id}** ${r.claim.slice(0, 80)} | ${icon(r.status)} | ${r.value} | ${r.threshold} |`),
    '',
    regressions.length ? `**REGRESSIONS vs previous run: ${regressions.join(', ')}**` : 'No regressions vs previous run.',
    conf?.unexpected_green?.length ? `Expect-red now GREEN (update claims.yaml): ${conf.unexpected_green.join(', ')}` : '',
  ].join('\n')
  writeFileSync(join(OUT, 'scorecard.md'), md)
  console.log(md)
  console.log(`\nResults in ${OUT}`)

  const hard = (stageResults.cargo?.failed || 0) + (conf?.hard_failures || 0) + regressions.length
  process.exit(hard > 0 ? 1 : 0)
}

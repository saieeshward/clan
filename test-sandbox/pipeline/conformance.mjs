#!/usr/bin/env node
// CLAN conformance & regression stage (deterministic, no LLM).
// Black-box assertions against the CLI binary covering core lifecycle and
// every fixed finding (F1..F15). Emits conformance.json for the scorecard.
//
//   node conformance.mjs --clan <path-to-clan-binary> [--out <dir>]
//
// Exit code: number of HARD failures among tests not marked expect-red.

import { spawnSync } from 'node:child_process'
import { mkdtempSync, writeFileSync, readFileSync, existsSync, mkdirSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'

const args = process.argv.slice(2)
function argOf(flag, dflt) { const i = args.indexOf(flag); return i >= 0 ? args[i + 1] : dflt }
const CLAN = argOf('--clan', process.platform === 'win32' ? 'clan.exe' : 'clan')
const OUT = argOf('--out', '.')

const work = mkdtempSync(join(tmpdir(), 'clan-conf-'))
const results = []
let counter = 0

function clan(cliArgs, opts = {}) {
  const r = spawnSync(CLAN, cliArgs, {
    cwd: work, encoding: 'utf8', input: opts.stdin,
    env: { ...process.env, CLAN_NO_HINTS: opts.hints ? '' : '1' },
    timeout: 30000,
  })
  return { code: r.status ?? -1, out: (r.stdout || '') + (r.stderr || ''), stdout: r.stdout || '', stderr: r.stderr || '' }
}
function tmpJson(obj) { const p = join(work, `p${counter++}.json`); writeFileSync(p, JSON.stringify(obj)); return p }
function record(id, name, pass, detail, expectRed = false) {
  results.push({ id, name, pass: !!pass, expect_red: expectRed, detail: pass ? '' : String(detail).slice(0, 400) })
  const mark = pass ? 'PASS' : (expectRed ? 'RED (expected)' : 'FAIL')
  console.log(`${id} ${mark}  ${name}${pass ? '' : ` :: ${String(detail).slice(0, 160)}`}`)
}
function exportStatic(file) {
  const out = join(work, `exp${counter++}.json`)
  const r = clan(['export-static', '--output', out, file])
  if (r.code !== 0) return { err: r.out }
  try { return { json: JSON.parse(readFileSync(out, 'utf8')) } } catch (e) { return { err: String(e) } }
}
const ATTR = ['--agent', 'conformance-bot', '--action', 'conformance check']

// ---------- T01 create + validate + info ----------
let r = clan(['create', '--title', 'Pipeline Doc', '--brief', 'Conformance test document', '--output', 'doc.clan'])
record('T01a', 'create --output', r.code === 0 && existsSync(join(work, 'doc.clan')), r.out)
r = clan(['validate', 'doc.clan'])
record('T01b', 'validate fresh file', r.code === 0 && /OK/i.test(r.out), r.out)
r = clan(['info', 'doc.clan'])
record('T01c', 'info', r.code === 0 && /Pipeline Doc/.test(r.out), r.out)

// ---------- T02 create --schema seeds real schema (F9) ----------
const schema = tmpJson({ type: 'object', properties: { mode: { type: 'string' }, structured: { type: 'object', properties: { price: { type: 'number' } } } } })
r = clan(['create', '--title', 'Schema Doc', '--brief', 'b', '--schema', schema, '--output', 'sdoc.clan'])
{
  const e = exportStatic('sdoc.clan')
  record('T02', 'create --schema seeds schema (F9)', r.code === 0 && e.json && JSON.stringify(e.json.output_schema || e.json).includes('price'), e.err || r.out)
}

// ---------- T03 patch-data --set inline scalar (F13) ----------
r = clan(['patch-data', 'doc.clan', '--set', 'price=42', ...ATTR])
{
  const d = clan(['read', 'data', 'doc.clan'])
  record('T03', 'patch-data --set key=value (F13)', r.code === 0 && /price:\s*42/.test(d.out), r.out + d.out)
}

// ---------- T04 patch-data inline JSON string (F13) ----------
r = clan(['patch-data', 'doc.clan', '{"vendor":"Zoho"}', ...ATTR])
{
  const d = clan(['read', 'data', 'doc.clan'])
  record('T04', 'patch-data inline JSON (F13)', r.code === 0 && /vendor:\s*Zoho/.test(d.out), r.out + d.out)
}

// ---------- T05 mutation without attribution is rejected (F15) ----------
r = clan(['patch-data', 'doc.clan', '{"x":1}'])
record('T05', 'patch-data w/o --agent/--action rejected (F15)', r.code !== 0 && /(--agent|--action|--no-decision)/.test(r.out), `code=${r.code} ${r.out}`)

// ---------- T06 --no-decision adds no chain entry; F1: no unknown-agent ----------
{
  const before = (clan(['read', 'chain', 'doc.clan']).out.match(/- agent:/g) || []).length
  r = clan(['patch-data', 'doc.clan', '{"y":2}', '--no-decision'])
  const after = clan(['read', 'chain', 'doc.clan'])
  const n = (after.out.match(/- agent:/g) || []).length
  record('T06', '--no-decision: no entry, no unknown-agent (F1/F15)', r.code === 0 && n === before && !/unknown-agent/.test(after.out), `before=${before} after=${n} ${r.out}`)
}

// ---------- T07 attributed mutation lands in chain with fields_changed ----------
r = clan(['patch-data', 'doc.clan', '{"budget_eur":40000}', '--agent', 'finance-bot', '--action', 'set budget cap'])
{
  const c = clan(['read', 'chain', 'doc.clan'])
  record('T07', 'attributed patch -> chain entry + fields_changed (F15)', r.code === 0 && /finance-bot/.test(c.out) && /budget_eur/.test(c.out), r.out + c.out.slice(0, 300))
}

// ---------- T08 --append concatenates arrays (F14) ----------
clan(['patch-data', 'doc.clan', tmpJson({ risks: [{ risk: 'one', severity: 'High' }] }), ...ATTR])
r = clan(['patch-data', 'doc.clan', tmpJson({ risks: [{ risk: 'two', severity: 'Low' }] }), '--append', 'risks', ...ATTR])
{
  const e = exportStatic('doc.clan')
  const risks = e.json?.shared_data?.risks ?? e.json?.data?.risks
  record('T08', 'patch-data --append grows array (F14)', r.code === 0 && Array.isArray(risks) && risks.length === 2, e.err || `len=${risks?.length} ${r.out}`)
}

// ---------- T09 UTF-8 BOM tolerated (F3) ----------
{
  const p = join(work, 'bom.json'); writeFileSync(p, '﻿{"bom_ok":true}')
  r = clan(['patch-data', 'doc.clan', p, ...ATTR])
  record('T09', 'BOM-prefixed JSON accepted (F3)', r.code === 0, r.out)
}

// ---------- T10 read agent --skip-guide is materially smaller ----------
{
  const full = clan(['read', 'agent', 'doc.clan'])
  const skip = clan(['read', 'agent', 'doc.clan', '--skip-guide'])
  record('T10', 'skip-guide saves >1k chars', full.code === 0 && skip.code === 0 && full.stdout.length - skip.stdout.length > 1000, `full=${full.stdout.length} skip=${skip.stdout.length}`)
}

// ---------- T11/T12 patch-html: bad selector fails loudly; good patch applies ----------
{
  const frag = join(work, 'frag.html')
  writeFileSync(frag, '---\nmode: patch-html\npatch_selector: "#does-not-exist-xyz"\npatch_action: append\n---\n<p>x</p>')
  r = clan(['patch-html', 'doc.clan', frag, ...ATTR])
  record('T11', 'patch-html non-matching selector exits non-zero', r.code !== 0, `code=${r.code} ${r.out}`)
  writeFileSync(frag, '---\nmode: patch-html\npatch_selector: "body"\npatch_action: append\n---\n<p id="conf-frag">appended-by-conformance</p>')
  r = clan(['patch-html', 'doc.clan', frag, ...ATTR])
  const h = clan(['read', 'human', 'doc.clan'])
  record('T12', 'patch-html append applies', r.code === 0 && /appended-by-conformance/.test(h.out), r.out)
}

// ---------- T13–T17 fork / namespace guard / merge / conflict report / policy ----------
mkdirSync(join(work, 'branches'), { recursive: true })
r = clan(['fork', 'doc.clan', '--agents', 'alpha,beta', '--output-dir', 'branches'])
const bA = join('branches', 'alpha.clan'), bB = join('branches', 'beta.clan')
record('T13', 'fork creates branch per agent', r.code === 0 && existsSync(join(work, bA)) && existsSync(join(work, bB)), r.out)

r = clan(['patch-data', bA, '{"direct":1}', ...ATTR])
record('T14', 'branch rejects direct shared write (guard)', r.code !== 0 && /namespace/i.test(r.out), `code=${r.code} ${r.out}`)

r = clan(['patch-data', bA, '{"verdict":"HubSpot","alpha_score":9}', '--namespace', ...ATTR])
const r2 = clan(['patch-data', bB, '{"verdict":"Zoho","beta_notes":["n1"]}', '--namespace', ...ATTR])
record('T15', 'namespace writes accepted on branches', r.code === 0 && r2.code === 0, r.out + r2.out)

r = clan(['merge', bA, bB, '--output', 'merged.clan'])
{
  const rep = clan(['read', 'report', 'merged.clan'])
  record('T16', 'merge detects contested key with provenance (verdict)', r.code === 0 && /verdict/.test(rep.out) && /(alpha|beta)/.test(rep.out), r.out + rep.out.slice(0, 300))
}

clan(['patch-data', bA, '{"notes":["a"]}', '--namespace', ...ATTR])
clan(['patch-data', bB, '{"notes":["b"]}', '--namespace', ...ATTR])
r = clan(['merge', bA, bB, '--output', 'merged2.clan', '--policy', 'notes=append'])
{
  const e = exportStatic('merged2.clan')
  const notes = e.json?.shared_data?.notes ?? e.json?.data?.notes
  record('T17', 'merge --policy notes=append folds both', r.code === 0 && Array.isArray(notes) && notes.length === 2, e.err || `notes=${JSON.stringify(notes)} ${r.out}`)
}

// ---------- T18 patch-decision appends pinned entry ----------
r = clan(['patch-decision', 'doc.clan', '--agent', 'lead', '--action', 'signed off', '--rationale', 'conformance', '--pinned'])
{
  const c = clan(['read', 'chain', 'doc.clan'])
  record('T18', 'patch-decision appends pinned entry', r.code === 0 && /signed off/.test(c.out) && /pinned: true/.test(c.out), r.out)
}

// ---------- T19 no-render -> render materialises view ----------
r = clan(['create', '--title', 'AgentOnly', '--brief', 'b', '--no-render', '--output', 'ao.clan'])
{
  const rr = clan(['render', 'ao.clan'])
  const h = clan(['read', 'human', 'ao.clan'])
  record('T19', 'create --no-render + render materialises view', r.code === 0 && rr.code === 0 && /<html|<body|<h1/i.test(h.out), rr.out + h.out.slice(0, 200))
}

// ---------- T20 F2b: no stale hint when patched keys are data-bound (EXPECT RED) ----------
{
  clan(['create', '--title', 'BindDoc', '--brief', 'b', '--output', 'bind.clan'])
  const html = join(work, 'bound.html')
  writeFileSync(html, '<!DOCTYPE html><html><body><h1>Report</h1><p>Price: <span data-adf-id="p">{{price}}</span></p></body></html>')
  clan(['pack-html', 'bind.clan', html, '--delta', 'seed view', '--output', 'bind2.clan'].filter(Boolean))
  const target = existsSync(join(work, 'bind2.clan')) ? 'bind2.clan' : 'bind.clan'
  const p = clan(['patch-data', target, '{"price":99}', ...ATTR], { hints: true })
  const falseHint = /stale/i.test(p.out)
  record('T20', 'no stale-view hint for bound-key patch (F2b)', p.code === 0 && !falseHint, `hint-output: ${p.out.slice(0, 200)}`, true /* expect red until F2b lands */)
}

// ---------- T23 requirements declared + surfaced (F8 / layer 5) ----------
{
  const req = join(work, 'req.yaml')
  writeFileSync(req, 'requirements:\n  - capability: web-search\n    level: none\n  - capability: filesystem\n    level: read-write\n')
  r = clan(['patch-requirements', 'doc.clan', req])
  const a = clan(['read', 'agent', 'doc.clan', '--skip-guide'])
  record('T23', 'patch-requirements surfaced in read agent (F8/L5)', r.code === 0 && /(requirement|web-search|filesystem)/i.test(a.out), r.out + a.out.slice(0, 200))
}

// ---------- T21 read decisions alias (F12) ----------
r = clan(['read', 'decisions', 'doc.clan'])
record('T21', 'read decisions == read chain (F12)', r.code === 0 && /- agent:/.test(r.out), r.out.slice(0, 200))

// ---------- T22 export-static carries the full handoff surface ----------
{
  const e = exportStatic('doc.clan')
  const keys = Object.keys(e.json || {})
  const need = ['task', 'shared_data', 'decision_history_toon', 'output_schema'].filter(k => !keys.some(x => x.includes(k.split('_')[0])))
  record('T22', 'export-static keys complete', e.json && need.length === 0, e.err || `missing=${need} have=${keys}`)
}

// ---------- write results ----------
const hardFails = results.filter(x => !x.pass && !x.expect_red).length
const summary = {
  ran_at: new Date().toISOString(),
  clan_binary: CLAN,
  clan_version: clan(['--version']).out.trim(),
  total: results.length,
  passed: results.filter(x => x.pass).length,
  hard_failures: hardFails,
  expected_red: results.filter(x => !x.pass && x.expect_red).map(x => x.id),
  unexpected_green: results.filter(x => x.pass && x.expect_red).map(x => x.id), // expect-red that now passes -> flip claims.yaml
  tests: results,
}
mkdirSync(OUT, { recursive: true })
writeFileSync(join(OUT, 'conformance.json'), JSON.stringify(summary, null, 2))
console.log(`\n${summary.passed}/${summary.total} passed, ${hardFails} hard failure(s).` +
  (summary.unexpected_green.length ? ` NOTE: expect-red now GREEN: ${summary.unexpected_green} — update claims.yaml.` : ''))
rmSync(work, { recursive: true, force: true })
process.exit(hardFails)

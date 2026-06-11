#!/usr/bin/env node
// Node.js port of measure-lite.ps1 — works on macOS/Linux.
import { execSync } from 'node:child_process'
import { readdirSync, statSync, readFileSync, writeFileSync, existsSync } from 'node:fs'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const HERE = dirname(fileURLToPath(import.meta.url))
process.env.CLAN_NO_HINTS = '1'

function charsOfCmd(cmd) {
  try { return execSync(cmd, { encoding: 'utf8', stdio: ['pipe','pipe','ignore'] }).length } catch { return -1 }
}
function dirChars(dir) {
  if (!existsSync(dir)) return 0
  let c = 0
  for (const f of readdirSync(dir, { recursive: true, withFileTypes: true })) {
    if (f.isFile()) {
      try { c += readFileSync(join(f.parentPath ?? f.path, f.name), 'utf8').length } catch {}
    }
  }
  return c
}

const result = { h1: { clan: [], adhoc: [] }, h2: { clan: [], adhoc: [] }, receipts: [] }

// ---------- H1: output chars per hop ----------
for (const arm of ['clan', 'adhoc']) {
  const armDir = join(HERE, `h1-${arm}`)
  const recDir = join(armDir, 'receipts')
  for (let n = 1; n <= 8; n++) {
    const nn = String(n).padStart(2, '0')
    const outDir = join(recDir, `hop${nn}-output`)
    const authored = dirChars(outDir)
    let reported = null, role = null
    const recFiles = existsSync(recDir) ? readdirSync(recDir).filter(f => f.startsWith(`receipt-hop${nn}`)) : []
    if (recFiles.length) {
      try {
        const j = JSON.parse(readFileSync(join(recDir, recFiles[0]), 'utf8'))
        reported = j.output_chars; role = j.role
      } catch {}
    }
    result.h1[arm].push({ hop: n, role, authored_chars: authored, reported_output_chars: reported })
  }
  if (arm === 'clan') {
    const doc = join(armDir, 'work', 'doc.clan')
    if (existsSync(doc)) {
      let validates = false
      try { execSync(`clan validate ${doc}`, { stdio: 'ignore' }); validates = true } catch {}
      const chainText = charsOfCmd(`clan read chain ${doc}`) > 0
        ? execSync(`clan read chain ${doc}`, { encoding: 'utf8', stdio: ['pipe','pipe','ignore'] }) : ''
      result.h1_clan_final = {
        validates,
        data_chars: charsOfCmd(`clan read data ${doc}`),
        human_chars: charsOfCmd(`clan read human ${doc}`),
        n_decisions: (chainText.match(/- agent:/g) || []).length,
        bytes: statSync(doc).size,
      }
    }
  } else {
    const work = join(armDir, 'work')
    result.h1_adhoc_final = {
      report_chars: existsSync(join(work,'report.html')) ? readFileSync(join(work,'report.html'),'utf8').length : 0,
      data_chars: existsSync(join(work,'data.json')) ? readFileSync(join(work,'data.json'),'utf8').length : 0,
      log_lines: existsSync(join(work,'decisions.log')) ? readFileSync(join(work,'decisions.log'),'utf8').split('\n').filter(Boolean).length : 0,
    }
  }
}

// ---------- H2: injected chars per hop ----------
const clanSnapDir = join(HERE, 'h2-clan', 'snapshots')
if (existsSync(clanSnapDir)) {
  const snaps = readdirSync(clanSnapDir).filter(f => f.startsWith('hop-') && f.endsWith('.clan')).sort()
  for (const s of snaps) {
    const full = join(clanSnapDir, s)
    result.h2.clan.push({
      hop: s.replace('.clan',''),
      injected_skipguide: charsOfCmd(`clan read agent ${full} --skip-guide`),
      injected_full: charsOfCmd(`clan read agent ${full}`),
      data_chars: charsOfCmd(`clan read data ${full}`),
      artifact_bytes: statSync(full).size,
    })
  }
}
const adhocSnapDir = join(HERE, 'h2-adhoc', 'snapshots')
const briefFile = join(HERE, 'h2-adhoc', 'work', 'brief.md')
const briefChars = existsSync(briefFile) ? readFileSync(briefFile, 'utf8').length : 0
if (existsSync(adhocSnapDir)) {
  const dirs = readdirSync(adhocSnapDir, { withFileTypes: true }).filter(d => d.isDirectory()).map(d=>d.name).sort()
  for (const d of dirs) {
    const full = join(adhocSnapDir, d)
    result.h2.adhoc.push({
      hop: d,
      injected_full: briefChars + dirChars(full),
      artifact_bytes: dirChars(full),
    })
  }
}

// ---------- receipts ----------
for (const flowArm of ['h1-clan','h1-adhoc','h2-clan','h2-adhoc']) {
  const recDir = join(HERE, flowArm, 'receipts')
  if (!existsSync(recDir)) continue
  for (const f of readdirSync(recDir).filter(f => f.startsWith('receipt-') && f.endsWith('.json'))) {
    try {
      const j = JSON.parse(readFileSync(join(recDir, f), 'utf8'))
      result.receipts.push({ arm: flowArm, role: j.role, hop: j.hop,
        n_errors: (j.errors || []).length, friction: j.problems_and_friction,
        n_files_read: (j.files_read || []).length })
    } catch { result.receipts.push({ arm: flowArm, file: f, parse_error: true }) }
  }
}

const out = join(HERE, 'metrics-lite.json')
writeFileSync(out, '﻿' + JSON.stringify(result, null, 2))
const clanSum = result.h1.clan.reduce((s,e)=>s+(e.authored_chars||0),0)
const adhocSum = result.h1.adhoc.reduce((s,e)=>s+(e.authored_chars||0),0)
console.log(`metrics-lite.json written: ${statSync(out).size} bytes`)
console.log(`H1 ratio: ${adhocSum > 0 ? (clanSum/adhocSum).toFixed(3) : 'n/a'} (clan ${clanSum} / adhoc ${adhocSum})`)
console.log(`H1 n_decisions: ${result.h1_clan_final?.n_decisions ?? 'n/a'} / 8 hops`)
if (result.h2.clan.length && result.h2.adhoc.length) {
  const cl = result.h2.clan.at(-1), al = result.h2.adhoc.at(-1)
  const sr = (cl.injected_skipguide ?? cl.injected_full) / al.injected_full
  console.log(`H2 synth_ratio: ${sr.toFixed(3)}`)
}

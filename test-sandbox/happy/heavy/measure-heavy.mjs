#!/usr/bin/env node
// Node.js port of measure-heavy.ps1 — works on macOS/Linux.
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
function mean(a) { return a.length ? a.reduce((x,y)=>x+y,0)/a.length : 0 }
function stdev(a) {
  if (a.length < 2) return 0
  const m = mean(a)
  return Math.sqrt(a.reduce((s,x)=>s+(x-m)**2, 0)/(a.length-1))
}

const N_REPS = 5
const result = { h1: { reps: [] }, h2: { clan: [], adhoc: [], reps: [] }, receipts: [] }
const h1Ratios = []

// ---- H1 ----
for (let rep = 1; rep <= N_REPS; rep++) {
  const clanDir = join(HERE, `h1-clan-rep${rep}`)
  const adhocDir = join(HERE, `h1-adhoc-rep${rep}`)
  const repRow = { rep, clan_hops: [], adhoc_hops: [] }
  for (const arm of ['clan','adhoc']) {
    const armDir = arm === 'clan' ? clanDir : adhocDir
    const recDir = join(armDir, 'receipts')
    for (let n = 1; n <= 8; n++) {
      const nn = String(n).padStart(2,'0')
      const outDir = join(recDir, `hop${nn}-output`)
      const authored = dirChars(outDir)
      let reported = null, role = null
      if (existsSync(recDir)) {
        const recs = readdirSync(recDir).filter(f=>f.startsWith(`receipt-hop${nn}`))
        if (recs.length) {
          try { const j = JSON.parse(readFileSync(join(recDir,recs[0]),'utf8')); reported=j.output_chars; role=j.role } catch {}
        }
      }
      repRow[`${arm}_hops`].push({ hop:n, role, authored_chars:authored, reported })
    }
  }
  const clanTotal = repRow.clan_hops.reduce((s,e)=>s+(e.authored_chars||0),0)
  const adhocTotal = repRow.adhoc_hops.reduce((s,e)=>s+(e.authored_chars||0),0)
  const ratio = adhocTotal > 0 ? Math.round(clanTotal/adhocTotal*10000)/10000 : null
  repRow.clan_total_chars = clanTotal; repRow.adhoc_total_chars = adhocTotal; repRow.ratio = ratio
  if (ratio !== null) h1Ratios.push(ratio)
  const doc = join(clanDir, 'work', 'doc.clan')
  if (existsSync(doc)) {
    let validates = false
    try { execSync(`clan validate ${doc}`, {stdio:'ignore'}); validates=true } catch {}
    const chainText = (() => { try { return execSync(`clan read chain ${doc}`,{encoding:'utf8',stdio:['pipe','pipe','ignore']}) } catch { return '' } })()
    repRow.clan_final = { validates, n_decisions:(chainText.match(/- agent:/g)||[]).length, bytes:statSync(doc).size }
  }
  result.h1.reps.push(repRow)
}
if (h1Ratios.length) {
  result.h1.ratio_mean = Math.round(mean(h1Ratios)*10000)/10000
  result.h1.ratio_stdev = Math.round(stdev(h1Ratios)*10000)/10000
  result.h1.win = result.h1.ratio_mean <= 0.50
}

// ---- H2 (first rep only for injected-chars curve; all reps for crossover) ----
const crossoverHops = []
for (let rep = 1; rep <= N_REPS; rep++) {
  const clanDir = join(HERE, `h2-clan-rep${rep}`)
  const adhocDir = join(HERE, `h2-adhoc-rep${rep}`)
  const repRow = { rep, clan:[], adhoc:[], crossover_hop:null }
  const clanSnaps = existsSync(join(clanDir,'snapshots')) ?
    readdirSync(join(clanDir,'snapshots')).filter(f=>f.startsWith('hop-')&&f.endsWith('.clan')).sort() : []
  for (const s of clanSnaps) {
    const full = join(clanDir,'snapshots',s)
    repRow.clan.push({ hop:s.replace('.clan',''), injected_skipguide:charsOfCmd(`clan read agent ${full} --skip-guide`), artifact_bytes:statSync(full).size })
  }
  const briefFile = join(adhocDir,'work','brief.md')
  const briefChars = existsSync(briefFile) ? readFileSync(briefFile,'utf8').length : 0
  const adhocSnaps = existsSync(join(adhocDir,'snapshots')) ?
    readdirSync(join(adhocDir,'snapshots'),{withFileTypes:true}).filter(d=>d.isDirectory()).map(d=>d.name).sort() : []
  for (const d of adhocSnaps) {
    const full = join(adhocDir,'snapshots',d)
    repRow.adhoc.push({ hop:d, injected_full:briefChars+dirChars(full), artifact_bytes:dirChars(full) })
  }
  // crossover
  const minH = Math.min(repRow.clan.length, repRow.adhoc.length)
  for (let i = 0; i < minH-1; i++) {
    const ci=repRow.clan[i], ai=repRow.adhoc[i]
    if ((ci.injected_skipguide||0)>0 && (ai.injected_full||0)>0 && ci.injected_skipguide < ai.injected_full) {
      repRow.crossover_hop = i+1; break
    }
  }
  if (repRow.crossover_hop !== null) crossoverHops.push(repRow.crossover_hop)
  result.h2.reps.push(repRow)
  // use rep1 for the summary curve
  if (rep === 1) { result.h2.clan = repRow.clan; result.h2.adhoc = repRow.adhoc }
}
if (crossoverHops.length) {
  result.h2.crossover_mean = Math.round(mean(crossoverHops)*100)/100
  result.h2.crossover_stdev = Math.round(stdev(crossoverHops)*100)/100
  result.h2.win = result.h2.crossover_mean <= 10
}

// ---- receipts ----
for (const arm of ['h1-clan','h1-adhoc','h2-clan','h2-adhoc']) {
  for (let rep = 1; rep <= N_REPS; rep++) {
    const recDir = join(HERE, `${arm}-rep${rep}`, 'receipts')
    if (!existsSync(recDir)) continue
    for (const f of readdirSync(recDir).filter(f=>f.startsWith('receipt-')&&f.endsWith('.json'))) {
      try {
        const j = JSON.parse(readFileSync(join(recDir,f),'utf8'))
        result.receipts.push({ arm, rep, role:j.role, hop:j.hop, n_errors:(j.errors||[]).length, friction:j.problems_and_friction })
      } catch { result.receipts.push({ arm, rep, file:f, parse_error:true }) }
    }
  }
}

const out = join(HERE, 'metrics-heavy.json')
writeFileSync(out, '﻿' + JSON.stringify(result, null, 2))
console.log(`metrics-heavy.json written: ${statSync(out).size} bytes`)
console.log(`H1 ratio: mean=${result.h1.ratio_mean} stdev=${result.h1.ratio_stdev} win=${result.h1.win}`)
console.log(`H2 crossover: mean=${result.h2.crossover_mean ?? 'none'} win=${result.h2.win ?? false}`)

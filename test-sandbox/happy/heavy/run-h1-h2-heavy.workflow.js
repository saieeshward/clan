// MEASUREMENT NOTE (2026-06-11 revision): the patch-html hops (H1 hops 2/5/6)
// now hand agents PRE-WRITTEN fragment files (_inputs/h1-supplied/) and the
// exact `--selector`/`--patch-action` command, rather than asking them to
// compose HTML+frontmatter under pressure. This isolates the CLI/format token
// cost from agent-composition noise (the F16 structural failures). Both arms
// get the same supplied fragments for fairness. H2 free hops (10/11) carry a
// ≤300-char data budget to bound injection growth. Results from this revision
// are NOT directly comparable to the 2026-06-11-H baseline (which measured
// composition too) — report them as a separate, composition-controlled run.
export const meta = {
  name: 'happy-path-heavy-h1-h2',
  description: 'HEAVY benchmark (composition-controlled): H1 patch-vs-rewrite with supplied fragments (5 reps x2 arms, 8 hops) + H2 long-chain crossover with data budgets (5 reps x2 arms, 12 hops), verifier per rep, measure pass',
  phases: [
    { title: 'H1-clan' },
    { title: 'H1-adhoc' },
    { title: 'H2-clan' },
    { title: 'H2-adhoc' },
    { title: 'Verify' },
    { title: 'Measure' },
  ],
}

const N_REPS = 5
const HEAVY = 'test-sandbox/happy/heavy'
const INPUTS = 'test-sandbox/happy/lite/_inputs'
const SUP = `${INPUTS}/h1-supplied`
const H2SUP = `${INPUTS}/h2-supplied`

const RECEIPT_NOTE = `
Write a receipt JSON to receipts/receipt-hopNN.json (NN zero-padded hop number) with EXACTLY these keys:
{ "role": string, "hop": int, "arm": string, "flow": string,
  "commands_run": [string],
  "files_read": [{"file": string, "approx_chars": int}],
  "files_written": [{"file": string, "chars": int}],
  "output_chars": int,
  "errors": [{"what": string, "recovered_by": string}],
  "problems_and_friction": "1-3 sentences",
  "context_understood": "1 sentence" }
Also save the EXACT text you authored to effect the change into receipts/hopNN-output/ (one file per payload:
the patch-data JSON file, the patch-html fragment, the decision text, or for ad-hoc only the fragment(s) you
actually typed — NOT whole files unless you genuinely rewrote them from scratch). The measure pass sums these
as ground-truth output bytes. Be honest and minimal: authored chars only.`

// ---- H1: 8-hop change spec (byte-identical across arms) ----
const H1 = [
  { n: 1, role: 'pricing-update',
    change: 'Zoho CRM per-seat price drops to 38 (from 45); zoho_annual_total becomes 18240 (=38x40x12).',
    clan: 'Both are data-bound {{}} fields. Write {"zoho_price_per_seat":38,"zoho_annual_total":18240} to /tmp/h1patch1.json and run: clan patch-data DOC /tmp/h1patch1.json --agent pricing-agent --action "update Zoho seat price to 38" --rationale "New vendor quote received". pack-html is FORBIDDEN.',
    adhoc: 'Edit data.json (zoho_price_per_seat=38, zoho_annual_total=18240) AND update the literal "$45"/"$21600" figures in the report.html cost table.' },
  { n: 2, role: 'risk-add',
    change: 'Add risk: {"risk":"Vendor lock-in","severity":"Medium","mitigation":"Quarterly data-export clause in contract"}.',
    clan: `risks is an array — merge-patch REPLACES arrays. First: clan read data DOC to get current risks[]. Then write the full new risks array (existing + new entry) to /tmp/h1patch2.json and clan patch-data DOC /tmp/h1patch2.json --agent risk-agent --action "add vendor lock-in risk" --rationale "Identified during procurement review". THEN add the matching view row from the SUPPLIED fragment file (do NOT compose HTML yourself): clan patch-html DOC ${SUP}/hop02-risk-fragment.html --selector "#risk-rows" --patch-action append --agent risk-agent --action "add risk table row" --rationale "Mirror data patch in view". The success line confirms it landed in <tbody id="risk-rows">. pack-html FORBIDDEN.`,
    adhoc: `Append the risk object to the risks[] array in data.json AND splice the SUPPLIED row fragment ${SUP}/hop02-risk-fragment.html into the risk table tbody (id="risk-rows") in report.html.` },
  { n: 3, role: 'date-update',
    change: 'rollout_phase2_date changes to 2026-09-15 (was 2026-08-01).',
    clan: 'Data-bound field. Write {"rollout_phase2_date":"2026-09-15"} to /tmp/h1patch3.json and clan patch-data DOC /tmp/h1patch3.json --agent date-agent --action "update phase-2 date" --rationale "Timeline slipped by 6 weeks". pack-html FORBIDDEN.',
    adhoc: 'Edit data.json (rollout_phase2_date) AND update the literal date in report.html (element id="phase2" or wherever it appears).' },
  { n: 4, role: 'summary-rewrite',
    change: `Replace exec_summary with the exact text in ${SUP}/exec-summary.txt.`,
    clan: `Read ${SUP}/exec-summary.txt. Write {"exec_summary":"<verbatim content>"} to /tmp/h1patch4.json and clan patch-data DOC /tmp/h1patch4.json --agent summary-agent --action "rewrite exec summary" --rationale "Board requested clearer opening". pack-html FORBIDDEN.`,
    adhoc: `Read ${SUP}/exec-summary.txt. Edit data.json exec_summary AND the <p id="exec-summary"> text in report.html with this exact text.` },
  { n: 5, role: 'residency-section',
    change: `Add a new "Data residency" subsection. Exact HTML fragment is in ${SUP}/residency-section.html.`,
    clan: `Add the SUPPLIED section fragment (do NOT compose HTML yourself): clan patch-html DOC ${SUP}/residency-section.html --selector "body" --patch-action append --agent residency-agent --action "add data residency section" --rationale "Legal requirement to document residency". pack-html FORBIDDEN.`,
    adhoc: `Read ${SUP}/residency-section.html. Insert its content into report.html after the Data governance section.` },
  { n: 6, role: 'chart-update',
    change: 'Update SVG bar: rect id="bar-zoho" must become y="78" height="82" (Zoho is now cheapest).',
    clan: `Replace the bar with the SUPPLIED fragment (do NOT compose HTML yourself): clan patch-html DOC ${SUP}/hop06-svg-fragment.html --selector "#bar-zoho" --patch-action replace --agent chart-agent --action "shrink Zoho bar to reflect lower cost" --rationale "Price drop from hop 1 reflected in chart". The success line confirms it replaced <rect id="bar-zoho">. pack-html FORBIDDEN.`,
    adhoc: `Replace the #bar-zoho <rect> in report.html with the SUPPLIED fragment ${SUP}/hop06-svg-fragment.html (y="78" height="82").` },
  { n: 7, role: 'verdict-flip',
    change: `Flip verdict to "Zoho CRM" and set verdict_rationale to the text in ${SUP}/verdict-rationale.txt.`,
    clan: `Read ${SUP}/verdict-rationale.txt. Write {"verdict":"Zoho CRM","verdict_rationale":"<verbatim text>"} to /tmp/h1patch7.json and clan patch-data DOC /tmp/h1patch7.json --agent verdict-agent --action "flip verdict to Zoho CRM" --rationale "Re-scored: Zoho wins on 40-seat cost with EU residency". pack-html FORBIDDEN.`,
    adhoc: `Read ${SUP}/verdict-rationale.txt. Edit data.json (verdict + verdict_rationale) AND update verdict-value and verdict-rationale elements in report.html.` },
  { n: 8, role: 'signoff',
    change: 'Record final sign-off: steering committee approved Zoho CRM on 2026-06-11.',
    clan: 'Run: clan patch-decision DOC --agent steering-committee --action "approved Zoho CRM recommendation" --rationale "Re-scored cost vs time-to-value; Zoho wins on 40-seat cost with EU residency included." --pinned',
    adhoc: 'Append to decisions.log: "2026-06-11 | steering-committee | approved Zoho CRM recommendation | Re-scored cost vs time-to-value; Zoho wins on 40-seat cost with EU residency included."' },
]

// ---- H2: supplied roles (hops 1-9) ----
const H2_SUPPLIED = [
  '01-market-researcher','02-pricing-analyst','03-risk-analyst','04-gdpr-reviewer',
  '05-integrations-assessor','06-customer-discovery','07-competitive-intel',
  '08-finance-modeler','09-rollout-planner',
]

// H2 free roles (hops 10-11) — agents author findings from accumulated context
// DATA BUDGET (problem 3 fix): free-authored hops are capped at ≤300 chars of
// findings JSON to bound per-hop injection growth and remove the agent-verbosity
// variance that tipped C-SYNTH-WIN over 1.0. Prefer numeric/enum fields to prose.
const H2_FREE = [
  { n: 10, role: '10-procurement-reviewer',
    desc: 'You are the PROCUREMENT REVIEWER. Read what prior analysts found, then author your own ≤15-field findings JSON covering procurement_lead_time_weeks (int), procurement_contract_term_years (int), procurement_key_commitments (array of strings), procurement_negotiation_leverage (array of strings), procurement_risks (array of {risk, severity, mitigation}), procurement_export_clause_feasible (bool). Namespace all keys with prefix "procurement_". HARD BUDGET: your findings JSON must be ≤300 chars total — prefer numeric/enum/bool fields over prose, keep array entries terse.' },
  { n: 11, role: '11-legal-reviewer',
    desc: 'You are the LEGAL REVIEWER. Read what prior analysts found, then author your own ≤15-field findings JSON covering legal_dpa_required (bool), legal_gdpr_transfer_mechanism (string), legal_liability_cap_eur (int), legal_ip_ownership_clear (bool), legal_indemnification_adequate (bool), legal_risks (array of {risk, severity, mitigation}). Namespace all keys with prefix "legal_". HARD BUDGET: your findings JSON must be ≤300 chars total — prefer numeric/enum/bool fields over prose, keep array entries terse.' },
]

// ---- Prompt builders ----

function h1ClanPrompt(rep, hop) {
  const dir = `${HEAVY}/h1-clan-rep${rep}`
  const doc = `${dir}/work/doc.clan`
  return `You are the CLAN H1 revision agent, rep ${rep}, hop ${hop.n} (role: ${hop.role}). Repo root is cwd; clan binary is on PATH (v1.1.0).
Document: ${doc}

THE CHANGE (identical to the ad-hoc arm): ${hop.change}

CLAN ARM PROTOCOL (guided, patch-first):
1. Orient: clan read agent ${doc} --skip-guide  (also clan read data if you need the current array values)
2. Make the change: ${hop.clan.replace(/DOC/g, doc)}
3. Validate: clan validate ${doc}  (must print OK — fix and retry if it fails)
4. Snapshot: copy ${doc} to ${dir}/snapshots/hop-0${hop.n}-${hop.role}.clan (create dir if needed)
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: 0${hop.n}  Rep: ${rep}  Flow: H1  Arm: clan
Keep authored output MINIMAL — that is the metric. Report concisely when done.`
}

function h1AdhocPrompt(rep, hop) {
  const dir = `${HEAVY}/h1-adhoc-rep${rep}`
  return `You are the AD-HOC H1 revision agent, rep ${rep}, hop ${hop.n} (role: ${hop.role}). Repo root is cwd.
Working dir: ${dir}/work/ — contains report.html, data.json, decisions.log.

THE CHANGE (identical to the CLAN arm): ${hop.change}

AD-HOC ARM PROTOCOL (guided, best-case): ${hop.adhoc}
Splice surgically where possible — that is the fair best case for ad-hoc.
After the change: append a one-line entry to ${dir}/work/decisions.log.
Snapshot: copy report.html + data.json + decisions.log to ${dir}/snapshots/hop-0${hop.n}-${hop.role}/ (create dir).
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: 0${hop.n}  Rep: ${rep}  Flow: H1  Arm: adhoc
Save ONLY the text you actually typed to effect the change (splices or minimal fragments, not whole files unless fully rewritten). Report concisely.`
}

function h2ClanPromptSupplied(rep, idx) {
  const role = H2_SUPPLIED[idx]
  const dir = `${HEAVY}/h2-clan-rep${rep}`
  const doc = `${dir}/work/doc.clan`
  const n = idx + 1
  const nn = n < 10 ? `0${n}` : `${n}`
  return `You are the CLAN H2 chain agent, rep ${rep}, hop ${n} (role: ${role}). Repo root is cwd; clan on PATH.
Document: ${doc} (data-only; no view until hop 12).

Supplied findings file: ${H2SUP}/${role}.json (≤15 role-namespaced fields)
Supplied handoff note: ${H2SUP}/${role}.handoff.txt

PROTOCOL (guided, patch-first):
1. Orient: clan read agent ${doc} --skip-guide  (do NOT re-transcribe prior analysts data; patch-data merge-patches — omitted keys are kept)
2. Read ${H2SUP}/${role}.handoff.txt — you will use its text as the rationale
3. Patch findings: clan patch-data ${doc} ${H2SUP}/${role}.json --agent ${role} --action "added ${role} findings" --rationale "<content of the handoff.txt file>"
4. Validate: clan validate ${doc}
5. Snapshot: copy ${doc} to ${dir}/snapshots/hop-${nn}-${role}.clan
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: ${n}  Rep: ${rep}  Flow: H2  Arm: clan
Report concisely.`
}

function h2ClanPromptFree(rep, free) {
  const dir = `${HEAVY}/h2-clan-rep${rep}`
  const doc = `${dir}/work/doc.clan`
  const nn = `${free.n}`
  return `You are the CLAN H2 chain agent, rep ${rep}, hop ${free.n} (role: ${free.role}). Repo root is cwd; clan on PATH.
Document: ${doc} (accumulated findings from hops 1-${free.n - 1}).

${free.desc}

PROTOCOL:
1. Orient: clan read agent ${doc} --skip-guide ; clan read data ${doc}
2. Author your findings JSON: write to /tmp/h2-r${rep}-hop${free.n}.json
3. Patch: clan patch-data ${doc} /tmp/h2-r${rep}-hop${free.n}.json --agent ${free.role} --action "added ${free.role} findings" --rationale "<one-line summary of your key finding>"
4. Validate: clan validate ${doc}
5. Snapshot: copy ${doc} to ${dir}/snapshots/hop-${nn}-${free.role}.clan
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: ${free.n}  Rep: ${rep}  Flow: H2  Arm: clan
Report concisely.`
}

function h2ClanLeadPrompt(rep) {
  const dir = `${HEAVY}/h2-clan-rep${rep}`
  const doc = `${dir}/work/doc.clan`
  return `You are the CLAN H2 LEAD PARTNER, rep ${rep}, hop 12. Repo root is cwd; clan on PATH.
Document: ${doc} — holds 11 analysts findings.

1. Orient: clan read agent ${doc} --skip-guide ; clan read chain ${doc}
2. Write final recommendation: {"final_recommendation":"Zoho CRM","final_y1_cost_eur":18240,"final_top_risks":["migration data quality","Zoho workflow maturity","integration latency"],"final_rationale":"Lowest 40-seat cost with EU residency included; low switching cost via export clause offsets maturity risk."} to /tmp/h2-r${rep}-lead.json then: clan patch-data ${doc} /tmp/h2-r${rep}-lead.json --agent lead-partner --action "final recommendation: Zoho CRM" --rationale "Synthesised 11 analyst inputs"
3. Materialise view: clan render ${doc}
4. Validate: clan validate ${doc}
5. Record decision: clan patch-decision ${doc} --agent lead-partner --action "final recommendation: Zoho CRM" --rationale "Synthesised 11 analyst inputs." --pinned
6. Snapshot: copy ${doc} to ${dir}/snapshots/hop-12-lead-partner.clan
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: 12  Rep: ${rep}  Flow: H2  Arm: clan
Report concisely.`
}

function h2AdhocPromptSupplied(rep, idx) {
  const role = H2_SUPPLIED[idx]
  const dir = `${HEAVY}/h2-adhoc-rep${rep}`
  const n = idx + 1
  const nn = n < 10 ? `0${n}` : `${n}`
  return `You are the AD-HOC H2 chain agent, rep ${rep}, hop ${n} (role: ${role}). Repo root is cwd.
Working dir: ${dir}/work/ — has brief.md, findings.md, handoff.md (growing each hop).

Supplied findings: ${H2SUP}/${role}.json    Supplied handoff: ${H2SUP}/${role}.handoff.txt

PROTOCOL (best-case ad-hoc): To orient you MUST read brief.md + findings.md + handoff.md in full.
1. Append your findings to findings.md under a "## ${role}" heading — write as key: value list (same content as the supplied JSON).
2. Append your handoff note (from ${H2SUP}/${role}.handoff.txt) to handoff.md under "## ${role}".
3. Snapshot: copy findings.md and handoff.md to ${dir}/snapshots/hop-${nn}-${role}/ (create dir).
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: ${n}  Rep: ${rep}  Flow: H2  Arm: adhoc
In files_read, record approx_chars for EVERY file you read (this is the injected-context metric). Report concisely.`
}

function h2AdhocPromptFree(rep, free) {
  const dir = `${HEAVY}/h2-adhoc-rep${rep}`
  const nn = `${free.n}`
  return `You are the AD-HOC H2 chain agent, rep ${rep}, hop ${free.n} (role: ${free.role}). Repo root is cwd.
Working dir: ${dir}/work/ — has brief.md, findings.md (hops 1-${free.n - 1} accumulated), handoff.md.

${free.desc}

PROTOCOL (best-case ad-hoc): Read brief.md, findings.md, handoff.md in full to orient.
1. Author your findings as a markdown key: value list and append to findings.md under "## ${free.role}".
2. Append your one-line handoff summary to handoff.md under "## ${free.role}".
3. Snapshot: copy findings.md and handoff.md to ${dir}/snapshots/hop-${nn}-${free.role}/ (create dir).
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: ${free.n}  Rep: ${rep}  Flow: H2  Arm: adhoc
Record approx_chars for EVERY file you read. Report concisely.`
}

function h2AdhocLeadPrompt(rep) {
  const dir = `${HEAVY}/h2-adhoc-rep${rep}`
  return `You are the AD-HOC H2 LEAD PARTNER, rep ${rep}, hop 12. Repo root is cwd.
Working dir: ${dir}/work/ — brief.md + accumulated findings.md and handoff.md from 11 analysts.

1. Read brief.md, findings.md, and handoff.md in FULL.
2. Write a final report.html: recommendation page for Zoho CRM, year-1 EUR 18,240, top risks, full rationale synthesising all 11 analysts.
3. Write decisions.log (create if absent): "2026-06-11 | lead-partner | final recommendation: Zoho CRM | synthesised 11 analyst inputs"
4. Snapshot findings.md, handoff.md, report.html, decisions.log to ${dir}/snapshots/hop-12-lead-partner/ (create dir).
${RECEIPT_NOTE}
Receipts dir: ${dir}/receipts/  Hop: 12  Rep: ${rep}  Flow: H2  Arm: adhoc
Record approx_chars for EVERY file you read. Report concisely.`
}

function verifierPrompt(rep) {
  const clanDir = `${HEAVY}/h1-clan-rep${rep}`
  const adhocDir = `${HEAVY}/h1-adhoc-rep${rep}`
  return `You are the H1 fidelity verifier for rep ${rep}. Repo root is cwd; clan on PATH.

CLAN final:  ${clanDir}/work/doc.clan  — inspect with: clan read data, clan read human, clan read chain
AD-HOC final: ${adhocDir}/work/ — inspect report.html + data.json + decisions.log

The 8 required changes to verify (present / partial / absent in each arm):
1. zoho_price_per_seat=38 AND zoho_annual_total=18240 (data AND view)
2. risks[] contains {"risk":"Vendor lock-in","severity":"Medium",...} AND a matching <tr> is in the HTML table
3. rollout_phase2_date="2026-09-15"
4. exec_summary matches ${SUP}/exec-summary.txt verbatim (read the file to compare)
5. A "Data residency" subsection is present in the HTML view
6. SVG #bar-zoho has y="78" height="82"
7. verdict="Zoho CRM" AND verdict_rationale matches ${SUP}/verdict-rationale.txt
8. A final sign-off decision is recorded (agent: steering-committee or a decisions.log line)

For each arm, for each of the 8 changes, score: "present" / "partial" / "absent".
Also flag any UNINTENDED diffs: data fields or HTML sections that changed but should not have.
Return structured JSON only — no prose.`
}

const VERDICT_SCHEMA = {
  type: 'object',
  required: ['clan', 'adhoc'],
  properties: {
    clan: {
      type: 'object',
      required: ['changes_present', 'partial', 'absent', 'unintended_diffs'],
      properties: {
        changes_present: { type: 'integer' },
        partial: { type: 'integer' },
        absent: { type: 'integer' },
        unintended_diffs: { type: 'array', items: { type: 'string' } },
        notes: { type: 'string' },
      },
    },
    adhoc: {
      type: 'object',
      required: ['changes_present', 'partial', 'absent', 'unintended_diffs'],
      properties: {
        changes_present: { type: 'integer' },
        partial: { type: 'integer' },
        absent: { type: 'integer' },
        unintended_diffs: { type: 'array', items: { type: 'string' } },
        notes: { type: 'string' },
      },
    },
  },
}

// ---- Chain runners (serial within each chain) ----

async function runH1Clan(rep) {
  const results = []
  for (let i = 0; i < H1.length; i++) {
    const r = await agent(h1ClanPrompt(rep, H1[i]), { label: `h1clan-r${rep}-h${H1[i].n}`, phase: 'H1-clan' })
    results.push(r)
  }
  return results
}

async function runH1Adhoc(rep) {
  const results = []
  for (let i = 0; i < H1.length; i++) {
    const r = await agent(h1AdhocPrompt(rep, H1[i]), { label: `h1adhoc-r${rep}-h${H1[i].n}`, phase: 'H1-adhoc' })
    results.push(r)
  }
  return results
}

async function runH2Clan(rep) {
  const results = []
  for (let i = 0; i < H2_SUPPLIED.length; i++) {
    const r = await agent(h2ClanPromptSupplied(rep, i), { label: `h2clan-r${rep}-h${i + 1}`, phase: 'H2-clan' })
    results.push(r)
  }
  for (let i = 0; i < H2_FREE.length; i++) {
    const r = await agent(h2ClanPromptFree(rep, H2_FREE[i]), { label: `h2clan-r${rep}-h${H2_FREE[i].n}`, phase: 'H2-clan' })
    results.push(r)
  }
  const lead = await agent(h2ClanLeadPrompt(rep), { label: `h2clan-r${rep}-lead`, phase: 'H2-clan' })
  results.push(lead)
  return results
}

async function runH2Adhoc(rep) {
  const results = []
  for (let i = 0; i < H2_SUPPLIED.length; i++) {
    const r = await agent(h2AdhocPromptSupplied(rep, i), { label: `h2adhoc-r${rep}-h${i + 1}`, phase: 'H2-adhoc' })
    results.push(r)
  }
  for (let i = 0; i < H2_FREE.length; i++) {
    const r = await agent(h2AdhocPromptFree(rep, H2_FREE[i]), { label: `h2adhoc-r${rep}-h${H2_FREE[i].n}`, phase: 'H2-adhoc' })
    results.push(r)
  }
  const lead = await agent(h2AdhocLeadPrompt(rep), { label: `h2adhoc-r${rep}-lead`, phase: 'H2-adhoc' })
  results.push(lead)
  return results
}

// ---- Orchestration ----
// pipeline: stage 1 runs all 4 chains per rep in parallel; stage 2 runs verifier per rep

const reps = Array.from({ length: N_REPS }, (_, i) => i + 1)

const repResults = await pipeline(
  reps,
  (rep) => parallel([
    () => runH1Clan(rep),
    () => runH1Adhoc(rep),
    () => runH2Clan(rep),
    () => runH2Adhoc(rep),
  ]),
  (chains, rep) => agent(verifierPrompt(rep), {
    label: `verify-r${rep}`, phase: 'Verify', schema: VERDICT_SCHEMA,
  })
)

// ---- Measure pass ----
log('All reps complete — running measure pass')
const measure = await agent(
  `You are the HEAVY measure agent. Repo root is cwd; PowerShell is available.
Run: powershell -NonInteractive -File test-sandbox/happy/heavy/measure-heavy.ps1
Read the output and the resulting test-sandbox/happy/heavy/metrics-heavy.json.
Report the following as JSON: H1 ratio mean/stdev/win (target <=0.50), H2 crossover hop mean/stdev/win (target <=10), total errors across all receipts, and any critical failures (chains that produced zero authored chars or validation failures).`,
  { label: 'measure', phase: 'Measure' }
)

return {
  n_reps: N_REPS,
  rep_verdicts: repResults.filter(Boolean),
  measure,
}

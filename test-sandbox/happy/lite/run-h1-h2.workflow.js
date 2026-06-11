export const meta = {
  name: 'happy-path-lite-h1-h2',
  description: 'LITE benchmark: H1 patch-vs-rewrite (8 hops x2 arms) + H2 long-chain crossover (10 hops x2 arms)',
  phases: [
    { title: 'H1-clan' },
    { title: 'H1-adhoc' },
    { title: 'H2-clan' },
    { title: 'H2-adhoc' },
    { title: 'Verify' },
  ],
}

const ROOT = 'test-sandbox/happy/lite'
const SUP = `${ROOT}/_inputs/h1-supplied`

const RECEIPT_NOTE = `
Write a receipt JSON to receipts/receipt-hopNN.json (NN zero-padded) with EXACTLY these keys:
{ "role","hop","arm","flow","commands_run":[..],"files_read":[{"file","approx_chars"}],
  "files_written":[{"file","chars"}],"output_chars":<int total chars YOU authored this hop>,
  "errors":[{"what","recovered_by"}],"problems_and_friction":"<1-3 sentences>","context_understood":"<1 sentence>" }
ALSO: save the EXACT text you authored to effect the change into receipts/hopNN-output/ (one file per payload:
the patch-data JSON, the patch-html fragment, the decision text, or — for ad-hoc — only the fragment(s) you actually
typed, NOT whole files unless you genuinely rewrote them). The measure pass sums these as ground-truth output bytes.
Be honest and minimal: authored chars only.`

// ---- H1: shared change spec (byte-identical instruction across arms) ----
const H1 = [
  { n: 1, role: 'pricing-update',
    change: 'Zoho CRM per-seat price drops to 38 (from 45); zoho_annual_total becomes 18240 (=38*40*12).',
    clan: `Both are data-bound {{}} fields, so patch-data ALONE updates the view too. Write {"zoho_price_per_seat":38,"zoho_annual_total":18240} to a tmp json and run: clan patch-data <doc> <tmp>. pack-html is FORBIDDEN.`,
    adhoc: `Edit data.json (zoho_price_per_seat=38, zoho_annual_total=18240) AND the literal "$45"/"$21600" in report.html cost table.` },
  { n: 2, role: 'risk-add',
    change: 'Add risk {"risk":"Vendor lock-in","severity":"Medium","mitigation":"Quarterly data-export clause in contract"}.',
    clan: `risks is an array — patch-data merge-patch REPLACES arrays, so read the current risks (clan read data <doc>), append the new entry, and patch-data the FULL new risks array. THEN patch-html append one <tr> to selector "#risk-rows" (table rows are literal HTML, not bound). pack-html FORBIDDEN.`,
    adhoc: `Append the risk object to data.json risks[] AND add a matching <tr> to the risk table in report.html.` },
  { n: 3, role: 'date-update',
    change: 'rollout_phase2_date becomes 2026-09-15 (was 2026-08-01).',
    clan: `Data-bound field — patch-data ALONE: {"rollout_phase2_date":"2026-09-15"}. pack-html FORBIDDEN.`,
    adhoc: `Edit data.json AND the literal date in report.html element id="phase2".` },
  { n: 4, role: 'summary-rewrite',
    change: `Replace exec_summary with the supplied text in ${SUP}/exec-summary.txt (read it verbatim).`,
    clan: `exec_summary is data-bound — patch-data ALONE with the supplied text as the value. pack-html FORBIDDEN.`,
    adhoc: `Edit data.json exec_summary AND the <p id="exec-summary"> text in report.html.` },
  { n: 5, role: 'residency-section',
    change: `Add a new "Data residency" subsection. The exact HTML fragment is in ${SUP}/residency-section.html.`,
    clan: `This is NEW view content (not a bound field) — patch-html append the fragment to selector "body". pack-html FORBIDDEN.`,
    adhoc: `Insert the supplied section HTML into report.html (after the Data governance section).` },
  { n: 6, role: 'chart-update',
    change: 'Zoho is now cheapest, so its bar shrinks: the SVG rect id="bar-zoho" must become y="78" height="82".',
    clan: `patch-html with action "replace", selector "#bar-zoho", fragment: <rect id="bar-zoho" x="170" y="78" width="80" height="82" fill="#16a085"></rect>. pack-html FORBIDDEN.`,
    adhoc: `Edit the bar-zoho <rect> in report.html SVG to y="78" height="82".` },
  { n: 7, role: 'verdict-flip',
    change: `Flip verdict from "HubSpot" to "Zoho CRM" and set verdict_rationale to the supplied text in ${SUP}/verdict-rationale.txt.`,
    clan: `Both are data-bound — patch-data ALONE: {"verdict":"Zoho CRM","verdict_rationale":"<supplied>"}. pack-html FORBIDDEN.`,
    adhoc: `Edit data.json (verdict + verdict_rationale) AND the verdict-value + verdict-rationale spans in report.html.` },
  { n: 8, role: 'signoff',
    change: 'Record final sign-off: the steering committee approved the Zoho CRM recommendation on 2026-06-11.',
    clan: `clan patch-decision --agent steering-committee --action "approved Zoho CRM recommendation" --rationale "Re-scored cost vs time-to-value; Zoho wins on 40-seat cost with EU residency included." --pinned <doc>`,
    adhoc: `Append a sign-off line to decisions.log: "2026-06-11 | steering-committee | approved Zoho CRM recommendation | ...".` },
]

function h1ClanPrompt(hop) {
  const dir = `${ROOT}/h1-clan`
  return `You are the CLAN H1 revision agent for hop ${hop.n} (role: ${hop.role}). Repo root is the cwd; the 'clan' binary is on PATH (v1.1.0).
Document: ${dir}/work/doc.clan (a full-html .clan with ~25 data-bound {{key}} fields).

THE CHANGE (identical to the ad-hoc arm): ${hop.change}

CLAN ARM PROTOCOL (guided, patch-first):
1. Orient: clan read agent ${dir}/work/doc.clan --skip-guide   (and clan read data / read human as needed).
2. Make the change EXACTLY as: ${hop.clan}
3. Validate: clan validate ${dir}/work/doc.clan  (must print OK).
4. Snapshot: copy ${dir}/work/doc.clan to ${dir}/snapshots/hop-0${hop.n}-${hop.role}.clan
${RECEIPT_NOTE}
Receipts go in ${dir}/receipts/ . This is hop 0${hop.n}. Keep authored output minimal — that is the metric. Report concisely when done.`
}

function h1AdhocPrompt(hop) {
  const dir = `${ROOT}/h1-adhoc`
  return `You are the AD-HOC H1 revision agent for hop ${hop.n} (role: ${hop.role}). Repo root is the cwd.
Working directory: ${dir}/work/ contains report.html (a static HTML report), data.json (structured data), decisions.log.

THE CHANGE (identical to the CLAN arm): ${hop.change}

AD-HOC ARM PROTOCOL (guided, best-case): read whatever files you need, then edit in place. ${hop.adhoc}
You MAY splice surgically (use minimal Edit operations) — that is the fair best case for ad-hoc.
After the change: append a one-line entry to decisions.log describing what you did.
Then snapshot: copy ${dir}/work/report.html, data.json, decisions.log into ${dir}/snapshots/hop-0${hop.n}-${hop.role}/ (create the dir).
${RECEIPT_NOTE}
Receipts go in ${dir}/receipts/ . This is hop 0${hop.n}. Save ONLY the text you actually typed to effect the change (splices, not whole files, unless you truly rewrote a file). Report concisely when done.`
}

// ---- H2: long chain. 9 specialist data hops + 1 lead-partner final. ----
const H2_ROLES = [
  '01-market-researcher','02-pricing-analyst','03-risk-analyst','04-gdpr-reviewer',
  '05-integrations-assessor','06-customer-discovery','07-competitive-intel','08-finance-modeler',
  '09-rollout-planner',
]
const H2SUP = `${ROOT}/_inputs/h2-supplied`

function h2ClanPrompt(idx) {
  const role = H2_ROLES[idx]
  const dir = `${ROOT}/h2-clan`
  const n = idx + 1
  return `You are the CLAN H2 chain agent, hop ${n}, role ${role}. Repo root is cwd; 'clan' is on PATH.
Document: ${dir}/work/doc.clan (data-only; no view until the final hop).

Your supplied findings: ${H2SUP}/${role}.json  (≤15 fields, role-namespaced keys).
Your supplied handoff note: ${H2SUP}/${role}.handoff.txt

PROTOCOL (guided, patch-first, --skip-guide):
1. Orient: clan read agent ${dir}/work/doc.clan --skip-guide   (this is the WHOLE point — see how little you must read).
   Do NOT re-transcribe prior analysts' data; patch-data merge-patches (omitted keys are kept).
2. Add your findings: clan patch-data ${dir}/work/doc.clan ${H2SUP}/${role}.json
3. Record your handoff as a decision: clan patch-decision --agent ${role} --action "added ${role} findings" --rationale "<the handoff note text>" ${dir}/work/doc.clan
4. Validate: clan validate ${dir}/work/doc.clan
5. Snapshot: copy ${dir}/work/doc.clan to ${dir}/snapshots/hop-0${n}-${role}.clan
${RECEIPT_NOTE}
Receipts go in ${dir}/receipts/ . Hop 0${n}. Report concisely.`
}

function h2AdhocPrompt(idx) {
  const role = H2_ROLES[idx]
  const dir = `${ROOT}/h2-adhoc`
  const n = idx + 1
  return `You are the AD-HOC H2 chain agent, hop ${n}, role ${role}. Repo root is cwd.
Working dir: ${dir}/work/ has brief.md, findings.md, handoff.md (these GROW as the chain proceeds).

Your supplied findings: ${H2SUP}/${role}.json    Your supplied handoff note: ${H2SUP}/${role}.handoff.txt

PROTOCOL (guided, best-case ad-hoc): To orient you MUST read the accumulated state — read brief.md, findings.md AND handoff.md (that is how state is carried here).
1. Append your findings to findings.md under a "## ${role}" heading — write them as a readable key: value list (same content as the supplied JSON).
2. Append your handoff note to handoff.md under a "## ${role}" heading.
3. Snapshot: copy ${dir}/work/findings.md and handoff.md into ${dir}/snapshots/hop-0${n}-${role}/ (create the dir).
${RECEIPT_NOTE}
Receipts go in ${dir}/receipts/ . Hop 0${n}. In files_read, record approx_chars of EVERY file you read (this is the injected-context metric). Report concisely.`
}

function h2LeadClanPrompt() {
  const dir = `${ROOT}/h2-clan`
  return `You are the CLAN H2 LEAD PARTNER (final hop 10). Repo root is cwd; 'clan' on PATH.
Document: ${dir}/work/doc.clan now holds 9 analysts' findings.
1. Orient from the artifact alone: clan read agent ${dir}/work/doc.clan --skip-guide ; clan read chain ${dir}/work/doc.clan
2. Add the final recommendation as data: write {"final_recommendation":"Zoho CRM","final_y1_cost_eur":18240,"final_top_risks":["migration data quality","Zoho workflow maturity","integration latency"],"final_rationale":"Lowest 40-seat cost with EU residency included; low switching cost via export clause offsets maturity risk."} to a tmp json and clan patch-data.
3. Materialise the human view in DESIGNED mode (no hand-authored full-html): clan render ${dir}/work/doc.clan
4. Record the decision: clan patch-decision --agent lead-partner --action "final recommendation: Zoho CRM" --rationale "Synthesised 9 analyst inputs." --pinned ${dir}/work/doc.clan
5. Validate, then snapshot to ${dir}/snapshots/hop-10-lead-partner.clan
${RECEIPT_NOTE}
Receipts in ${dir}/receipts/ . Hop 10. Report concisely.`
}

function h2LeadAdhocPrompt() {
  const dir = `${ROOT}/h2-adhoc`
  return `You are the AD-HOC H2 LEAD PARTNER (final hop 10). Repo root is cwd.
Working dir ${dir}/work/ has brief.md + the accumulated findings.md and handoff.md from 9 analysts.
1. To orient you MUST read brief.md, findings.md and handoff.md in full.
2. Write a final report.html (a readable recommendation page) summarising the recommendation: Zoho CRM, year-1 cost EUR 18,240, top risks, rationale.
3. Append the final decision to a decisions.log (create it): "2026-06-11 | lead-partner | final recommendation: Zoho CRM | synthesised 9 analyst inputs".
4. Snapshot findings.md, handoff.md, report.html, decisions.log into ${dir}/snapshots/hop-10-lead-partner/ .
${RECEIPT_NOTE}
Receipts in ${dir}/receipts/ . Hop 10. Record approx_chars of every file you read in files_read (injected-context metric). Report concisely.`
}

// ---- Run the four independent chains concurrently; hops within a chain are serial ----
async function runSerialChain(promptFn, count, phase, labelPrefix) {
  const results = []
  for (let i = 0; i < count; i++) {
    const r = await agent(promptFn(i), { label: `${labelPrefix}-hop${i + 1}`, phase })
    results.push(r)
  }
  return results
}

const [h1c, h1a, h2c, h2a] = await parallel([
  () => runSerialChain((i) => h1ClanPrompt(H1[i]), H1.length, 'H1-clan', 'h1clan'),
  () => runSerialChain((i) => h1AdhocPrompt(H1[i]), H1.length, 'H1-adhoc', 'h1adhoc'),
  async () => {
    const specialists = await runSerialChain((i) => h2ClanPrompt(i), H2_ROLES.length, 'H2-clan', 'h2clan')
    const lead = await agent(h2LeadClanPrompt(), { label: 'h2clan-lead', phase: 'H2-clan' })
    return [...specialists, lead]
  },
  async () => {
    const specialists = await runSerialChain((i) => h2AdhocPrompt(i), H2_ROLES.length, 'H2-adhoc', 'h2adhoc')
    const lead = await agent(h2LeadAdhocPrompt(), { label: 'h2adhoc-lead', phase: 'H2-adhoc' })
    return [...specialists, lead]
  },
])

// ---- H1 fidelity verifier ----
const verdict = await agent(
  `You are the H1 fidelity verifier. Repo root is cwd; 'clan' on PATH. Compare both H1 final artifacts against the 8 required changes.
CLAN final: ${ROOT}/h1-clan/work/doc.clan  (use clan read data + clan read human + clan read chain)
AD-HOC final: ${ROOT}/h1-adhoc/work/  (report.html + data.json + decisions.log)
The 8 changes: (1) zoho_price_per_seat=38 & zoho_annual_total=18240; (2) a "Vendor lock-in" risk row present in BOTH data and the view table; (3) rollout_phase2_date=2026-09-15; (4) exec_summary rewritten (matches ${SUP}/exec-summary.txt); (5) a "Data residency" subsection present in the view; (6) SVG bar-zoho y=78 height=82; (7) verdict="Zoho CRM" with the supplied rationale; (8) a final sign-off decision recorded.
For EACH arm, for EACH of the 8 changes, report present/absent/partial. Also flag any UNINTENDED diffs (fields/sections that should NOT have changed but did). Return your findings as JSON.`,
  {
    label: 'h1-verifier', phase: 'Verify',
    schema: {
      type: 'object',
      required: ['clan', 'adhoc'],
      properties: {
        clan: { type: 'object', required: ['changes_present', 'unintended_diffs'],
          properties: { changes_present: { type: 'integer' }, partial: { type: 'integer' }, absent: { type: 'integer' },
            unintended_diffs: { type: 'array', items: { type: 'string' } }, notes: { type: 'string' } } },
        adhoc: { type: 'object', required: ['changes_present', 'unintended_diffs'],
          properties: { changes_present: { type: 'integer' }, partial: { type: 'integer' }, absent: { type: 'integer' },
            unintended_diffs: { type: 'array', items: { type: 'string' } }, notes: { type: 'string' } } },
      },
    },
  },
)

return {
  h1_clan_hops: h1c.length, h1_adhoc_hops: h1a.length,
  h2_clan_hops: h2c.length, h2_adhoc_hops: h2a.length,
  fidelity: verdict,
}

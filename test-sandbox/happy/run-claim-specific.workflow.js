export const meta = {
  name: 'claim-specific-agentic',
  description: 'B-FORKMERGE (contested keys) + B-UNGUIDED (teachable) + B-META (metamorphosis) + L-H3 (cold resume), each with a structured verifier',
  phases: [
    { title: 'ForkMerge' },
    { title: 'Unguided' },
    { title: 'Meta' },
    { title: 'Resume' },
    { title: 'Verify' },
  ],
}

const H = 'test-sandbox/happy'
const RECEIPT = `When done, write receipts/receipt-<role>.json with keys:
{ "role","commands_run":[],"files_read":[{"file","approx_chars"}],"files_written":[{"file","chars"}],
  "output_chars":<int you authored>,"errors":[{"what","recovered_by"}],"problems_and_friction":"","context_understood":"" }`

// ===================== B-FORKMERGE =====================
const FM = `${H}/forkmerge2`
const FM_BRANCHES = ['analyst-a', 'analyst-b', 'analyst-c', 'analyst-d']

function fmBranchPrompt(agent) {
  const doc = `${FM}/branches/${agent}.clan`
  return `You are ${agent}, a build-vs-buy analyst. Repo root is cwd; 'clan' is on PATH.
Your branch file: ${doc}
1. Read your task: clan read context ${doc}  (it tells you the EXACT values to write).
2. Read agent guide if unsure: clan agent-help
3. Write your four findings into YOUR namespace (branches are locked on shared data — you MUST use --namespace):
   Build a JSON with keys recommendation, budget_eur, top_risks (array), assumptions EXACTLY as your context says,
   write it to /tmp/${agent}.json, then:
   clan patch-data ${doc} /tmp/${agent}.json --namespace --agent ${agent} --action "wrote build-vs-buy assessment"
4. Validate: clan validate ${doc}
${RECEIPT}  Receipts dir: ${FM}/receipts/  Report concisely.`
}

async function runForkMerge() {
  // 4 branches write in parallel
  await parallel(FM_BRANCHES.map(a => () => agent(fmBranchPrompt(a), { label: `fm-${a}`, phase: 'ForkMerge' })))
  // merge + adjudicate (one agent does the deterministic merge then reads the report)
  const verdict = await agent(
    `You are the build-vs-buy SYNTHESIZER. Repo root is cwd; 'clan' on PATH.
1. Merge the 4 branches deterministically:
   clan merge ${FM}/branches/analyst-a.clan ${FM}/branches/analyst-b.clan ${FM}/branches/analyst-c.clan ${FM}/branches/analyst-d.clan --output ${FM}/work/merged.clan --delta "fold 4 analyst assessments"
2. Inspect conflicts: clan read report ${FM}/work/merged.clan   (this lists contested keys + per-branch values/provenance)
3. The FOUR keys analysts disagreed on: recommendation, budget_eur, top_risks, assumptions.
   For EACH, report how many distinct branch values the merge-report preserved and whether it names the winning + losing branches.
4. Adjudicate: pick a final recommendation and record it:
   write {"final_recommendation":"<pick>","final_budget_eur":<int>} to /tmp/fm-final.json then
   clan patch-data ${FM}/work/merged.clan /tmp/fm-final.json --agent synthesizer --action "adjudicated build-vs-buy" --rationale "<why>"
   and clan patch-decision ${FM}/work/merged.clan --agent synthesizer --action "final: <pick>" --rationale "<cite the contested values>" --pinned
5. Validate the merged file.
Return JSON: for each of the 4 contested keys, {key, distinct_values_in_report, winner_named (bool), loser_provenance (bool)}, plus adjudication_recorded (bool).`,
    {
      label: 'fm-synth', phase: 'ForkMerge',
      schema: {
        type: 'object', required: ['contested', 'adjudication_recorded'],
        properties: {
          contested: { type: 'array', items: { type: 'object', required: ['key', 'distinct_values_in_report', 'winner_named', 'loser_provenance'],
            properties: { key: { type: 'string' }, distinct_values_in_report: { type: 'integer' }, winner_named: { type: 'boolean' }, loser_provenance: { type: 'boolean' } } } },
          adjudication_recorded: { type: 'boolean' }, notes: { type: 'string' },
        },
      },
    })
  // ad-hoc comparison: 4 sequential writers to one flat summary.json -> scalar keys overwrite
  return verdict
}

// ===================== B-UNGUIDED =====================
const UG = `${H}/unguided`
const UG_ROLES = ['market-analyst', 'risk-analyst', 'lead-synthesizer']
function ugPrompt(idx) {
  const role = UG_ROLES[idx]
  const doc = `${UG}/work/doc-fresh.clan`
  return `There is a command-line tool called \`clan\` on your PATH, and a document at ${doc}.
You are the ${role} (hop ${idx + 1} of 3) on a market-entry analysis for Brightline Logistics expanding to Germany.
Figure out how to use the tool yourself. Add your ${role} findings to the document and record what you did so the next agent can continue.
Do not corrupt the file — whatever you do, the document must still validate afterwards. Knowledge only; no web.
${RECEIPT}  Receipts dir: ${UG}/receipts/
In problems_and_friction, say what you had to discover and whether anything blocked you. Report concisely.`
}
async function runUnguided() {
  const results = []
  for (let i = 0; i < UG_ROLES.length; i++) {
    results.push(await agent(ugPrompt(i), { label: `ug-${UG_ROLES[i]}`, phase: 'Unguided' }))
  }
  // verifier
  const v = await agent(
    `You audit a 3-hop UNGUIDED run on ${UG}/work/doc-fresh.clan. Repo root is cwd; 'clan' on PATH.
Inspect: clan read data, clan read chain, clan validate on that file.
Report JSON: { validates (bool), n_chain_entries (int), unknown_agent_count (int), all_hops_attributed (bool),
namespace_or_guard_violations (int, e.g. corrupted file or failed writes that were forced through),
used_pack_when_patch_suffices (bool), stuck_on_attribution_error (bool, true if any agent visibly failed to record attribution) }`,
    { label: 'ug-verify', phase: 'Verify', schema: {
      type: 'object', required: ['validates', 'n_chain_entries', 'unknown_agent_count', 'all_hops_attributed'],
      properties: { validates: { type: 'boolean' }, n_chain_entries: { type: 'integer' }, unknown_agent_count: { type: 'integer' },
        all_hops_attributed: { type: 'boolean' }, namespace_or_guard_violations: { type: 'integer' },
        used_pack_when_patch_suffices: { type: 'boolean' }, stuck_on_attribution_error: { type: 'boolean' }, notes: { type: 'string' } } } })
  return { hops: results.length, verdict: v }
}

// ===================== B-META =====================
const META = `${H}/meta`
function metaHop1() {
  return `You are the ACCOUNT PLANNER (hop 1 of a 3-hop metamorphosis). Repo root is cwd; 'clan' on PATH.
Parent: ${META}/work/doc-fresh.clan . You will produce ${META}/snapshots/hop-01-agency-brief.clan .
This document will later transform into a concept deck then a client pitch — but YOUR hop-1 data fields must survive verbatim to the end.
1. Author an HTML agency brief for "Lumen" (a sustainable lighting startup) into /tmp/meta-hop1.html. At the TOP put a YAML frontmatter block (between --- markers) supplying structured data:
   single_minded_proposition: "Light that pays for itself"
   budget_eur: 120000
   persona: "Facilities managers at mid-size offices"
   The HTML body should reference the logo via <img src="assets/logo-mark.svg">.
2. Pack it, carrying the logo asset and a new schema:
   First write schema to /tmp/meta-schema1.json: {"type":"object","properties":{"single_minded_proposition":{"type":"string"},"budget_eur":{"type":"integer"},"persona":{"type":"string"}}}
   Then: clan pack-html ${META}/work/doc-fresh.clan /tmp/meta-hop1.html --output ${META}/snapshots/hop-01-agency-brief.clan --assets ${META}/work --schema /tmp/meta-schema1.json --agent account-planner --action "agency brief" --rationale "hop 1 metamorphosis"
   (the --assets dir ${META}/work contains logo-mark.svg)
3. Validate the output and confirm the asset is inside: clan validate <out> ; unzip -l <out> | grep logo
${RECEIPT}  Receipts dir: ${META}/receipts/  Report concisely.`
}
function metaHop2() {
  return `You are the CREATIVE DIRECTOR (hop 2 of 3). Repo root is cwd; 'clan' on PATH.
Parent: ${META}/snapshots/hop-01-agency-brief.clan . Produce ${META}/snapshots/hop-02-concept-deck.clan .
Transform the document into a completely different CONCEPT DECK view with a NEW schema — but do NOT re-pass --assets (the logo must carry automatically; this is the F10 regression test) and do NOT re-transcribe hop-1 data (merge-patch keeps omitted keys).
1. Author /tmp/meta-hop2.html: a concept-deck page presenting THREE named concepts. Add frontmatter data: concept_names: ["Dawn", "Halo", "Beacon"].
2. Write schema /tmp/meta-schema2.json: {"type":"object","properties":{"concept_names":{"type":"array","items":{"type":"string"}}}}
3. Pack WITHOUT --assets: clan pack-html ${META}/snapshots/hop-01-agency-brief.clan /tmp/meta-hop2.html --output ${META}/snapshots/hop-02-concept-deck.clan --schema /tmp/meta-schema2.json --agent creative-director --action "concept deck" --rationale "hop 2 metamorphosis"
4. Validate; confirm the logo asset still present (unzip -l | grep logo) AND hop-1 data still present (clan read data | grep single_minded).
${RECEIPT}  Receipts dir: ${META}/receipts/  Report concisely.`
}
function metaHop3() {
  return `You are the PITCH LEAD (hop 3 of 3, final). Repo root is cwd; 'clan' on PATH.
Parent: ${META}/snapshots/hop-02-concept-deck.clan . Produce ${META}/snapshots/hop-03-client-pitch.clan .
Transform into a CLIENT PITCH view, again a new schema, again WITHOUT re-passing --assets and WITHOUT re-transcribing prior data.
1. Author /tmp/meta-hop3.html: a client pitch page. Frontmatter data: recommended_concept: "Halo", pitch_ask_eur: 120000.
2. Schema /tmp/meta-schema3.json: {"type":"object","properties":{"recommended_concept":{"type":"string"},"pitch_ask_eur":{"type":"integer"}}}
3. clan pack-html ${META}/snapshots/hop-02-concept-deck.clan /tmp/meta-hop3.html --output ${META}/snapshots/hop-03-client-pitch.clan --schema /tmp/meta-schema3.json --agent pitch-lead --action "client pitch" --rationale "hop 3 metamorphosis" --pinned
4. Validate.
${RECEIPT}  Receipts dir: ${META}/receipts/  Report concisely.`
}
async function runMeta() {
  await agent(metaHop1(), { label: 'meta-hop1', phase: 'Meta' })
  await agent(metaHop2(), { label: 'meta-hop2', phase: 'Meta' })
  await agent(metaHop3(), { label: 'meta-hop3', phase: 'Meta' })
  const v = await agent(
    `You verify a 3-hop METAMORPHOSIS. Repo root is cwd; 'clan' on PATH. Final: ${META}/snapshots/hop-03-client-pitch.clan
Checks (use clan read data / read chain / validate / unzip -l on the final):
- hop1_data_survived: are single_minded_proposition, budget_eur, persona STILL in the final data verbatim?
- hop2_data_survived: is concept_names ["Dawn","Halo","Beacon"] still present?
- asset_carried: is human/assets/logo-mark.svg present in the final (it was only passed via --assets at hop 1)?
- lineage_unbroken: does read chain show all 3 hops attributed (account-planner, creative-director, pitch-lead)?
- final_validates: does clan validate pass?
Return JSON with those 5 booleans + notes.`,
    { label: 'meta-verify', phase: 'Verify', schema: {
      type: 'object', required: ['hop1_data_survived', 'hop2_data_survived', 'asset_carried', 'lineage_unbroken', 'final_validates'],
      properties: { hop1_data_survived: { type: 'boolean' }, hop2_data_survived: { type: 'boolean' }, asset_carried: { type: 'boolean' },
        lineage_unbroken: { type: 'boolean' }, final_validates: { type: 'boolean' }, notes: { type: 'string' } } } })
  return v
}

// ===================== L-H3 cold resume =====================
async function runResume() {
  const doc = `${H}/lite/h3-clan/work/doc.clan`
  const v = await agent(
    `You are taking over an in-progress analysis. Everything known is in ${doc}. Repo root is cwd; a 'clan' CLI is on PATH.
Work out where the analysis stands and complete the NEXT step only (one more analyst hop), then record your handoff.
Before your first productive write, orient yourself from the artifact alone.
${RECEIPT}  Receipts dir: ${H}/lite/h3-clan/  Set role to "resume-agent".
In context_understood, state what stage you found it at and what the correct next step was. In problems_and_friction, count how many read/probe commands you needed before your first write. Report concisely.`,
    { label: 'h3-resume', phase: 'Resume' })
  // judge orientation quality from the receipt + artifact
  const judge = await agent(
    `Judge a COLD-RESUME attempt. Repo root is cwd; 'clan' on PATH. Artifact now: ${H}/lite/h3-clan/work/doc.clan
The resume agent's receipt is in ${H}/lite/h3-clan/receipts/ (read it). Inspect the artifact (clan read data/chain/validate).
Return JSON: { oriented_correctly (bool — did it identify the right next step?), n_orientation_reads (int from receipt), produced_valid_write (bool), redid_prior_work (bool), decision_recorded (bool), notes }`,
    { label: 'h3-judge', phase: 'Verify', schema: {
      type: 'object', required: ['oriented_correctly', 'produced_valid_write', 'redid_prior_work', 'decision_recorded'],
      properties: { oriented_correctly: { type: 'boolean' }, n_orientation_reads: { type: 'integer' }, produced_valid_write: { type: 'boolean' },
        redid_prior_work: { type: 'boolean' }, decision_recorded: { type: 'boolean' }, notes: { type: 'string' } } } })
  return judge
}

// ===================== orchestrate (4 independent flows in parallel) =====================
const [forkmerge, unguided, metaResult, resume] = await parallel([
  () => runForkMerge(),
  () => runUnguided(),
  () => runMeta(),
  () => runResume(),
])

return { forkmerge, unguided, meta: metaResult, resume }

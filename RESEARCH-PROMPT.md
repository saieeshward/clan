# CLAN Research Experiment — Claude Code Prompt

## Context (read before running)

This prompt runs a controlled experiment comparing two approaches to multi-agent document generation:

- **Track A**: CLAN pipeline — agents pass `.clan` files; context, provenance, and human rendering travel together
- **Track B**: Baseline pipeline — agents pass plain text/markdown/HTML files (industry standard)

Both tracks produce the same document: **"How to Invest €100 in Dublin, Ireland"**.

The output is a **technical report** formatted as a `.clan` file, comparing the two approaches across token usage, wall-clock time, context fidelity, and output quality.

All agents are Claude Code subagents (no external API key needed). Timing and token counts are recorded at each hop.

---

## Prerequisites

Before running, verify these are available in the working directory:

```bash
# From the clan/ repo root:
./target/release/clan --version   # clan CLI v1.0.0
ls app/dist/index.html             # Tauri frontend built
```

If the release binary doesn't exist:
```bash
. "$HOME/.cargo/env" && cargo build --release -p clan-cli
```

---

## The Prompt (paste this into Claude Code)

---

You are the **Orchestrator** for a research experiment. Your job is to run two parallel multi-agent pipelines, record their behaviour, and produce a technical research report as a `.clan` file.

### Experiment Goal

Produce a high-quality document: **"How to Invest €100 in Dublin, Ireland"**

This document should cover:
- Available investment vehicles accessible to Irish residents (ETFs, index funds, stocks, savings accounts, pension contributions)
- Relevant Irish platforms (DEGIRO, Trading212, Revolut, Zurich, etc.)
- Key Irish tax and regulatory context (exit tax 41%, deemed disposal, CGT, Revenue reporting) — note this is a simulation, not financial advice
- Practical recommendations for a €100 starting amount
- Risk profile options (conservative / balanced / growth)

---

### Track A: CLAN Pipeline

Run these 5 agents **sequentially**. Each agent receives a `.clan` file and produces the next one. Record the wall-clock time and estimated token count for each hop.

**Setup**: Create the initial `.clan` file:
```bash
./target/release/clan create \
  --title "How to Invest €100 in Dublin, Ireland" \
  --brief "Produce a comprehensive, practical guide for investing €100 in Dublin, Ireland. Cover investment vehicles, Irish platforms, Irish tax rules, and practical recommendations for conservative/balanced/growth risk profiles. This is a simulation for research purposes." \
  --doc-type investment-guide \
  /tmp/invest-v0.clan
```

**Agent 1 — Research Agent**
- Read context: `./target/release/clan read agent /tmp/invest-v0.clan`
- Task: Research investment options available to Irish residents. Populate `shared/data.yaml` with structured facts: platforms, vehicles, tax_rules, typical_returns. Produce a `data-update` output.
- Pack: `./target/release/clan pack --output /tmp/invest-v1.clan --delta "Research complete: platforms, vehicles, tax rules structured" /tmp/invest-v0.clan /tmp/agent1-output.json`

**Agent 2 — Regulatory Agent**
- Read context: `./target/release/clan read agent /tmp/invest-v1.clan`  
- Task: Verify and expand the Irish regulatory and tax data. Add exit_tax_rate, deemed_disposal_rules, cgt_rate, revenue_requirements to the structured data. Produce a `data-update` output. Pin this decision (it's a critical compliance checkpoint).
- Pack: `./target/release/clan pack --output /tmp/invest-v2.clan --delta "Regulatory review complete: tax and compliance data verified" /tmp/invest-v1.clan /tmp/agent2-output.json`

**Agent 3 — Strategy Agent**
- Read context: `./target/release/clan read agent /tmp/invest-v2.clan`
- Task: Using the structured research and regulatory data, produce a specific investment strategy for €100 across three risk profiles (conservative, balanced, growth). Add recommendations, allocation_percentages, expected_1yr_return to the data. Produce a `data-update` output.
- Pack: `./target/release/clan pack --output /tmp/invest-v3.clan --delta "Strategy complete: three risk profiles with allocations" /tmp/invest-v2.clan /tmp/agent3-output.json`

**Agent 4 — Writer Agent**
- Read context: `./target/release/clan read agent /tmp/invest-v3.clan`
- Task: Using all structured data from previous agents, write a polished, rich HTML document for human readers. Use `full-html` output mode. The HTML should be well-structured, styled with CSS (dark theme, professional), and include: an executive summary, investment options table, risk profile cards, tax considerations section, and a getting-started checklist.
- Pack: `./target/release/clan pack --output /tmp/invest-v4.clan --delta "Writer complete: full HTML document produced" /tmp/invest-v3.clan /tmp/agent4-output.json`

**Agent 5 — QA Agent**
- Read context: `./target/release/clan read agent /tmp/invest-v4.clan`
- Task: Review the document for accuracy, completeness, and clarity. Check that all three risk profiles are addressed, tax rules are mentioned, and platforms are named. Produce a `data-update` with a `quality_score` (0-1), `issues_found` (list), and `verdict` (approved/needs-revision). Pin this decision.
- Pack: `./target/release/clan pack --output /tmp/invest-final.clan --delta "QA complete: document reviewed and signed off" /tmp/invest-v4.clan /tmp/agent5-output.json`

**Validate final file**:
```bash
./target/release/clan validate /tmp/invest-final.clan
./target/release/clan info /tmp/invest-final.clan
```

---

### Track B: Baseline Pipeline (no CLAN)

Run the **same 5 agents** with the same tasks, but context is passed as plain files — the industry standard approach.

**Setup**: Create a working directory:
```bash
mkdir -p /tmp/baseline
```

**Agent 1 — Research Agent (Baseline)**
- Input: A plain text brief (write it inline)
- Output: `/tmp/baseline/research.md` — markdown document with investment options, platforms, and data
- No structured schema. Agent decides format.
- Record time and estimated tokens.

**Agent 2 — Regulatory Agent (Baseline)**
- Input: Read `/tmp/baseline/research.md` as context
- Output: `/tmp/baseline/regulatory.md` — appends/amends regulatory info
- Record time and tokens.

**Agent 3 — Strategy Agent (Baseline)**
- Input: Read `/tmp/baseline/research.md` + `/tmp/baseline/regulatory.md`
- Output: `/tmp/baseline/strategy.md`
- Record time and tokens.

**Agent 4 — Writer Agent (Baseline)**
- Input: Read all three markdown files as context
- Output: `/tmp/baseline/document.html` — full HTML document
- Record time and tokens.

**Agent 5 — QA Agent (Baseline)**
- Input: Read `/tmp/baseline/document.html`
- Output: `/tmp/baseline/qa-report.txt` — plain text QA verdict
- Record time and tokens.

---

### Measurement Protocol

For each agent in both tracks, record:

| Metric | How to measure |
|--------|---------------|
| Wall-clock time | `time` bash builtin or `date +%s%N` before/after |
| Estimated input tokens | `wc -w` on the context passed × 1.3 (rough word-to-token ratio) |
| Estimated output tokens | `wc -w` on the agent's output × 1.3 |
| Context size (bytes) | `wc -c` on the full context string |
| Provenance available? | Yes if decision chain exists, No otherwise |
| Human view at this stage? | Yes/No |

Record all measurements in a structured format as you go.

---

### Report Generation

After both tracks complete, produce a **technical research report** as a `.clan` file.

**Create the report container**:
```bash
./target/release/clan create \
  --title "CLAN vs Baseline: Multi-Agent Pipeline Comparison" \
  --brief "Technical research report comparing CLAN-based and baseline multi-agent pipelines for document generation. Includes quantitative metrics (tokens, time, context size) and qualitative assessment (provenance, reproducibility, human readability at each stage)." \
  --doc-type research-report \
  /tmp/report-v0.clan
```

**Report Writer Agent**:
Using all measurements from both tracks, produce a `full-html` agent output that generates the research report. The report must contain:

1. **Abstract** (~150 words): What was tested, key finding
2. **Methodology**: Pipeline design, agent roles, measurement approach
3. **Results Table**: Side-by-side metrics for each agent hop (CLAN vs Baseline)
4. **Findings**:
   - Token efficiency: total tokens consumed, per-hop breakdown
   - Time: total wall time, per-hop breakdown  
   - Context fidelity: how much context was lost/degraded between hops
   - Provenance: what can be audited in each approach
   - Human readability: when in the pipeline could a human view the document
   - Structured data: was schema enforced? Did data drift between agents?
5. **Discussion**: Strengths and limitations of each approach
6. **Conclusion**: When would you use CLAN vs the baseline?
7. **Appendix**: Raw measurement data

Pack the report:
```bash
./target/release/clan pack \
  --output /tmp/clan-research-report.clan \
  --delta "Research report complete: CLAN vs baseline comparison" \
  /tmp/report-v0.clan /tmp/report-output.json
```

**Validate and open**:
```bash
./target/release/clan validate /tmp/clan-research-report.clan
./target/release/clan info /tmp/clan-research-report.clan
# Export human view:
./target/release/clan read human /tmp/clan-research-report.clan > /tmp/clan-research-report.html
open /tmp/clan-research-report.html   # macOS
```

---

### Deliverables

When the experiment is complete, report back with:

1. `/tmp/invest-final.clan` — the investment guide produced by the CLAN pipeline
2. `/tmp/baseline/document.html` — the investment guide produced by the baseline pipeline  
3. `/tmp/clan-research-report.clan` — the technical report as a CLAN file
4. `/tmp/clan-research-report.html` — the human-readable report
5. A summary table of all measurements

The research report `.clan` file should be openable in the CLAN Viewer app (run `cargo tauri dev` from the `app/` directory, or build with `cargo tauri build`).

---

### Notes for agents

- This is a **simulation** for research purposes. The investment content should be realistic and informative but is not financial advice.
- For CLAN agents: the `clan read agent` command outputs the full assembled context. Pipe it to your reasoning, then write the JSON output to a file.
- For baseline agents: read the previous agent's output files in full as context.
- If an agent produces invalid JSON for the CLAN pack step, fix it and retry — the pack command will tell you what's wrong.
- All agent decision entries should include `agent` (your role name), `action`, and `rationale`.
- Pin (`pinned: true`) the Regulatory Agent and QA Agent decisions in the CLAN track — these are critical checkpoints.

---

*CLAN v1.0 — Context and Live Agent Notation*  
*Experiment design: CLAN Research Team, 2026*

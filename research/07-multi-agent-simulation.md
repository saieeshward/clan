# Multi-Agent Simulation — Full Pipeline Details

Two simulations were run. Both used the CLAN CLI from an isolated workspace.

---

## Simulation 1: 3-Stage Sequential Pipeline

**Workspace**: `/tmp/adtech-ie-sim/`  
**Pipeline**: Market Researcher → Risk Analyst → Lead Partner (Board Memo)  
**Human override**: Partner override applied via `patch-html` after Stage 3

### Stage 0 — Root Document
```bash
clan create \
  --title "Irish AdTech Agency OS: Market Fit Analysis" \
  --brief "Evaluate whether an AI-powered OS for Irish advertising agencies..." \
  ie-adtech.clan
# Output: created ie-adtech.clan (8659 bytes) id=18fdeb31-...
```

### Stage 1 — Market Researcher (JSON path)

Agent output: `researcher-output.json` — full-html mode JSON with structured market data and a complete `<!DOCTYPE html>` report.

Key structured data produced:
- `market_overview`: €1.4B ad spend, 320 agencies, 72% digital share
- `pain_points`: 5 validated pain points
- `competitive_landscape`: 5 competitors
- `willingness_to_pay`: TAM €46M, SAM €11M, SOM €2.8M

```bash
clan pack \
  --delta "Market Researcher completed analysis. €46M TAM, clear whitespace." \
  --output ie-adtech-stage1.clan \
  ie-adtech.clan researcher-output.json
# Output: packed ie-adtech-stage1.clan (13664 bytes)
```

### Stage 2 — Risk Analyst (HTML path, with frontmatter)

Agent output: `risk-analyst.html` — HTML file with YAML frontmatter containing structured risk data.

Key structured data produced:
- `risks`: 5 risks (severity/likelihood/mitigation)
- `go_to_market`: 3-phase GTM strategy
- `overall_risk_rating`: MEDIUM
- `recommended_verdict`: PROCEED

```bash
clan pack-html \
  --delta "Risk Analyst: MEDIUM risk rating. PROCEED verdict. GTM phased." \
  --output ie-adtech-stage2.clan \
  ie-adtech-stage1.clan risk-analyst.html
# Output: packed ie-adtech-stage2.clan (14375 bytes)
```

### Stage 3 — Lead Partner Board Memo (HTML path)

Agent output: `board-memo.html` — premium two-column board memo with sidebar navigation, investment thesis, milestones, use-of-funds.

Key structured data produced:
- `document_class`: Investment Recommendation
- `ask_eur`: 750000
- `valuation_cap_eur`: 4500000
- `verdict`: INVEST
- `use_of_funds`: 45% product, 35% GTM, 20% ops

```bash
clan pack-html \
  --delta "Lead Partner: INVEST recommendation. €750K at €4.5M cap." \
  --output ie-adtech-final.clan \
  ie-adtech-stage2.clan board-memo.html
# Output: packed ie-adtech-final.clan (15391 bytes)
```

### Human Override — Partner `patch-html`

```bash
clan patch-html ie-adtech-final.clan - << 'EOF'
---
mode: patch-html
patch_selector: "[data-adf-id='exec-summary']"
patch_action: replace
---
<p data-adf-id="exec-summary" style="...">
  <strong>Partner Override (Human Review — 1 Jun 2026):</strong> 
  Following board discussion, recommend milestone-linked tranche structure. 
  €450K at signing; €300K on 15 signed clients (Month 6).
</p>
EOF
# Output: Patched ie-adtech-final.clan in-place
```

### Validation & Export

```bash
clan validate ie-adtech-final.clan  # → OK
clan export-static --output ie-adtech-export.json ie-adtech-final.clan
# Keys: agent_guide, clan_version, decision_history_toon, output_schema, patches, shared_data, task
```

### Decision Chain (final state)

```yaml
decisions:
  - agent: lead-partner
    action: "Issued INVEST recommendation for AgencyOS Ireland pre-seed round"
    rationale: "Two-stage analysis confirms product-market fit..."
    pinned: true
    fields_changed: [ask_eur, ask_type, author, company_name, ...]
  - agent: risk-analyst
    action: "Completed risk assessment for Irish AdTech OS"
    fields_changed: [analyst, go_to_market, overall_risk_rating, risks, ...]
  - agent: market-researcher
    action: "Completed Irish AdTech OS market research"
    fields_changed: [analysis_title, competitive_landscape, market_overview, pain_points, ...]
  - agent: unknown-agent
    action: "processed document"
```

### File sizes produced

| File | Size |
|---|---|
| `ie-adtech.clan` (root) | 8.5KB |
| `ie-adtech-stage1.clan` | 13.3KB |
| `ie-adtech-stage2.clan` | 14.0KB |
| `ie-adtech-final.clan` | 15.5KB |
| `ie-adtech-export.json` | 15.4KB |

---

## Simulation 2: 6-Agent Fan-Out Pipeline

**Workspace**: `/tmp/adtech-ie-deep/`  
**Pipeline**: Root → 4 parallel specialists → Synthesis (chained from financial branch)

### Architecture

```
root.clan
├── branch-financial-v2.clan    (Financial Analyst)
│   └── synthesis.clan          (Synthesis Lead, chained here)
├── branch-competitive.clan     (Competitive Intel Analyst)
├── branch-customer-v2.clan     (Customer Discovery Analyst)
└── branch-regulatory-v2.clan   (Regulatory & Product Analyst)
```

The parallel branches all share the same parent (`root.clan`). Each produces an independent `.clan` file. The synthesis agent reads `export-static` from all four branches and chains its output from the financial branch.

### Root Document

```bash
clan create \
  --title "AgencyOS Ireland — Deep Investment Analysis" \
  --brief "Multi-specialist deep-dive analysis..." \
  root.clan
# Output: created root.clan (8713 bytes) id=2afdaaa4-...
```

### Parallel Branch Agents (run simultaneously)

All 4 agents received the same instruction: `clan read agent root.clan`, produce a magazine-quality HTML report with YAML frontmatter, then `clan pack-html`.

#### Branch A: Financial Analyst
- HTML: `financial.html` (54,588 chars)
- Structured data: pricing tiers, unit economics, 5-year ARR projection, funding ask
- SVG assets: bar chart (Y1–Y5 ARR), donut chart (use of funds)
- First pack: flat frontmatter (bug) → repacked as `branch-financial-v2.clan`
- Final size: 18,943 bytes

#### Branch B: Competitive Intel Analyst  
- HTML: `competitive.html` (38,547 chars)
- Structured data: 5 direct competitors, 2 adjacent threats, 4 moat factors
- SVG assets: positioning quadrant chart, threat timeline
- Packed correctly on first attempt: `branch-competitive.clan`
- Size: 18,119 bytes

#### Branch C: Customer Discovery Analyst
- HTML: `customer.html` (45,367 chars)
- Structured data: 3 personas, 5 interview themes, 4 JTBD statements, 3 adoption blockers
- SVG assets: WTP heatmap, interview themes chart
- First pack: flat frontmatter (bug) → repacked as `branch-customer-v2.clan`
- Final size: 20,934 bytes

#### Branch D: Regulatory & Product Analyst
- HTML: `regulatory-product.html` (43,263 chars)
- Structured data: EU AI Act compliance (5 requirements), GDPR requirements, product architecture, MVP scope
- SVG assets: GDPR data flow diagram, tech stack architecture, compliance timeline
- First pack: flat frontmatter (bug) → repacked as `branch-regulatory-v2.clan`
- Final size: 19,778 bytes

### The Frontmatter Bug — Discovery During This Simulation

3 of 4 agents wrote valid YAML frontmatter but without the `structured:` wrapper key:

**What agents wrote (wrong)**:
```yaml
---
stage: "Financial Modeling"
analyst: "Financial Analyst"
pricing_tiers:
  - name: Starter
    ...
decision:
  agent: "financial-analyst"
  ...
---
```

**What CLAN requires**:
```yaml
---
structured:
  stage: "Financial Modeling"
  analyst: "Financial Analyst"
  pricing_tiers:
    - name: Starter
      ...
decision:
  agent: "financial-analyst"
  ...
---
```

**Result**: `clan pack-html` accepted the HTML, applied the decision entry, but silently discarded all structured data. `clan read data` returned only `$schema`.

**Fix applied**: Python script to prepend `structured:` and indent YAML body, then repack all three branches. All v2 branches confirmed correct via `clan read data`.

### Synthesis Agent (chained from financial-v2 branch)

```bash
clan pack-html \
  --delta "Synthesis Lead: All 4 branches converge on STRONG INVEST. Confidence 87/100." \
  --output synthesis.clan \
  branch-financial-v2.clan \
  synthesis.html
# Output: packed synthesis.clan (22935 bytes)
clan validate synthesis.clan  # → OK
```

Synthesis document includes:
- 4 convergence cards (K1–K4) with cross-branch insight
- SVG confidence arc gauge (87/100)
- 3×3 SVG decision matrix
- Side-by-side evidence panels from customer + financial branches
- Critical risks section with convergence evidence

### Complete Pipeline Stats

| Metric | Value |
|---|---|
| Total agents (including failed competitive retry) | 6 |
| Total `.clan` files produced | 10 |
| Total HTML produced | ~235KB raw |
| All `clan validate` passed | Yes (all valid files) |
| Structured data correctly packed (first try) | 1 of 4 specialist branches |
| Structured data correctly packed (after fix) | 4 of 4 |
| End-to-end wall time | ~45 minutes (including API call time) |

---

## Observations on CLAN's Multi-Agent Behavior

### What worked exactly as designed
1. Each agent's `clan read agent` returned clean context — guide + task + data + chain
2. The synthesis agent could call `export-static` on each branch to get clean JSON
3. Decision chain recorded all agents correctly
4. Validation passed on every correctly-formed file
5. The parallel branches were completely independent — no interference between agents

### What required intervention
1. **Frontmatter format trap**: 75% of agents got the `structured:` key wrong on first try without explicit example in the prompt
2. **Competitive agent API overload**: Had to retry one agent due to Anthropic API overload (not a CLAN issue)
3. **Branch chaining choice**: The synthesis agent had to pick one branch as parent (chose financial-v2) — CLAN has no native multi-parent merge

### What CLAN made significantly easier
1. **No orchestration code**: Zero Python/Node orchestration written for either simulation
2. **No context assembly**: No code to merge prior agent outputs
3. **No schema validation**: `clan pack` caught missing fields automatically
4. **No lineage tracking**: Every decision automatically recorded

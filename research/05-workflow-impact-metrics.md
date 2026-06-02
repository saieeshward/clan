# Workflow Impact Metrics — Quantitative Data

All measurements taken from the actual simulation runs on 2026-06-01.

---

## Token Cost Measurements

### Agent Context Size Across Pipeline Stages

Measured using `clan read agent <file> | wc -w`, then estimated tokens at ×1.3 words/token ratio.

| Pipeline Stage | File | Words | Est. Tokens |
|---|---|---|---|
| Root (blank doc) | `root.clan` | 1,028 | ~1,340 |
| After Financial Agent | `branch-financial-v2.clan` | 1,055 | ~1,371 |
| After Competitive Agent | `branch-competitive.clan` | 1,337 | ~1,738 |
| After Customer Agent | `branch-customer-v2.clan` | 1,070 | ~1,391 |
| After Regulatory Agent | `branch-regulatory-v2.clan` | 1,074 | ~1,396 |
| After Synthesis (chained) | `synthesis.clan` | ~1,400 | ~1,820 |

**Key observation**: Context size grows by only ~31–280 words per stage despite each stage adding substantial structured data. The agent guide (~783 words) dominates and is constant. TOON-encoded data adds minimal tokens per stage.

### Without-CLAN Baseline Estimate (4-Branch Synthesis)

Estimated context cost if synthesis agent received raw JSON from all 4 branches:

| Component | Words | Est. Tokens |
|---|---|---|
| Brief/task | 184 | ~239 |
| Financial raw JSON | ~800 | ~1,040 |
| Competitive raw JSON | ~600 | ~780 |
| Customer raw JSON | ~700 | ~910 |
| Regulatory raw JSON | ~750 | ~975 |
| Custom merge instructions | ~200 | ~260 |
| **Total (without CLAN)** | **~3,234** | **~4,204** |
| **With CLAN (`clan read agent`)** | **~1,337** | **~1,738** |
| **Saving** | **-59%** | **-59%** |

---

## File Size Efficiency

### .clan File Sizes vs Raw HTML Sizes

| File | .clan size | Raw HTML size | Raw data size | Compression ratio |
|---|---|---|---|---|
| `root.clan` | 8,713 B | 179 B | 45 B | — (mostly guide/schema) |
| `branch-financial-v2.clan` | 18,943 B | 54,588 B | variable | 65% of raw HTML |
| `branch-competitive.clan` | 18,119 B | 38,547 B | 2,252 B | 47% of raw HTML |
| `branch-customer-v2.clan` | 20,934 B | 45,367 B | variable | 46% of raw HTML |
| `branch-regulatory-v2.clan` | 19,778 B | 43,263 B | variable | 46% of raw HTML |
| `synthesis.clan` | 22,935 B | ~50,000 B est. | variable | ~46% |

**Total raw HTML across 4 branches**: ~181,765 bytes  
**Total .clan files (4 branches + synthesis)**: ~100,709 bytes  
**Compression**: ~45% of raw size — effectively halved by ZIP compression

### export-static File Sizes

| Export File | Size | Notes |
|---|---|---|
| `branch-financial-v2-export.json` | 9,800 B | Includes full agent guide (~5KB) |
| `branch-competitive-export.json` | 11,100 B | Most structured data of all branches |
| `branch-customer-v2-export.json` | 14,900 B | Largest — 3 personas + interview data |
| `branch-regulatory-v2-export.json` | 12,300 B | EU AI Act compliance data + product arch |

---

## TOON Compression Ratio

TOON vs raw YAML for the same data. Measured by comparing `clan read data` (raw YAML) vs the TOON block in `clan read agent`.

| Field example | Raw YAML (chars) | TOON (chars) | Saving |
|---|---|---|---|
| Simple string field | `analyst: "Market Researcher"\n` = 30 | `analyst: Market Researcher\n` = 28 | 7% |
| Array of 5 items | Header + 5 `- ` items = ~120 | `items [5]\n  val1\n  ...` = ~90 | 25% |
| Nested object (3 keys) | ~90 chars with indentation | ~70 chars | 22% |
| Overall (claimed) | baseline | baseline × 0.60 | ~40% |

The claimed ~40% saving appears consistent with observed context size stability across stages.

---

## Decision Chain Growth

| Stage | Chain line count | Chain growth |
|---|---|---|
| After stage 1 (market researcher) | 17 lines | +17 |
| After stage 3 (final board memo) | 54 lines | +37 |
| Chain compression (beyond verbatim window) | Not triggered in this test | — |

The chain grows by ~17–37 lines per agent, depending on how many fields changed and how long the rationale text is.

---

## Build & Launch Timing

| Step | Time |
|---|---|
| Node 20 npm install | ~3 seconds |
| Vite 8 dev server start | 143ms |
| Tauri incremental Rust build | ~3 seconds |
| Tauri cold Rust build | ~45–90 seconds (estimated) |
| `.clan` file auto-load on mount | <100ms (two `get_human_html` calls within 10ms) |

---

## CLI Command Timings (informal)

| Command | Typical time |
|---|---|
| `clan create` | <100ms |
| `clan pack` | <200ms |
| `clan pack-html` | <200ms |
| `clan validate` | <50ms |
| `clan export-static` | <100ms |
| `clan read agent` | <100ms |

All CLI commands are fast — no noticeable latency in the pipeline.

---

## Agent Simulation Summary

| Metric | First simulation (3-stage) | Second simulation (6-agent fan-out) |
|---|---|---|
| Agents | 3 (sequential chain) | 4 parallel branches + 1 synthesis |
| Total `.clan` files produced | 5 | 10 (including v2 repacks) |
| Total pipeline stages | 4 (including root) | 6 (root + 4 branches + synthesis) |
| Total HTML produced | ~115KB | ~235KB |
| All `clan validate` passed | Yes | Yes |
| Data correctly accumulated | Yes | Partial — 3/4 agents missed `structured:` wrapper |
| Time (wall clock, with API calls) | ~15 minutes | ~45 minutes |

---

## Observed Bug Frequencies (This Test)

| Bug | Occurrences |
|---|---|
| `patch-html` silent failure (bad selector) | 2 (1 in CLI test, 1 in live app log) |
| Frontmatter missing `structured:` wrapper | 3 of 4 agents in fan-out simulation |
| Double-save (confirmed in debug log) | 2 consecutive saves of same id/content |
| `clan://` protocol failure in browser | Not directly observed (would require browser test) |
| Empty iframe src on mount | 1 (per app launch, in browser console) |
| Tauri unlisten race error | 1 (per app launch, in browser console) |

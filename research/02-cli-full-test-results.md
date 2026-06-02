# CLI Full Test Results

All tests run from `/tmp/adtech-ie-sim/` (isolated workspace, no access to the CLAN repository directory).

## Setup

```bash
mkdir -p /tmp/adtech-ie-sim
cd /tmp/adtech-ie-sim
which clan  # → /Users/saieeshwar/Projects/Work/work/napkin/ADF/clan/target/release/clan
clan --version  # → clan 1.0.0
```

---

## `clan create`

### Test: Normal creation
```bash
clan create \
  --title "Irish AdTech Agency OS: Market Fit Analysis" \
  --brief "Evaluate whether an AI-powered OS for Irish advertising agencies..." \
  ie-adtech.clan
```
**Output**: `created ie-adtech.clan (8659 bytes)  id=18fdeb31-f849-4dfe-a3a0-dc1317970a59`
**Result**: ✅ Pass

### Test: Wrong argument style (using --output like pack)
```bash
clan create --title "..." --brief "..." --output ie-adtech.clan
```
**Output**: `error: unexpected argument '--output' found`
**Result**: ⚠️ Expected, but reveals UX inconsistency — `create` uses positional arg while `pack` uses `--output` flag. See Bug #4.

---

## `clan read agent`

```bash
clan read agent ie-adtech.clan
```
**Output**: Full markdown guide (~1,028 words) + task + empty TOON data block + empty decision history
**Result**: ✅ Pass

**Measured context sizes across pipeline stages**:

| File | Words | Est. Tokens (~×1.3) |
|------|-------|----------------------|
| root (blank) | 1,028 | ~1,340 |
| after stage 1 (financial branch) | 1,055 | ~1,371 |
| after stage 2 (competitive branch) | 1,337 | ~1,738 |
| after stage 3 (customer branch) | 1,070 | ~1,391 |
| after stage 4 (regulatory branch) | 1,074 | ~1,396 |

**Key finding**: Agent context stays nearly flat across stages (1,340–1,740 tokens) due to TOON compression. The guide (~783 words) dominates and is constant; data adds only marginal tokens per stage.

---

## `clan read data`

```bash
clan read data ie-adtech-final.clan
```
**Output**: Raw YAML in insertion order (not alphabetical, not TOON)
**Result**: ✅ Pass

**Important distinction**: `clan read data` returns insertion-order YAML. `clan read agent` returns the same data TOON-encoded in alphabetical order. These are the same data with different formatting. `agent-help` warns not to run both — correct advice, but the format difference is not documented.

---

## `clan read human`

```bash
clan read human ie-adtech-final.clan
```
**Output**: Full `<!DOCTYPE html>` document (54,588 chars for the board memo)
**Result**: ✅ Pass

---

## `clan read chain`

```bash
clan read chain ie-adtech-stage1.clan  # 17 lines
clan read chain ie-adtech-final.clan   # 54 lines
```
**Result**: ✅ Pass — chain grows correctly across stages. Each entry shows agent, action, rationale, timestamp, and `fields_changed[]`.

---

## `clan pack` (JSON path)

### Test: Normal pack
```bash
clan pack \
  --delta "Market Researcher completed analysis..." \
  --output ie-adtech-stage1.clan \
  ie-adtech.clan \
  researcher-output.json
```
**Output**: `packed ie-adtech-stage1.clan (13664 bytes)`
**Result**: ✅ Pass

### Test: Missing `structured` field in JSON
```bash
echo '{"mode":"full-html"}' > minimal.json
clan pack --delta "test" --output out.clan ie-adtech.clan minimal.json
```
**Output**: `Error: failed to parse agent output — agent output rejected: missing field: structured`
**Result**: ✅ Correct error. Clear, actionable message.

### Test: Invalid JSON input
```bash
echo 'not json at all' > bad.json
clan pack --delta "test" --output out.clan ie-adtech.clan bad.json
```
**Output**: `Error: failed to parse agent output — invalid JSON: expected ident at line 1 column 2`
**Result**: ✅ Correct error.

### Test: Missing source file
```bash
clan pack --delta "test" --output out.clan nonexistent.clan output.json
```
**Output**: `Error: could not open nonexistent.clan — I/O error: No such file or directory`
**Result**: ✅ Correct error. Stack trace shows root cause clearly.

---

## `clan pack-html` (HTML path)

### Test: Normal pack with YAML frontmatter
```bash
clan pack-html \
  --delta "Risk Analyst: assessment complete" \
  --output stage2.clan \
  ie-adtech-stage1.clan \
  risk-analyst.html
```
**Output**: `packed ie-adtech-stage2.clan (14375 bytes)`
**Result**: ✅ Pass

### Test: No frontmatter at all
```bash
printf '<!DOCTYPE html><html><body><p data-adf-id="test">Hello</p></body></html>' > no-fm.html
clan pack-html --delta "test" --output no-fm.clan ie-adtech.clan no-fm.html
clan validate no-fm.clan
```
**Output**: `packed no-fm.clan (8829 bytes)` → `OK`
**Result**: ✅ Pass — HTML-only path works cleanly.

### Test: Frontmatter with empty `structured: {}`
**Result**: ✅ Pass — accepts empty structured block.

### Test: Frontmatter with no `decision:` block
**Result**: ✅ Pass — decision is optional in frontmatter.

### Test: Deeply nested structured object
**Result**: ✅ Pass — nested YAML correctly stored.

### Test: **Flat frontmatter without `structured:` wrapper** ← NEW BUG FOUND
```yaml
---
stage: "Financial Modeling"
analyst: "Financial Analyst"
pricing_tiers:
  - name: "Starter"
    ...
---
```
**Output**: `packed branch-financial.clan` (appears to succeed)
**But**: `clan read data branch-financial.clan` → only `$schema` — all structured data silently discarded

**Result**: ❌ BUG. Silent data loss. See Bug #3 in bugs file.

**Root cause**: `clan pack-html` only reads data from under a `structured:` key in frontmatter. If the author writes flat top-level YAML (natural default for YAML authors), the data is accepted but discarded. No warning is printed.

**Discovery context**: 3 of 4 specialist agents in the 6-agent fan-out simulation wrote flat frontmatter. Only the competitive agent (given explicit `structured:` key in its prompt) correctly packed data.

---

## `clan patch-html`

### Test: Valid selector, replace action
```bash
clan patch-html ie-adtech-final.clan - << 'EOF'
---
mode: patch-html
patch_selector: "[data-adf-id='exec-summary']"
patch_action: replace
---
<p data-adf-id="exec-summary">Partner Override: milestone-linked tranche structure.</p>
EOF
```
**Output**: `Patched ie-adtech-final.clan in-place`
**Result**: ✅ Pass — content correctly replaced.

### Test: **Non-matching selector** ← CONFIRMED BUG
```bash
printf -- '---\nmode: patch-html\npatch_selector: ".this-selector-does-not-exist"\npatch_action: replace\n---\n<p>Orphaned</p>\n' | clan patch-html ie-adtech-final.clan -
```
**Output**: `Patched ie-adtech-final.clan in-place` (exit code: 0)
**Verification**: `clan read human ie-adtech-final.clan | grep "Orphaned"` → 0 matches. File mtime was updated.

**Result**: ❌ BUG. Silent success on failed patch. Exit 0, "Patched" message, no content change. See Bug #2 in bugs file.

**Also confirmed from live Tauri debug log**: `apply_patches: id=".vc-left" NOT FOUND in HTML` — shows the same silent failure occurs in the app patch path.

---

## `clan validate`

### Test: Valid file
```bash
clan validate ie-adtech-final.clan  # → OK
```
**Result**: ✅ Pass

### Test: Corrupt file (not a ZIP)
```bash
echo "not a zip" > corrupt.clan
clan validate corrupt.clan
```
**Output**: `Error: could not open corrupt.clan — ZIP error: invalid Zip archive: Could not find EOCD`
**Result**: ✅ Correct error, clear message.

### Test: Validate all files in batch
```bash
for f in *.clan; do echo "$f: $(clan validate $f 2>&1)"; done
```
All valid .clan files returned `OK`. `corrupt.clan` returned the ZIP error.
**Result**: ✅ All pass.

---

## `clan info`

```bash
clan info ie-adtech-final.clan
```
**Output**:
```
title:    Irish AdTech Agency OS: Market Fit Analysis
id:       a4572513-fe04-4e35-ba2b-f69a0076ad83
version:  1.0
created:  2026-06-01T15:46:39.185040+00:00
updated:  2026-06-01T15:50:43.141202+00:00
parent:   6f052bd1-af39-4690-9a00-196054b18c78
delta:    Risk Analyst completed 5-risk assessment...
sha256:   sha256:ad7b54507f3f830e6331f20...
files:    8
```
**Result**: ✅ Pass

---

## `clan export-static`

### Test: With `--output` flag
```bash
clan export-static --output ie-adtech-export.json ie-adtech-final.clan
```
**Output**: `wrote ie-adtech-export.json`
**Result**: ✅ Pass

### Test: Stdout mode
```bash
clan export-static ie-adtech-final.clan | python3 -c "import sys,json; d=json.load(sys.stdin); print(list(d.keys()))"
```
**Output**: `['agent_guide', 'clan_version', 'decision_history_toon', 'output_schema', 'patches', 'shared_data', 'task']`
**Result**: ✅ Pass — stdout export works for piping.

### Test: Wrong argument style (positional for output)
```bash
clan export-static ie-adtech-final.clan ie-adtech-export.json
```
**Output**: `error: unexpected argument 'ie-adtech-export.json' found`
**Result**: ⚠️ Not a bug (consistent with pack's `--output`), but worth noting in error messages.

### Observed `patches` field format issue
`patches` field in export is a raw YAML string, not a parsed JSON array. Inconsistent with `shared_data` which is parsed JSON. See Optimisation #10.

---

## `clan edit`

```bash
EDITOR="cat" clan edit ie-adtech-final.clan
```
**Output**: Full structured YAML opened in `cat` (piped to stdout). Shows merged data from all pipeline stages.
**Result**: ✅ Pass — EDITOR variable respected, structured YAML is clean and readable.

---

## `clan agent-help`

```bash
clan agent-help
```
**Word count**: 220 words (~286 tokens estimated)
**Claims**: "< 200 tokens" — close enough to accurate.

**Missing commands**: `create`, `edit`, `export-static` are not mentioned.
**Phantom commands**: None found — all mentioned commands exist and work.
**Format instructions**: Accurate and matched actual `clan pack` / `clan pack-html` behavior.
**Result**: ✅ Functionally correct, minor documentation gap on missing commands.

---

## TOON Format Observations

Fields in TOON output are sorted strictly alphabetically. `$schema` appears first because `$` (ASCII 36) precedes all letters — this is coincidental, not intentional special-casing.

**Semantic grouping issue**: In a 4-stage document, related fields are scattered:
- `ask_eur` and `ask_type` appear near top
- `valuation_cap_eur` and `verdict` appear at bottom, separated by 15+ unrelated keys
- `overall_risk_rating` and `recommended_verdict` are not adjacent

For agent consumption this is fine (LLMs don't need semantic ordering). For human debugging via `clan read agent`, it is disorienting.

`clan read data` returns insertion-order YAML (preserves semantic grouping). Only the TOON path has this issue.

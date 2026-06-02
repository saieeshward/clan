# New Patch Commands — Update Test Report

Tested on the updated CLAN binary (still reports `clan 1.0.0`).

---

## What Was Added

Six new surgical patch subcommands:

| Command | Patches | Protocol |
|---|---|---|
| `clan patch-data <file> <json>` | `shared/data.yaml` | RFC 7396 JSON Merge Patch |
| `clan patch-schema <file> <schema.json>` | `agent/output-schema.json` | Replace |
| `clan patch-decision <file> --agent ... --action ... --rationale ...` | `agent/decision-chain.yaml` | Append |
| `clan patch-state <file> <json>` | `agent/state.yaml` | RFC 7396 JSON Merge Patch |
| `clan patch-context <file> <md> [--append]` | `agent/context.md` | Overwrite or append |
| `clan patch-asset <file> <internal_path> <local_file>` | `human/assets/` | Inject/replace |

The `agent-help` was also completely redesigned into structured sections (READ / WRITE / PATCH / VERIFY).

---

## Test Results

| Command | Status | Notes |
|---|---|---|
| `patch-context` (overwrite) | ✅ Works | `Patched context in-place` |
| `patch-context --append` | ✅ Works | Appends correctly |
| `patch-state` | ✅ Works | `Patched state in-place`, merge applied |
| `patch-asset` | ⚠️ Works with security issue | Path traversal not sanitized |
| `patch-data` | ❌ Blocked | `$schema` reference bug |
| `patch-schema` | ❌ Blocked | `$schema` reference bug |
| `patch-decision` | ❌ Blocked | `$schema` reference bug |

---

## Bug 1 (Critical) — `patch-data` / `patch-schema` / `patch-decision` Blocked on Every File

### Error
```
Error: agent output rejected: False schema does not allow "spec/schemas/document.schema.json"
```

### Root cause
Every `.clan` file has `$schema: "spec/schemas/document.schema.json"` as the first line of `shared/data.yaml` (written by `clan create`). When `patch-data`, `patch-schema`, or `patch-decision` run, they validate the data against this reference. The validator tries to fetch `spec/schemas/document.schema.json` from inside the ZIP archive. This file does not exist. The JSON Schema spec says an unresolvable `$schema` reference should be treated as a `false` schema — which rejects everything.

### Scope
Affects **all** `.clan` files, both old (pre-update) and new. Every file created by `clan create` has this `$schema` reference.

### Why the other three commands work
`patch-context`, `patch-state`, and `patch-asset` do not validate against `shared/data.yaml` — they write directly to their respective files without a data schema check.

### No workaround available
The SHA256 per-file integrity check in `manifest.yaml` prevents manually editing the ZIP (validate reports mismatch). The only fix is in the CLAN binary.

### Reproduction
```bash
clan create --title "Test" --brief "Test" test.clan
echo '{"key":"value"}' | clan patch-data test.clan -
# → Error: agent output rejected: False schema does not allow "spec/schemas/document.schema.json"
```

### Fix
The new patch commands should not follow the `$schema` field in `shared/data.yaml` as a JSON Schema reference. Options:
1. Strip `$schema` from the YAML before running validation
2. Only validate against `agent/output-schema.json` (the intended schema)
3. Treat `$schema` in YAML data files as a metadata annotation, not a JSON Schema `$ref`

---

## Bug 2 (High) — New `clan create` Emits `additionalProperties: false` on `structured`

### What changed
New `clan create` output-schema.json:
```json
"structured": {
  "type": "object",
  "additionalProperties": false   ← NEW
}
```

Old output-schema.json:
```json
"structured": {
  "type": "object"
}
```

### Impact
With `additionalProperties: false` and no `properties` defined, **zero fields are allowed** in `structured`. This would block `clan pack` from accepting any structured data (once Bug 1 is fixed). An agent's output like `{"mode":"data-update","structured":{"field":"value"}}` would be rejected because `field` is an additional property on `structured`.

### Intended design (likely)
The `additionalProperties: false` is probably intended to be extended via `patch-schema` — you set the allowed fields explicitly before running agents. But:
1. `patch-schema` is blocked by Bug 1
2. This creates a chicken-and-egg deadlock: you must fix the schema to pack data, but you can't fix the schema because the schema blocks schema-patching

### Reproduced on
`test.clan` created by new `clan create`, confirmed by inspecting `agent/output-schema.json` directly.

---

## Security Issue — `patch-asset` Path Traversal

### Description
`patch-asset`'s `INTERNAL_PATH` argument is stored verbatim in the ZIP without path sanitization:

```bash
clan patch-asset file.clan ../../../spec/schemas/document.schema.json payload.json
```

Inspecting the ZIP afterwards:
```
human/assets/logo.png
human/assets/../../../spec/schemas/document.schema.json   ← written at this literal path
```

The `../` traversal is stored as-is in the ZIP. A tool that normalises paths when extracting would place the file at `spec/schemas/document.schema.json` — outside the intended `human/assets/` scope. Effectively, `patch-asset` can write to `agent/`, `shared/`, `spec/`, or any other path inside the archive.

### Impact
- Allows injecting arbitrary files into sensitive ZIP paths (`agent/context.md`, `shared/data.yaml`, `agent/output-schema.json`)
- Bypasses the intended restriction to `human/assets/`
- Could be used to silently override pipeline instructions or schema

### Fix
Sanitize `INTERNAL_PATH` in `patch-asset`: reject any path containing `..`, prefix with `human/assets/` unconditionally, and normalise the result.

---

## Agent-Help Redesign — Positive Changes

The `agent-help` output was substantially improved in this update.

### Before (old)
- Step-based narrative format (STEP 1, 2, 3, 4)
- Only mentioned `read agent`, `pack`, `pack-html`, `patch-html`
- No mention of `patch-data`, `patch-state`, `patch-context`, etc.
- Buried "OTHER COMMANDS" at the bottom

### After (new)
```
CLAN v1.0.0 AGENT PROTOCOL

# READ
clan read agent <file>    => Context, state, data, history (USE THIS FIRST)
clan read human <file>    => Rendered HTML
clan info <file>          => Manifest/lineage

# WRITE (Full Replace)
1. JSON Mode: clan pack --output <out> [--schema <schema>] <in> <json_file>
2. HTML Mode (Token-efficient): clan pack-html ...

# PATCH (In-place, Lowest Token Cost, Preferred)
1. DOM: clan patch-html
2. Data: clan patch-data  (RFC7396)
3. State: clan patch-state
4. Notes: clan patch-context [--append]
5. History: clan patch-decision
6. Asset: clan patch-asset
7. Schema: clan patch-schema

# VERIFY
clan validate <file>
```

### Improvements
- Structured READ / WRITE / PATCH / VERIFY sections — scannable
- All 7 patch commands documented with syntax
- Documents `--schema` flag on `pack`/`pack-html` (previously undocumented)
- Mentions `window.__CLAN__.data` for JS access to structured data
- References `{{key}}` templating

### Remaining gap
`clan create` still not mentioned. An agent bootstrapping a new pipeline from scratch has no documented path.

---

## Updated Command Status Matrix

| Command | Pre-Update | Post-Update | Change |
|---|---|---|---|
| `create` | ✅ | ✅ | No change |
| `pack` | ✅ | ✅ (old files) / ❌ (new files, Bug 2) | Regression on new files |
| `pack-html` | ✅ | ✅ | No change |
| `patch-html` | ✅ (silent failure bug) | ✅ (silent failure bug) | Bug persists |
| `read agent/human/data/chain` | ✅ | ✅ | No change |
| `validate` | ✅ | ✅ | No change |
| `export-static` | ✅ | ✅ | No change |
| `edit` | ✅ | ✅ | No change |
| `agent-help` | ⚠️ (missing commands) | ✅ (much improved) | Major improvement |
| `patch-context` | — | ✅ | New, working |
| `patch-state` | — | ✅ | New, working |
| `patch-asset` | — | ⚠️ (path traversal) | New, security issue |
| `patch-data` | — | ❌ (Bug 1 blocker) | New, broken |
| `patch-schema` | — | ❌ (Bug 1 blocker) | New, broken |
| `patch-decision` | — | ❌ (Bug 1 blocker) | New, broken |

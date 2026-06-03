# Handoff: CLAN logo (static + live)

## Overview
A complete logo system for **CLAN** — a lineage-graph mark that depicts a *clan of connected nodes* descending from a root (mapping directly to CLAN's DAG / provenance model) and simultaneously reads as a **"C"** monogram. Ships in two forms:

- **Static mark** — for favicon, app icon, titlebar, README header, docs.
- **Live mark** — the lineage grows in, holds, then recedes in a continuous loop, with a pulse ring from the spine node. For the app splash / loading state and the marketing hero.

## About the design files
The files in this bundle are **production-ready design references**, not a drop-in feature. The SVGs are clean and final — you can ship them directly. The `.tsx` is a reference React component matching a Vite + TypeScript setup; **recreate/adapt it using the codebase's existing component conventions** rather than pasting blindly. Where this repo already has placeholder logo files, the task is to **replace them** with these assets (see *Files to replace* below).

## Fidelity
**High-fidelity.** Final geometry, colours, typography, and motion. Reproduce exactly — every node coordinate, radius, and colour stop is specified below and baked into the SVGs.

---

## The mark — exact geometry
Drawn on a **100×100 viewBox**, `fill="none"`. Five nodes (circles) joined by four edges (lines). Edges are drawn *behind* nodes.

| Node | cx | cy | r | generation | fill (dual-tone) |
|------|----|----|----|-----------|------------------|
| 0 (top-right)   | 70 | 24 | 5.6 | 0 | `#6366f1` |
| 1 (top-left)    | 37 | 21 | 5.6 | 1 | `#5682e9` |
| 2 (spine/left)  | 18 | 50 | 7.0 | 2 | `#489de0` |
| 3 (bottom-left) | 37 | 79 | 5.6 | 3 | `#3bb9d8` |
| 4 (bottom-right)| 70 | 76 | 5.6 | 4 | `#2dd4cf` |

**Edges** (node index pairs): `[0,1] [1,2] [2,3] [3,4]`
- stroke `rgba(120,160,230,0.42)`, `stroke-width="2.8"`, `stroke-linecap="round"`
- The open right side + larger spine node (node 2) is what makes it read as a "C".

The dual-tone fills are `#6366f1` (indigo) → `#2dd4cf` (teal) interpolated linearly by generation (0→4). This colours the lineage from root to leaves.

### Tone variants
- **duo** — the gradient fills above (primary, on dark backgrounds).
- **mono** — every node + edge uses `currentColor`; edges at `stroke-opacity="0.42"`. Inherits surrounding text colour. Use for monochrome contexts (light docs, single-colour print).
- **knockout** — white nodes + `#ffffff` edges at `0.6` opacity, on an indigo field. Used inside the app icon.

---

## The live mark — motion spec
One loop = **4.6s**, `infinite`. Each node/edge is delayed by `generation × 0.13s` so the lineage assembles root → leaves.

| Element | Keyframes (% of 4.6s) | Easing |
|---------|----------------------|--------|
| **Edge** (`stroke-dashoffset` 1→0, `pathLength=1`, `stroke-dasharray=1`) | `0%`: off=1,opacity 0 · `10%`: opacity 1 · `30%`: off=0 (drawn) · `78%`: hold · `93%`: opacity 1 · `100%`: off=1,opacity 0 | `cubic-bezier(.6,.02,.3,1)` |
| **Node** (`transform: scale`) | `0%`: scale 0 · `16%`: scale 1 · `82%`: hold · `100%`: scale 0 | `cubic-bezier(.34,1.4,.5,1)` |
| **Pulse ring** (circle on node 2, `scale` + fade) | `0–38%`: scale 1, opacity 0 · `52%`: opacity .5 · `100%`: scale 2.6, opacity 0 | `ease-out` |

Notes:
- Nodes need `transform-box: fill-box; transform-origin: center` so they scale about their own centre.
- Ring: `fill="none" stroke="#7dd6e8" stroke-width="1.96"` on node 2 (`cx18 cy50 r7`).
- **`prefers-reduced-motion: reduce`** must disable all three animations and show the fully-drawn static mark (edges drawn, nodes at scale 1, ring hidden). This is already handled in both `clan-mark-live.svg` and `ClanMark.tsx`.

---

## Wordmark & lockup
- **Wordmark:** the literal text `CLAN`, all caps.
- **Font:** **Space Grotesk**, weight **600**. Fallback `system-ui, sans-serif`.
- **Letter-spacing:** `0.16em` (apply equal `text-indent: 0.16em` so the word stays optically centred when the tracking adds trailing space).
- **Optional tagline** (mono contexts / hero): `CONTEXT · LIVE · AGENT · NOTATION` in **Space Mono** 400, `letter-spacing: 0.22em`, uppercase, in the muted colour.
- **Lockup spacing:** gap between mark and wordmark = `0.5 × wordmark font-size`. Mark height ≈ `1.45 × wordmark font-size`. Vertically centred.

---

## Design tokens
```
/* Brand */
--clan-indigo:   #6366f1;   /* root node, primary accent (matches app --accent) */
--clan-teal:     #2dd4cf;   /* leaf node */
--clan-edge:     rgba(120,160,230,0.42);
--clan-ring:     #7dd6e8;
/* node gradient stops (gen 0→4) */
--clan-n0:#6366f1; --clan-n1:#5682e9; --clan-n2:#489de0; --clan-n3:#3bb9d8; --clan-n4:#2dd4cf;

/* Surfaces — from the existing app (src/index.css), unchanged */
--bg:      #0f1117;   --surface: #141922;   --border: #1e2d45;
--text:    #e2e8f0;   --word:    #eceefb;   --muted:  #6b7283;
--icon-field: #6366f1;  /* app-icon squircle */
--readme-bg:  #f5f4ef;  /* light/paper contexts */

/* Geometry */
--mark-viewbox: 100;
--mark-stroke:  2.8;        /* at 100 viewBox; scales with the svg */
--icon-radius:  0.26;       /* squircle corner = 0.26 × icon size (133/512) */
--icon-inset:   0.17;       /* mark padding inside icon = 0.17 × icon size */
--wordmark-tracking: 0.16em;
```

Fonts (Google Fonts): `Space Grotesk` (400–700), `Space Mono` (400, 700).

---

## Clear space & minimum sizes
- **Clear space:** keep padding of at least **one node diameter** (the spine node, ~14% of mark width) clear on all sides of the mark / lockup.
- **Minimum mark size:** 16px (favicon). Below ~20px, prefer a slightly bolder favicon if edges look faint — a dedicated heavier `favicon-16` is an acceptable optimization.
- **App icon inset:** mark occupies the centre 66%; squircle corner radius = 26% of icon size.
- Do not recolour nodes outside the indigo→teal ramp, rotate the mark, add drop shadows to the mark itself (the app-icon squircle may carry a subtle shadow), or close the right-hand opening (it would stop reading as a "C").

---

## Assets in this bundle
| File | What it is | Use |
|------|-----------|-----|
| `assets/clan-mark.svg` | Static dual-tone mark, transparent, 100×100 | General brand mark on dark |
| `assets/clan-mark-mono.svg` | Single-colour (`currentColor`) mark | Monochrome / light / print |
| `assets/clan-mark-live.svg` | Self-contained animated mark (CSS in SVG) | Splash / loading / hero |
| `assets/clan-icon.svg` | Indigo squircle + white knockout mark, 512×512 | App icon source (export PNGs) |
| `assets/favicon.svg` | Mark sized 32px | Browser favicon |
| `ClanMark.tsx` | React component: `<ClanMark>` + `<ClanLogo>`, `tone` + `animated` props | In-app usage (titlebar, panels) |
| `_preview.html` | Renders every asset together | Quick visual check |
| `CLAN Logo.html` | The full exploration (4 symbol directions, colour treatments, contexts) | Design rationale / alternates |

---

## Files to replace in this repo
This repo (`clan/app`) ships placeholder logo files — swap them for the new assets:

1. **`app/public/favicon.svg`** → replace with `assets/favicon.svg`.
   (`app/index.html` already links `<link rel="icon" type="image/svg+xml" href="/favicon.svg" />` — no change needed.)
2. **`app/public/logo-static.svg`** → replace with `assets/clan-mark.svg` (or keep the filename and overwrite contents). Update any importers.
3. **`app/public/icons.svg`** → review; if it's the old mark, regenerate from `assets/clan-icon.svg`.
4. **App icon / `app/src-tauri/icons/`** → `tauri.conf.json` points `bundle.icon` at `icons/icon.png`. Regenerate the Tauri icon set from `assets/clan-icon.svg`:
   ```
   npx @tauri-apps/cli icon design_handoff_clan_logo/assets/clan-icon.svg
   ```
   (writes `icon.png`, `.ico`, `.icns`, and all platform sizes into `src-tauri/icons/`).
5. **In-app chrome** — drop `ClanMark.tsx` into `app/src/components/` and use `<ClanLogo />` in the titlebar/header; use `<ClanMark animated />` for the loading/splash state. Load the two Google Fonts in `index.html` or via your existing font setup.

> CSP note: `tauri.conf.json` already allows `style-src 'unsafe-inline'` and `img-src data: blob:`, so both the inline-animated component and the SVG-as-`<img>` will render without CSP changes.

## Verify after implementing
- Favicon legible at 16px in a browser tab.
- App icon crisp at 512 / 128 / 32 (the knockout "C" stays clear).
- Live mark loops smoothly and **freezes to the static mark** under reduced-motion.
- Wordmark renders in Space Grotesk (not the fallback) once fonts load.

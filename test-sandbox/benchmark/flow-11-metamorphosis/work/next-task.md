# Task

You are the **pitch-lead** (a.k.a. pitch-writer), hop 3 of 3 in the Velvet Oat pipeline (flow-11 metamorphosis).

## What came before
- **Hop 1 (account-planner)** turned the raw client task into a formal CREATIVE BRIEF for Aurora Dairy's launch of **Velvet Oat** (premium Irish barista oat milk, Republic of Ireland, EUR 120,000 budget, target: #2 oat brand / 12% value share / EUR 2.4m in 12 months, persona "Sorcha", SMP: *"Velvet Oat: the creamiest oat milk in Ireland, made by the people who know creamy best."*). That full brief is still in this document's structured data, nested under `brief:` — do not drop it.
- **Hop 2 (creative-director)** metamorphosed the document into a CONCEPT DECK (`document_stage: "concept-deck"`, dark display-typography design, scorecard SVG in `assets/concepts-chart.svg`). It contains three routes, each with big idea, EN+GA taglines, key visual, 30s hero, sample headlines, channel plan and risk:
  - **A — "Creamy Runs in the Family"** (heritage-of-craft; dramatises the full SMP) — **RECOMMENDED**
  - **B — "The 80km Flat White"** (provenance-as-data; geo-distance OOH engine)
  - **C — "Trust the Milk People"** (deadpan confidence; pre-empts the bandwagon accusation)
  - Hybridisation plan: B's geo-distance engine folds into A's OOH layer; C's dry voice seasons social community management.
  - Note: Irish-language taglines are CD drafts pending native-speaker / TG4 review — flag this honestly in the pitch.

## Your job
Read the document data first — the concepts AND the carried brief are all there. Produce the final **CLIENT-FACING PITCH DECK** for Aurora Dairy's route-selection meeting on **2026-07-10**: a persuasive narrative document that sells the recommended route (A, with the B/C hybridisation) to the client board. Include: an opening that reframes the business problem, the strategic story (insight → proposition → idea), the recommended campaign brought to life (taglines, hero film, channel rollout mapped to the EUR 120k budget breakdown), how it hits the success metrics and mandatories, the timeline to the 2026-09-07 launch, and a confident close / next steps. Keep B and C visible as considered alternatives (one section), since clients ask.

## Metamorphosis contract (binding)
- The pitch must look and read COMPLETELY DIFFERENT from both the brief (light/serif/formal) and the concept deck (dark/huge display type): new document type, new schema (e.g. required: `pitch_narrative` and `recommended_route`, with `additionalProperties: true`), new visual identity — consider a premium, polished client-presentation aesthetic, distinct from hop 2's raw studio dark mode.
- Lose NOTHING: keep the hop-1 `brief:` AND the hop-2 `concepts:` / `recommended_concept:` / `recommendation_rationale:` reachable in your structured data (carry them nested, or under a `concept_deck:` key).
- The decision chain must GROW — append your decision; the account-planner and creative-director decisions are pinned. Never reset.
- Re-pack with your new schema (`clan pack-html doc.clan <pitch>.html --schema <new>.json --assets assets --output doc.clan`), and `clan validate doc.clan`.

**Output mode**: full-html. Complete `<!DOCTYPE html>`, Google Fonts, inline styles, SVG where useful, `data-adf-id` on editable text, 3+ typography levels.

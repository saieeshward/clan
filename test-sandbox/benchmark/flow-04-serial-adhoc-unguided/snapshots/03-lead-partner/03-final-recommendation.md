# Brightline CRM Decision — Final Recommendation & Rollout Plan
**For:** Brightline — 40-person advertising agency, Dublin, Ireland
**Stage:** 3 of 3 (lead-partner). Inputs: `brief.md`, `01-market-research.md`, `02-risk-analysis.md`.
**Pricing caveat:** All figures are knowledge-only (cutoff ~Jan 2026), EUR, annual billing, ex-VAT, **±20% — to validate with written quotes before signature** (risk R12).

## 1. Executive summary

We recommend **HubSpot (Sales Hub Professional for the selling team + Marketing Hub Professional)**, at an estimated **year-1 total cost of ~€40,000** (range €33k–€47k at ±20%). HubSpot is not the cheapest option — Zoho would cost roughly half — but it is the only platform that scores first or second on every criterion in the brief: native Meta/Google ad-ops integration (decisive for an agency), best-in-class adoption for a creative-led team, workable GDPR posture via the Frankfurt EU data centre, and mid-range cost. Salesforce is eliminated on cost-plus-risk grounds; Zoho is retained as the negotiation lever and documented fallback. Two conditions are non-negotiable at signature: the portal **must** be created in the EU region (R2), and the marketing-contact tier **must** be contractually controlled (R6).

**Assumption to confirm before Phase 1 (carry-forward from stage 2):** Brightline has no incumbent CRM — current state is spreadsheets/inbox. If an incumbent exists, add a 2–3 week migration workstream and ~€3–5k to Phase 1; the recommendation itself does not change.

## 2. The decision and how the specialists were weighed

The two prior stages disagreed in emphasis, not in conclusion, and the disagreements were resolved as follows:

- **Cost (research) vs risk (analysis) on Zoho.** Stage 1 showed Zoho is 2–3x cheaper (~€11–19k/yr). Stage 2 rated its risks "absorbable but costing staff time" (R4, R11). I weighed this against Brightline's economics: at 40 billable-adjacent staff, even 1–2 hours/week of agency time lost to shallow integrations, slow support during campaign-critical incidents, and DPO evidence-assembly erodes the ~€14k/yr saving. For an *ad agency* specifically, stage 1's finding that HubSpot has the only native Meta/Google ads sync (audience sync, lead-ad capture, attribution) is core-workflow value, not nice-to-have — that tipped it. Zoho remains the right answer if budget is hard-capped under €15k.
- **Salesforce capability vs everything else.** Stage 1 acknowledged Salesforce's power; stage 2 rated it highest aggregate risk (adoption R9=4, renewal uplift R5=4, lock-in R7=4, no-admin R10=4) and stage 1 priced the true cost at €38–79k licences **plus** €10–20k/yr admin/partner. Both specialists effectively converged: Salesforce is over-spec for a 40-seat agency without an in-house admin. Eliminated with high confidence.
- **HubSpot's weak points.** Stage 2's two severity-4 HubSpot risks (R2 EU portal, R6 contact-tier creep) are both *controllable at signature and by process*, unlike Salesforce's risks which are structural. Where a risk is cheap to mitigate, I discounted it; where it is structural, I did not. That asymmetry is the core of the decision.

## 3. Costed recommendation — year 1 (EUR, ex-VAT, ±20%)

| Item | Basis | Year-1 cost |
|---|---|---|
| Sales Hub Professional | 15 paid seller seats × ~€95/seat/mo | ~€17,100 |
| Free CRM seats | Remaining ~25 staff (view/log access) | €0 |
| Marketing Hub Professional | Flat ~€800/mo incl. starting contact tier | ~€9,600 |
| Mandatory onboarding fee | HubSpot Pro requirement | ~€3,000 |
| Implementation partner (light) | Data import, pipelines, ads + consent-field setup | ~€5,000 |
| Training & change management | Champions, workshops, 30/60/90 KPIs | ~€3,000 |
| Contingency: contact-tier headroom + middleware (GA4 via Zapier/reverse-ETL) | R6 buffer | ~€2,300 |
| **Year-1 total** | | **~€40,000** |

Ongoing year-2+ run-rate: ~€29–32k/yr (licences + contingency), assuming contact hygiene holds and renewal uplift is capped (see checklist).

## 4. Three-phase rollout plan

**Phase 1 — Contract & foundation (Weeks 1–4).**
Confirm incumbent-CRM assumption. Obtain written quotes from HubSpot *and* Zoho (negotiation lever). Execute contract checklist (below). Create portal **in EU/Frankfurt region — verify before any data import** (R2). Sign DPA + SCCs; DPO files TIA; update Brightline's client DPAs to disclose HubSpot as sub-processor (R13). Configure consent/lawful-basis fields before any ads sync (stage 2 §1).

**Phase 2 — Pilot (Weeks 5–10).**
8–10 users: one new-business pod + one account team (R9 mitigation). Import contacts/deals; mark non-marketing contacts to control tier (R6). Connect Meta + Google Ads native integrations; stand up GA4 reporting via Zoho Analytics-equivalent or Zapier/BigQuery path. Weekly usage KPIs; go/no-go gate at week 10 (≥70% weekly active pilot usage, pipeline data quality spot-check).

**Phase 3 — Full rollout & hardening (Weeks 11–20).**
Roll out to all 40 staff (free seats for non-sellers; upgrade only on demonstrated need). Named champion per team. Quarterly contact-hygiene routine instituted (R6). 30/60/90-day adoption reviews; renewal-notice deadline calendared. Decision review at month 6: confirm Enterprise features are *not* needed (custom-object paywall creep) and contact tier is within headroom.

**Contract checklist (bake into signature):** EU portal region in writing (R2); renewal uplift capped at CPI or ≤5% — the R5 mitigation applied to HubSpot, since promo pricing commonly steps up at first renewal; contact-tier price-per-tier locked with headroom (R6); 1-year initial term, no auto-renew beyond 60-day notice; onboarding fee scope itemised; sub-processor list reviewed, non-EU AI features disabled pending assessment (R3).

## 5. Conditions that would change this decision

- Hard budget cap < €15k/yr → **Zoho CRM Professional** (~€11k licences), accept support/evidence friction.
- HubSpot refuses EU-region guarantee or contact-tier cap → re-open Zoho.
- Discovery of complex incumbent CRM with heavy custom process → re-score; Salesforce only re-enters if a funded admin role exists.

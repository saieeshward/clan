# Brightline CRM Decision — Final Recommendation
**Prepared by:** lead-partner (synthesis of market-research.md, risk-analysis.md, team-adoption-analysis.md)
**Client:** Brightline, 40-person advertising agency, Dublin, Ireland | **Date:** 2026-06-10
**Caveat:** All figures are knowledge-based estimates (no live pricing checks), ±15%; verify against vendor EUR price books before signature.

## Executive Summary

Brightline should adopt **HubSpot** (Sales Hub Professional + Marketing Hub Professional), provisioned in the **EU (Frankfurt) data centre at portal creation**, with an estimated **year-1 total cost of ~EUR 46,500** (range EUR 42–52k). All three specialist workstreams — market/pricing, risk/GDPR, and team adoption — independently recommended HubSpot at 4/5 confidence, an unusually clean convergence. HubSpot is roughly half Salesforce's realistic all-in cost, carries the lowest adoption risk for a creative agency (1–2 weeks to AM fluency vs 4–8 for Salesforce), is the only option with native Meta/Google/LinkedIn Ads integration at its core tier, and offers an acceptable GDPR posture provided the EU region is selected on day one — a decision that is effectively irreversible for HubSpot. Zoho remains the budget fallback (~EUR 22k/yr licences) if cost dominates every other criterion; Salesforce is rejected as an adoption-and-cost outlier at this headcount.

## Costed Recommendation (Year 1, EUR)

| Item | Basis | Cost |
|---|---|---|
| Sales Hub Professional — 22 paid seats | 10 AMs + 8 media buyers + 4 leadership/ops × ~EUR 95/seat/mo × 12; creatives use free view-only seats | 25,100 |
| Marketing Hub Professional | ~EUR 800/mo platform fee, entry contact tier | 9,600 |
| Mandatory onboarding fees (Sales Pro + Marketing Pro) | One-off, required at Pro tier | 4,000 |
| Training & change management | ~1 day AMs, half-day buyers, 1-hr creatives; champion time; per adoption analysis (EUR 2–4k) plus light consulting | 3,500 |
| Contingency: marketing-contact tier creep + scope buffer (~10%) | Risk register #4 mitigation | 4,300 |
| **Year-1 total** | | **~EUR 46,500** |

Steady-state year-2 run-rate falls to ~EUR 36–40k (one-offs drop away). Comparators: Salesforce Enterprise ~EUR 74k licences alone, realistically EUR 100–130k year 1 with admin/partner and marketing add-ons; Zoho Enterprise ~EUR 22k licences + EUR 4–8k training + premium support.

## Three-Phase Rollout Plan (12 weeks to full cutover)

**Phase 1 — Foundation & Account Managers (Weeks 0–4).** Provision EU-Frankfurt portal *before any data import* (risk #2 — verify region first). Sign DPA with SCCs; document a transfer impact assessment. Configure pipeline stages, required fields, and dedupe rules. Migrate ruthlessly little: active clients and open opportunities only. Onboard the 10 AMs; appoint two champions (one AM, one media buyer) with 10% protected time for 90 days. Exit criteria: all AMs logging deals weekly.

**Phase 2 — Media Buyers, Ads & Marketing Hub (Weeks 4–8).** Connect Google/Meta/LinkedIn Ads accounts; automate the pipeline-to-campaign handoff so buyers never double-enter data (their #1 resistance trigger). Activate Marketing Hub: forms, email, attribution. Set contact-hygiene policy from day one — mark non-active contacts non-marketing, schedule quarterly list cleanses (risk #4). Train buyers (half-day each).

**Phase 3 — Creatives, Cutover & Measurement (Weeks 8–12+).** Give creatives 1-hour orientation, limited to tasks/comments via free seats — do not force activity logging. Enforce the pre-announced hard cutover date killing parallel spreadsheets. Track weekly for the first quarter: % of deals updated in last 7 days and login rate by role; intervene per-person. Leadership consumes dashboards instead of requesting email updates. Quarter-end: vendor review against the documented exit path (risk #10).

## How I Weighed the Specialists Where They Differed

The three agents agreed on the vendor, so reconciliation concerned magnitudes, not direction:

1. **Paid-seat count (cost driver).** The market researcher assumed 25 paid seats (~EUR 28.5k licences); the adoption analyst's role breakdown (10 AMs, 8 buyers, 15 creatives, ~7 leadership/ops) implies fewer daily users. I sided closer to the adoption analyst's ground-truth headcount — 22 paid seats, creatives on free view-only seats — trimming licences to ~EUR 25.1k. This is HubSpot's seat model working as intended and is the lower-risk contractual position (seats can be added mid-term, not removed).
2. **"All-in" figures.** The researcher's ~EUR 38k/yr is a steady-state run-rate; the risk analyst flagged mandatory Pro-tier onboarding fees and contact-tier creep the researcher omitted; the adoption analyst added EUR 2–4k training. My EUR 46.5k year-1 figure unifies all three: run-rate + one-offs + a contingency sized to the risk analyst's register. Where risk and market views conflicted on hidden costs, I weighted the risk analyst — hidden-cost pessimism is the cheaper error.
3. **Salesforce admin cost.** Estimates differed (researcher: +EUR 15–30k partner; adoption: EUR 25–45k part-time admin) but both point the same way; the discrepancy only widens the gap against Salesforce and needed no resolution to decide.
4. **Zoho's case.** The researcher kept Zoho live on price; the risk analyst (support weakness, execution risk) and adoption analyst (UX friction, 15–20 person-days training) outvoted it. At a ~EUR 14–18k/yr saving against materially worse adoption odds and the weakest support, the expected cost of a failed rollout exceeds the saving. Zoho stays as documented fallback only.
5. **Binding conditions adopted from the risk analyst:** EU portal region verified before import; marketing-contact tier negotiated at signature; 1-year initial term; processes documented outside the tool to cap functional lock-in.

**Decision: HubSpot. Year-1 budget: EUR 46,500 (approve up to EUR 52,000). Confidence: 4/5.**

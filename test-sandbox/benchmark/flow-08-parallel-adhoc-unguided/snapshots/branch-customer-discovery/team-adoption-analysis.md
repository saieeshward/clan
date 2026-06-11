# Team Adoption Analysis — Brightline CRM Evaluation
Role: customer-discovery (team-adoption angle) | Date: 2026-06-10
Scope: learning curve per platform for ad-agency roles, training burden, expected adoption resistance, and change-management recommendations for a 40-person Dublin agency.

**Assumptions** (no web access; knowledge-based, current to early 2026): Brightline is ~40 staff split roughly 10 account managers (AMs), 8 media buyers, 15 creatives, plus leadership/ops/finance. Today they likely run on spreadsheets, email, and possibly a lightweight tool — so this is a first "real CRM," not a migration from an incumbent enterprise system. Creatives will be light/occasional users; AMs are the daily power users; media buyers care mainly about pipeline-to-campaign handoff data.

## Learning curve per platform, by role

**HubSpot — easiest overall.** Its consumer-grade UI is the closest to tools agency staff already know (Gmail-style inbox, Canva-like simplicity). AMs are typically productive in 1–2 weeks for pipeline, contact, and email-logging workflows. Media buyers pick up deal-stage and budget fields in days; the native ad-account integrations (Google/Meta/LinkedIn Ads) match their mental model. Creatives can be confined to tasks/comments and need under an hour of orientation. Admin burden is low: an office-manager-level "accidental admin" can run it; no dedicated hire required.

**Zoho CRM — moderate.** Functionally approachable but the UI is denser and less polished; navigation and module naming confuse non-technical users early on. AMs need 2–4 weeks to fluency; expect more "where do I click" tickets in month one. Creatives find it utilitarian and tend to avoid it unless workflows force engagement. Zoho's breadth (the wider Zoho One suite) is a double-edged sword: cheap, but configuration sprawl creates inconsistent experiences unless someone curates it.

**Salesforce — steepest by a wide margin.** Sales Cloud assumes a structured sales org and an admin function. AMs realistically need 4–8 weeks plus formal training to work confidently; media buyers and especially creatives will find it heavyweight and opaque. For a 40-person agency, Salesforce practically demands either a part-time certified admin (~EUR 25–45k/yr part-time in the Dublin market) or an implementation partner — a hidden cost the licence price never shows. Mis-scoped Salesforce rollouts at sub-50-person firms are the classic "shelfware" failure mode.

## Training burden (estimated)

- HubSpot: ~1 day formal training for AMs, half-day for buyers, 1-hour session for creatives; free HubSpot Academy covers self-serve refreshers. Total: ~6–8 person-days org-wide.
- Zoho: ~2 days for AMs, 1 day for buyers, half-day creatives; documentation is weaker, so budget for an internal champion writing Brightline-specific guides. Total: ~15–20 person-days.
- Salesforce: 3–5 days for AMs plus admin training/partner onboarding; Trailhead is good but generic. Total: 30+ person-days plus ongoing admin time.

## Expected adoption resistance

Agency-specific friction points: (1) Creatives resent "sales tooling" and will not log activity — keep them out of the CRM except for project visibility, or adoption metrics will look falsely bad. (2) AMs guard client relationships; they resist anything that feels like surveillance. Frame the CRM as protecting them (handover continuity, fewer status-update meetings), and have leadership visibly use dashboards instead of asking for email updates. (3) Media buyers will reject double data entry — adoption hinges on the CRM-to-ad-ops handoff being automated, which favours HubSpot's native ad integrations. (4) Senior staff "we've always used spreadsheets" inertia: the strongest predictor of failure; counter by killing the parallel spreadsheet on a fixed date.

## Change-management recommendations

1. **Phase by role**: AMs first (weeks 1–4), media buyers next (weeks 4–8), creatives last and lightly (week 8+). Don't big-bang 40 people.
2. **Appoint two champions** (one AM, one buyer) with 10% protected time for 90 days; route questions to them, not IT.
3. **Migrate ruthlessly little**: import active clients and open opportunities only; archive the rest. Dirty legacy data kills trust in week one.
4. **Hard cutover date** for retiring spreadsheets, announced upfront, leadership-enforced.
5. **Measure adoption weekly** for the first quarter: % deals updated in last 7 days, login rate by role; intervene per-person, not by memo.
6. **Budget reality**: HubSpot needs ~EUR 2–4k of training/consulting; Zoho ~EUR 4–8k; Salesforce EUR 15–40k+ with partner involvement.

## Verdict (adoption lens)

**Recommended vendor: HubSpot. Confidence: 4/5.** Lowest learning curve for every role, lowest training burden, weakest resistance profile, and no admin hire needed. Zoho is the acceptable budget fallback; Salesforce is an adoption-risk outlier at this headcount. Confidence is 4 not 5 because pricing/GDPR angles (sibling analyses) could shift the total-cost picture.

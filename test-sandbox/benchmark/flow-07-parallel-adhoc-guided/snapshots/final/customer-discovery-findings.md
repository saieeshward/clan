# Customer Discovery: Team-Adoption Analysis — Brightline CRM Evaluation

Scope: learning curve per platform (Salesforce, HubSpot, Zoho CRM) for ad-agency roles, training burden, expected adoption resistance, and change-management recommendations for a 40-person Dublin agency. Knowledge-based analysis (no web access); assumptions stated at the end.

## Learning curve by platform and role

**Salesforce (Sales Cloud).** Steepest curve of the three. Account managers face a dense object model (Leads vs Contacts vs Opportunities, record types, page layouts) that typically takes 4–8 weeks to genuine fluency, and the platform effectively requires a part-time admin — a role a 40-person agency does not have and would need to contract (~EUR 600–900/day in Dublin) or train internally via Trailhead (free but time-hungry, 30–60 hours for admin basics). Media buyers get little native value: ad-ops data lives outside Salesforce unless AppExchange connectors or custom integration are added, which raises the learning surface further. Creatives would touch it rarely and resent every login. Salesforce adoption failures in sub-50-person firms are common and usually stem from over-configuration, not the product itself.

**HubSpot.** Shallowest curve. The UI is consumer-grade; account managers reach working competence in 3–7 days and fluency in 2–3 weeks. HubSpot Academy provides free, role-specific, certificate-based training that maps well to AM and media-buyer workflows (deal pipelines, email sequences, meeting links, native Google/Microsoft 365 mail sync). Marketing-agency staff often already know HubSpot from client work — a real accelerator at an ad agency. Creatives can consume context (briefs, client history) read-only with near-zero training. No dedicated admin needed; one power-user AM can own configuration in ~2–4 hours/week.

**Zoho CRM.** Middle of the pack. Cheaper, capable, but the UI is denser and less polished; AMs typically need 2–4 weeks to fluency and discoverability is weaker (features exist but are buried). Documentation quality is uneven and the broader Zoho One ecosystem invites scope creep ("we also got Zoho Projects/Desk/Books...") that multiplies the training burden. Fine for a tools-tolerant team; riskier for a creative-heavy culture with low patience for clunky software.

## Training burden (estimated, 40 people: ~12 AMs/client-service, ~8 media buyers, ~14 creatives, ~6 ops/leadership)

- Salesforce: 2–3 days formal training for AMs, 1 day for buyers, plus ongoing admin overhead. Realistic first-year training + admin cost: EUR 15k–30k beyond licences.
- HubSpot: half-day kickoff + self-serve Academy paths (2–6 hours per person). First-year training cost effectively EUR 2k–5k (mostly internal time).
- Zoho: 1–2 days for AMs, with a designated internal champion needed to compensate for weaker discoverability. EUR 5k–12k first year.

## Expected adoption resistance

- AMs resist anything that adds data-entry without visible payoff; Salesforce scores worst here, HubSpot best (auto-logging of email/calls reduces manual entry).
- Media buyers will judge the CRM by whether it talks to their ad-ops stack (Meta/Google Ads, DV360, reporting tools). All three need connectors; resistance is platform-agnostic but amplified if the tool is also hard to use (Salesforce, Zoho).
- Creatives are passive users; resistance shows as silent non-use. Keep them out of mandatory workflows on any platform.
- A 40-person agency has no slack for a 2-month productivity dip; Salesforce's dip is the deepest and longest.

## Change-management recommendations

1. Appoint two champions (one senior AM, one media buyer); give them config ownership and a weekly 30-min office hour for the first 8 weeks.
2. Phase rollout: AMs/pipeline first (weeks 1–4), media buyers + ad-ops integration (weeks 5–8), creatives read-only last (week 9+). Do not launch all-hands.
3. Migrate only live clients and the last 12 months of history at cutover; archive the rest to avoid data-cleanup paralysis.
4. Set 3 non-negotiable behaviours (e.g., every client interaction logged, pipeline updated weekly, no side spreadsheets) and have leadership model them visibly.
5. Measure adoption at weeks 4/8/12 (weekly active users, % deals updated in last 7 days) and treat <70% AM weekly-active at week 8 as a trigger for intervention.

## Recommendation (adoption lens only)

**HubSpot** — lowest learning curve, best free training ecosystem, least resistance from the AM-heavy core, no admin headcount required. Zoho is the budget fallback; Salesforce is an adoption risk Brightline cannot staff for.

## Assumptions

- Role mix estimated (no headcount breakdown provided).
- Pricing/training figures are 2025-era knowledge-based estimates in EUR, not quotes.
- No incumbent CRM specified; assumed migration from spreadsheets/inbox-based tracking.
- "Adoption risk" assessed independently of price and GDPR, which sibling agents cover.

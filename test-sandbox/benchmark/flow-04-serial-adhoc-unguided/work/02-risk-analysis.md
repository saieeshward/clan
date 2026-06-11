# Risk Analysis: Salesforce vs HubSpot vs Zoho CRM
**For:** Brightline — 40-person advertising agency, Dublin, Ireland
**Stage:** 2 of 3 (risk-analyst). Inputs: `brief.md`, `01-market-research.md`. Next stage: lead-partner (costed recommendation + phased rollout).
**Knowledge caveat:** No web access; all claims from training knowledge (cutoff ~Jan 2026). Assumptions stated inline. Verify DPA terms, current data-centre locations, and renewal clauses before signature.

## 1. GDPR posture and EU data residency

All three vendors are workable for an Irish controller; differences are in depth, default posture, and proof effort.

- **Salesforce.** Hyperforce offers EU residency (Frankfurt/Paris); the EU Operating Zone (EU support staff, EU-only access) is a *paid add-on*. Mature DPA with SCCs, strong sub-processor transparency, broad certifications (ISO 27001/27701, SOC 2). Caveat: US parent means Schrems II / CLOUD Act exposure remains; mitigated by Hyperforce encryption and, at extra cost, Shield Platform Encryption with BYOK. GDPR risk is low but *configuration-dependent* — granular permissions cut both ways: misconfiguration is on you.
- **HubSpot.** EU data centre (Frankfurt, AWS) selectable **only at portal creation** — choose EU region day one; migrating a US-hosted portal later is painful. DPA with SCCs, SOC 2, ISO 27001. Some sub-services (e.g. certain AI features, call recording transcription) may process outside the EU — review the sub-processor list if AI features are enabled. Built-in consent/cookie tooling is genuinely useful for an ad agency handling prospect data.
- **Zoho.** EU DC (Amsterdam, with Dublin presence) — Zoho runs its **own** data centres, no AWS/US hyperscaler dependency, which is a quiet GDPR advantage (simpler sub-processor chain). DPA and SCCs available; ISO 27001, SOC 2. Weakness: privacy documentation and audit-support responsiveness are thinner; a DPO doing a TIA will spend more effort assembling evidence. Zoho Corporation is India-headquartered — assess India as a third country in the TIA, though EU-DC data is stated to stay in-region.

**Agency-specific note:** Brightline will likely be *processor* for client campaign data (custom audiences, lead-ad capture). Whichever CRM is chosen becomes a sub-processor Brightline must disclose in its own client DPAs. HubSpot's native Meta/Google ads sync moves personal data to ad platforms — ensure lawful-basis and consent flags are mapped in CRM fields from day one.

## 2. Vendor lock-in and exit costs

- **Salesforce:** highest lock-in. Custom objects, Flows, Apex, and AppExchange dependencies do not port. Data export is easy (weekly export, APIs); *process* export is not. Multi-year agreements with auto-renewal and 5–15% uplifts; downgrading seats mid-term is contractually hard. Exit cost estimate: 3–6 months consultancy effort.
- **HubSpot:** moderate. Good APIs and exports, but marketing assets (workflows, landing pages, attribution history) don't port, and contact-tier pricing creates *cost* lock-in as the database grows. Annual terms; no perpetual discounts — promo pricing at signature commonly steps up at first renewal.
- **Zoho:** lowest. Cheap enough that sunk cost is small; exports are straightforward. Risk inverts: the temptation to adopt the whole Zoho One suite (books, projects, campaigns) spreads lock-in across the business quietly.

## 3. Migration-in risk

Greenfield-or-spreadsheet assumed (brief doesn't state an incumbent CRM — **assumption to confirm**). Zoho and HubSpot imports are self-serve in days; Salesforce realistically needs a partner (€5–15k). The bigger migration risk is *adoption*, not data: Salesforce's admin burden (research doc: +€10–20k/yr partner/admin) is itself a continuity risk for a 40-person agency with no in-house admin.

## 4. Hidden-cost and support risk

- **Salesforce:** premier support is ~30% of net licence extra; standard support is slow. Sandboxes, API-call limits, and storage overages bite below Enterprise. Renewal uplift is the single biggest 3-year cost risk.
- **HubSpot:** marketing-contact tier creep (agencies hoard prospect lists), Enterprise-feature paywalls (custom objects), onboarding fee (~€1.5–3k mandatory on Pro+). Support on Pro is decent (email/chat); phone needs higher tiers.
- **Zoho:** support is the weak point — slow first-line responses, EU-hours coverage thinner, small Irish partner network. Paid premium support (~20–25% of licence) partially mitigates. Hidden costs are few; the cost is in staff time working around rough edges.

## 5. Risk register

| # | Risk | Vendor | Sev (1–5) | Mitigation |
|---|---|---|---|---|
| R1 | US transfer exposure (CLOUD Act) despite EU hosting | Salesforce, HubSpot | 3 | EU DC/Hyperforce EU; SCCs + TIA on file; encryption add-ons if client contracts demand; document in client DPAs |
| R2 | HubSpot portal created in US region by accident | HubSpot | 4 | Explicitly select EU data residency at portal creation; verify before any data import |
| R3 | AI/sub-features processing data outside EU | HubSpot, Zoho | 2 | Review sub-processor list; disable non-EU AI features until assessed |
| R4 | Thin audit evidence slows client/DPO due diligence | Zoho | 2 | Collect DPA, ISO/SOC reports up front; budget DPO hours for TIA |
| R5 | Renewal uplift 5–15%/yr + multi-year auto-renew | Salesforce | 4 | Cap uplift in contract; 1-yr initial term; calendar renewal-notice deadlines |
| R6 | Marketing-contact tier creep inflates cost 2x | HubSpot | 4 | Quarterly contact hygiene; mark non-marketing contacts (free); contract tier headroom |
| R7 | Process/customisation lock-in raises exit cost | Salesforce | 4 | Minimise Apex/custom objects; document config; keep ad-ops logic in middleware not CRM |
| R8 | Suite sprawl (Zoho One) spreads lock-in org-wide | Zoho | 2 | Adopt modules deliberately; keep finance/projects decisions separate |
| R9 | Adoption failure / shelfware in creative-led team | Salesforce 4 / HubSpot 2 / Zoho 3 | – | Pilot with 8–10 users; champion per team; usage KPIs at 30/60/90 days |
| R10 | No in-house admin → config drift, broken automations | Salesforce | 4 | Budget partner retainer (€10–20k/yr) or choose lower-admin platform |
| R11 | Slow support during campaign-critical incidents | Zoho | 3 | Premium support tier; local partner SLA; runbook for export/fallback |
| R12 | Pricing assumptions wrong (±20%, knowledge-only) | All | 3 | Stage 3 must mark all figures "to validate"; get written quotes before recommendation is final |
| R13 | Sub-processor disclosure gap in Brightline's client DPAs | All | 3 | Update client DPAs to list chosen CRM; map consent fields before ads sync enabled |

## 6. Net risk view (for stage 3)

- **Lowest composite risk for Brightline's profile: HubSpot**, *provided* R2 (EU portal) and R6 (contact creep) are controlled at signature.
- **Zoho** is the lowest financial-exposure option; its risks (support, evidence depth) are absorbable but cost staff time.
- **Salesforce** carries the highest aggregate risk for a 40-seat agency without an admin: adoption (R9), hidden cost (R5), and lock-in (R7) all rate 4.
- Carry-forward for lead-partner: confirm incumbent-CRM assumption (Section 3), validate pricing (R12), and bake R2/R5/R6 mitigations into the contract checklist of the rollout plan.

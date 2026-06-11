# Brightline CRM Evaluation — Report Draft

## market-researcher

**Assumptions and caveats.** No web access was used; all figures are from model knowledge with a cutoff of January 2026. Vendor list prices change frequently and EUR figures below are list-price ballparks (annual billing, ex-VAT) — verify against current vendor quotes before any decision. Where vendors price in USD, I assume rough EUR parity. "40-seat" costing assumes all 40 staff get a paid seat; in practice an agency could licence ~25 sales/account-management seats and use free/read-only seats for the rest, materially cutting cost (especially on HubSpot and Salesforce).

### Capabilities overview

**Salesforce (Sales Cloud).** The deepest and most configurable platform: custom objects, advanced workflow (Flow), territory management, forecasting, CPQ add-ons, and the largest ISV ecosystem (AppExchange). Reporting and permissioning are best-in-class. The cost is complexity: it usually requires an admin (often 0.25-0.5 FTE at this size) or a partner for setup, and the UI is the least loved by non-sales staff.

**HubSpot (Sales Hub + Marketing Hub).** Strongest usability and fastest time-to-value. Native marketing tooling (email, forms, landing pages, ad management) lives in the same data model as the CRM, which suits an agency that runs campaigns. Automation is good but shallower than Salesforce; custom objects and granular permissions only arrive at Enterprise tier. Marketing Hub pricing scales with contact volume, which can surprise agencies holding large prospect lists.

**Zoho CRM.** Best value for money. Covers ~80% of Salesforce's functionality (workflows, blueprints, scoring, decent API) at a fraction of the price, plus the wider Zoho One suite (projects, finance, desk) if Brightline wants an all-in-one stack. Weaknesses: less polished UX, weaker ecosystem and partner network in Ireland/EU, and reporting/AI features (Zia) lag the other two.

### Ballpark pricing (EUR per user/month, annual billing)

| Platform | Realistic tier for Brightline | List EUR/user/mo | 40 seats, EUR/year (approx.) |
|---|---|---|---|
| Salesforce | Sales Cloud Enterprise | ~165 | ~79,000 |
| Salesforce (lighter) | Pro Suite | ~100 | ~48,000 |
| HubSpot | Sales Hub Professional (~90/seat) + Marketing Hub Pro (~800/mo flat) | blended ~110 | ~53,000 |
| Zoho CRM | Enterprise | ~40 | ~19,000 |

Implementation/onboarding is extra: expect EUR 10-30k partner cost for Salesforce, ~EUR 3-6k mandatory onboarding for HubSpot Pro, and near-zero to EUR 5k for Zoho.

### Integration fit with ad-ops tooling

- **Meta/Google Ads:** HubSpot is the standout — native ads integration syncs audiences, captures lead-ad submissions, and attributes ad spend to CRM deals out of the box. Salesforce needs Marketing Cloud Advertising (a paid add-on, formerly Advertising Studio) or third-party connectors. Zoho has a native Google Ads integration in CRM and Meta lead-ad sync via Zoho Social/Marketing Automation — functional but more assembly required.
- **GA4:** No CRM has a deep native GA4 integration; all three rely on UTM capture plus connectors (Zapier/Make, or GA4 BigQuery export into the CRM reporting layer). HubSpot attribution partially substitutes for GA4; Salesforce CRM Analytics can ingest GA4 data with effort.
- **Programmatic (DV360, The Trade Desk, etc.):** None offer native DSP integrations. The realistic pattern is CRM -> CDP/clean-room (LiveRamp, Hightouch-style reverse ETL) -> DSP audiences. Salesforce has the strongest enterprise path here (Data Cloud with audience activation), but at significant extra cost; HubSpot and Zoho depend on middleware. For a 40-person agency, middleware (Zapier/Make/Fivetran-lite) is the pragmatic answer regardless of CRM.

### Strengths/weaknesses summary

- Salesforce: + depth, ecosystem, scalability; - cost, admin burden, adoption risk for non-sales staff.
- HubSpot: + usability, native ad-ops fit, fast rollout; - contact-based marketing pricing, Enterprise-gated permissions.
- Zoho: + price, suite breadth; - polish, weaker EU partner ecosystem, lighter ad-ops integrations.

**Steer for next agents:** HubSpot looks like the best capability-per-euro fit for an ad agency workflow; Zoho is the budget hedge; Salesforce is justifiable only if Brightline expects enterprise-client procurement requirements demanding it.

## risk-analyst

**Assumptions.** Knowledge-only analysis, cutoff January 2026; no web access. Vendor residency options, certifications, and contract norms below reflect publicly known positions as of that date and must be confirmed in the DPA/order form before signature. I assume Brightline currently runs on spreadsheets or a lightweight legacy tool, holds EU-resident personal data (client contacts plus prospect lists used for ad targeting), and is the data controller with the CRM vendor as processor.

### GDPR posture and data residency (EU hosting)

All three vendors offer GDPR-adequate paperwork: DPAs with SCCs, sub-processor lists, and ISO 27001 programmes. The differences are residency mechanics and parent-company jurisdiction.

- **Salesforce:** EU hosting via Hyperforce EU (Frankfurt/Paris on public cloud); an optional "EU Operating Zone" restricts support access to EU personnel at extra cost. US parent means CLOUD Act / Schrems II exposure persists despite the EU-US Data Privacy Framework; mitigated by strong certifications (ISO 27701, SOC 2, C5).
- **HubSpot:** Offers an EU data centre (Frankfurt, AWS eu-central-1), but the region is fixed at portal creation — moving an existing portal later is effectively a migration project. Same US-parent transfer caveat. EMEA HQ is in Dublin, which helps with EU-facing support and contracting.
- **Zoho:** EU data centres (Amsterdam, with EU-region redundancy); privately held Indian parent, so transfer-risk analysis runs to India rather than the US — less litigated territory, cover it explicitly via SCCs and a transfer impact assessment.

Cross-cutting: syncing prospect lists to Meta/Google custom audiences (the ad-ops integrations the market section praises) creates joint-controller obligations and consent/legitimate-interest questions that sit with Brightline, not the CRM vendor, whichever platform is chosen.

### Vendor lock-in and exit costs

Lock-in ranks Salesforce > HubSpot > Zoho. Salesforce data exports cleanly (API/weekly export), but configuration — Flows, Apex, AppExchange dependencies — is non-portable, and contracts are typically annual-to-multi-year with auto-renewal and ~5-7% renewal uplifts; budget EUR 15-40k for a future exit. HubSpot exports contacts/deals easily, but marketing assets (workflows, emails, landing pages, attribution history) do not port; exit ~EUR 8-20k. Zoho is cheapest to leave (~EUR 5-12k) unless Brightline adopts the wider Zoho One suite, which couples CRM exit to finance/projects tooling. Mitigation for all: keep integration logic in middleware (Zapier/Make) rather than vendor-native automation where practical, and take quarterly full data exports.

### Migration and adoption risk

Inbound migration risk is mostly Brightline-side: dirty spreadsheet data, no dedup discipline, and 40 staff of whom many are creatives who will resist a heavy tool. Salesforce carries the highest adoption risk (complex UI, needs an admin); HubSpot the lowest; Zoho mid-pack (acceptable UX, fewer local partners to rescue a stalled rollout).

### Hidden-cost risk

- **Salesforce:** admin time (0.25-0.5 FTE), Premier Support (+30% of net licence), paid sandboxes, API limits, Marketing Cloud Advertising add-on. Realistic year-1 total can be 1.5-2x licence.
- **HubSpot:** Marketing Hub contact-tier pricing climbs as prospect lists grow; granular permissions and custom objects are Enterprise-gated, creating upgrade pressure; mandatory paid onboarding.
- **Zoho:** the sticker price is honest, but middleware to close ad-ops gaps, configuration time, and premium support (~20-25% of licence) erode the gap somewhat.

### Support risk

Salesforce standard support is slow without Premier, but the EU/Irish partner ecosystem is the deepest. HubSpot has strong in-app support and a Dublin EMEA HQ — lowest support risk for Brightline specifically. Zoho support quality is the weakest of the three and its EU partner bench is thin; mitigate by buying Zoho's premium support tier and contracting a named EU partner.

### Risk register

| # | Risk | Vendor | Severity (1-5) | Mitigation |
|---|---|---|---|---|
| 1 | US CLOUD Act / transfer challenge to EU data | Salesforce, HubSpot | 3 | EU residency option + SCCs + transfer impact assessment; minimise special-category data in CRM |
| 2 | EU region fixed at signup; later residency change = re-migration | HubSpot | 3 | Select EU data centre at portal creation, day one |
| 3 | Transfer-risk analysis to India under-scrutinised | Zoho | 2 | SCCs, TIA, confirm EU DC and EU-only support access in DPA |
| 4 | Joint-controller exposure from ad-audience syncs | All | 4 | Consent/LIA review before enabling Meta/Google audience sync; document in RoPA |
| 5 | Config lock-in (Flows/Apex/AppExchange) raises exit cost | Salesforce | 4 | Keep automations in middleware; quarterly exports; avoid multi-year terms initially |
| 6 | Marketing-asset lock-in and contact-tier price creep | HubSpot | 3 | Cap synced contact lists; archive cold contacts; negotiate contact-tier pricing at renewal |
| 7 | Suite coupling via Zoho One spreads lock-in beyond CRM | Zoho | 2 | Adopt CRM only; evaluate suite modules individually |
| 8 | Adoption failure among non-sales staff stalls rollout | Salesforce | 4 | Phased rollout, named admin, role-trimmed UI; reconsider Pro Suite over Enterprise |
| 9 | Hidden year-1 costs (admin FTE, support uplift, add-ons) | Salesforce | 4 | Budget 1.5-2x licence year 1; fixed-price implementation partner |
| 10 | Enterprise-tier upgrade pressure for permissions/objects | HubSpot | 3 | Validate Professional-tier permission model in trial before contract |
| 11 | Weak EU support/partner bench delays issue resolution | Zoho | 3 | Premium support tier + named EU partner on retainer |
| 12 | Dirty source data corrupts migration | All | 3 | Pre-migration dedup/cleanse sprint; migrate in waves with validation gates |

**Steer for next agent:** risk profile does not overturn the market steer — HubSpot's residual risks (contact-tier creep, Enterprise gating, region-at-signup) are all mitigable by contract and configuration choices, whereas Salesforce's dominant risks (cost blow-out, adoption) are structural at 40 seats. Zoho is risk-acceptable only with paid support and middleware budget priced in.
